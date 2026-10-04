import { ReferenceTypeId } from "./ua/ns0";
import { statusCodeIsGood } from "./ua/status_codes";
import {
  BrowseDirection,
  type BrowseNextRequest,
  type BrowseNextResponse,
  type BrowseRequest,
  type BrowseResponse,
  type NodeId,
  type ReferenceDescription,
  type RequestHeader,
  type ResponseHeader,
  type StatusCode,
} from "./ua/types";

export abstract class Client {
  abstract connect(url: string): Promise<void>;
  abstract disconnect(): Promise<void>;

  protected abstract sendBrowseRequest(
    req: BrowseRequest,
  ): Promise<BrowseResponse>;

  protected abstract sendBrowseNextRequest(
    req: BrowseNextRequest,
  ): Promise<BrowseNextResponse>;

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
    checkStatusCode(result);

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
      checkStatusCode(contResult);
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

function checkStatusCode({ StatusCode }: { StatusCode?: StatusCode }): void {
  if (!StatusCode) {
    throw new Error("Missing StatusCode");
  }
  if (StatusCode.Code && !statusCodeIsGood(StatusCode.Code)) {
    throw new Error(`StatusCode : ${StatusCode.Symbol} (${StatusCode.Code})`);
  }
}

function defaultHeader(): RequestHeader {
  return {
    Timestamp: new Date().toISOString(),
  };
}
