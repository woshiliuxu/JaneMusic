<template>
  <div class="small-wrap">
    <TitleBar @settings="openModal('settings')" @theme="toggleTheme" @pin="togglePin" />

    <div class="player">
      <div ref="coverWrap" class="cover-wrap" @click="openModal('lyr')" @wheel="wheelSwitch">
        <CoverArt :song="store.uiSong" variant="vinyl" :force-loading="store.coverLoading" />
      </div>
      <div ref="songInfo" class="song-info">
        <div class="song-name ellipsis" :title="uiTitle">{{ uiTitle }}</div>
        <div class="song-artist ellipsis" :title="uiArtist">{{ uiArtist }}</div>
        <div class="lyric-line ellipsis" title="点开看整首词" @click="openModal('lyr')">{{ store.curLyric || " " }}</div>
      </div>

      <ProgressBar class="small-seek" />

      <div class="ctrl-row">
        <div class="side">
          <button class="icon-btn sm" title="走过的路" @click="openModal('hist')">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round"><path d="M3 12a9 9 0 1 0 3-6.7L3 8"/><path d="M3 3v5h5M12 7v5l3.5 2"/></svg>
          </button>
        </div>
        <Transport class="center-btns" gap="22px" />
        <div class="side right">
          <VolumeControl btn-cls="sm" />
        </div>
      </div>

      <StatusLine />
    </div>
  </div>
</template>

<script setup>
/* 小窗 380×600：与 3.0 相同的竖版布局（圆形羽化封面 + 单行歌词） */
import { computed, ref, watch } from "vue";
import TitleBar from "../components/TitleBar.vue";
import CoverArt from "../components/CoverArt.vue";
import ProgressBar from "../components/ProgressBar.vue";
import Transport from "../components/Transport.vue";
import VolumeControl from "../components/VolumeControl.vue";
import StatusLine from "../components/StatusLine.vue";
import { store, openModal } from "../lib/store.js";
import { wheelSwitch } from "../lib/player.js";
import { toggleTheme, togglePin } from "../lib/ui.js";

const uiTitle = computed(() => (store.uiSong ? store.uiSong.title : "Jane Music"));
const uiArtist = computed(() => (store.uiSong ? store.uiSong.author || "" : ""));

/* 切歌动效：封面 + 歌名一起滑（WAAPI，同 3.0） */
const coverWrap = ref(null);
const songInfo = ref(null);
watch(
  () => store.uiVer,
  () => {
    if (!store.switchAnim || !store.motionFrame) return;
    const { oX, oY, iX, iY } = store.motionFrame;
    [coverWrap.value, songInfo.value].forEach((n) => {
      if (!n) return;
      n.getAnimations().forEach((a) => a.cancel());
      n.animate([
        { opacity: 1, transform: "translate3d(0,0,0)", easing: "cubic-bezier(.4,0,.8,1)" },
        { opacity: 0, transform: `translate3d(${oX}px,${oY}px,0)`, offset: .4, easing: "linear" },
        { opacity: 0, transform: `translate3d(${iX}px,${iY}px,0)`, offset: .42, easing: "cubic-bezier(.2,.8,.3,1)" },
        { opacity: 1, transform: "translate3d(0,0,0)", offset: 1 },
      ], { duration: 380 });
    });
  }
);
</script>

<style scoped>
.small-wrap { height: 100%; display: flex; flex-direction: column; }
.player {
  flex: 1; min-height: 0;
  display: flex; flex-direction: column;
  /* 左右各 24：与标题栏 logo、窗口按钮的边线一致，两侧对称 */
  padding: 2px 24px 16px 24px;
}
.cover-wrap {
  flex: 1; min-height: 0;
  display: grid; place-items: center;
  cursor: pointer;
}
.cover-wrap :deep(.cover) {
  width: min(45vh, 288px);
  height: min(45vh, 288px);
}
.song-info {
  text-align: center;
  display: flex; flex-direction: column; gap: 1px;
  min-height: 58px;
}
.song-name { font-size: 16.5px; font-weight: 600; letter-spacing: .2px; }
.song-artist { font-size: 12px; color: var(--muted); }
.lyric-line {
  font-size: 11.5px; color: var(--dim);
  min-height: 17px; margin-top: 2px;
  transition: opacity .2s;
  cursor: pointer;
}
.lyric-line:hover { color: var(--muted); }
.small-seek { padding-top: 14px; }
.ctrl-row {
  display: flex; align-items: center;
  gap: 10px;
  padding: 8px 0 2px;
}
.side { flex: 1; display: flex; justify-content: flex-start; }
.side.right {
  justify-content: flex-end;
  /* 同大窗：把音量图标的可见图形推到与右时间数字同一条右缘线（sm 按钮内缩 8px） */
  margin-right: -8px;
}
.center-btns { display: flex; align-items: center; justify-content: center; }
</style>
