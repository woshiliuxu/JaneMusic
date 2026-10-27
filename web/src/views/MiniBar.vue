<template>
  <div class="mini-bar" @wheel="wheelSwitch">
    <img v-if="miniPic" :src="miniPic" class="mini-cover" alt="" draggable="false" data-tauri-drag-region />
    <div class="mini-info" data-tauri-drag-region>
      <b class="ellipsis">{{ titleLine }}</b>
      <div class="mini-lyric ellipsis">{{ store.curLyric || " " }}</div>
    </div>
    <button class="icon-btn sm" title="放 / 停" @click="togglePlay">
      <svg v-if="!store.playing" viewBox="0 0 24 24" fill="currentColor"><path d="M8 5.1v13.8c0 .8.9 1.3 1.6.9l10.9-6.9c.6-.4.6-1.4 0-1.8L9.6 4.2c-.7-.4-1.6.1-1.6.9z"/></svg>
      <svg v-else viewBox="0 0 24 24" fill="currentColor"><path d="M7 4h3.5v16H7zM13.5 4H17v16h-3.5z"/></svg>
    </button>
    <button class="icon-btn sm" title="换一首" @click="playNext('x', true)">
      <svg viewBox="0 0 24 24" fill="currentColor"><path d="M16 5h2v14h-2zM4 5l11 7-11 7z"/></svg>
    </button>
    <button class="icon-btn sm" title="展开看看" @click="expand">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round"><path d="M14 4h6v6M10 20H4v-6M20 4l-7 7M4 20l7-7"/></svg>
    </button>
  </div>
</template>

<script setup>
/* 胶囊 340×84：封面 + 「歌名 - 歌手」 + 当前歌词 + 三键；封面/文字可拖动窗口 */
import { computed } from "vue";
import { store } from "../lib/store.js";
import { proxied } from "../lib/tauri.js";
import { wheelSwitch, togglePlay, playNext } from "../lib/player.js";
import { setMode } from "../lib/modes.js";

const song = computed(() => store.uiSong);
const titleLine = computed(() => {
  const s = song.value;
  if (!s) return "Jane Music";
  const t = s.title || "";
  const a = s.author || "";
  return a ? t + " - " + a : t;
});
const miniPic = computed(() => (song.value && song.value.pic ? proxied(song.value.pic) : ""));

function expand() {
  setMode(store.prevWin || "big");
}
</script>

<style scoped>
.mini-bar {
  position: fixed; top: 0; left: 0;
  width: 340px; height: 84px; /* = MINI_SIZE：窗口怎么变胶囊都不变形 */
  z-index: 60;
  display: flex;
  visibility: visible;
  align-items: center;
  gap: 9px;
  padding: 0 7px 0 9px;
  border: 1px solid var(--glass-border);
  border-radius: 17px;
  background: var(--glass);
  box-shadow: var(--glass-inset), 0 6px 18px var(--shadow);
}
html:not([data-mode="mini"]) .mini-bar { display: none; }
.mini-cover {
  width: 44px; height: 44px; flex: 0 0 44px;
  border-radius: 12px;
  object-fit: cover;
  -webkit-user-drag: none;
  box-shadow: 0 2px 8px var(--shadow);
  cursor: grab;
}
.mini-info {
  flex: 1; min-width: 0;
  display: flex; flex-direction: column; justify-content: center;
  line-height: 1.3;
  cursor: grab;
}
.mini-info b { font-size: 12.5px; font-weight: 600; }
.mini-lyric { font-size: 11px; color: var(--dim); }
</style>
