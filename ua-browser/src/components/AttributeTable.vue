<script lang="ts" setup>
import { useConnectionStore } from "@/stores/connection";
import { useNodeTreeStore } from "@/stores/node-tree";
import { AttributeId } from "@/ua/attribute_ids";
import { statusCodeIsGood, type Variant } from "@/ua/types";
import { ref, watch } from "vue";

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

const ALL_ATTRS_NAMES = [
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
  async (newSelected, oldSelected) => {
    if (newSelected) {
      if (!connectionStore.client) {
        console.warn(
          `No OPC/UA client available. Cannot read node attributes.`,
        );
        return;
      }
      try {
        const attrs = await connectionStore.client.readAttributes(
          newSelected,
          ALL_ATTRS,
        );
        attributes.value = attrs
          .filter((val) => val !== null && statusCodeIsGood(val.Status))
          .map((val, index) => [ALL_ATTRS[index], val as Variant]);
      } catch (error) {
        console.error(`Read node attributes failed:`, error);
      }
    }
  },
);
</script>
<template>
  <table>
    <thead>
      <tr>
        <th>Attribute</th>
        <th>Value</th>
      </tr>
    </thead>
    <tbody>
      <tr v-for="(attr, index) in attributes" :key="index">
        <td>{{ ALL_ATTRS_NAMES[index] }}</td>
        <td>{{ JSON.stringify(attr) }}</td>
      </tr>
    </tbody>
  </table>
</template>
