import { invoke } from "@tauri-apps/api/core";

export type ConnectionState =
  "connected" | "disconnected" | "connecting" | "error";

export async function connect(url: string): Promise<ConnectionState> {
  return invoke<ConnectionState>("connect", { url });
}

export async function disconnect(): Promise<ConnectionState> {
  return invoke<ConnectionState>("disconnect");
}

export const rootFolderNodeId = "i=84";

export type AttributeId =
  | "NodeId"
  | "NodeClass"
  | "BrowseName"
  | "DisplayName"
  | "Description"
  | "WriteMask"
  | "UserWriteMask"
  | "IsAbstract"
  | "Symmetric"
  | "InverseName"
  | "ContainsNoLoops"
  | "EventNotifier"
  | "Value"
  | "DataType"
  | "ValueRank"
  | "ArrayDimensions"
  | "AccessLevel"
  | "UserAccessLevel"
  | "MinimumSamplingInterval"
  | "Historizing"
  | "Executable"
  | "UserExecutable"
  | "DataTypeDefinition"
  | "RolePermissions"
  | "UserRolePermissions"
  | "AccessRestrictions"
  | "AccessLevelEx";
