import { invoke } from "@tauri-apps/api/core";
import { defineStore } from "pinia";
import { computed, ref } from "vue";

const defaultAddress = import.meta.env.VITE_DEFAULT_ADDRESS || "192.168.0.1";
const defaultPort = import.meta.env.VITE_DEFAULT_PORT || 4840;

export type ConnectionState =
  "connected" | "disconnected" | "connecting" | "error";

export const useConnectionStore = defineStore("connection", () => {
  const state = ref<ConnectionState>("disconnected");
  const address = ref(defaultAddress);
  const port = ref(defaultPort);
  const error = ref<string | null>(null);

  const url = computed(() => `opc.tcp://${address.value}:${port.value}`);

  async function connect() {
    error.value = null;
    state.value = "connecting";
    const timeout = setTimeout(() => {
        state.value = "error";
        error.value = "Connection timed out";
    }, 5000); // 5 seconds timeout
    try {
        await invoke("connect", { url: url.value });
        clearTimeout(timeout);
        state.value = "connected";
    } catch (e) {
        clearTimeout(timeout);
        state.value = "error";
        console.error(e);
        error.value = (e as Error).message;
    }
  }

  async function disconnect() {
    try {
        await invoke("disconnect");
        state.value = "disconnected";
    } catch (e) {
        state.value = "error";
        console.error(e);
        error.value = (e as Error).message;
    }
  }

  function resetError() {
    if (state.value === "error") {
        error.value = null;
        state.value = "disconnected";
    }
  }

  return {
    state,
    address,
    port,
    url,
    error,
    connect,
    disconnect,
    resetError,
  };
});
