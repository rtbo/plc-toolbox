<script setup lang="ts">
import { computed, ref } from "vue";
import {
  mdiLanConnect,
  mdiLanDisconnect,
  mdiLoading,
  mdiClipboardOutline,
  mdiClipboardCheckOutline,
} from "@mdi/js";
import Icon from "./Icon.vue";

const props = defineProps<{
  state: "connected" | "disconnected" | "connecting" | "error";
}>();
const emit = defineEmits<{
  (e: "connect", url: string): void;
  (e: "disconnect"): void;
  (e: "reset-error"): void;
}>();

const address = ref("192.168.0.1");
const port = ref(4840);

function connect() {
  const url = `opc.tcp://${address.value}:${port.value}`;
  emit("connect", url);
}
function disconnect() {
  emit("disconnect");
}

const iconPath = computed(() => {
  if (props.state === "connected") {
    return mdiLanConnect;
  } else {
    return mdiLanDisconnect;
  }
});
const iconClass = computed(() => {
  if (props.state === "error") {
    return "text-error";
  } else if (props.state === "connected") {
    return "text-primary";
  } else {
    return "text-on-surface/50";
  }
});
const addressClass = computed(() => {
  if (props.state === "error") {
    return "border-error";
  } else if (props.state === "connected") {
    return "border-primary";
  } else {
    return "border-on-surface/50";
  }
});

const canEdit = computed(
  () => props.state !== "connected" && props.state !== "connecting",
);

const copied = ref(false);
const flipped = ref(false);
const FLIP_DURATION = 150;
const HOLD_DURATION = 2000;

function flipTo(showChecked: boolean) {
  return new Promise<void>((resolve) => {
    flipped.value = true;
    setTimeout(() => {
      copied.value = showChecked;
      flipped.value = false;
      setTimeout(resolve, FLIP_DURATION);
    }, FLIP_DURATION);
  });
}

async function copyToClipboard() {
  const url = `opc.tcp://${address.value}:${port.value}`;
  await navigator.clipboard.writeText(url);
  await flipTo(true);
  setTimeout(() => {
    flipTo(false);
  }, HOLD_DURATION);
}
function capitalize(str: string) {
  return str.charAt(0).toUpperCase() + str.slice(1);
}
</script>

<template>
  <div class="address-bar w-full px-4 py-4">
    <div class="mx-auto flex max-w-4xl items-center">

      <!-- Icon indicating connection state -->
      <div class="flex items-center">
        <Icon
          :pathData="iconPath"
          class="mx-2 text-xl"
          :class="iconClass"
          :title="capitalize(props.state)"
        />
        <Icon
          v-if="props.state === 'connecting'"
          :pathData="mdiLanConnect"
          class="text-primary absolute mx-2 animate-ping text-xl"
          :class="iconClass"
          title="Connecting"
        />
      </div>

      <!-- Address and port input fields -->
      <div
        class="mx-auto flex items-center rounded-lg border px-2 py-1"
        :class="addressClass"
      >
        <span class="mr-2">opc.tcp://</span>
        <input
          @input="$emit('reset-error')"
          class="bg-surface w-50 px-2"
          :disabled="!canEdit"
          type="text"
          placeholder="Search..."
          v-model="address"
        />
        <span class="mx-1">:</span>

        <input
          @input="$emit('reset-error');"
          min="1"
          max="65535"
          pattern="[1-9][0-9]{0,4}"
          step="1"
          type="number"
          inputmode="numeric"
          class="port-input bg-surface w-18"
          v-model="port" 
          :disabled="!canEdit"
        />
        <button
          class="ml-3 flex cursor-pointer items-center text-lg"
          @click="copyToClipboard"
        >
          <Icon
            :pathData="copied ? mdiClipboardCheckOutline : mdiClipboardOutline"
            class="mr-2 transition-transform duration-150 ease-in"
            :class="flipped ? 'scale-x-0' : 'scale-x-100'"
            title="Copy"
          />
        </button>
      </div>

      <!-- Connect/Disconnect button -->
      <div class="mx-2 flex w-36 items-center justify-center">
        <button
          v-if="props.state !== 'connected'"
          class="bg-primary text-on-primary right-4 inline-flex cursor-pointer items-center rounded-lg px-4 py-1"
          :disabled="props.state === 'connecting'"
          @click="connect"
        >
          <Icon
            :pathData="
              props.state === 'connecting' ? mdiLoading : mdiLanConnect
            "
            class="mr-2"
            :class="{ 'animate-spin': props.state === 'connecting' }"
            title="Connect"
          />
          Connect
        </button>
        <button
          v-else
          class="border-primary text-on-surface right-4 inline-flex cursor-pointer items-center rounded-lg border px-4 py-1"
          @click="disconnect"
        >
          <Icon :pathData="mdiLanDisconnect" class="mr-2" title="Disconnect" />
          Disconnect
        </button>
      </div>
    </div>
  </div>
</template>

<style lang="css" scoped>
.port-input::-webkit-inner-spin-button,
.port-input::-webkit-outer-spin-button {
  -webkit-appearance: none;
  margin: 0;
}
.port-input {
  -moz-appearance: textfield;
  appearance: textfield;
}
</style>