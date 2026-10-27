<template>
  <div class="src-line" :class="{ off: !store.status }">
    <span v-if="store.status" class="status" :class="store.status.kind">{{ store.status.text }}</span>
  </div>
</template>

<script setup>
import { store } from "../lib/store.js";
</script>
