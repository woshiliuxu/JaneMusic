/* 托盘悬停气泡：在放歌就报「歌名 - 歌手」，没在放歌就轮换一句趣味提示。

   为什么用定时轮换而不是「悬停时再换」：Windows 的托盘 tip 是悬停那一刻读一次，
   没有事件能赶在气泡弹出前改字（Tauri 的 Enter 事件比气泡晚），所以闲时每 4 分钟
   自己换一句，用户每次瞄到的都不一样；放歌时根本不轮换，免得把歌名顶掉。 */

import { watch } from "vue";
import { store } from "./store.js";
import { invoke } from "./tauri.js";

/* 闲时提示：短、口语、不卖萌过头，也不要 emoji */
const IDLE = [
  "快来听歌吧~",
  "耳朵闲着呢，点我放一首",
  "随机是一种浪漫，试试？",
  "今天的歌单还没开张",
  "静音的世界少了点什么",
  "发呆不如听歌",
  "点一下，让房间有点声音",
  "歌都排好队了，就差你点头",
  "一首歌三分钟，换口气",
  "抽到哪首算哪首，挺好",
  "这个心情，适合来首什么？",
  "让耳朵先下班",
  "没在放歌，但随时可以",
  "点我，下一首就是惊喜",
  "歌单在抽屉里落灰了",
  "三分钟，把脑子清空一下",
  "随机播放也是种浪漫",
  "手边有歌，何必安静",
];

/* 托盘 tip 太长 Windows 会截断，自己先收一下 */
const TIP_MAX = 58;

let last = "";
let idx = Math.floor(Math.random() * IDLE.length);
let timer = null;

function clip(s) {
  return s.length > TIP_MAX ? s.slice(0, TIP_MAX - 1) + "…" : s;
}

function push(text) {
  if (!text || text === last) return;
  last = text;
  invoke("set_tray_tip", { text }).catch(() => {});
}

/* 正在放歌时的文案；暂停/没歌返回空串 */
function playingTip() {
  if (!store.playing) return "";
  const s = store.uiSong || store.current;
  if (!s || !s.title) return "";
  const author = s.author || "";
  return clip(author ? `${s.title} - ${author}` : s.title);
}

export function refreshTrayTip() {
  const t = playingTip();
  if (t) {
    push(t);
    return;
  }
  push(IDLE[idx % IDLE.length]);
}

/* 闲时才有意义：放着歌的时候别把歌名换掉 */
function tick() {
  if (playingTip()) return;
  idx = (idx + 1) % IDLE.length;
  push(IDLE[idx]);
}

export function initTrayTip() {
  refreshTrayTip();
  /* uiVer 换歌、playing 播放/暂停，两条都盯 */
  watch(() => [store.playing, store.uiVer], refreshTrayTip);
  if (timer) clearInterval(timer);
  timer = setInterval(tick, 4 * 60 * 1000);
}
