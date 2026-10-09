<script lang="ts" setup>
import { useConnectionStore } from "@/stores/connection";
import { useNodeTreeStore } from "@/stores/node-tree";
import { AttributeId } from "@/ua/attribute_ids";
import { builtinTypeName, statusCodeIsGood, type Variant } from "@/ua/types";
import { ref, watch } from "vue";
import AttributeValue from "./AttributeValue.vue";

const connectionStore = useConnectionStore();
const nodeTreeStore = useNodeTreeStore();

const ALL_ATTRS = [
  AttributeId.NodeId,
  AttributeId.NodeClass,
  AttributeId.BrowseName,
  AttributeId.DisplayName,
  AttributeId.Description,
  AttributeId.WriteMask,
  AttributeId.UserWriteMask,
  AttributeId.IsAbstract,
  AttributeId.Symmetric,
  AttributeId.InverseName,
  AttributeId.ContainsNoLoops,
  AttributeId.EventNotifier,
  AttributeId.Value,
  AttributeId.DataType,
  AttributeId.ValueRank,
  AttributeId.ArrayDimensions,
  AttributeId.AccessLevel,
  AttributeId.UserAccessLevel,
  AttributeId.MinimumSamplingInterval,
  AttributeId.Historizing,
  AttributeId.Executable,
  AttributeId.UserExecutable,
  AttributeId.DataTypeDefinition,
  AttributeId.RolePermissions,
  AttributeId.UserRolePermissions,
  AttributeId.AccessRestrictions,
  AttributeId.AccessLevelEx,
];

const ATTRS_NAMES = [
  "NodeId",
  "NodeClass",
  "BrowseName",
  "DisplayName",
  "Description",
  "WriteMask",
  "UserWriteMask",
  "IsAbstract",
  "Symmetric",
  "InverseName",
  "ContainsNoLoops",
  "EventNotifier",
  "Value",
  "DataType",
  "ValueRank",
  "ArrayDimensions",
  "AccessLevel",
  "UserAccessLevel",
  "MinimumSamplingInterval",
  "Historizing",
  "Executable",
  "UserExecutable",
  "DataTypeDefinition",
  "RolePermissions",
  "UserRolePermissions",
  "AccessRestrictions",
  "AccessLevelEx",
];

const attributes = ref<[AttributeId, Variant][]>([]);

watch(
  () => nodeTreeStore.selectedNode,
  async (newSelected, _) => {
    if (!newSelected) {
      attributes.value = [];
      return;
    }
    if (!connectionStore.client) {
      console.warn(`No OPC/UA client available. Cannot read node attributes.`);
      attributes.value = [];
      return;
    }
    try {
      const attrs = await connectionStore.client.readAttributes(
        newSelected,
        ALL_ATTRS,
      );
      attributes.value = attrs
        .filter(
          (val) => val !== null && statusCodeIsGood(val?.Status),
        )
        .map((val, index) => [index, val as Variant]);
    } catch (error) {
      console.error(`Read node attributes failed:`, error);
    }
  },
);
</script>
<template>
  <table class="table-auto border-separate border-spacing-x-3">
    <thead>
      <tr>
        <th class="text-start">Attribute</th>
        <th class="text-start">Value</th>
        <th class="text-start">DataType</th>
      </tr>
    </thead>
    <tbody>
      <tr v-for="[index, attr] in attributes" :key="index">
        <td>{{ ATTRS_NAMES[index] }}</td>
        <td><AttributeValue :value="attr" /></td>
        <td>{{ builtinTypeName(attr.UaType) }}</td>
      </tr>
    </tbody>
  </table>
</template>
