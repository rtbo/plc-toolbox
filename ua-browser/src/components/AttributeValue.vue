<script setup lang="ts">
import { BuiltinTypeId, builtinTypeName, type LocalizedText, type Variant } from "@/ua/types";

const props = defineProps<{
  value: Variant;
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
        {{ value.Value }}
      </span>
      <span v-else-if="typeof value.Value === 'string'">
        {{ elidedString(value.Value as string, 50) }}
      </span>
      <span v-else-if="value.UaType === BuiltinTypeId.LocalizedText">
        {{ (value.Value as LocalizedText).Text }}
        <span v-if="(value.Value as LocalizedText).Locale" class="text-sm text-content/50">
          (Locale: {{ (value.Value as LocalizedText).Locale }})
        </span>
      </span>
      <span v-else>
          Unsupported type: {{ builtinTypeName(value.UaType) }}
      </span>
    </div>
  </div>
</template>
