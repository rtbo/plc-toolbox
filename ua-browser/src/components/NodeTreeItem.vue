<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useConnectionStore } from "../stores/connection";
import type {
  BrowseRequest,
  BrowseResponse,
  ReferenceDescription,
} from "../ua/types";
import { BrowseDirection } from "../ua/types";
import { clientBrowse } from "../client";
import { ReferenceTypeId } from "../ua/ns0";
import { mdiTriangleDown } from "@mdi/js";
import Icon from "./Icon.vue";

const props = defineProps<{
  node: ReferenceDescription;
}>();

const connectionStore = useConnectionStore();

const children = ref<ReferenceDescription[]>([]);

async function browse() {
  const req: BrowseRequest = {
    RequestedMaxReferencesPerNode: 100,
    NodesToBrowse: [
      {
        BrowseDirection: BrowseDirection.Forward,
        IncludeSubtypes: true,
        ReferenceTypeId: ReferenceTypeId.HierarchicalReferences,
        NodeId: props.node.NodeId,
        ResultMask: 63,
      },
    ],
  };
  try {
    const response: BrowseResponse = await clientBrowse(req);
    console.log(`Browse response:`, response);
    children.value = response?.Results?.[0]?.References || [];
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
    <div class="flex items-center" :class="toggleBtnClass" @click="toggleExpand">
      <button
        class="mr-2 w-4 text-center"
      >
        <Icon :pathData="mdiTriangleDown" class="text-xs" :class="toggleIconClass" />
      </button>
      <div>{{ props.node.DisplayName?.Text || props.node.BrowseName }}</div>
    </div>
    <ul v-if="children.length > 0 && expanded" class="ml-4">
      <li v-for="child in children" :key="child.NodeId">
        <NodeTreeItem :node="child" />
      </li>
    </ul>
  </div>
</template>
