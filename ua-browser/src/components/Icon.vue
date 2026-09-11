<script setup lang="ts">
defineProps<{
  pathData: string;
  title?: string;
}>();
</script>

<template>
  <span
    v-bind="$attrs"
    :aria-hidden="title ? undefined : 'true'"
    :aria-label="title"
    class="ua-icon"
    role="img"
    @click="$emit('click', $event)"
  >
    <svg
      class="ua-icon__svg"
      viewBox="0 0 24 24"
    >
      <path :d="pathData">
        <title v-if="title">{{ title }}</title>
      </path>
    </svg>
  </span>
</template>

<style scoped>
.ua-icon {
  display: inline-flex;
}

.ua-icon__svg {
  width: 1em;
  height: 1em;
  fill: currentColor;
}
</style>
