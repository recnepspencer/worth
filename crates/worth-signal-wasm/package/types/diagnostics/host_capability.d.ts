import type { SignalValue } from "../model.js";
import type {
  HostCapabilityCompatibility,
} from "./policy.js";

export interface HostCapabilityReadSummary {
  family: string;
  registrationId: string;
  compatibility: HostCapabilityCompatibility;
}

export type HostCapabilityEventKind =
  | "InvalidationFlushed"
  | "InvalidationNoOpSuppressed"
  | "InvalidationIgnoredStale"
  | "PortableImportDenied";

export type HostCapabilityInvalidationMode = "push-driven" | "polled" | "manually-committed";

export type HostCapabilityPortableImportOutcome =
  | "Live"
  | "Reattached"
  | "Unavailable"
  | "Incompatible"
  | "Denied";

export interface HostCapabilityTransportArtifact {
  family: string;
  registrationId: string;
  compatibility: HostCapabilityCompatibility;
  exactRestoreOutcome: "Live" | "Reattached" | "Unavailable" | "Incompatible";
  portableImportOutcome: HostCapabilityPortableImportOutcome;
  portableImportReason: string;
}

export interface HostCapabilityDiagnosticsEvent {
  sequence: number;
  kind: HostCapabilityEventKind;
  family: string;
  registrationId: string;
  compatibility: HostCapabilityCompatibility;
  invalidationMode: HostCapabilityInvalidationMode | null;
  queuedInvalidationCount: number;
  previousState: SignalValue | null;
  nextState: SignalValue | null;
  touchedNodes: number;
  reevaluatedNodes: number;
  portableImportOutcome?: HostCapabilityPortableImportOutcome;
  portableImportReason?: string;
  deniedCallbackIds?: ReadonlyArray<string>;
}

export interface HostCapabilityDiagnosticsFamilyReport {
  family: string;
  eventCount: number;
  latestKind: HostCapabilityEventKind | null;
  latestCompatibility: HostCapabilityCompatibility | null;
  invalidationModes: ReadonlyArray<HostCapabilityInvalidationMode>;
  maxQueuedInvalidationCount: number;
  maxTouchedNodes: number;
  maxReevaluatedNodes: number;
  deniedCallbackIds: ReadonlyArray<string>;
}

export interface HostCapabilityLineageEntry {
  sequence: number;
  family: string;
  registrationId: string;
  kind: HostCapabilityEventKind | null;
  compatibility: HostCapabilityCompatibility | null;
  invalidationMode: HostCapabilityInvalidationMode | null;
  queuedInvalidationCount: number;
  touchedNodes: number;
  reevaluatedNodes: number;
  portableImportOutcome: HostCapabilityPortableImportOutcome | null;
  deniedCallbackIds: ReadonlyArray<string>;
}

export interface HostCapabilityBreadthFamilyReport {
  family: string;
  eventCount: number;
  maxQueuedInvalidationCount: number;
  maxTouchedNodes: number;
  maxReevaluatedNodes: number;
}

export interface HostCapabilityBreadthReport {
  maxQueuedInvalidationCount: number;
  maxTouchedNodes: number;
  maxReevaluatedNodes: number;
  families: ReadonlyArray<HostCapabilityBreadthFamilyReport>;
}

export interface HostCapabilityDiagnosticsReport {
  totals: {
    registrationCount: number;
    disposalCount: number;
    readCount: number;
    pollCount: number;
    noOpPollCount: number;
    manualCommitCount: number;
    noOpManualCommitCount: number;
    invalidationCount: number;
    invalidationBatchFlushCount: number;
    reevaluationCount: number;
    invalidationTouchedNodeCount: number;
    noOpInvalidationSuppressedCount: number;
    staleInvalidationIgnoredCount: number;
    compatibilityDenialCount: number;
    unavailabilityArtifactCount: number;
    broadFanoutDenialCount: number;
    retainedEventCount: number;
  };
  lineage: ReadonlyArray<HostCapabilityLineageEntry>;
  lineageDigest: string;
  breadth: HostCapabilityBreadthReport;
  breadthDigest: string;
  families: ReadonlyArray<HostCapabilityDiagnosticsFamilyReport>;
  digest: string;
}

export interface HostCapabilityTransportFamilyReport {
  family: string;
  callbackIds: ReadonlyArray<string>;
  compatibilities: ReadonlyArray<HostCapabilityCompatibility>;
  exactRestoreOutcomes: ReadonlyArray<string>;
  portableImportOutcomes: ReadonlyArray<HostCapabilityPortableImportOutcome>;
  deniedCallbackIds: ReadonlyArray<string>;
  unavailableCallbackIds: ReadonlyArray<string>;
}

export interface HostCapabilityTransportReport {
  totals: {
    unavailableArtifactCount: number;
    transportEntryCount: number;
    deniedFamilyCount: number;
    unavailableFamilyCount: number;
    snapshotPortableFamilyCount: number;
  };
  families: ReadonlyArray<HostCapabilityTransportFamilyReport>;
  digest: string;
}
