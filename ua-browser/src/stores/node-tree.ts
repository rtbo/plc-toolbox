import { defineStore } from "pinia";
import { reactive, ref } from "vue";
import type { NodeId, ReferenceDescription } from "../ua/types";

export const useNodeTreeStore = defineStore('node-tree', () => {
  const browseCache = reactive<Map<NodeId, ReferenceDescription[]>>(new Map());
  function cacheBrowseResult(nodeId: NodeId, references: ReferenceDescription[]) {
    browseCache.set(nodeId, references);
  }
  
  const selectedNode = ref<NodeId | null>(null);
  function setSelectedNode(node: NodeId | null) {
    selectedNode.value = node;
  }

  return {
    browseCache,
    cacheBrowseResult,
    selectedNode,
    setSelectedNode,
  };
});
