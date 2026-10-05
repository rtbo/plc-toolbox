import { AttributeId } from "./ua/attribute_ids";
import { DataTypeId, ObjectId, ReferenceTypeId, VariableId } from "./ua/ns0";
import { statusCodeIsGood } from "./ua/status_codes";
import {
  BrowseDirection,
  BuiltinTypeId,
  type BrowseNextRequest,
  type BrowseNextResponse,
  type BrowseRequest,
  type BrowseResponse,
  type NodeId,
  type ReadRequest,
  type ReadResponse,
  type ReferenceDescription,
  type RequestHeader,
  type ResponseHeader,
  type StatusCode,
  type Variant,
} from "./ua/types";

export abstract class Client {
  _nsArray: string[] = [];

  get nsArray(): string[] {
    return this._nsArray;
  }
  
  protected abstract doConnect(url: string): Promise<void>;
  protected abstract doDisconnect(): Promise<void>;

  protected abstract sendReadRequest(req: ReadRequest): Promise<ReadResponse>;

  protected abstract sendBrowseRequest(
    req: BrowseRequest,
  ): Promise<BrowseResponse>;

  protected abstract sendBrowseNextRequest(
    req: BrowseNextRequest,
  ): Promise<BrowseNextResponse>;

  async connect(url: string): Promise<void> {
    await this.doConnect(url);
    try {
      const nsArray = await this.readAttribute(
        VariableId.Server_NamespaceArray,
        AttributeId.Value,
      );
      if (!nsArray || nsArray.UaType !== BuiltinTypeId.String) {
        throw new Error("NamespaceArray is not an array of strings");
      }
      this._nsArray = nsArray?.Value || [];
      console.log("NamespaceArray:", this._nsArray);
    } catch (e) {
      console.error("Error reading namespace array:", e);
      await this.disconnect();
      throw e;
    }
  }

  async disconnect(): Promise<void> {
    await this.doDisconnect();
  }

  async readAttribute(nodeId: NodeId, attrId: AttributeId): Promise<Variant | null> {
    const req: ReadRequest = {
      RequestHeader: defaultHeader(),
      NodesToRead: [
        {
          NodeId: nodeId,
          AttributeId: attrId,
        },
      ],
    };
    const resp = await this.sendReadRequest(req);
    console.log("Read response:", resp);
    checkResponse(resp.ResponseHeader);
    if (resp.Results?.length !== 1) {
      throw new Error("Unexpected number of results in read response");
    }
    const result = resp.Results[0];
    if (!result) {
      return null;
    }
    checkStatusCode(result?.Status);
    return result;
  }

  async browseNode(nodeId: NodeId): Promise<ReferenceDescription[]> {
    const req: BrowseRequest = {
      RequestHeader: defaultHeader(),
      RequestedMaxReferencesPerNode: 100,
      NodesToBrowse: [
        {
          BrowseDirection: BrowseDirection.Forward,
          IncludeSubtypes: true,
          ReferenceTypeId: ReferenceTypeId.HierarchicalReferences,
          NodeId: nodeId,
          ResultMask: 63,
        },
      ],
    };
    const resp = await this.sendBrowseRequest(req);
    checkResponse(resp.ResponseHeader);
    if (resp.Results?.length !== 1) {
      throw new Error("Unexpected number of results in browse response");
    }
    const result = resp.Results[0];
    if (!result) {
      // null result: no references found for the node
      return [];
    }
    checkStatusCode(result?.StatusCode);

    const nodeRefs = result.References || [];
    let continuation = result.ContinuationPoint;
    while (continuation) {
      const contReq: BrowseNextRequest = {
        RequestHeader: defaultHeader(),
        ReleaseContinuationPoints: true,
        ContinuationPoints: [continuation],
      };
      const contResp = await this.sendBrowseNextRequest(contReq);
      checkResponse(contResp.ResponseHeader);
      if (contResp.Results?.length !== 1) {
        throw new Error("Unexpected number of results in browse next response");
      }
      const contResult = contResp.Results[0];
      checkStatusCode(contResult?.StatusCode);
      if (contResult.References) {
        nodeRefs?.push(...contResult.References);
      }
      continuation = contResult.ContinuationPoint;
    }
    return nodeRefs;
  }
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
    throw new Error(`ServiceResult: ${result}`);
  }
}

function checkStatusCode(Status?: StatusCode): void {
  if (Status?.Code && !statusCodeIsGood(Status.Code)) {
    throw new Error(`StatusCode : ${Status.Symbol} (${Status.Code})`);
  }
}

function defaultHeader(): RequestHeader {
  return {
    Timestamp: new Date().toISOString(),
  };
}
