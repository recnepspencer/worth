import type {
  ComputedSignalHandle,
  Signal,
} from "../callable_surface.js";
import type { SignalValue } from "../model.js";
import type {
  LineageSummary,
  ReplaySummary,
} from "../diagnostics.js";
import type {
  ResourceLineDescriptor,
  ResourcePolicyProfileName,
  ResourceProcessingCompletionKind,
  ResourceBinaryDescriptor,
  ResourceRequestDescriptor,
  ResourceUploadDescriptor,
  ResourceUploadTransportKind,
} from "./resource_postures.js";
import type {
  ResourceItemAspectMap,
  ResourceLineReconciliation,
} from "./resource_reconciliation.js";
import type { ResourceLineVerificationPackage } from "./resource_verification.js";
import type {
  ResourceLineHistory,
  ResourceLineHistoryAvailability,
} from "./resource_line_history.js";
import type {
  ResourceLineDiagnosticsSummary,
  ResourceLineSummary,
} from "./resource_line_summary.js";
import type { ResourceLineDiagnostics } from "./resource_line_diagnostics.js";
import type {
  ResourceMutationResponseConfirmationKind,
  ResourceMutationResponsePlan,
} from "./resource_mutation_response.js";
import type {
  ResourceLineAwaitSettlementOptions,
  ResourceLineAwaitSettlementResult,
  ResourceLineExecution,
  ResourceLineExecutionOptions,
} from "./resource_await_settlement.js";

export type {
  ResourceLineAwaitSettlementFailedResult,
  ResourceLineAwaitSettlementFulfilledResult,
  ResourceLineAwaitSettlementOptions,
  ResourceLineAwaitSettlementResult,
  ResourceLineExecution,
  ResourceLineExecutionOptions,
} from "./resource_await_settlement.js";

export type ResourceLineOperation =
  | "initialLoad"
  | "refresh"
  | "revalidate"
  | "replay"
  | "restore"
  | "delivery";
export type ResourceLineContinuity =
  | "preservedVisibleValue"
  | "noVisibleValueYet";

export interface ResourceLinePendingStatus {
  readonly kind: "pending";
  readonly operation: ResourceLineOperation;
  readonly continuity: ResourceLineContinuity;
}

export interface ResourceLineFulfilledStatus {
  readonly kind: "fulfilled";
  readonly operation: ResourceLineOperation;
}

export interface ResourceLineTimedOutStatus {
  readonly kind: "timedOut";
  readonly operation: ResourceLineOperation;
  readonly continuity: ResourceLineContinuity;
}

export interface ResourceLineRejectedStatus {
  readonly kind: "rejected";
  readonly operation: ResourceLineOperation;
  readonly message: string;
  readonly continuity: ResourceLineContinuity;
}

export type ResourceLineStatus =
  | ResourceLinePendingStatus
  | ResourceLineFulfilledStatus
  | ResourceLineTimedOutStatus
  | ResourceLineRejectedStatus;

export interface ResourceLineFresh {
  readonly kind: "fresh";
}

export interface ResourceLineStaleFreshness {
  readonly kind: "stale";
  readonly reason:
    | "policyProfile"
    | "initialLoadPending"
    | "initialLoadRejected"
    | "initialLoadTimedOut"
    | "refreshPending"
    | "refreshRejected"
    | "refreshTimedOut"
    | "revalidatePending"
    | "revalidateRejected"
    | "revalidateTimedOut"
    | "replayPending"
    | "replayRejected"
    | "replayTimedOut"
    | "deliveryInvalidate"
    | "manualLineInvalidate"
    | "manualFamilyInvalidate"
    | "manualFamilyInvalidateAll"
    | "manualRuntimeInvalidateAll";
}

export type ResourceLineFreshness =
  | ResourceLineFresh
  | ResourceLineStaleFreshness;

export interface ResourceLineReadyProcessing {
  readonly kind: "ready";
  readonly completionKind: ResourceProcessingCompletionKind;
  readonly jobId: null;
  readonly message: null;
}

export interface ResourceLineAcceptedProcessing {
  readonly kind: "accepted";
  readonly completionKind: Exclude<ResourceProcessingCompletionKind, "none">;
  readonly jobId: string;
  readonly message: string | null;
}

export interface ResourceLineInProgressProcessing {
  readonly kind: "processing";
  readonly completionKind: Exclude<ResourceProcessingCompletionKind, "none">;
  readonly jobId: string;
  readonly message: string | null;
}

