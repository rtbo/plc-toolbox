<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import AddressBar from "./components/AddressBar.vue";
import type { ConnectionState } from "./ua";
import { uaBrowse, uaReadAttribute } from "./ua";

const state = ref<ConnectionState>("disconnected");

const typeDefs: Map<string, any> = new Map();

async function handleConnect(url: string) {
  state.value = "connecting";
  try {
    await invoke("connect", { url });
    state.value = "connected";
    let refs = await uaBrowse("i=84");
    for (const refDesc of refs) {
      if (!typeDefs.has(refDesc.typeDefinition)) {
        const td = await uaReadAttribute(refDesc.typeDefinition, "browse-name");
        typeDefs.set(refDesc.typeDefinition, td);
      }
    }
  } catch (e) {
    console.error(e);
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
