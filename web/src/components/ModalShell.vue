<template>
  <div class="overlay" @click.self="$emit('close')">
    <div class="modal" :class="{ tall }">
      <div class="modal-h">
        <div>
          <div class="modal-t">{{ title }}</div>
          <div class="modal-s ellipsis">{{ subtitle }}</div>
        </div>
        <button class="win-btn close" title="关掉" @click="$emit('close')">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round"><path d="M6 6l12 12M18 6L6 18"/></svg>
        </button>
      </div>
      <slot />
    </div>
  </div>
</template>

<script setup>
defineProps({
  title: String,
  subtitle: String,
  tall: Boolean,
});
defineEmits(["close"]);
</script>
