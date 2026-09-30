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
    Code?: number;
    Symbol?: number;
}
export type QualifiedName = string;

export interface LocalizedText {
    Locale?: string;
    Text?: string;
}

export interface ExtensionObject {
    UaTypeId: NodeId;
    UaEncoding?: number;
    UaBody?: ByteString;
}

export interface DataValue {
    UaType: Byte;
    Value?: any;
    Dimensions?: UInt32[];
    Status?: StatusCode;
    SourceTimestamp?: DateTime;
    SourcePicoseconds?: UInt16;
    ServerTimestamp?: DateTime;
    ServerPicoseconds?: UInt16;
}

export interface Variant {
    UaType: number;
    Value?: any;
    Dimensions?: number[];
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
