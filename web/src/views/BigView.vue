<template>
  <div class="big-wrap">
    <TitleBar thin @settings="openModal('settings')" @theme="toggleTheme" @pin="togglePin" />

    <div class="big-main">
      <div class="big-left">
        <div ref="coverZone" class="big-cover-zone" @wheel="wheelSwitch" data-tauri-drag-region-exclude>
          <CoverArt :song="store.uiSong" variant="vinyl" :force-loading="store.coverLoading" />
        </div>
      </div>
      <div class="big-right">
        <div ref="metaEl" class="big-meta">
          <div class="big-name ellipsis" :title="uiTitle">{{ uiTitle }}</div>
          <div class="big-artist ellipsis" :title="uiArtist">{{ uiArtist }}</div>
        </div>
        <LyricList
          class="big-lyr"
          :lines="store.lrcLines"
          :cur="store.lyrIdx"
          align="left"
          :loading="store.lrcLoading"
          dense
          empty="这首歌没留词，就听旋律吧"
        />
      </div>
    </div>

    <div class="big-foot">
      <ProgressBar class="big-seek" />
      <div class="big-ctrl">
        <div class="g-left foot-meta">
          <div class="fm-txt">
            <div class="fm-name ellipsis" :title="uiTitle">{{ uiTitle }}</div>
            <div class="fm-artist ellipsis" :title="uiArtist">{{ uiArtist }}</div>
          </div>
          <button class="icon-btn bare fm-fav" :class="{ faved: fav }" :title="fav ? '从抽屉里拿出来' : '收进小抽屉'" @click="toggleFav(store.uiSong)">
            <svg viewBox="0 0 24 24" :fill="fav ? 'currentColor' : 'none'" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round"><path d="M12 20.5S3.5 15.4 3.5 9.6A4.6 4.6 0 0 1 12 6.9a4.6 4.6 0 0 1 8.5 2.7c0 5.8-8.5 10.9-8.5 10.9z"/></svg>
          </button>
        </div>
        <Transport gap="18px" />
        <div class="g-right">
          <button class="icon-btn bare" title="我的小抽屉" @click="openModal('favs')">
            <!-- 书签（收藏标记）+ 音符（音乐）合成：书签压淡做底，音符走实线 -->
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round">
              <!-- 书签外框 3.2..20.8 / 音符 5.5..16：与历史/列表图标的 18/24 视觉框对齐 -->
              <path d="M5.2 3.3H18.8A2 2 0 0 1 20.8 5.3V19.3A1.25 1.25 0 0 1 18.75 20.2L12 15.3L5.25 20.2A1.25 1.25 0 0 1 3.2 19.3V5.3A2 2 0 0 1 5.2 3.3Z" opacity=".45" />
              <g stroke-width="2.1">
                <ellipse cx="10.4" cy="14" rx="2.6" ry="2.05" transform="rotate(-20 10.4 14)" />
                <path d="M12.95 13.6V5.5" />
                <path d="M12.95 5.5c2 .45 3.2 1.6 3.1 3.3" />
              </g>
            </svg>
          </button>
          <button class="icon-btn bare" title="走过的路" @click="openModal('hist')">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round"><path d="M3 12a9 9 0 1 0 3-6.7L3 8"/><path d="M3 3v5h5M12 7v5l3.5 2"/></svg>
          </button>
          <button class="icon-btn bare" title="当前榜单全曲" @click="openModal('list')">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round"><path d="M3 6h13M3 12h13M3 18h9"/><path d="M19 8v8M16 13.5 21 16"/></svg>
          </button>
          <VolumeControl />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed, ref, watch } from "vue";
import TitleBar from "../components/TitleBar.vue";
import CoverArt from "../components/CoverArt.vue";
import LyricList from "../components/LyricList.vue";
import ProgressBar from "../components/ProgressBar.vue";
import Transport from "../components/Transport.vue";
import VolumeControl from "../components/VolumeControl.vue";
import { store, openModal } from "../lib/store.js";
import { wheelSwitch, toggleFav, isFav } from "../lib/player.js";
import { toggleTheme, togglePin } from "../lib/ui.js";

