import type { DeclaredLocalTruthSchema } from "./schema.js";

declare const localTruthBasisBrand: unique symbol;
declare const localTruthCommitBrand: unique symbol;

export type LocalTruthAuthorityKind = "typescriptInMemoryLocalTruth";
export type LocalTruthSupportPosture = "inMemoryProcessLocal";

export interface LocalTruthBasis {
  readonly artifactFamily: "LocalTruthBasis";
  readonly authorityId: string;
  readonly schemaIdentity: string;
  readonly branchId: string;
  readonly branch: LocalTruthBranchReceipt;
  readonly headCommitId: string;
  readonly snapshotId: string;
  readonly revision: number;
  readonly identityDigest: string;
  readonly [localTruthBasisBrand]: true;
}

export interface LocalTruthBranchReceipt {
  readonly artifactFamily: "LocalTruthBranchReceipt";
  readonly id: string;
  readonly name: string;
  readonly kind: "ordinary";
  readonly parentBranchId: string | null;
  readonly forkCommitId: string;
  readonly forkSnapshotId: string;
  readonly forkRevision: number;
  readonly headCommitId: string;
  readonly snapshotId: string;
  readonly retired: boolean;
  readonly basis: LocalTruthBasis;
  readonly derivation?: LocalTruthSignalProjectionReceipt | null;
}

export interface LocalTruthAspectOperation<T = unknown> {
  readonly entityId: string;
  readonly aspectId: string;
  readonly before?: T;
  readonly after?: T;
  readonly value?: T;
  readonly evidenceDigest?: string;
}

export interface LocalTruthCommit {
  readonly artifactFamily: "LocalTruthCommit";
  readonly id: string;
  readonly integrityDigest: string;
  readonly authorityId: string;
  readonly authorityKind: LocalTruthAuthorityKind;
  readonly schemaIdentity: string;
  readonly branchId: string;
  readonly parentCommitId: string | null;
  readonly beforeSnapshotId: string | null;
  readonly afterSnapshotId: string;
  readonly kind: "genesis" | "mutation";
  readonly operations: ReadonlyArray<LocalTruthAspectOperation>;
  readonly [localTruthCommitBrand]: true;
}

export interface LocalTruthSignalProjectionReceipt {
  readonly artifactFamily: "LocalTruthSignalProjectionReceipt";
  readonly branchId: string;
  readonly commitId: string | null;
  readonly posture: "Current" | "CommittedDerivationPending" | "RebuildRequired" | "Unavailable" | "Failed";
  readonly digest?: string;
  readonly reason?: string;
  readonly binding?: {
    readonly signalBranchId: number | bigint;
    readonly signalBasisDigest: string;
  };
  readonly projectedEntities?: number;
  readonly invalidatedAspects?: number;
}

export interface LocalTruthCommitOutcome {
  readonly artifactFamily: "LocalTruthCommitOutcome";
  readonly commit: LocalTruthCommit;
  readonly derivation: LocalTruthSignalProjectionReceipt | null;
}

export interface LocalTruthCheckpoint<T extends object = Record<string, unknown>> {
  readonly artifactFamily: "LocalTruthCheckpoint";
  readonly authorityId: string;
  readonly schemaIdentity: string;
  readonly branchId: string;
  readonly headCommitId: string;
  readonly snapshotId: string;
  readonly values: Readonly<Record<string, T>>;
  readonly lineage: readonly [];
  readonly locusHeads: ReadonlyArray<readonly [string, string]>;
  readonly compactedCommitCount: number;
  readonly compactedCommitDigests: ReadonlyArray<string>;
  readonly compactedSegmentDigest: string;
  readonly priorCheckpointDigest: string | null;
  readonly digest: string;
}

export interface LocalTruthHistorySegment<T extends object = Record<string, unknown>> {
  readonly artifactFamily: "LocalTruthHistorySegment";
  readonly branchId: string;
  readonly checkpoint: LocalTruthCheckpoint<T> | null;
  readonly fromCommitId: string | null;
  readonly toCommitId: string | null;
  readonly commits: ReadonlyArray<LocalTruthCommit>;
  readonly digest: string;
}

export interface LocalTruthHistoricalSnapshot<T extends object = Record<string, unknown>> {
  readonly artifactFamily: "LocalTruthHistoricalSnapshot";
  readonly authorityId: string;
  readonly schemaIdentity: string;
  readonly branchId: string;
  readonly commitId: string;
  readonly snapshotId: string;
  readonly values: Readonly<Record<string, T>>;
  readonly counters: Readonly<{ readonly visitedCommits: number }>;
  readonly digest: string;
}

export type LocalTruthOutcome<T> =
  | { readonly posture: "success"; readonly value: T }
  | { readonly posture: "advisory"; readonly value: T; readonly advisories: ReadonlyArray<{ readonly code: string; readonly message: string }> }
  | { readonly posture: "denied" | "unavailable" | "failed"; readonly code: string; readonly message: string; readonly evidence: unknown };

export interface LocalTruthMutationRequest {
  readonly requestId: string;
  readonly branchId: string;
  readonly expectedBasis: LocalTruthBasis;
  readonly operations: ReadonlyArray<{
    readonly entityId: string;
    readonly aspectId: string;
    readonly value: unknown;
  }>;
  readonly metadata?: Readonly<Record<string, unknown>>;
}

export interface LocalTruthInspection<T extends object> {
  readonly artifactFamily: "LocalTruthInspection";
  readonly authorityId: string;
  readonly authorityKind: LocalTruthAuthorityKind;
  readonly schemaIdentity: string;
  readonly supportPosture: LocalTruthSupportPosture;
  readonly revision: number;
  readonly branches: ReadonlyArray<LocalTruthBranchReceipt>;
  readonly heads: Readonly<Record<string, LocalTruthBasis>>;
  readonly values: Readonly<Record<string, Readonly<Record<string, T>>>>;
  readonly decisionLog: ReadonlyArray<unknown>;
  readonly counters: Readonly<Record<string, number>>;
  readonly bridgeCounters?: Readonly<{
    readonly roundTrips: number;
    readonly serializedBreadth: number;
  }>;
  readonly digest: string;
}

export interface LocalTruthOptions<T extends object> {
  readonly authorityId: string;
  readonly schema: DeclaredLocalTruthSchema<T>;
  readonly initialEntities: Readonly<Record<string, T>>;
  readonly bindings: ReadonlyArray<{
    readonly entityId: string;
    readonly input: { readonly id: string; value(): T };
    readonly aspectMap: Readonly<Record<string, number>>;
  }>;
}
