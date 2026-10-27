<template>
  <!-- 两个窗口视图常驻渲染（visibility 切换，布局树不拆 → 切换零重排）；
       胶囊盖在最上层；弹层和 Toast 挂根上，三态共用。 -->
  <div class="views">
    <BigView class="view" :class="{ on: store.mode === 'big' }" />
    <SmallView class="view" :class="{ on: store.mode === 'small' }" />
  </div>

  <MiniBar v-show="store.mode === 'mini'" />

  <SettingsModal v-if="store.modals.settings" />
  <SongListModal
    v-if="store.modals.hist"
    title="播放历史"
    subtitle="走过的每一步，点一下就重走"
    :songs="store.hist"
    empty="还没留下脚印，先去听一首"
    clearable
    clear-text="清空脚印"
    @close="closeModal('hist')"
    @play="playFromList"
    @clear="clearHist"
  />
  <SongListModal
    v-if="store.modals.favs"
    title="我的小抽屉"
    subtitle="喜欢过的都在这儿，点一下就唱"
    :songs="store.favs"
    empty="抽屉是空的，听见喜欢的就点个心"
    clearable
    clear-text="清空抽屉"
    @close="closeModal('favs')"
    @play="playFromList"
    @clear="clearFavs"
  />
  <SongListModal
    v-if="store.modals.list"
    title="播放列表"
    subtitle="当前榜单的整卡曲目，点谁唱谁"
    :songs="currentList"
    empty="榜单还在路上，再等一会儿"
    @close="closeModal('list')"
    @play="playFromList"
  />
  <LyricModal v-if="store.modals.lyr" :song="store.uiSong" />

  <ToastHost />
  <DebugPanel />
</template>

<script setup>
import { computed, onMounted, watch } from "vue";
import { listen } from "@tauri-apps/api/event";
import BigView from "./views/BigView.vue";
import SmallView from "./views/SmallView.vue";
import MiniBar from "./views/MiniBar.vue";
import SettingsModal from "./components/SettingsModal.vue";
import SongListModal from "./components/SongListModal.vue";
import LyricModal from "./components/LyricModal.vue";
import ToastHost from "./components/ToastHost.vue";
import DebugPanel from "./components/DebugPanel.vue";
import { store, kv, kvSave, closeModal } from "./lib/store.js";
import { boot, playSong } from "./lib/player.js";
import { setMode } from "./lib/modes.js";
import { invoke } from "./lib/tauri.js";
import { SIZES } from "./lib/constants.js";
import { scheduleUpdateCheck } from "./lib/update.js";

const currentList = computed(() => store.playlists[store.catId] || []);

function playFromList(song) {
  playSong(song);
  ["hist", "favs", "list"].forEach(closeModal);
}
function clearHist() {
  store.hist = [];
  kv.hist = [];
  kvSave();
}
function clearFavs() {
  store.favs = [];
  kv.favs = [];
  kvSave();
}

onMounted(async () => {
  bindModeAttr();
  await boot();
  /* 模式恢复必须放在 boot 之后：kv.mode 要等 load_state 读回才是上次的值 */
  restoreMode();
  /* 托盘菜单请求切模式 */
  try {
    await listen("jm-mode", (e) => setMode(String(e.payload || "")));
  } catch { /* 事件桥不可用就算了 */ }
  /* 启动稳了之后探一次新版本（模块内部自己错峰） */
  scheduleUpdateCheck();
});

/* html[data-mode] 驱动全局 CSS（背景透明/胶囊显隐/展开淡入） */
function bindModeAttr() {
  watch(
    () => store.mode,
    (m) => { document.documentElement.dataset.mode = m; },
    { immediate: true }
  );
}

/* 启动恢复上次模式（big/small/mini）；没有记录就按大窗把窗口样式/尺寸落到位 */
function restoreMode() {
  const saved = ["big", "small", "mini"].includes(kv.mode) ? kv.mode : null;
  if (!saved) { applyWinStyle(); return; }
  if (saved === "mini") {
    /* 先假装身在上次那个窗口，setMode 才会把 lastWin 记对（展开时回到同一档） */
    store.mode = kv.lastWin === "small" ? "small" : "big";
    setMode("mini");
    return;
  }
  if (saved !== store.mode) setMode(saved);
  else applyWinStyle();
}

function applyWinStyle() {
  const s = SIZES[store.mode] || SIZES.big;
  invoke("set_window_size", { w: s.w, h: s.h, rounded: store.mode !== "mini" }).catch(() => {});
}
</script>

<style scoped>
.views { position: absolute; inset: 0; z-index: 1; }
</style>
