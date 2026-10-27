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

      <!-- 关于：版本号 / 检查更新 / 仓库地址（放进 .modal-b，内距与字段对齐） -->
      <div class="about">
        <span class="ver">v{{ APP_VERSION }}</span>
        <span class="ab-gap"></span>
        <button v-if="!store.update" class="lnk" data-tauri-drag-region-exclude @click="manualCheck">检查更新</button>
        <button
          v-else
          class="upd"
          data-tauri-drag-region-exclude
          :title="store.updating ? `正在下载 ${store.updPct || 0}%，好了自动重启` : `点这自动更新到 ${store.update.latest}`"
          @click="applyUpdate"
        >{{ store.updating ? `下载中 ${store.updPct || 0}%` : `NEW ${store.update.latest} →` }}</button>
        <button class="lnk" data-tauri-drag-region-exclude @click="openRepo">GitHub</button>
      </div>
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
import { checkUpdate, applyUpdate } from "../lib/update.js";
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
async function manualCheck() {
  const has = await checkUpdate(true);
  if (has) toast("标题栏那颗 NEW 就是入口", "ok");
}
</script>

<style scoped>
/* 关于行：与上方字段同行距，横内距交给 .modal-b，左右自然对齐 */
.about {
  display: flex; align-items: center; gap: 16px;
  padding: 12px 0 7px;
  border-top: 1px solid var(--glass-border);
}
.ver { font-size: 12px; line-height: 17px; color: var(--dim); letter-spacing: .3px; }
.ab-gap { flex: 1; }
.lnk, .upd {
  flex: none;
  border: none; background: none; padding: 0;
  cursor: pointer;
  font-size: 12.5px; line-height: 17px; letter-spacing: .3px;
  white-space: nowrap;
  transition: color .15s;
}
.lnk { color: var(--dim); }
.lnk:hover { color: var(--accent); }
/* 有新版本时：主色文字按钮 */
.upd { color: var(--accent); font-weight: 600; }
.upd:hover { text-decoration: underline; }
</style>
