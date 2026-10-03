import { invoke } from "@tauri-apps/api/core";
import type { BrowseRequest, BrowseResponse, RequestHeader, ResponseHeader } from "./ua/types";
import { statusCodeIsGood } from "./ua/status_codes";

export async function clientConnect(url: string): Promise<void> {
  return invoke("connect", { url });
}

export async function clientDisconnect(): Promise<void> {
  return invoke("disconnect");
}

function serializeJSON(value: unknown): Uint8Array {
  const json = JSON.stringify(value);
  return new TextEncoder().encode(json);
}

function defaultHeader(): RequestHeader {
  return {
    Timestamp: new Date().toISOString(),
  };
}

function checkResponse(header?: ResponseHeader): void {
  if (typeof header === "undefined") {
    throw new Error("Response is missing ServiceResult");
  }
  const result = header.ServiceResult;
  if (typeof result === "undefined") {
    throw new Error("Response is missing ServiceResult");
  }
  if (!result.Code) {
    return;
  }
  if (!statusCodeIsGood(result.Code)) {
    throw new Error(`ServiceResult: ${header.ServiceResult}`);
  }  
}

export async function clientBrowse(req: BrowseRequest): Promise<BrowseResponse> {
  if (!req.RequestHeader) {
    req.RequestHeader = defaultHeader();
  }
  const resp = await invoke<BrowseResponse>("browse", serializeJSON(req));
  console.log(resp);
  checkResponse(resp.ResponseHeader)
  return resp;
}
