<script setup lang="ts">
import AddressBar from "./components/AddressBar.vue";
import AttributeTable from "./components/AttributeTable.vue";
import NodeTreeView from "./components/NodeTreeView.vue";
</script>

<template>
  <div
    class="bg-background text-content flex h-screen w-full flex-col overflow-hidden"
  >
    <header class="w-full shrink-0">
      <AddressBar />
    </header>
    <div class="flex min-h-0 flex-1 overflow-hidden">
      <section class="min-h-0 w-1/2 overflow-auto">
        <NodeTreeView />
      </section>
      <section class="min-h-0 w-1/2 overflow-auto">
        <AttributeTable />
      </section>
    </div>
  </div>
</template>
