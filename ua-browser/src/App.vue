<script setup lang="ts">
import AddressBar from "./components/AddressBar.vue";
import NodeTreeView from "./components/NodeTreeView.vue";
</script>

<template>
  <div class="bg-background text-content flex h-screen w-full flex-col">
    <AddressBar />
    <NodeTreeView />
  </div>
</template>
