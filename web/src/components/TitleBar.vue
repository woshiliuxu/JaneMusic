<template>
  <div class="titlebar" :class="{ thin }">
    <h1 data-tauri-drag-region-exclude @click="$emit('settings')">Jane Music</h1>
    <div class="tb-space" data-tauri-drag-region></div>
    <!-- 榜单入口唯一位置：标题栏右上角，紧挨窗口按钮组 -->
    <ChartMenu :variant="thin ? 'chip' : 'icon'" />
    <button
      class="win-btn"
      data-tauri-drag-region-exclude
      :title="mode === 'big' ? '缩成小窗' : '放大一点'"
      @click="toggleWin"
    >
      <!-- 图标画的是“将切换到的形态”：大窗态显示竖长小窗，小窗态显示宽扁大窗 -->
      <svg v-if="mode === 'big'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linejoin="round">
        <rect x="8.5" y="3" width="7" height="18" rx="2"/>
        <path d="M8.5 7.2h7"/>
      </svg>
      <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linejoin="round">
        <rect x="3" y="7.5" width="18" height="9" rx="2"/>
        <path d="M8.2 7.5v9"/>
      </svg>
    </button>
    <button class="win-btn" data-tauri-drag-region-exclude title="换个光线" @click="$emit('theme')">
      <svg v-if="theme === 'dark'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round"><circle cx="12" cy="12" r="4.2"/><path d="M12 2.5v2.2M12 19.3v2.2M2.5 12h2.2M19.3 12h2.2M5 5l1.6 1.6M17.4 17.4 19 19M19 5l-1.6 1.6M6.6 17.4 5 19"/></svg>
      <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round"><path d="M21 12.8A9 9 0 1 1 11.2 3 7 7 0 0 0 21 12.8z"/></svg>
    </button>
    <button class="win-btn" :class="{ pinned }" data-tauri-drag-region-exclude title="钉在最前面" @click="$emit('pin')">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round"><path d="M12 17v5M9 3h6l1 7 3 2H5l3-2z"/></svg>
    </button>
    <button class="win-btn" data-tauri-drag-region-exclude title="缩到任务栏" @click="minimize">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round"><path d="M5 12h14"/></svg>
    </button>
    <button class="win-btn close" data-tauri-drag-region-exclude title="收进托盘，不占地方" @click="hideWin">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round"><path d="M6 6l12 12M18 6L6 18"/></svg>
    </button>
  </div>
</template>

<script setup>
import { computed } from "vue";
import ChartMenu from "./ChartMenu.vue";
import { store } from "../lib/store.js";
import { invoke } from "../lib/tauri.js";
import { setMode } from "../lib/modes.js";

const props = defineProps({ thin: Boolean });
defineEmits(["settings", "theme", "pin"]);

const mode = computed(() => store.mode);
const theme = computed(() => store.theme);
const pinned = computed(() => store.pinned);

function toggleWin() {
  setMode(store.mode === "big" ? "small" : "big");
}
function minimize() { invoke("minimize_window").catch(() => {}); }
function hideWin() { invoke("close_window").catch(() => {}); }
</script>

<style scoped>
.titlebar { height: 60px; }
.titlebar.thin { height: 50px; }
</style>
