<script lang="ts" setup>
import { useConnectionStore } from "../stores/connection";
import { NodeClass, type ReferenceDescription } from "../ua/types";
import { ObjectId } from "../ua/ns0";
import NodeTreeItem from "./NodeTreeItem.vue";
const connectionStore = useConnectionStore();

const rootNode: ReferenceDescription = {
  NodeId: ObjectId.RootFolder,
  BrowseName: "Root",
  DisplayName: { Text: "Root" },
  NodeClass: NodeClass.Object,
};
</script>

<template>
  <div>
    <NodeTreeItem
      v-if="connectionStore.state === 'connected'"
      :node="rootNode"
    />
  </div>
</template>
