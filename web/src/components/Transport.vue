<template>
  <div class="transport" :style="{ gap }">
    <button class="icon-btn" :class="btnCls" :disabled="histLen < 2" title="退回上一首" @click="playPrev(axis)">
      <svg viewBox="0 0 24 24" fill="currentColor"><path d="M6 5h2v14H6zM20 5v14L9 12z"/></svg>
    </button>
    <button class="icon-btn big" :class="btnCls" title="放 / 停" @click="togglePlay">
      <svg v-if="!store.playing" viewBox="0 0 24 24" fill="currentColor"><path d="M8 5.1v13.8c0 .8.9 1.3 1.6.9l10.9-6.9c.6-.4.6-1.4 0-1.8L9.6 4.2c-.7-.4-1.6.1-1.6.9z"/></svg>
      <svg v-else viewBox="0 0 24 24" fill="currentColor"><path d="M7 4h3.5v16H7zM13.5 4H17v16h-3.5z"/></svg>
    </button>
    <button class="icon-btn" :class="btnCls" title="随机下一首" @click="playNext(axis, true)">
      <svg viewBox="0 0 24 24" fill="currentColor"><path d="M16 5h2v14h-2zM4 5l11 7-11 7z"/></svg>
    </button>
  </div>
</template>

<script setup>
import { computed } from "vue";
import { store } from "../lib/store.js";
import { playPrev, playNext, togglePlay } from "../lib/player.js";

const props = defineProps({
  axis: { type: String, default: "x" },
  btnCls: { type: String, default: "" },
  gap: { type: String, default: "22px" },
});
const histLen = computed(() => store.hist.length);
</script>

<style scoped>
.transport { display: flex; align-items: center; }
</style>
