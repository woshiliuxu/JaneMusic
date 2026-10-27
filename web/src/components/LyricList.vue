<template>
  <div class="lyr-host">
    <div
      ref="list"
      class="lyr-list"
      :class="{ dense, left: align === 'left', single: lines.length === 1 }"
    >
      <div v-if="!lines.length" class="empty-tip">{{ loading ? "歌词正在飞奔来的路上~" : empty }}</div>
      <div
        v-for="(l, i) in lines"
        :key="i"
        class="lyr-line"
        :class="{ cur: i === cur }"
        @click="seek(l)"
      >
        <span class="lyr-main">{{ l.txt }}</span>
        <span v-if="l.sub" class="lyr-sub">{{ l.sub }}</span>
      </div>
    </div>
  </div>
</template>

<script setup>
/* 歌词列表：当前句高亮 + 平滑跟随滚动；点任意一句跳到那句。滚动条完全隐藏。 */
import { ref, watch, nextTick } from "vue";
import { audio } from "../lib/store.js";

const props = defineProps({
  lines: { type: Array, default: () => [] },
  cur: { type: Number, default: -1 },
  empty: { type: String, default: "这首歌没留词，就听旋律吧" },
  /* 紧凑排版（大窗右栏是左对齐，弹层居中） */
  align: { type: String, default: "center" },
  /* 歌词还在拉：空列表时显示「正在加载」 */
  loading: Boolean,
  dense: Boolean,
});

const list = ref(null);

function seek(l) {
  if (!audio.duration) return;
  /* 钳在 0 ~ 结尾前 0.3s：时间戳异常时最多跳到曲尾附近，绝不触发 ended 连播 */
  const target = Math.min(Math.max(l.t + 0.05, 0), audio.duration - 0.3);
  if (!(target >= 0)) return;
  audio.currentTime = target;
  if (audio.paused) audio.play().catch(() => {});
}

watch(
  () => props.cur,
  async () => {
    await nextTick();
    const host = list.value;
    if (!host) return;
    const el = host.querySelector(".lyr-line.cur");
    if (!el) return;
    /* 当前句停靠在「中心偏上一行」= 上一句原本的位置，下方多露一句后续歌词。
       必须每句按绝对坐标重算（居中点 - 一行），不能用相邻行 rect 差值累加：
       cur 行有 +2px 字号过渡，长句折行数会变（行高 72↔102），差值法每句多滚
       一个膨胀量，越滚越偏。 */
    const prev = el.previousElementSibling;
    const rowH = prev ? prev.getBoundingClientRect().height : el.getBoundingClientRect().height;
    const hostRect = host.getBoundingClientRect();
    const elRect = el.getBoundingClientRect();
    const target = host.scrollTop
      + (elRect.top + elRect.height / 2)
      - (hostRect.top + hostRect.height / 2)
      + rowH;
    host.scrollTo({ top: target, behavior: "smooth" });
  }
);

watch(
  () => props.lines,
  async () => {
    await nextTick();
    if (list.value) list.value.scrollTop = 0;
  }
);
</script>

<style scoped>
.lyr-host {
  position: relative;
  display: flex; flex-direction: column;
  min-height: 0;
  min-width: 0;
}
.lyr-list.left { text-align: left; }
/* 左对齐布局（大窗）：加载/无歌词提示跟歌词文字左缘走，别在容器里居中浮到右边 */
.lyr-list.left .empty-tip { justify-content: flex-start; padding-left: 6px; }
/* 只有一句歌词时整句水平垂直居中 */
.lyr-list.single { text-align: center; }
</style>
