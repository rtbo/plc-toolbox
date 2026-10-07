<script setup lang="ts">
import {
  BuiltinTypeId,
  builtinTypeName,
  NodeClass,
  type LocalizedText,
  type Variant,
} from "@/ua/types";

type IntFmt = "hex" | "node-class";

const props = defineProps<{
  value: Variant;
  intFmt?: IntFmt;
}>();

function isNumberType(typeId: BuiltinTypeId): boolean {
  return (
    typeId === BuiltinTypeId.SByte ||
    typeId === BuiltinTypeId.Int16 ||
    typeId === BuiltinTypeId.Int32 ||
    typeId === BuiltinTypeId.Int64 ||
    typeId === BuiltinTypeId.Byte ||
    typeId === BuiltinTypeId.UInt16 ||
    typeId === BuiltinTypeId.UInt32 ||
    typeId === BuiltinTypeId.UInt64 ||
    typeId === BuiltinTypeId.Float ||
    typeId === BuiltinTypeId.Double
  );
}

function hexLen(typeId: BuiltinTypeId): number {
  switch (typeId) {
    case BuiltinTypeId.SByte:
    case BuiltinTypeId.Byte:
      return 2;
    case BuiltinTypeId.Int16:
    case BuiltinTypeId.UInt16:
      return 4;
    case BuiltinTypeId.Int32:
    case BuiltinTypeId.UInt32:
      return 8;
    case BuiltinTypeId.Int64:
    case BuiltinTypeId.UInt64:
      return 16;
    default:
      return 0;
  }
}

function nodeClassName(nodeClass: NodeClass): string {
  switch (nodeClass) {
    case NodeClass.Object:
      return "Object";
    case NodeClass.Variable:
      return "Variable";
    case NodeClass.Method:
      return "Method";
    case NodeClass.ObjectType:
      return "ObjectType";
    case NodeClass.VariableType:
      return "VariableType";
    case NodeClass.ReferenceType:
      return "ReferenceType";
    case NodeClass.DataType:
      return "DataType";
    case NodeClass.View:
      return "View";
    default:
      return "Unspecified";
  }
}

function numberToString(
  value: number,
  typeId: BuiltinTypeId,
  intFmt?: IntFmt,
): string {
  if (intFmt === "hex") {
    return "0x" + value.toString(16).padStart(hexLen(typeId), "0");
  } else {
    return value.toString();
  }
}

function elidedString(str: string, maxLength: number): string {
  if (str.length <= maxLength) {
    return str;
  }
  const half = Math.floor(maxLength / 2);
  return str.slice(0, half) + "..." + str.slice(str.length - half);
}
</script>
<template>
  <div class="flex flex-col gap-2">
    <div v-if="value.Dimensions && value.Dimensions.length > 0">
      Array of {{ value.Dimensions.reduce((a, b) => a * b, 1) }} elements ({{
        value.Dimensions.join(" x ")
      }})
    </div>
    <div v-else>
      <span v-if="value.UaType === BuiltinTypeId.Boolean">
        {{ value.Value ? "true" : "false" }}
      </span>
      <span v-else-if="isNumberType(value.UaType)">
        {{ numberToString(value.Value as number, value.UaType, props.intFmt) }}
        <span
          v-if="props.intFmt === 'node-class'"
          class="text-content/50 text-sm"
        >
          ({{ nodeClassName(value.Value as NodeClass) }})
        </span>
      </span>
      <span v-else-if="typeof value.Value === 'string'">
        {{ elidedString(value.Value as string, 50) }}
      </span>
      <span v-else-if="value.UaType === BuiltinTypeId.LocalizedText">
        {{ (value.Value as LocalizedText).Text }}
        <span
          v-if="(value.Value as LocalizedText).Locale"
          class="text-content/50 text-sm"
        >
          (Locale: {{ (value.Value as LocalizedText).Locale }})
        </span>
      </span>
      <span v-else>
        Unsupported type: {{ builtinTypeName(value.UaType) }}
      </span>
    </div>
  </div>
</template>
