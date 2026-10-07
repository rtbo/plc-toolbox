<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useConnectionStore } from "../stores/connection";
import { useNodeTreeStore } from "../stores/node-tree";
import type { ReferenceDescription } from "../ua/types";
import { mdiTriangleDown } from "@mdi/js";
import Icon from "./Icon.vue";

const props = defineProps<{
  node: ReferenceDescription;
}>();

const connectionStore = useConnectionStore();
const nodeTreeStore = useNodeTreeStore();

const children = ref<ReferenceDescription[]>([]);

async function browse() {
  if (!connectionStore.client) {
    console.warn(`No OPC/UA client available. Cannot browse nodes.`);
    children.value = [];
    return;
  }
  if (!props.node?.NodeId) {
    console.warn(`No NodeId provided. Cannot browse nodes.`);
    children.value = [];
    return;
  }
  try {
    children.value = await connectionStore.client.browseNode(props.node.NodeId);
  } catch (error) {
    console.error(`Browse request failed:`, error);
  }
}

const expanded = ref(false);
const toggleBtnClass = computed(() => {
  if (children.value.length === 0) {
    return "pointer-events-none";
  }
  return "cursor-pointer";
});
const toggleIconClass = computed(() => {
  if (children.value.length === 0) {
    return "opacity-0";
  }
  return expanded.value ? "rotate-0" : "-rotate-90";
});
const toggleExpand = () => {
  expanded.value = !expanded.value;
  console.log("expanded: ", expanded.value);
};

const selected = computed(() => {
  return nodeTreeStore.selectedNode === props.node?.NodeId;
});
const selectedClass = computed(() => {
  return selected.value ? "bg-surface" : "";
});

onMounted(async () => {
  if (connectionStore.state !== "connected") {
    console.warn(`Not connected to OPC UA server. Cannot browse nodes.`);
    return;
  }
  await browse();
});
</script>

<template>
  <div class="ml-2">
    <div class="flex items-center">
      <button
        class="mr-2 w-4 text-center"
        :class="toggleBtnClass"
        @click="toggleExpand"
      >
        <Icon
          :pathData="mdiTriangleDown"
          class="text-xs"
          :class="toggleIconClass"
        />
      </button>
      <div
        @click="nodeTreeStore.setSelectedNode(props.node?.NodeId || null)"
        :class="['rounded px-2 py-1', selectedClass]"
      >
        {{ props.node.DisplayName?.Text || props.node.BrowseName }}
      </div>
    </div>
    <ul v-if="children.length > 0 && expanded" class="ml-4">
      <li v-for="child in children" :key="child.NodeId">
        <NodeTreeItem :node="child" />
      </li>
    </ul>
  </div>
</template>
