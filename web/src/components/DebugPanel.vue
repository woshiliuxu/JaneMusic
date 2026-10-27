<template>
  <div v-if="store.dbgOn" class="dbg-panel">{{ store.dbgLines.join("\n") }}</div>
</template>

<script setup>
import { store } from "../lib/store.js";
</script>
