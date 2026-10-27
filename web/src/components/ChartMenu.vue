<template>
  <div ref="root" class="chart-menu" :class="variant">
    <!-- 大窗：标题栏右侧胶囊，直接显示当前榜单名 -->
    <button
      v-if="variant === 'chip'"
      class="chart-chip"
      :class="{ open }"
      data-tauri-drag-region-exclude
      :title="'正在听「' + curName + '」，点我换一个'"
      @click="toggle"
    >
      <svg class="ck-bar" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
        <path d="M4 20V11M9.3 20V5M14.7 20v-6M20 20V8" />
      </svg>
      <span class="ellipsis">{{ curName }}</span>
      <svg class="ck-caret" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M6 9.5 12 15.5 18 9.5" />
      </svg>
    </button>

    <!-- 小窗：标题栏右侧同规格圆形按钮（宽度不够放胶囊） -->
    <button
      v-else
      class="win-btn chart-btn"
      :class="{ open }"
      data-tauri-drag-region-exclude
      :title="'正在听「' + curName + '」，点我换一个'"
      @click="toggle"
    >
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round">
        <path d="M4 20V11M9.3 20V5M14.7 20v-6M20 20V8" />
      </svg>
    </button>

    <Transition name="pop">
      <div v-if="open" class="chart-pop">
        <div class="cp-head">换一个榜单听听</div>
        <button
          v-for="c in CATS"
          :key="c.id"
          class="cp-row"
          :class="{ on: c.id === store.catId }"
          @click="pick(c.id)"
        >
          <span class="ellipsis">{{ c.name }}</span>
          <svg
            v-if="c.id === store.catId"
            class="cp-tick"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.2"
            stroke-linecap="round"
            stroke-linejoin="round"
          ><path d="M4.5 12.5 9.5 17.5 19.5 6.5" /></svg>
        </button>
      </div>
    </Transition>
  </div>
</template>

<script setup>
/* 榜单切换：全应用只有这一个入口，固定在标题栏右上角。
   chip = 大窗（显示当前榜单名），icon = 小窗（同规格圆形按钮，宽度不够）。
   两者共用 setCat / store.catId，弹层右对齐向下展开。 */
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { CATS } from "../lib/constants.js";
import { store } from "../lib/store.js";
import { setCat } from "../lib/player.js";

defineProps({
  variant: { type: String, default: "chip" },
});

const open = ref(false);
const root = ref(null);

const curName = computed(() => {
  const c = CATS.find((x) => x.id === store.catId);
  return c ? c.name : "榜单";
});

function toggle() { open.value = !open.value; }
function pick(id) {
  open.value = false;
  setCat(id);
}

/* 点外面关闭 */
function onDocDown(e) {
  if (open.value && root.value && !root.value.contains(e.target)) open.value = false;
}
onMounted(() => document.addEventListener("mousedown", onDocDown));
onBeforeUnmount(() => document.removeEventListener("mousedown", onDocDown));
</script>

<style scoped>
.chart-menu { position: relative; display: flex; }
.chart-btn.open { color: var(--accent); border-color: var(--accent-soft); background: var(--accent-soft); }

/* 胶囊态 */
.chart-chip {
  display: inline-flex; align-items: center; gap: 5px;
  max-width: 142px;
  height: 28px; padding: 0 8px 0 9px;
  border-radius: 14px;
  border: 1px solid var(--glass-border);
  background: var(--glass);
  backdrop-filter: var(--blur);
  -webkit-backdrop-filter: var(--blur);
  box-shadow: var(--glass-inset);
  color: var(--ink);
  font-family: inherit;
  font-size: 11.5px;
  cursor: pointer;
  transition: color .15s, border-color .15s, background .15s;
}
.chart-chip:hover, .chart-chip.open { color: var(--accent); border-color: var(--accent-soft); background: var(--accent-soft); }
.ck-bar { width: 12px; height: 12px; flex: 0 0 12px; color: var(--accent); }
/* 展开指示：很小的一个下箭头，展开时翻转 */
.ck-caret {
  width: 9px; height: 9px; flex: 0 0 9px;
  color: var(--dim);
  transition: transform .18s ease, color .15s;
}
.chart-chip.open .ck-caret { transform: rotate(180deg); color: var(--accent); }

.chart-pop {
  position: absolute;
  top: calc(100% + 8px);
  right: -2px;
  width: 156px;
  padding: 6px;
  border-radius: 14px;
  border: 1px solid var(--glass-border);
  background: var(--glass);
  backdrop-filter: blur(40px) saturate(170%);
  -webkit-backdrop-filter: blur(40px) saturate(170%);
  box-shadow: var(--glass-inset), 0 14px 34px var(--shadow-strong);
  z-index: 40;
}
.cp-head {
  font-size: 10.5px;
  color: var(--dim);
  letter-spacing: .4px;
  padding: 3px 8px 5px;
}
.cp-row {
  width: 100%;
  height: 30px;
  padding: 0 8px;
  display: flex; align-items: center; justify-content: space-between;
  gap: 6px;
  border: none; background: none;
  border-radius: 9px;
  color: var(--ink);
  font-size: 12.5px;
  font-family: inherit;
  cursor: pointer;
  text-align: left;
  transition: background .15s, color .15s;
}
.cp-row:hover { background: var(--row-hover); }
.cp-row.on { color: var(--accent); font-weight: 600; }
.cp-tick { width: 13px; height: 13px; flex: 0 0 13px; }

.pop-enter-active, .pop-leave-active { transition: opacity .14s ease, transform .14s ease; }
.pop-enter-from, .pop-leave-to { opacity: 0; transform: translateY(-5px); }
</style>
