/* 播放器引擎：榜单、抽歌、摆歌、播放、死链、限流、歌词、滚轮、快捷键、启动。
   只操作 store/audio/kv，不碰 DOM —— 视图组件 watch store 自己渲染。 */
import { audio, store, kv, kvSave, toast, setStatus, hideStatus, setMotion, openModal, closeModal, misc } from "./store.js";
import { invoke, setStreamPort, proxied, proxiedSong } from "./tauri.js";
import { CATS, MAX_HIST, MAX_RETRY, PL_TTL, WHEEL_STEP, WHEEL_COOL } from "./constants.js";
import { isDead, markDead } from "./dead.js";
import { rateReset, rateClearCool, rateHit } from "./rate.js";
import { parseLrc, indexAt, fmt } from "./lyrics.js";
import { cancelRamp, rampTo, applyVol } from "./volume.js";

/* ---------------- 榜单 ---------------- */

/* 接口源跟随后端启动测速：多个 Meting 源谁快用谁（lib.rs 的 src-probe）。
   历史会话（state.json）和本地歌单缓存里存的 pic/lrc/音频还是旧源地址，
   按当前首选源基址重写——音频在 Rust 侧早这么做（rewrite_audio_url），
   这里补齐封面和歌词；这份是给「恢复会话」用的，那条路绕过 Rust 解析。 */
let srcBase = "https://api.qijieya.cn/meting/"; // boot 时由后端告知首选源基址
function markSource(b) {
  if (typeof b === "string" && b) srcBase = b;
}
function fixSource(u) {
  if (!u || u.indexOf("meting") < 0 || u.indexOf(srcBase) === 0) return u; // 非 Meting 或已是首选源
  const id = (u.match(/[?&]id=(\d+)/) || [])[1];
  const ty = (u.match(/[?&]type=(url|pic|lrc)/) || [])[1];
  return id && ty ? srcBase + "?server=netease&type=" + ty + "&id=" + id : u;
}
function fixSong(s) {
  if (s && typeof s === "object") {
    s.url = fixSource(s.url);
    s.alt = fixSource(s.alt);
    s.pic = fixSource(s.pic);
    s.lrc = fixSource(s.lrc);
    /* 旧版 state.json 里可能存着拉词中途被杀留下的 __lrcFetching:true，
       不清掉的话 loadLrc 直接返回，歌词永远停在加载态（落盘侧也已剥离） */
    delete s.__lrcFetching;
  }
  return s;
}

export async function ensurePlaylist(catId, silent) {
  /* 上次失败会被写成空数组，这里要按长度判断，否则这个榜单再也拉不起来 */
  if (store.playlists[catId] && store.playlists[catId].length) return;
  store.loading = true;
  try {
    const songs = await invoke("fetch_playlist", { id: catId });
    if (!Array.isArray(songs) || songs.length === 0) throw new Error("empty");
    store.playlists[catId] = songs.filter((s) => s && s.url && s.title);
    kv.playlists[catId] = { ts: Date.now(), songs: store.playlists[catId] };
    kvSave();
    prefetchSongs(store.playlists[catId]);
    prepareNext();
    if (!store.startedOnce && !store.current && !store.lastSong) stageRandom();
  } catch {
    if (silent) {
      if (!store.playlists[catId] || store.playlists[catId].length === 0) setStatus("榜单跑丢了", "bad");
    } else {
      setStatus("榜单跑丢了", "bad");
      toast("榜单没接上信号，看看网络再来一次", "err");
      store.playlists[catId] = [];
    }
  }
  store.loading = false;
}

export function setCat(catId) {
  if (store.loading) { toast("榜单还在路上，稍等一下~", "err"); return; }
  const cat = CATS.find((c) => c.id === catId);
  const name = cat ? cat.name : "新榜单";
  if (catId === store.catId) { toast("已经在「" + name + "」了，没走错", "ok"); return; }
  store.catId = catId;
  localStorage.setItem("jm.cat", catId);
  kv.cat = catId;
  kvSave();
  /* 预选的下一首属于旧榜单，作废后重新抽 */
  store.nextSong = null;
  switchChart(catId, name);
}