export type ResourceLineProcessing =
  | ResourceLineReadyProcessing
  | ResourceLineAcceptedProcessing
  | ResourceLineInProgressProcessing;

export interface ResourceLineReadyUpload {
  readonly kind: "ready";
  readonly transportKind: ResourceUploadTransportKind;
  readonly uploadId: null;
  readonly descriptor: null;
  readonly finalizeRequired: boolean;
  readonly awaitingProcessing: boolean;
  readonly message: null;
}

export interface ResourceLinePreparedUpload {
  readonly kind: "prepared";
  readonly transportKind: Exclude<ResourceUploadTransportKind, "none">;
  readonly uploadId: string;
  readonly descriptor: ResourceUploadDescriptor;
  readonly finalizeRequired: boolean;
  readonly awaitingProcessing: false;
  readonly message: string | null;
}

export interface ResourceLineUploadedUpload {
  readonly kind: "uploaded";
  readonly transportKind: Exclude<ResourceUploadTransportKind, "none">;
  readonly uploadId: string;
  readonly descriptor: null;
  readonly finalizeRequired: boolean;
  readonly awaitingProcessing: boolean;
  readonly message: string | null;
}

export type ResourceLineUpload =
  | ResourceLineReadyUpload
  | ResourceLinePreparedUpload
  | ResourceLineUploadedUpload;

export interface ResourceUploadDescriptorDiagnostics {
  readonly kind: "signed" | "directMultipart";
  readonly url: string;
  readonly method: "PUT" | "POST";
  readonly headerNames: readonly string[];
  readonly fieldNames: readonly string[];
  readonly objectKey: string | null;
  readonly expiresAt: string | null;
}

export interface ResourceLineReadyUploadDiagnostics {
  readonly kind: "ready";
  readonly transportKind: ResourceUploadTransportKind;
  readonly uploadId: null;
  readonly descriptor: null;
  readonly finalizeRequired: boolean;
  readonly awaitingProcessing: boolean;
  readonly message: null;
}

export interface ResourceLinePreparedUploadDiagnostics {
  readonly kind: "prepared";
  readonly transportKind: Exclude<ResourceUploadTransportKind, "none">;
  readonly uploadId: string;
  readonly descriptor: ResourceUploadDescriptorDiagnostics;
  readonly finalizeRequired: boolean;
  readonly awaitingProcessing: false;
  readonly message: string | null;
}

export interface ResourceLineUploadedUploadDiagnostics {
  readonly kind: "uploaded";
  readonly transportKind: Exclude<ResourceUploadTransportKind, "none">;
  readonly uploadId: string;
  readonly descriptor: null;
  readonly finalizeRequired: boolean;
  readonly awaitingProcessing: boolean;
  readonly message: string | null;
}

export type ResourceLineUploadDiagnostics =
  | ResourceLineReadyUploadDiagnostics
  | ResourceLinePreparedUploadDiagnostics
  | ResourceLineUploadedUploadDiagnostics;

export interface ResourceLineDownloadReadyDescriptor {
  readonly kind: "file" | "media" | "export";
  readonly id: string;
  readonly label: string | null;
  readonly fileName: string | null;
  readonly mediaType: string | null;
  readonly byteLength: number | null;
  readonly download: {
    readonly kind: "ready";
    readonly transportKind: "simple" | "directMultipart";
    readonly url: string;
    readonly method: "GET" | "POST";
    readonly headers: Readonly<Record<string, string>>;
    readonly fields: Readonly<Record<string, string>>;
    readonly objectKey: string | null;
    readonly expiresAt: string | null;
  };
}

export interface ResourceLineDownloadUnavailableDescriptor {
  readonly kind: "file" | "media" | "export";
  readonly id: string;
  readonly label: string | null;
  readonly fileName: string | null;
  readonly mediaType: string | null;
  readonly byteLength: number | null;
  readonly download: {
    readonly kind: "unavailable";
    readonly reason: "notReady" | "unavailable";
    readonly detail: string;
  };
}

export interface ResourceLineDownloadIncompatibleDescriptor {
  readonly kind: "file" | "media" | "export";
  readonly id: string;
  readonly label: string | null;
  readonly fileName: string | null;
  readonly mediaType: string | null;
  readonly byteLength: number | null;
  readonly download: {
    readonly kind: "incompatible";
    readonly reason: "staleDescriptor" | "transportBoundary";
    readonly detail: string;
  };
}

