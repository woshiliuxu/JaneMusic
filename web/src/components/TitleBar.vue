<template>
  <div class="titlebar" :class="{ thin }" @mousedown="onDrag">
    <h1 data-tauri-drag-region-exclude @click="$emit('settings')">Jane Music</h1>
    <!-- 有新版本时在标题后跟一颗小 NEW，点它下载替换并自动重启 -->
    <button
      v-if="store.update"
      class="new-badge"
      data-tauri-drag-region-exclude
      :title="`出了新版本 ${store.update.latest}，点这自动更新`"
      @click="applyUpdate"
    >NEW</button>
    <div class="tb-space"></div>
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
import { applyUpdate } from "../lib/update.js";

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

/* 整条标题栏按下即拖。小窗 380 宽被按钮挤满，data-tauri-drag-region 依赖的
   空白区几乎没有，拖不动 —— 改成统一 mousedown 走自定义命令。
   排除：左键以外、按钮/NEW 徽标（各自有点击职责）、标题 h1（单击 = 打开设置，
   拖动循环会吞掉 click 或 click 误触发在拖动结束后，都不干净）。 */
function onDrag(e) {
  if (e.button !== 0) return;
  if (e.target.closest("button, .new-badge, h1")) return;
  invoke("drag_window").catch(() => {});
}
</script>

<style scoped>
/* NEW 徽标：横着跟在标题后的小胶囊，比标题字小一号 */
.new-badge {
  border: none; flex: none; cursor: pointer;
  margin-left: 6px; padding: 1.5px 6.5px;
  border-radius: 999px;
  font-size: 9.5px; font-weight: 700; letter-spacing: .8px;
  line-height: 1.5;
  color: #fff; background: var(--accent);
  box-shadow: 0 2px 10px color-mix(in srgb, var(--accent) 45%, transparent);
  animation: new-pulse 2.2s ease-in-out infinite;
  transition: transform .15s;
}
.new-badge:hover { transform: scale(1.08); }
.new-badge:active { transform: scale(.95); }
@keyframes new-pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: .72; }
}
</style>

<style scoped>
.titlebar { height: 60px; }
.titlebar.thin { height: 50px; }
</style>