/* 切榜要有明确结果反馈：拉取中显示状态条，成功/失败都 toast 一句 */
async function switchChart(catId, name) {
  const cached = store.playlists[catId];
  if (!cached || !cached.length) setStatus("正在把「" + name + "」搬过来…");
  await ensurePlaylist(catId, true);
  const list = store.playlists[catId] || [];
  if (!list.length) {
    setStatus("榜单跑丢了", "bad");
    toast("「" + name + "」没搬成，看看网络再来一次", "err");
    return;
  }
  hideStatus();
  toast("换到「" + name + "」了，" + list.length + " 首随便挑", "ok");
}

/* 让后端悄悄把几首歌下进磁盘缓存：下次抽到秒开。随机抽，别总顶着榜单头几首。 */
function prefetchSongs(songs) {
  const alive = songs.filter((s) => s && s.url && !isDead(s.url));
  for (let i = alive.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [alive[i], alive[j]] = [alive[j], alive[i]];
  }
  const urls = alive.slice(0, 3).map((s) => s.url);
  if (urls.length) invoke("prefetch", { urls }).catch(() => {});
}

/* ---------------- 抽歌（下一首提前定好并预热） ---------------- */

function playlist() { return store.playlists[store.catId]; }

function pickAlive(exclude) {
  const list = playlist();
  if (!list || list.length === 0) return null;
  const alive = list.filter((s) => !isDead(s.url) && (!exclude || s.url !== exclude.url));
  return alive.length ? alive : null;
}

function warmSong(song) {
  if (!song || !getStreamPort0()) return;
  const urls = [];
  if (song.pic && !misc.warm.has("pic:" + song.pic)) {
    misc.warm.add("pic:" + song.pic);
    const im = new Image();
    im.crossOrigin = "anonymous";
    im.src = proxied(song.pic);
    urls.push(song.pic);
  }
  if (song.url && !misc.warm.has("snd:" + song.url)) {
    misc.warm.add("snd:" + song.url);
    urls.push(song.url);
  }
  if (urls.length) invoke("prefetch", { urls }).catch(() => {});
}

function prepareNext() {
  const alive = pickAlive(store.current || store.lastSong);
  if (!alive) { store.nextSong = null; return; }
  const song = alive[Math.floor(Math.random() * alive.length)];
  store.nextSong = song;
  warmSong(song);
}

export function pickRandom() {
  const alive = pickAlive(null);
  if (!alive) {
    const list = playlist();
    if (list && list.length) toast("这个榜单的歌全哑了，换一个吧", "err");
    return;
  }
  let song = null;
  if (store.nextSong) {
    const hit = alive.find((s) => s.url === store.nextSong.url);
    if (hit) song = hit;
  }
  if (!song) song = alive[Math.floor(Math.random() * alive.length)];
  playSong(song);
}

/* ---------------- 摆歌（不自动播）与界面切换 ---------------- */

/* 把界面（含封面/歌名/歌词）切到某首歌；anim=false 开局摆歌不打动画 */
function applySongUI(song) {
  store.uiSong = song;
  store.lrcLines = parseLrc(song.lrcText || "");
  /* 有歌词地址但还没到手 → 是「正在加载」，不是「没有歌词」 */
  store.lrcLoading = !!(song && song.lrc) && song.lrcText === undefined;
  store.lyrIdx = -1;
  store.curLyric = "";
  store.uiVer++;
  /* 只在“正在放、但音频还没缓冲好”时转圈；开局摆歌不设 src，readyState 恒为 0，
     这里若无条件判断会让封面上的加载圈一直转下去 */
  if (!audio.paused && audio.readyState < 3) store.coverLoading = true;
  /* 一切到这首歌就开始拉歌词（以前只在 playing 事件里拉，切歌后那几百毫秒
     会被误判成「这首歌没留词」） */
  loadLrc(song);
}

function stageSong(song) {
  if (!song || !song.url) return;
  store.lastSong = song;
  applySongUI(song);
  loadLrc(song);
  prepareNext();
}

export function stageRandom() {
  const list = playlist();
  if (!list || list.length === 0) return;
  const alive = list.filter((s) => !isDead(s.url));
  if (alive.length === 0) return;
  stageSong(alive[Math.floor(Math.random() * alive.length)]);
}

