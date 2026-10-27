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
  </ModalShell>
</template>

<script setup>
import { computed } from "vue";
import ModalShell from "./ModalShell.vue";
import { store, kv, kvSave, toast, closeModal } from "../lib/store.js";
import { applyVol } from "../lib/volume.js";
import { applyLyrSize } from "../lib/ui.js";

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
</script>