const uiTitle = computed(() => (store.uiSong ? store.uiSong.title : "Jane Music"));
const uiArtist = computed(() => (store.uiSong ? store.uiSong.author || "" : ""));
const fav = computed(() => isFav(store.uiSong));

/* 切歌动效：封面 + 歌名一組一起滑（WAAPI，同 3.0 的四关键帧） */
const coverZone = ref(null);
const metaEl = ref(null);
watch(
  () => store.uiVer,
  () => {
    if (!store.switchAnim || !store.motionFrame) return;
    const { oX, oY, iX, iY } = store.motionFrame;
    [coverZone.value, metaEl.value].forEach((n) => {
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
.big-wrap { height: 100%; display: flex; flex-direction: column; }

.big-main {
  flex: 1; min-height: 0;
  display: flex;
  /* 视觉重心右移：封面是重色块、歌词文本短而疏，纯几何居中会显得整体偏左、右侧发空 */
  padding-left: 48px;
}
.big-left {
  width: 340px; flex: none;
  padding: 0 0 0 30px;
  display: flex; flex-direction: column; justify-content: center;
}
/* 唱片往里挪一点（居中基础上整体下移 ~15px），别和右上歌名抢视觉重心 */
.big-cover-zone { width: 300px; height: 300px; margin-top: 30px; position: relative; }
.big-cover-zone :deep(.cover) { width: 100%; height: 100%; }

.big-right {
  flex: 1; min-width: 0;
  display: flex; flex-direction: column;
/* 右内距 24 与标题栏/底栏同一右缘线；左内距加大，把歌词和唱片拉开距离 */
  padding: 2px 24px 0 48px;
}
/* 歌名块：左缘与歌词文字左缘（列表内距 2 + 行内距 6）对齐 */
.big-meta { padding-left: 8px; margin-bottom: 16px; }
.big-name { font-size: 21px; font-weight: 600; letter-spacing: .2px; }
.big-artist { margin-top: 3px; font-size: 13px; color: var(--muted); }

.big-lyr {
  flex: 1; min-height: 0;
  /* 大窗歌词比小窗/弹层再大 2px，仍受设置里的字号控制 */
  --lyr-size: calc(var(--lyr-base, 15px) + 2px);
}
/* 歌词文字左缘 = 2(容器) + 6(行内距) = 8，与歌名块 padding-left 8 对齐。
   上下各留 40px 渐隐：滚动容器高度不是行高的整数倍，首尾行必然各露半截，
   硬边界看着像"被遮住"，改成渐隐后是自然的淡出。 */
.big-lyr :deep(.lyr-list) {
  padding: 0 2px 12px;
  -webkit-mask-image: linear-gradient(180deg, transparent 0, #000 40px, #000 calc(100% - 46px), transparent 100%);
  mask-image: linear-gradient(180deg, transparent 0, #000 40px, #000 calc(100% - 46px), transparent 100%);
}

.big-foot {
  flex: 0 0 auto;
  /* 左右各 24：与标题栏 logo / 窗口按钮同一对边线，两侧对称 */
  padding: 16px 24px 14px 24px;
}
.big-ctrl {
  /* 三列 grid：中列（播放三键）锚定窗口正中，左右列贴边 */
  display: grid;
  grid-template-columns: 1fr auto 1fr;
  align-items: center;
  margin-top: 10px;
}
.big-ctrl > .g-left { justify-self: start; }
/* 底栏左侧：歌名/歌手两行 + 收藏心形（裸图标，与右侧图标组同尺寸） */
.foot-meta {
  min-width: 0; max-width: 300px;
  display: flex; align-items: center; gap: 4px;
}
.fm-txt { min-width: 0; line-height: 1.3; }
.fm-name { font-size: 13px; font-weight: 600; letter-spacing: .2px; }
.fm-artist { font-size: 11.5px; color: var(--dim); margin-top: 1px; }
.fm-fav { width: 36px; height: 36px; flex: 0 0 36px; }
.big-ctrl > .g-right {
  justify-self: end;
  display: flex; align-items: center; gap: 12px;
  /* 裸图标无可见边框，图标缩在 36px 命中区里距盒边 9px；
     负外距把可见图形推到 846 线，与右时间数字、标题栏关闭钮圆圈同一条右缘线 */
  margin-right: -9px;
}
</style>