/* 切歌动效调度：152ms 处换内容（滑出正好结束），460ms 兜底 */
let switchToken = 0;
const switchTimers = [];
function scheduleSwitch(song, frame) {
  const my = ++switchToken;
  store.switchAnim = true;
  store.motionFrame = frame;
  switchTimers.forEach(clearTimeout);
  switchTimers.length = 0;
  const swap = () => {
    if (my !== switchToken) return;
    applySongUI(song);
    store.switchAnim = false;
  };
  switchTimers.push(setTimeout(swap, 152));
  switchTimers.push(setTimeout(swap, 460));
}

/* ---------------- 播放 ---------------- */

export function playSong(song) {
  try {
    if (!getStreamPort0()) { toast("播放服务还在热身，再点一次", "err"); return; }
    const prevUrl = store.current ? store.current.url : (store.lastSong ? store.lastSong.url : null);
    const same = prevUrl === song.url;
    store.current = song;
    store.startedOnce = true;
    hideStatus();
    store.coverLoading = !same;
    if (song.pic && !same) {
      const pre = new Image();
      pre.crossOrigin = "anonymous";
      pre.src = proxied(song.pic);
    }
    if (same) {
      applySongUI(song);
    } else {
      scheduleSwitch(song, store.motionFrame || { oX: 30, oY: 0, iX: -30, iY: 0 });
      setMotion("x", 1); // 复位：下一次默认左右滑
    }
    audio.volume = 0; // 淡入的起点
    audio.src = proxiedSong(song);
    const p = audio.play();
    if (p && p.catch) p.catch((e) => dbg("play拒绝: " + (e && e.name)));
  } catch (e) {
    dbg("playSong错误: " + (e && e.message));
  }
  store.lastSong = song;
  const i = store.hist.findIndex((h) => h.url === song.url);
  if (i >= 0) store.hist.splice(i, 1);
  store.hist.unshift(Object.assign({}, song));
  if (store.hist.length > MAX_HIST) store.hist.pop();
  kv.last = song;
  kv.hist = store.hist.slice(0, 60);
  kvSave();
  prepareNext();
}

/* 歌词按需抓 */
export async function loadLrc(song) {
  if (!song || !song.lrc || song.lrcText !== undefined || song.__lrcFetching) return;
  song.__lrcFetching = true;
  /* 按歌曲 id 而不是对象身份判断“还是不是当前显示的歌”：
     恢复会话/重建歌单会生成同歌不同对象，对象比较会让 finally 清不掉 lrcLoading，
     歌词永远停在「正在飞奔来的路上」 */
  const mine = () => !!store.uiSong && store.uiSong.id === song.id;
  if (mine()) store.lrcLoading = true;
  try {
    const text = await invoke("fetch_text", { url: song.lrc });
    song.lrcText = String(text);
    if (mine()) {
      store.lrcLines = parseLrc(song.lrcText);
      store.lyrIdx = -1;
    }
  } catch {
    /* 拿不到就当这首歌没歌词，别反复重试 */
    song.lrcText = "";
  } finally {
    song.__lrcFetching = false;
    if (mine()) store.lrcLoading = false;
  }
}

/* ---------------- 播放器事件 ---------------- */

const audioState = () =>
  "t=" + audio.currentTime.toFixed(2) + " rdy=" + audio.readyState + " net=" + audio.networkState +
  (audio.buffered.length ? " buf=0-" + audio.buffered.end(audio.buffered.length - 1).toFixed(1) : " buf=-");

