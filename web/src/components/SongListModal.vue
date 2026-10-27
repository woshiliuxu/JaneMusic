<template>
  <ModalShell :title="title" :subtitle="subtitle" tall @close="$emit('close')">
    <div class="row-list">
      <div v-if="!songs.length" class="empty-tip">{{ empty }}</div>
      <div
        v-for="(s, i) in songs"
        :key="s.url + i"
        class="list-item"
        :class="{ now: store.current && s.url === store.current.url }"
        @click="$emit('play', s)"
      >
        <div class="idx">{{ store.current && s.url === store.current.url ? "♪" : i + 1 }}</div>
        <div class="ht">
          <b class="ellipsis">{{ s.title }}</b>
          <span class="ellipsis">{{ s.author || "" }}</span>
        </div>
      </div>
    </div>
    <div v-if="clearable && songs.length" class="modal-f center">
      <button class="link" @click="$emit('clear')">{{ clearText }}</button>
    </div>
  </ModalShell>
</template>

<script setup>
/* 歌曲列表弹层：历史 / 收藏 / 播放列表共用一个壳 */
import ModalShell from "./ModalShell.vue";
import { store } from "../lib/store.js";

defineProps({
  title: String,
  subtitle: { type: String, default: "" },
  songs: { type: Array, default: () => [] },
  empty: { type: String, default: "这里空空的，什么都没有" },
  clearable: Boolean,
  clearText: { type: String, default: "清空" },
});
defineEmits(["close", "play", "clear"]);
</script>
