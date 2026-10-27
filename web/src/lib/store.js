/* 全局状态：唯一数据源（reactive），组件只读它 + 调 player 的动作。
   audio 元素、warm 缓存集合、各种 timer 放模块级（不进响应式）。 */
import { reactive } from "vue";
import { invoke } from "./tauri.js";
import { CATS } from "./constants.js";

/* 播放器引擎只有一个 <audio>，所有视图共用 */
export const audio = new Audio();

/* 持久化载荷（state.json）：恢复态 + 全部设置 */
export const kv = {
  last: null,
  playlists: {},
  vol: null,
  theme: null,
  cat: null,
  hist: [],
  favs: [],
  wheelSong: true,
  wheelVol: true,
  lyrSize: 15,      // 歌词基准字号（12~24）
  mode: "big",      // big | small | mini
  lastWin: "big",   // 进入胶囊前所在的窗口档位（大窗/小窗）
};

let kvTimer = null;
export function kvSave() {
  clearTimeout(kvTimer);
  kvTimer = setTimeout(() => {
    /* __lrcFetching 是「这轮会话里词拉到一半」的瞬态标记：程序在拉词中途被
       关掉/杀掉，它就以 true 落盘，下次启动 loadLrc 看到它直接返回，
       歌词永远卡在「飞奔来的路上」。落盘前剥掉，只留数据不留状态。 */
    const snapshot = JSON.stringify(kv, (k, v) => (k === "__lrcFetching" ? undefined : v));
    invoke("save_state", { json: snapshot }).catch(() => {});
  }, 900);
}

/* UI 状态（不落盘） */
export const store = reactive({
  /* 榜单 */
  catId: CATS[0].id,
  playlists: {},        // id -> [{title, author, url, pic, lrc}]

  /* 播放 */
  current: null,        // 音频目标（playSong 立即设置）
  uiSong: null,         // 界面正在展示的歌（切歌动效期间晚 current 一拍）
  uiVer: 0,             // uiSong 变化计数：视图 watch 它跑动效
  lastSong: null,
  startedOnce: false,
  hist: [],
  favs: [],
  loading: false,       // 榜单拉取中
  playing: false,
  curTime: 0,
  duration: 0,
  coverLoading: false,  // 封面/音频转圈
  seeking: false,
  failStreak: 0,
  sessionOk: false,
  lastPick: 0,
  vol: 70,
  nextSong: null,
  switchAnim: false,    // 本轮 uiVer 变化要不要带动效（摆歌不要）

  /* 歌词 */
  lrcLines: [],
  lyrIdx: -1,
  lrcLoading: false,    // 当前歌的歌词还在拉：空列表要显示「正在飞奔来的路上」而不是「没留词」
  curLyric: "",         // 单行歌词（小窗一行 / 胶囊第二行共用）
  lyrSize: 15,          // 歌词基准字号（设置里可调，落盘）

  /* 外观 */
  theme: "light",
  pinned: false,

  /* 模式 */
  mode: "big",          // big | small | mini
  prevWin: "big",       // 胶囊展开回哪个窗
  pinBeforeMini: false, // 进胶囊前的置顶状态，退出时恢复
  modeBusy: false,
  motionFrame: null,    // { oX, oY, iX, iY } 切歌动效偏移，视图用完即弃

  /* 弹层 */
  modals: { settings: false, hist: false, lyr: false, favs: false, list: false },

  /* 提示 */
  toasts: [],
  status: null,         // { text, kind } 只报错误

  /* 调试 */
  dbgLines: [],
  dbgOn: false,
});

/* 非响应式杂项 */
export const misc = {
  warm: new Set(),      // 已预热的封面/音频
  lrcCache: new Map(),  // url -> lrcText
};

/* 远程调试句柄（WebView2 --remote-debugging-port 下用 CDP 直接改状态验证 UI） */
if (typeof window !== "undefined") window.__jm = store;

export function toast(text, kind) {
  const t = { id: Date.now() + Math.random(), text, kind: kind || "" };
  store.toasts.push(t);
  if (store.toasts.length > 3) store.toasts.shift();
  setTimeout(() => {
    const i = store.toasts.indexOf(t);
    if (i >= 0) store.toasts.splice(i, 1);
  }, 3000);
}

export function setStatus(text, kind) {
  store.status = { text, kind: kind || "" };
}
export function hideStatus() {
  store.status = null;
}

/* 切歌动效方向（按钮=左右 / 滚轮=上下），用完复位成左右 */
export function setMotion(axis, dir) {
  const off = 30;
  store.motionFrame = {
    oX: axis === "x" ? -dir * off : 0,
    oY: axis === "y" ? -dir * off : 0,
    iX: axis === "x" ? dir * off : 0,
    iY: axis === "y" ? dir * off : 0,
  };
}

export function openModal(name) { store.modals[name] = true; }
export function closeModal(name) { store.modals[name] = false; }