export function bindAudio() {
  audio.addEventListener("playing", () => {
    store.failStreak = 0;
    store.sessionOk = true;
    rateReset();
    dbg("PLAYING " + audioState());
    hideStatus();
    store.playing = true;
    store.coverLoading = false;
    if (store.vol > 0) rampTo(store.vol, 420); else audio.volume = 0;
    loadLrc(store.current);
  });
  audio.addEventListener("pause", () => { store.playing = false; });
  audio.addEventListener("loadedmetadata", () => { store.duration = audio.duration || 0; });
  audio.addEventListener("timeupdate", () => {
    if (store.seeking) return;
    store.curTime = audio.currentTime || 0;
    store.duration = audio.duration || 0;
    updateLyric();
  });
  /* 验明正身：只有后端确认上游 404/410/415 才算真死链 */
  audio.addEventListener("error", () => {
    const song = store.current;
    const code = audio.error ? audio.error.code : -1;
    const msg = audio.error ? audio.error.message : "";
    dbg("audio err" + code + " [" + msg + "] " + audioState() + " cur=" + (song ? song.title : "null"));
    if (!song) return;
    store.current = null;
    store.failStreak++;
    store.playing = false;
    store.coverLoading = false;

    const hits = rateHit();
    if (hits >= 5) {
      stopWith("先喘口气", "一分钟内摔了太多次，歇会儿", "自动重试太频繁，暂停一分钟，避免被曲库拉黑");
      return;
    }

    verifyDead(song).then((dead404) => {
      dbg("verify " + song.title + " dead404=" + dead404 + " failStreak=" + store.failStreak);
      if (dead404) {
        markDead(song.url);
        toast("「" + song.title + "」人间蒸发了，跳下一首", "err");
      }
      const list = playlist() || [];
      const alive = list.filter((s) => !isDead(s.url)).length;
      if (alive > 0) {
        const wait = Math.min(4000, 800 * Math.pow(2, Math.min(store.failStreak - 1, 3)));
        setTimeout(() => { if (!store.current) pickRandom(); }, wait);
      } else {
        stopWith("这个榜单全军覆没", "右上角换一个榜单试试", "这个榜单没有能放的歌了");
      }
    });
  });
  audio.addEventListener("ended", () => pickRandom());
  ["abort", "emptied", "stalled", "suspend", "waiting", "loadeddata", "canplay", "loadstart"].forEach((ev) => {
    audio.addEventListener(ev, () => dbg("ev:" + ev + " " + audioState() + " " + (store.current ? store.current.title : "null")));
  });
}

async function verifyDead(song) {
  try {
    const st = await invoke("check_url", { url: song.url });
    return st === 404 || st === 410 || st === 415;
  } catch { return false; }
}

function stopWith(title, sub, tip) {
  store.current = null;
  store.failStreak = 0;
  store.playing = false;
  store.coverLoading = false;
  setStatus("暂停一下，马上回来", "bad");
  store.uiSong = { title, author: sub, url: "", pic: "", lrcText: "" };
  store.lrcLines = [];
  store.curLyric = "";
  store.uiVer++;
  toast(tip, "err");
}

/* 歌词跟随（timeupdate 每 ~250ms 调一次） */
function updateLyric() {
  const lines = store.lrcLines;
  if (!lines || lines.length === 0) { if (store.lyrIdx !== -1) { store.lyrIdx = -1; store.curLyric = ""; } return; }
  const idx = indexAt(lines, (audio.currentTime || 0) + 0.2);
  if (idx !== store.lyrIdx) {
    store.lyrIdx = idx;
    store.curLyric = idx >= 0 ? lines[idx].txt : "";
  }
}

/* ---------------- 三键与历史回退 ---------------- */

export function togglePlay() {
  if (!audio.src) {
    if (store.lastSong && !isDead(store.lastSong.url)) playSong(store.lastSong);
    else pickRandom();
    return;
  }
  if (audio.paused) playWithFade(); else pauseWithFade();
}

function playWithFade() {
  cancelRamp();
  audio.volume = 0;
  const p = audio.play();
  if (p && p.catch) p.catch(() => {});
}

export function pauseWithFade() {
  if (audio.paused) return;
  rampTo(0, 160, () => {
    audio.pause();
    audio.volume = store.vol / 100;
  });
}

function manualPick() {
  const now = Date.now();
  if (now - (store.lastPick || 0) < 1200) return;
  store.lastPick = now;
  rateClearCool();
  rateReset();
  store.failStreak = 0;
  pickRandom();
}

export function playPrev(axis) {
  setMotion(axis || "x", -1);
  const i = store.hist.findIndex((h) => store.current && h.url === store.current.url);
  const next = store.hist[Math.min(i + 1, store.hist.length - 1)];
  if (next && next !== store.current) playSong(next);
}

export function playNext(axis, throttled) {
  setMotion(axis || "x", 1);
  if (throttled) { manualPick(); return; }
  store.lastPick = 0;
  rateClearCool();
  rateReset();
  store.failStreak = 0;
  pickRandom();
}

/* ---------------- 收藏 ---------------- */

