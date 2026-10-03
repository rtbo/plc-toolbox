import { defineStore } from "pinia";
import { reactive, ref } from "vue";
import type { NodeId, ReferenceDescription } from "../ua/types";

const useNodeTreeStore = defineStore('node-tree', () => {
  const browseCache = reactive<Map<NodeId, ReferenceDescription[]>>(new Map());
  function cacheBrowseResult(nodeId: NodeId, references: ReferenceDescription[]) {
    browseCache.set(nodeId, references);
  }
  
  const selectedNode = ref<any | null>(null);
  function setSelectedNode(node: any | null) {
    selectedNode.value = node;
  }

  return {
    browseCache,
    cacheBrowseResult,
    selectedNode,
    setSelectedNode,
  };
});
