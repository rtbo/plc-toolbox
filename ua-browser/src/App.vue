<script setup lang="ts">
import AddressBar from "./components/AddressBar.vue";
import { ref } from "vue";
import { invoke } from '@tauri-apps/api/core';


const state = ref<"connected" | "disconnected" | "connecting" | "error">("disconnected");

async function handleConnect(url: string) {
  state.value = "connecting";
  try {
    await invoke("connect", { url });
    state.value = "connected";
  } catch {
    state.value = "error";
  }
}

async function handleDisconnect() {
  try {
    await invoke("disconnect");
    state.value = "disconnected";
  } catch {
    state.value = "error";
  }
}

function resetConnError() {
  if (state.value === "error") {
    state.value = "disconnected";
  }
}
</script>

<template>
  <div class="bg-background text-content flex h-screen w-full flex-col">
    <AddressBar
      :state="state"
      @connect="handleConnect"
      @disconnect="handleDisconnect"
      @reset-error="resetConnError"
    />
  </div>
</template>
