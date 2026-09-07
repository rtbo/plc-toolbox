<script setup lang="ts">
import AddressBar from './components/AddressBar.vue';

function handleBrowse(url: string) {
  console.log('Browsing to:', url);
}
</script>

<template>
  <AddressBar @browse="handleBrowse" />
</template>
