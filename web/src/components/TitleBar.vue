<template>
  <div class="titlebar" :class="{ thin }">
    <h1 data-tauri-drag-region-exclude @click="$emit('settings')">Jane Music</h1>
    <!-- 有新版本时亮一颗 NEW，点它直达下载页 -->
    <button
      v-if="store.update"
      class="new-badge"
      data-tauri-drag-region-exclude
      :title="`出了新版本 ${store.update.latest}，点这去下载`"
      @click="openReleases"
    >NEW</button>
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
function openReleases() { invoke("open_url", { url: store.update.url }).catch(() => {}); }
</script>

<style scoped>
/* NEW 徽标：主色小胶囊，轻微呼吸提醒但不吵 */
.new-badge {
  border: none; flex: none; cursor: pointer;
  margin-left: 7px; padding: 2.5px 8px;
  border-radius: 999px;
  font-size: 10px; font-weight: 700; letter-spacing: .8px;
  color: #fff; background: var(--accent);
  box-shadow: 0 2px 10px color-mix(in srgb, var(--accent) 45%, transparent);
  animation: new-pulse 2.2s ease-in-out infinite;
  transition: transform .15s;
}
.new-badge:hover { transform: scale(1.08); }
.new-badge:active { transform: scale(.96); }
@keyframes new-pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: .72; }
}
</style>

<style scoped>
.titlebar { height: 60px; }
.titlebar.thin { height: 50px; }
</style>
