import { Client } from "@/client";
import { OpcTcpClient } from "@/client/opctcp-client";
import { invoke } from "@tauri-apps/api/core";
import { defineStore } from "pinia";
import { computed, ref } from "vue";

const defaultAddress = import.meta.env.VITE_DEFAULT_ADDRESS || "192.168.0.1";
const defaultPort = import.meta.env.VITE_DEFAULT_PORT || 4840;

export type ConnectionState =
  "connected" | "disconnected" | "connecting" | "error";

export type Protocol = "opc.tcp";

export const useConnectionStore = defineStore("connection", () => {
  const protocol = ref<Protocol>("opc.tcp");
  const address = ref(defaultAddress);
  const port = ref(defaultPort);
  const state = ref<ConnectionState>("disconnected");
  const client = ref<Client | null>(null);
  const error = ref<string | null>(null);

  const url = computed(
    () => `${protocol.value}://${address.value}:${port.value}`,
  );

  async function connect() {
    error.value = null;
    state.value = "connecting";
    const timeout = setTimeout(() => {
      state.value = "error";
      error.value = "Connection timed out";
    }, 5000); // 5 seconds timeout
    try {
      if (protocol.value !== "opc.tcp") {
        throw new Error("Only opc.tcp protocol is supported in this version.");
      }
      const cl = new OpcTcpClient();
      cl.connect(url.value);
      clearTimeout(timeout);
      state.value = "connected";
      client.value = cl;
    } catch (e) {
      clearTimeout(timeout);
      state.value = "error";
      client.value = null;
      console.error(e);
      error.value = (e as Error).message;
    }
  }

  async function disconnect() {
    try {
      await invoke("disconnect");
      state.value = "disconnected";
      client.value = null;
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
    protocol,
    address,
    port,
    state,
    client,
    url,
    error,
    connect,
    disconnect,
    resetError,
  };
});
