import { StatusCodeId } from "./status_codes";

export type Boolean = boolean;
export type SByte = number;
export type Byte = number;
export type Int16 = number;
export type UInt16 = number;
export type Int32 = number;
export type UInt32 = number;
export type Int64 = string;
export type UInt64 = string;
export type Float = number;
export type Double = number;
export type String = string;

/**
 * A date and time represented as an ISO 8601 string.
 */
export type DateTime = string;

export type Guid = string;

/**
 * A sequence of bytes represented as a Base64-encoded string.
 */
export type ByteString = string;

export type XmlElement = string;

export type NodeId = string;

export type ExpandedNodeId = string;

export interface StatusCode {
  Code?: StatusCodeId;
  Symbol?: number;
}
export type QualifiedName = string;

export interface LocalizedText {
  Locale?: string;
  Text?: string;
}

export interface ExtensionObject {
  UaTypeId?: NodeId;
  UaEncoding?: number;
  UaBody?: ByteString;
}

export interface Variant {
  UaType: BuiltinTypeId;
  Value?: any;
  Dimensions?: number[];
}

export interface DataValue extends Variant {
  Status?: StatusCode;
  SourceTimestamp?: DateTime;
  SourcePicoseconds?: UInt16;
  ServerTimestamp?: DateTime;
  ServerPicoseconds?: UInt16;
}

export interface DiagnosticInfo {
  SymbolicId?: number;
  NamespaceUri?: number;
  Locale?: number;
  LocalizedText?: number;
  AdditionalInfo?: string;
  InnerStatusCode?: StatusCode;
  InnerDiagnosticInfo?: DiagnosticInfo;
}

export const enum BuiltinTypeId {
  Boolean = 1,
  SByte = 2,
  Byte = 3,
  Int16 = 4,
  UInt16 = 5,
  Int32 = 6,
  UInt32 = 7,
  Int64 = 8,
  UInt64 = 9,
  Float = 10,
  Double = 11,
  String = 12,
  DateTime = 13,
  Guid = 14,
  ByteString = 15,
  XmlElement = 16,
  NodeId = 17,
  ExpandedNodeId = 18,
  StatusCode = 19,
  QualifiedName = 20,
  LocalizedText = 21,
  ExtensionObject = 22,
  DataValue = 23,
  Variant = 24,
  DiagnosticInfo = 25,
}

export function statusCodeIsGood(statusCode: StatusCodeId | StatusCode): boolean {
    if (typeof statusCode === "number") {
        return (statusCode & 0xf0000000) === 0x00000000;
    } else {
        return statusCode.Code === undefined || (statusCode.Code & 0xf0000000) === 0x00000000;
    }
}

export function statusCodeIsUncertain(statusCode: StatusCodeId | StatusCode): boolean {
    if (typeof statusCode === "number") {
        return (statusCode & 0xf0000000) === 0x40000000;
    } else {
        return statusCode.Code !== undefined && (statusCode.Code & 0xf0000000) === 0x40000000;
    }
}

export function statusCodeIsBad(statusCode: StatusCodeId | StatusCode): boolean {
    if (typeof statusCode === "number") {
        return (statusCode & 0xf0000000) === 0x80000000;
    } else {
        return statusCode.Code !== undefined && (statusCode.Code & 0xf0000000) === 0x80000000;
    }
}