export function isFav(song) {
  return !!(song && store.favs.some((f) => f.url === song.url));
}
export function toggleFav(song) {
  if (!song || !song.url) return;
  const i = store.favs.findIndex((f) => f.url === song.url);
  if (i >= 0) {
    store.favs.splice(i, 1);
    toast("「" + song.title + "」从抽屉里放出去了", "ok");
  } else {
    store.favs.unshift(Object.assign({}, song));
    toast("「" + song.title + "」收进小抽屉了", "ok");
  }
  kv.favs = store.favs.slice(0, 500);
  kvSave();
}

/* ---------------- 滚轮：封面切歌 / 音量上调节 ---------------- */

const wheelState = { songAt: 0, acc: 0, volAcc: 0, popTimer: null };

export function wheelSwitch(e) {
  if (!kv.wheelSong) return;
  e.preventDefault();
  const now = Date.now();
  wheelState.acc += e.deltaY;
  if (Math.abs(wheelState.acc) < WHEEL_STEP) return;
  const dir = wheelState.acc > 0 ? 1 : -1;
  wheelState.acc = 0;
  if (now - wheelState.songAt < WHEEL_COOL) return;
  wheelState.songAt = now;
  if (dir > 0) playNext("y"); else playPrev("y");
}

export function wheelVol(e, popOpen, popClose) {
  if (!kv.wheelVol) return;
  e.preventDefault();
  wheelState.volAcc += e.deltaY;
  if (Math.abs(wheelState.volAcc) < 30) return;
  const dir = wheelState.volAcc > 0 ? -1 : 1;
  wheelState.volAcc = 0;
  const cur = typeof store.vol === "number" ? store.vol : Math.round(audio.volume * 100);
  const v = Math.max(0, Math.min(100, Math.round((cur + dir * 5) / 5) * 5));
  if (v === cur) return;
  applyVol(v);
  if (popOpen) popOpen();
  clearTimeout(wheelState.popTimer);
  wheelState.popTimer = setTimeout(() => { if (popClose) popClose(); }, 1000);
}

/* ---------------- 调试 ---------------- */

export function dbg(msg) {
  store.dbgLines.unshift(String(Date.now() % 1000000).padStart(6, "0") + " " + msg);
  if (store.dbgLines.length > 24) store.dbgLines.pop();
  try { invoke("log_line", { msg }).catch(() => {}); } catch { /* 忽略 */ }
}

/* ---------------- 快捷键 ---------------- */

export function bindKeys() {
  window.addEventListener("keydown", (e) => {
    const tag = e.target && e.target.tagName;
    if (tag === "INPUT" || tag === "TEXTAREA") return;
    if (e.code === "Space") {
      e.preventDefault();
      togglePlay();
    } else if (e.key === "n" || e.key === "N") {
      manualPick();
    } else if (e.key === "d" || e.key === "D") {
      store.dbgOn = !store.dbgOn;
    }
  });
  window.addEventListener("error", (e) => dbg("JS错误: " + e.message));
  window.addEventListener("unhandledrejection", (e) => dbg("Promise错误: " + (e.reason && e.reason.message ? e.reason.message : e.reason)));
}

/* ---------------- 启动 ---------------- */

function getStreamPort0() {
  // 端口在 boot 时写入 tauri.js；这里用闭包读不到就直接判 store
  return store._port || 0;
}
export function markPort(p) { store._port = p; }

export async function boot() {
  const say = (m) => { try { invoke("log_line", { msg: m }).catch(() => {}); } catch { /* */ } };
  say("boot start");
  try {
    await bootInner(say);
  } catch (e) {
    say("boot FAILED: " + (e && (e.stack || e.message || String(e))));
    console.error(e);
  }
}

