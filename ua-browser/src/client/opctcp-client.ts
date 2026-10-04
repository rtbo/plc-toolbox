import { invoke } from "@tauri-apps/api/core";
import type {
  BrowseNextRequest,
  BrowseNextResponse,
  BrowseRequest,
  BrowseResponse,
} from "@/ua/types";
import { Client } from "@/client";

export class OpcTcpClient extends Client {
  private url: string | null = null;

  async connect(url: string): Promise<void> {
    await invoke("connect", { url });
    this.url = url;
  }

  async disconnect(): Promise<void> {
    if (this.url) {
      await invoke("disconnect");
      this.url = null;
    }
  }

  async sendBrowseRequest(req: BrowseRequest): Promise<BrowseResponse> {
    return await invoke("browse", serializeJSON(req));
  }

  async sendBrowseNextRequest(
    req: BrowseNextRequest,
  ): Promise<BrowseNextResponse> {
    return await invoke("browse_next", serializeJSON(req));
  }
}

function serializeJSON(value: unknown): Uint8Array {
  const json = JSON.stringify(value);
  return new TextEncoder().encode(json);
}
