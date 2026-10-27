<template>
  <ModalShell :title="song ? song.title : '歌词'" :subtitle="song ? song.author || '' : ''" class="lyr-shell" @close="close">
    <LyricList :lines="store.lrcLines" :cur="store.lyrIdx" :loading="store.lrcLoading" class="lyr-body" />
  </ModalShell>
</template>

<script setup>
/* 完整歌词弹层（小窗用；大窗的歌词常驻右栏） */
import ModalShell from "./ModalShell.vue";
import LyricList from "./LyricList.vue";
import { store, closeModal } from "../lib/store.js";

defineProps({ song: Object });
function close() { closeModal("lyr"); }
</script>

<style scoped>
.lyr-shell :deep(.modal) { height: 500px; max-height: 88vh; }
.lyr-shell :deep(.modal-b) { flex: 1; min-height: 0; display: flex; }
.lyr-shell :deep(.lyr-body) { flex: 1; }
</style>
