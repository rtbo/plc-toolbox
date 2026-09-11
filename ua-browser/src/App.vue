<script setup lang="ts">
import AddressBar from "./components/AddressBar.vue";
import { ref } from "vue";

const state = ref<"connected" | "disconnected" | "connecting" | "error">("disconnected");

function handleConnect(url: string) {
  console.log("Browsing to:", url);
  state.value = "connected";
}
function handleDisconnect() {
  console.log("Disconnected");
  state.value = "disconnected";
}
</script>

<template>
  <div class="bg-background text-content flex h-screen w-full flex-col">
    <AddressBar
      :state="state"
      @connect="handleConnect"
      @disconnect="handleDisconnect"
    />
  </div>
</template>
