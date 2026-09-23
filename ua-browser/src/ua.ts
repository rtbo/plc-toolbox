import { invoke } from "@tauri-apps/api/core";

export type ConnectionState =
  "connected" | "disconnected" | "connecting" | "error";

export async function uaConnect(url: string): Promise<void> {
  return invoke("connect", { url });
}

export async function uaDisconnect(): Promise<void> {
  return invoke("disconnect");
}

export const rootFolderNodeId = "i=84";

export type AttributeId =
  | "node-id"
  | "node-class"
  | "browse-name"
  | "display-name"
  | "description"
  | "write-mask"
  | "user-write-mask"
  | "is-abstract"
  | "symmetric"
  | "inverse-name"
  | "contains-no-loops"
  | "event-notifier"
  | "value"
  | "data-type"
  | "value-rank"
  | "array-dimensions"
  | "access-level"
  | "user-access-level"
  | "minimum-sampling-interval"
  | "historizing"
  | "executable"
  | "user-executable"
  | "data-type-definition"
  | "role-permissions"
  | "user-role-permissions"
  | "access-restrictions"
  | "access-level-ex";

export interface ReferenceDescription {
  NodeId: string;
  BrowseName: string;
  DisplayName: string;
  TypeDefinition: string;
  NodeClass: string;
  IsForward: boolean;
  ReferenceTypeId: string;
}

export async function uaBrowse(nodeId: string): Promise<ReferenceDescription[]> {
  return invoke<ReferenceDescription[]>("browse", { nodeId });
}

export interface Variant {
  scalar: VariantScalar;
}

export type VariantScalar =
  | boolean
  | number
  | string;
  
export async function uaReadAttribute(nodeId: string, attributeId: AttributeId): Promise<Variant> {
  console.log(`Reading attribute: nodeId=${nodeId}, attributeId=${attributeId}`);
  return invoke<Variant>("read_attribute", { nodeId, attributeId });
}