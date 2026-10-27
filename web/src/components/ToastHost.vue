<template>
  <div class="toasts">
    <TransitionGroup name="tst">
      <div v-for="t in store.toasts" :key="t.id" class="toast" :class="t.kind">
        <span class="t-dot"></span>
        <span class="t-txt">{{ t.text }}</span>
      </div>
    </TransitionGroup>
  </div>
</template>

<script setup>
import { store } from "../lib/store.js";
</script>
