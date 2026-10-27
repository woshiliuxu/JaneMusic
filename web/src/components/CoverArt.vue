<template>
  <div class="cover" :class="[variant, { playing: store.playing, spin: store.playing && active }]">
    <div class="cover-glow"></div>

    <!-- 黑胶碟：整张碟（含唱片标）一起转，暂停即停在当前角度 -->
    <div v-if="variant === 'vinyl'" class="disc">
      <img v-if="shown" :src="shownSrc" class="disc-art" alt="" draggable="false" />
      <div v-else class="cover-ph">
        <svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 2a10 10 0 1 0 10 10A10 10 0 0 0 12 2zm0 14.5A4.5 4.5 0 1 1 16.5 12 4.5 4.5 0 0 1 12 16.5zm0-6a1.5 1.5 0 1 0 1.5 1.5A1.5 1.5 0 0 0 12 10.5z"/></svg>
      </div>
    </div>

    <template v-else>
      <img v-if="shown" :src="shownSrc" class="cover-img" alt="" draggable="false" />
      <div v-else class="cover-ph">
        <svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 2a10 10 0 1 0 10 10A10 10 0 0 0 12 2zm0 14.5A4.5 4.5 0 1 1 16.5 12 4.5 4.5 0 0 1 12 16.5zm0-6a1.5 1.5 0 1 0 1.5 1.5A1.5 1.5 0 0 0 12 10.5z"/></svg>
      </div>
    </template>

    <div v-if="spinning" class="cover-loading"></div>
  </div>
</template>

<script setup>
/* 封面：等新图解码完再上屏（<img> 直接换 src 会一直显示上一张），
   want 序号防乱序回填；解码成功顺手取主色给背景氛围光。 */
import { computed, ref, watch, onMounted, onBeforeUnmount } from "vue";
import { store } from "../lib/store.js";
import { proxied } from "../lib/tauri.js";
import { pickTint } from "../lib/tint.js";

const props = defineProps({
  /* 展示哪首歌的封面（= store.uiSong，跟随动效节奏） */
  song: Object,
  /* vinyl = 黑胶碟（播放时旋转）；plain = 直接用原图 */
  variant: { type: String, default: "vinyl" },
  /* 音频还在缓冲时强行转圈（原 showCoverLoading） */
  forceLoading: Boolean,
});

const shown = ref(false);
const shownSrc = ref("");
const decoding = ref(false);
let want = "";

const spinning = computed(() => decoding.value || (props.forceLoading && shown.value));

/* 只在窗口真正看得见时转：转盘旋转在这台机器上要走软件合成，实测吃掉约 0.7 个核
   （实测：播放 161% → 转盘停转 71%）。窗口失焦或被最小化时没人看，白烧 CPU 和电，
   所以失焦即停在当前角度，切回来接着转（animation-play-state 切回 running 会无缝续上）。 */
const active = ref(true);
function syncActive() {
  active.value = document.visibilityState === "visible" && document.hasFocus();
}
function onVis() {
  syncActive();
}
onMounted(() => {
  syncActive();
  window.addEventListener("focus", syncActive);
  window.addEventListener("blur", syncActive);
  document.addEventListener("visibilitychange", onVis);
});
onBeforeUnmount(() => {
  window.removeEventListener("focus", syncActive);
  window.removeEventListener("blur", syncActive);
  document.removeEventListener("visibilitychange", onVis);
});

watch(
  () => props.song && props.song.pic,
  (pic) => {
    if (!pic) {
      want = "";
      shown.value = false;
      shownSrc.value = "";
      decoding.value = false;
      return;
    }
    const src = proxied(pic);
    if (shownSrc.value === src && shown.value) {
      decoding.value = false;
      return;
    }
    want = src;
    const im = new Image();
    im.crossOrigin = "anonymous";
    im.onload = () => {
      if (want !== src) return;
      shownSrc.value = src;
      shown.value = true;
      decoding.value = false;
      pickTint(im, store.theme);
    };
    im.onerror = () => {
      if (want !== src) return;
      shown.value = false;
      shownSrc.value = "";
      decoding.value = false;
    };
    decoding.value = true;
    im.src = src;
  },
  { immediate: true }
);
</script>