async function bootInner(say) {
  /* 主题先按 localStorage 摆好，state.json 回来再覆盖 */
  const lsTheme = localStorage.getItem("jm.theme");
  if (lsTheme) { store.theme = lsTheme; document.documentElement.setAttribute("data-appearance", lsTheme); }
  const lsCat = localStorage.getItem("jm.cat");
  if (lsCat && CATS.some((c) => c.id === lsCat)) { store.catId = lsCat; kv.cat = lsCat; }
  for (const k of ["wheelSong", "wheelVol"]) {
    const s = localStorage.getItem("jm." + k);
    if (s !== null) kv[k] = s === "1";
  }
  const lsVol = localStorage.getItem("jm.vol");
  if (lsVol !== null) { store.vol = Math.max(0, Math.min(100, +lsVol)); audio.volume = store.vol / 100; }
  const lsLyr = localStorage.getItem("jm.lyrSize");
  if (lsLyr !== null) {
    store.lyrSize = Math.max(12, Math.min(24, Math.round(+lsLyr || 15)));
    document.documentElement.style.setProperty("--lyr-base", store.lyrSize + "px");
  }

  const [port, raw, src] = await Promise.all([
    invoke("get_stream_port").catch(() => 0),
    invoke("load_state").catch(() => ""),
    invoke("preferred_source").catch(() => ""),
  ]);
  say("port=" + port + " state=" + (raw ? raw.length : 0) + " src=" + src);
  markSource(src);
  setStreamPort(port);
  markPort(port);

  try {
    const o = JSON.parse(raw || "{}");
    if (o && typeof o === "object") {
      if (o.last && o.last.url) { kv.last = fixSong(o.last); store.lastSong = kv.last; }
      for (const [id, v] of Object.entries(o.playlists || {})) {
        if (v && Array.isArray(v.songs) && Date.now() - v.ts < PL_TTL)
          kv.playlists[id] = { ts: v.ts, songs: v.songs.map(fixSong) };
      }
      if (typeof o.vol === "number") {
        const v = Math.max(0, Math.min(100, Math.round(o.vol)));
        store.vol = v;
        audio.volume = v / 100;
        localStorage.setItem("jm.vol", String(v));
      }
      if (typeof o.theme === "string") {
        store.theme = o.theme;
        document.documentElement.setAttribute("data-appearance", o.theme);
        localStorage.setItem("jm.theme", o.theme);
      }
      if (typeof o.cat === "string" && CATS.some((c) => c.id === o.cat)) {
        kv.cat = o.cat;
        store.catId = o.cat;
        localStorage.setItem("jm.cat", o.cat);
      }
      for (const k of ["wheelSong", "wheelVol"]) {
        if (typeof o[k] === "boolean") { kv[k] = o[k]; localStorage.setItem("jm." + k, o[k] ? "1" : "0"); }
      }
      if (typeof o.lyrSize === "number") {
        store.lyrSize = Math.max(12, Math.min(24, Math.round(o.lyrSize)));
        kv.lyrSize = store.lyrSize;
        document.documentElement.style.setProperty("--lyr-base", store.lyrSize + "px");
        localStorage.setItem("jm.lyrSize", String(store.lyrSize));
      }
      if (Array.isArray(o.hist)) {
        kv.hist = o.hist.filter((h) => h && h.url && h.title).slice(0, MAX_HIST).map(fixSong);
        store.hist = kv.hist.slice();
      }
      if (Array.isArray(o.favs)) {
        kv.favs = o.favs.filter((f) => f && f.url && f.title).slice(0, 500).map(fixSong);
        store.favs = kv.favs.slice();
      }
      /* 窗口形态：上次停在哪个模式、胶囊是从哪个窗口进去的 */
      if (typeof o.mode === "string" && ["big", "small", "mini"].includes(o.mode)) kv.mode = o.mode;
      if (typeof o.lastWin === "string" && ["big", "small"].includes(o.lastWin)) kv.lastWin = o.lastWin;
    }
  } catch { /* 坏文件当没有 */ }

  const cachedPl = kv.playlists[store.catId];
  if (cachedPl && cachedPl.songs.length) {
    store.playlists[store.catId] = cachedPl.songs.filter((s) => !isDead(s.url));
  }

  bindAudio();
  bindKeys();
  hideStatus();

  /* 上次的歌直接摆出来（点播放秒响）；没有就等榜单拉完随机摆 */
  say("stage: last=" + (kv.last ? kv.last.title : "none"));
  if (kv.last) {
    stageSong(kv.last);
    prefetchSongs([kv.last]);
  } else {
    stageRandom();
  }
  say("staged ok");
  ensurePlaylist(store.catId, true);

  /* 调试采样：状态变化才记一条 */
  let lastTick = "";
  setInterval(() => {
    if (!store.current) return;
    const s = audioState() + " paused=" + audio.paused;
    if (s !== lastTick) { lastTick = s; dbg("tick " + s); }
  }, 500);
}
