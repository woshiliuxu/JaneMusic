<template>
  <div class="seek-row">
    <span class="time">{{ fmt(shown) }}</span>
    <div class="seek" @mousedown="down">
      <i :style="{ width: pct + '%' }"></i>
      <span class="seek-dot" :style="{ left: pct + '%' }"></span>
    </div>
    <span class="time">{{ fmt(store.duration) }}</span>
  </div>
</template>

<script setup>
/* 进度条：点击/拖动 seek；拖动中窗口失焦丢 mouseup 时用 e.buttons 兜底 */
import { computed, ref } from "vue";
import { audio, store } from "../lib/store.js";
import { fmt } from "../lib/lyrics.js";

const dragT = ref(0);
const shown = computed(() => (store.seeking ? dragT.value : store.curTime));
const pct = computed(() => (store.duration ? (shown.value / store.duration) * 100 : 0));

function seekFrom(e) {
  const r = e.currentTarget.getBoundingClientRect();
  const x = (e.clientX - r.left) / r.width;
  const d = audio.duration;
  if (!d) return;
  const p = Math.max(0, Math.min(1, x));
  dragT.value = p * d;
  audio.currentTime = p * d;
}
function down(e) {
  store.seeking = true;
  seekFrom(e);
  const move = (ev) => {
    if (!(ev.buttons & 1)) { up(); return; }
    seekFrom(ev);
  };
  const up = () => {
    store.seeking = false;
    window.removeEventListener("mousemove", move);
    window.removeEventListener("mouseup", up);
  };
  window.addEventListener("mousemove", move);
  window.addEventListener("mouseup", up);
}
</script>