<style scoped>
.cover { position: relative; display: grid; place-items: center; }
.cover .cover-glow {
  position: absolute; inset: -22px;
  border-radius: 50%;
  background: radial-gradient(circle, var(--accent-soft) 30%, transparent 68%);
  opacity: 0;
  transition: opacity .3s;
  pointer-events: none;
}
.cover.playing .cover-glow { opacity: 1; }
.cover img {
  width: 100%; height: 100%;
  object-fit: cover;
  -webkit-user-drag: none;
}

/* ---------------- 黑胶碟 ---------------- */
.cover.vinyl { border-radius: 50%; }
.disc {
  position: relative;
  width: 100%; height: 100%;
  border-radius: 50%;
  display: grid; place-items: center;
  /* 盘体不是死黑：左上受光、往右下压暗，胶感才出得来 */
  background: radial-gradient(circle at 36% 30%,
    #3b3d46 0%, #26272e 26%, #191a21 52%, #101117 78%, #0c0d12 100%);
  box-shadow:
    0 16px 38px var(--shadow),
    inset 0 1px 1px rgba(255, 255, 255, .10),  /* 上缘受光 */
    inset 0 0 16px rgba(0, 0, 0, .6);          /* 边缘暗角收边 */
  animation: vinylSpin 26s linear infinite;
  animation-play-state: paused;
  will-change: transform;
}
/* 转 = 正在播放 且 窗口可见并聚焦（失焦停在当前角度，省掉软件合成的开销） */
.cover.spin .disc { animation-play-state: running; }
/* 纹路：细密沟槽 + 几条稍宽的音轨分隔环；唱片标以内不画（mask 挖空） */
.disc::before {
  content: "";
  position: absolute; inset: 0;
  border-radius: 50%;
  background:
    repeating-radial-gradient(circle at 50% 50%,
      rgba(0, 0, 0, 0) 0 30px,
      rgba(0, 0, 0, .38) 30px 31.5px),
    repeating-radial-gradient(circle at 50% 50%,
      rgba(255, 255, 255, .05) 0 1px,
      rgba(255, 255, 255, 0) 1px 3.2px);
  -webkit-mask: radial-gradient(circle at 50% 50%, transparent 0 37.5%, #000 38%);
  mask: radial-gradient(circle at 50% 50%, transparent 0 37.5%, #000 38%);
  pointer-events: none;
}
/* 光：斜向两道反光扫过胶面 + 外圈一线轮廓光 */
.disc::after {
  content: "";
  position: absolute; inset: 0;
  border-radius: 50%;
  background:
    conic-gradient(from 208deg,
      transparent 0 9%,
      rgba(255, 255, 255, .10) 17%,
      transparent 27%,
      transparent 55%,
      rgba(255, 255, 255, .055) 67%,
      transparent 79%),
    radial-gradient(circle at 50% 50%,
      transparent 0 91%,
      rgba(255, 255, 255, .06) 96%,
      rgba(0, 0, 0, .35) 100%);
  pointer-events: none;
}
.cover .disc-art {
  position: relative; z-index: 1;
  width: 74%; height: 74%;
  border-radius: 50%;
  object-fit: cover;
  /* 唱片标压在胶面上：一圈窄黑边 + 轻微内凹感 */
  box-shadow:
    0 0 0 2px rgba(8, 9, 13, .9),
    0 0 0 3px rgba(255, 255, 255, .06),
    0 4px 14px rgba(0, 0, 0, .45);
  -webkit-user-drag: none;
}
@keyframes vinylSpin { to { transform: rotate(360deg); } }

.cover-ph {
  width: 38%; height: 38%;
  color: var(--dim);
  opacity: .45;
  z-index: 1;
}
.cover-ph svg { width: 100%; height: 100%; }
.cover-loading {
  position: absolute; inset: 0;
  border-radius: inherit;
  display: grid; place-items: center;
}
.cover-loading::before {
  content: "";
  width: 30px; height: 30px;
  border-radius: 50%;
  border: 3px solid var(--scrollbar);
  border-top-color: var(--accent);
  animation: spin .7s linear infinite;
}
@keyframes spin { to { transform: rotate(360deg); } }
</style>