export type ResourceLineDownloadDescriptor =
  | ResourceLineDownloadReadyDescriptor
  | ResourceLineDownloadUnavailableDescriptor
  | ResourceLineDownloadIncompatibleDescriptor;

export interface ResourceLineDownload {
  readonly count: number;
  readonly readyCount: number;
  readonly unavailableCount: number;
  readonly incompatibleCount: number;
  readonly descriptors: readonly ResourceLineDownloadDescriptor[];
}

export interface ResourceLineDownloadReadyDescriptorDiagnostics {
  readonly kind: "file" | "media" | "export";
  readonly id: string;
  readonly label: string | null;
  readonly fileName: string | null;
  readonly mediaType: string | null;
  readonly byteLength: number | null;
  readonly download: {
    readonly kind: "ready";
    readonly transportKind: "simple" | "directMultipart";
    readonly url: string;
    readonly method: "GET" | "POST";
    readonly headerNames: readonly string[];
    readonly fieldNames: readonly string[];
    readonly objectKey: string | null;
    readonly expiresAt: string | null;
  };
}

export interface ResourceLineDownloadUnavailableDescriptorDiagnostics {
  readonly kind: "file" | "media" | "export";
  readonly id: string;
  readonly label: string | null;
  readonly fileName: string | null;
  readonly mediaType: string | null;
  readonly byteLength: number | null;
  readonly download: {
    readonly kind: "unavailable";
    readonly reason: "notReady" | "unavailable";
    readonly detail: string;
  };
}

export interface ResourceLineDownloadIncompatibleDescriptorDiagnostics {
  readonly kind: "file" | "media" | "export";
  readonly id: string;
  readonly label: string | null;
  readonly fileName: string | null;
  readonly mediaType: string | null;
  readonly byteLength: number | null;
  readonly download: {
    readonly kind: "incompatible";
    readonly reason: "staleDescriptor" | "transportBoundary";
    readonly detail: string;
  };
}

export type ResourceLineDownloadDescriptorDiagnostics =
  | ResourceLineDownloadReadyDescriptorDiagnostics
  | ResourceLineDownloadUnavailableDescriptorDiagnostics
  | ResourceLineDownloadIncompatibleDescriptorDiagnostics;

export interface ResourceLineDownloadDiagnostics {
  readonly count: number;
  readonly readyCount: number;
  readonly unavailableCount: number;
  readonly incompatibleCount: number;
  readonly descriptors: readonly ResourceLineDownloadDescriptorDiagnostics[];
}

export interface ResourceLine<TParams = unknown, TValue = SignalValue> {
  value(): TValue;
  signal(): ComputedSignalHandle<TValue>;
  summarySignal(): ComputedSignalHandle<ResourceLineSummary<TParams>>;
  descriptor(): ResourceLineDescriptor<TParams>;
  request(): ResourceRequestDescriptor<TParams>;
  summary(): ResourceLineSummary<TParams>;
  download(): ResourceLineDownload;
  history(): ResourceLineHistory;
  processing(): ResourceLineProcessing;
  upload(): ResourceLineUpload;
  diagnostics(): ResourceLineDiagnostics;
  diagnosticsSummary(): ResourceLineDiagnosticsSummary;
  mutationResponse(): ResourceMutationResponsePlan | null;
  free(): void;
  invalidate(): ResourceLineFreshness;
  refresh(): ResourceLineStatus;
  revalidate(): ResourceLineStatus;
  awaitSettlement(
    options?: ResourceLineAwaitSettlementOptions,
  ): Promise<ResourceLineAwaitSettlementResult<TParams, TValue>>;
  execute(
    options?: ResourceLineExecutionOptions,
  ): ResourceLineExecution<TParams, TValue>;
  [Symbol.dispose](): void;
  status(): ResourceLineStatus;
  freshness(): ResourceLineFreshness;
  view<TView = SignalValue>(
    project: (value: TValue) => TView,
  ): ResourceLineView<TView>;
}

export interface ResourceLineView<TValue = SignalValue> extends Signal<TValue> {}

export type {
  ResourceLineEffectRollbackResult,
  ResourceLineHistory,
  ResourceLineHistoryAvailability,
} from "./resource_line_history.js";
export type {
  ResourceLineBasisDiagnostics,
  ResourceLineDiagnostics,
  ResourceLineVisibleSelection,
} from "./resource_line_diagnostics.js";
export type {
  ResourceLineDiagnosticsSummary,
  ResourceLineSummary,
} from "./resource_line_summary.js";
