<template>
  <div class="vol-wrap" :class="{ open }" @wheel="onWheel">
    <button class="icon-btn vol-btn" :class="btnCls" :title="store.vol === 0 ? '把声音放出来' : '静音'" @click="toggleMute">
      <svg v-if="store.vol === 0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round"><path d="M11.2 4.4 6 8.7H2.4v6.6H6l5.2 4.3z"/><path d="M15.4 9.3l5.2 5.4M20.6 9.3l-5.2 5.4"/></svg>
      <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round"><path d="M11.2 4.4 6 8.7H2.4v6.6H6l5.2 4.3z"/><path d="M15 8.8a4.6 4.6 0 0 1 0 6.4"/><path d="M18.2 6.2a8.4 8.4 0 0 1 0 11.6"/></svg>
    </button>
    <div class="vol-pop">
      <div
        ref="track"
        class="vol-track"
        :style="{ '--vol-pct': store.vol + '%' }"
        @mousedown="down"
      >
        <i></i>
      </div>
    </div>
  </div>
</template>

<script setup>
/* 音量：点图标=静音切换；移入弹竖条；竖条可点可拖；滚轮 ±5 吸附 5 的倍数。
   按钮样式由 btnCls 决定，跟同一行其它图标保持同一档（大窗 bare / 小窗 sm）。 */
import { ref } from "vue";
import { audio, store } from "../lib/store.js";
import { applyVol } from "../lib/volume.js";
import { wheelVol } from "../lib/player.js";

const props = defineProps({
  btnCls: { type: String, default: "bare" },
});

const track = ref(null);
const open = ref(false);
let popTimer = null;

function popOpen() { open.value = true; clearTimeout(popTimer); }
function popClose() { open.value = false; }

function toggleMute() {
  const v = audio.volume > 0 ? 0 : 70;
  applyVol(v);
  popOpen();
  clearTimeout(popTimer);
  popTimer = setTimeout(popClose, 1000);
}

function volFrom(e) {
  const r = track.value.getBoundingClientRect();
  const v = Math.round(Math.max(0, Math.min(1, 1 - (e.clientY - r.top) / r.height)) * 100);
  applyVol(v);
}

function down(e) {
  popOpen();
  volFrom(e);
  const move = (ev) => {
    if (!(ev.buttons & 1)) { up(); return; } // 失焦丢 mouseup 保护
    volFrom(ev);
  };
  const up = () => {
    window.removeEventListener("mousemove", move);
    window.removeEventListener("mouseup", up);
    clearTimeout(popTimer);
    popTimer = setTimeout(popClose, 700);
  };
  window.addEventListener("mousemove", move);
  window.addEventListener("mouseup", up);
}

function onWheel(e) {
  wheelVol(e, popOpen, popClose);
}
</script>

<style scoped>
.vol-btn { padding: 0; }
</style>
