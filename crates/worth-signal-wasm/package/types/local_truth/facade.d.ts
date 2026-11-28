import type {
  LocalTruthBranchReceipt,
  LocalTruthCommitOutcome,
  LocalTruthCheckpoint,
  LocalTruthHistorySegment,
  LocalTruthHistoricalSnapshot,
  LocalTruthInspection,
  LocalTruthMutationRequest,
  LocalTruthOptions,
  LocalTruthOutcome,
  LocalTruthSignalProjectionReceipt,
} from "./core.js";
import type {
  DeclaredLocalTruthSchema,
  LocalTruthSchemaDeclaration,
} from "./schema.js";

export interface LocalTruthAuthority<T extends object> {
  readonly kind: "typescriptInMemoryLocalTruth";
  readonly schema: DeclaredLocalTruthSchema<T>;
  ready?(): Promise<unknown>;
  inspect(): Promise<LocalTruthInspection<T>>;
  branch(branchId?: string): Promise<LocalTruthOutcome<LocalTruthBranchReceipt>>;
  commit(request: LocalTruthMutationRequest): Promise<LocalTruthOutcome<LocalTruthCommitOutcome>>;
  forkBranch(request: {
    readonly parentBranchId: string;
    readonly expectedParentBasis: LocalTruthBranchReceipt["basis"];
    readonly name: string;
  }): Promise<LocalTruthOutcome<LocalTruthBranchReceipt>>;
  derivation(branchId?: string): Promise<LocalTruthSignalProjectionReceipt>;
  destroyDerivation(branchId: string): Promise<LocalTruthSignalProjectionReceipt>;
  rebuildDerivation(branchId: string): Promise<LocalTruthSignalProjectionReceipt | LocalTruthOutcome<never>>;
  checkpoint(branchId: string): Promise<LocalTruthOutcome<LocalTruthCheckpoint<T>>>;
  history(branchId: string): Promise<LocalTruthOutcome<LocalTruthHistorySegment<T>>>;
  historicalSnapshot(request: {
    readonly branchId: string;
    readonly commitId: string;
  }): Promise<LocalTruthOutcome<LocalTruthHistoricalSnapshot<T>>>;
  terminate(): Promise<void>;
}

export interface LocalTruthFactory {
  <T extends object>(options: LocalTruthOptions<T>): LocalTruthAuthority<T>;
}

export function declareLocalTruthSchema<T extends object>(
  declaration: LocalTruthSchemaDeclaration<T>,
): DeclaredLocalTruthSchema<T>;

export { declareLocalTruthSchema as localTruthSchema };
