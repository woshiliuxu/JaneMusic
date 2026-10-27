<template>
  <ModalShell
    title="设置"
    subtitle="音量、字号，还有滚轮的小脾气"
    @close="close"
  >
    <div class="modal-b">
      <div class="field">
        <label>音量</label>
        <input
          class="rng"
          type="range"
          min="0"
          max="100"
          :value="store.vol"
          :style="{ '--rng-pct': store.vol + '%' }"
          @input="onVol"
        />
      </div>
      <div class="field">
        <label>歌词字号 <span class="val">{{ store.lyrSize }} px</span></label>
        <input
          class="rng"
          type="range"
          min="12"
          max="24"
          step="1"
          :value="store.lyrSize"
          :style="{ '--rng-pct': lyrPct + '%' }"
          @input="onLyrSize"
        />
      </div>
      <div class="field">
        <label>滚轮操作</label>
        <div class="set-row">
          <div class="set-txt">
            <b>滚封面换歌</b>
            <span>鼠标停在唱片上滚一滚，就换下一首</span>
          </div>
          <button class="sw" role="switch" :aria-checked="kv.wheelSong" @click="toggleWheel('wheelSong')"></button>
        </div>
        <div class="set-row">
          <div class="set-txt">
            <b>滚音量</b>
            <span>鼠标停在喇叭上滚一滚，声音跟着变</span>
          </div>
          <button class="sw" role="switch" :aria-checked="kv.wheelVol" @click="toggleWheel('wheelVol')"></button>
        </div>
      </div>
    </div>

    <!-- 关于：版本号 / 仓库地址 / 检查更新 -->
    <div class="about">
      <span class="ver">Jane Music v{{ APP_VERSION }}</span>
      <span class="sep">·</span>
      <button class="lnk" data-tauri-drag-region-exclude title="打开 GitHub 仓库" @click="openRepo">GitHub</button>
      <span class="sep">·</span>
      <button class="lnk" data-tauri-drag-region-exclude @click="manualCheck">检查更新</button>
      <button
        v-if="store.update"
        class="upd"
        data-tauri-drag-region-exclude
        :title="`点这去下载 ${store.update.latest}`"
        @click="openReleases"
      >有新版本 {{ store.update.latest }} →</button>
    </div>
  </ModalShell>
</template>

<script setup>
import { computed } from "vue";
import ModalShell from "./ModalShell.vue";
import { store, kv, kvSave, toast, closeModal } from "../lib/store.js";
import { applyVol } from "../lib/volume.js";
import { applyLyrSize } from "../lib/ui.js";
import { APP_VERSION, REPO_URL } from "../lib/constants.js";
import { checkUpdate } from "../lib/update.js";
import { invoke } from "../lib/tauri.js";

const emit = defineEmits(["close"]);
function close() { closeModal("settings"); }

/* 滑块填充比例（12~24 映射到 0~100） */
const lyrPct = computed(() => ((store.lyrSize - 12) / 12) * 100);

function onVol(e) {
  applyVol(+e.target.value);
}
function onLyrSize(e) {
  applyLyrSize(+e.target.value);
}
function toggleWheel(key) {
  kv[key] = !kv[key];
  localStorage.setItem("jm." + key, kv[key] ? "1" : "0");
  kvSave();
  toast(key === "wheelSong"
    ? (kv[key] ? "滚唱片换歌，开张了" : "滚唱片换歌，收摊了")
    : (kv[key] ? "滚轮调音量，拧一下试试" : "滚轮调音量，先歇着"), "ok");
}

function openRepo() { invoke("open_url", { url: REPO_URL }).catch(() => {}); }
function openReleases() { invoke("open_url", { url: store.update.url }).catch(() => {}); }
async function manualCheck() {
  const has = await checkUpdate(true);
  if (has) toast("标题栏那颗 NEW 就是入口", "ok");
}
</script>

<style scoped>
.about {
  display: flex; align-items: center; gap: 9px;
  margin-top: 4px; padding-top: 14px;
  border-top: 1px solid var(--glass-border);
}
.about .ver { font-size: 12.5px; color: var(--dim); letter-spacing: .3px; }
.about .sep { color: var(--dim); opacity: .5; font-size: 11px; }
.lnk, .upd {
  border: none; background: none; padding: 0; cursor: pointer;
  font-size: 12.5px; letter-spacing: .3px;
  color: var(--dim);
  transition: color .15s;
}
.lnk:hover { color: var(--accent); }
.upd {
  margin-left: auto;
  color: var(--accent); font-weight: 600;
}
.upd:hover { text-decoration: underline; }
</style>
