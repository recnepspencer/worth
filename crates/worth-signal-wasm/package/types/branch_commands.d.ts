import type { RuntimeBranchHandle } from "./diagnostics.js";
import type { RunSummary, TransactionOp } from "./model.js";

export interface WorkerBranchBasisReceipt {
  readonly branchId: number | bigint;
  readonly branchName: string;
  readonly snapshotId: number | bigint | null;
  readonly nativeHeadGeneration: number | bigint;
  readonly nativeHeadDigest: string;
  readonly authoredGraphGeneration: number | bigint;
  readonly authoredStateDigest: string;
}

export interface WorkerForkBranchRequest {
  readonly name: string;
  readonly parentBranchId: number | bigint;
  readonly expectedParentBasis: WorkerBranchBasisReceipt;
}

export interface WorkerForkBranchReceipt {
  readonly branch: RuntimeBranchHandle;
  readonly parentBasis: WorkerBranchBasisReceipt;
  readonly createdBasis: WorkerBranchBasisReceipt;
}

export interface WorkerApplyTransactionToBranchRequest {
  readonly branchId: number | bigint;
  readonly expectedBasis: WorkerBranchBasisReceipt;
  readonly transactionOps: ReadonlyArray<TransactionOp>;
}

export interface WorkerApplyTransactionToBranchReceipt {
  readonly beforeBasis: WorkerBranchBasisReceipt;
  readonly afterBasis: WorkerBranchBasisReceipt;
  readonly activeBranchIdBefore: number | bigint;
  readonly activeBranchIdAfter: number | bigint;
  readonly runSummary: RunSummary;
}

export type WorkerBranchRetirementReason =
  | "rejected"
  | "superseded"
  | "dependencyCancellation"
  | "projectionRebuild";

export interface WorkerRetireBranchRequest {
  readonly branchId: number | bigint;
  readonly expectedBasis: WorkerBranchBasisReceipt;
  readonly reason: WorkerBranchRetirementReason;
}

export interface WorkerRetireBranchReceipt {
  readonly retiredBranchId: number | bigint;
  readonly parentBranchId: number | bigint;
  readonly terminalBasis: WorkerBranchBasisReceipt;
  readonly closeoutDigest: string;
  readonly reclaimedBranchStateCount: number;
  readonly reclaimedSnapshotStateCount: number;
  readonly reclaimedRuntimeMetaCount: number;
  readonly retainedProofRecordCount: number;
}

export interface WorkerRetireBranchesRequest {
  readonly retirements: ReadonlyArray<WorkerRetireBranchRequest>;
}

export interface WorkerRetireBranchesReceipt {
  readonly retirements: ReadonlyArray<WorkerRetireBranchReceipt>;
}

export interface WorkerCloseoutEffectBranchRequest {
  readonly canonicalTransaction: WorkerApplyTransactionToBranchRequest;
  readonly effectRetirement: WorkerRetireBranchRequest;
  readonly dependencyBasisRetirement: WorkerRetireBranchRequest | null;
}

export interface WorkerCloseoutEffectBranchReceipt {
  readonly canonicalTransaction: WorkerApplyTransactionToBranchReceipt;
  readonly effectRetirement: WorkerRetireBranchReceipt;
  readonly dependencyBasisRetirement: WorkerRetireBranchReceipt | null;
}

declare module "./callable_surface.js" {
  interface CallableSignalHistory {
    worker_branch_basis(branchId: number | bigint): WorkerBranchBasisReceipt | Promise<WorkerBranchBasisReceipt>;
    fork_branch(request: WorkerForkBranchRequest): WorkerForkBranchReceipt | Promise<WorkerForkBranchReceipt>;
    apply_transaction_to_branch(request: WorkerApplyTransactionToBranchRequest): WorkerApplyTransactionToBranchReceipt | Promise<WorkerApplyTransactionToBranchReceipt>;
    retire_branch(request: WorkerRetireBranchRequest): WorkerRetireBranchReceipt | Promise<WorkerRetireBranchReceipt>;
    retire_branches(request: WorkerRetireBranchesRequest): WorkerRetireBranchesReceipt | Promise<WorkerRetireBranchesReceipt>;
    closeout_effect_branch(request: WorkerCloseoutEffectBranchRequest): WorkerCloseoutEffectBranchReceipt | Promise<WorkerCloseoutEffectBranchReceipt>;
  }
}

declare module "./worker_runtime_bridge.js" {
  interface WorkerRuntimeBridge {
    workerBranchBasis(branchId: number | bigint): Promise<WorkerBranchBasisReceipt>;
    forkBranch(request: WorkerForkBranchRequest): Promise<WorkerForkBranchReceipt>;
    applyTransactionToBranch(request: WorkerApplyTransactionToBranchRequest): Promise<WorkerApplyTransactionToBranchReceipt>;
    retireBranch(request: WorkerRetireBranchRequest): Promise<WorkerRetireBranchReceipt>;
    retireBranches(request: WorkerRetireBranchesRequest): Promise<WorkerRetireBranchesReceipt>;
    closeoutEffectBranch(request: WorkerCloseoutEffectBranchRequest): Promise<WorkerCloseoutEffectBranchReceipt>;
  }
  interface WorkerFirstHistoryFacade {
    worker_branch_basis(branchId: number | bigint): Promise<WorkerBranchBasisReceipt>;
    fork_branch(request: WorkerForkBranchRequest): Promise<WorkerForkBranchReceipt>;
    apply_transaction_to_branch(request: WorkerApplyTransactionToBranchRequest): Promise<WorkerApplyTransactionToBranchReceipt>;
    retire_branch(request: WorkerRetireBranchRequest): Promise<WorkerRetireBranchReceipt>;
    retire_branches(request: WorkerRetireBranchesRequest): Promise<WorkerRetireBranchesReceipt>;
    closeout_effect_branch(request: WorkerCloseoutEffectBranchRequest): Promise<WorkerCloseoutEffectBranchReceipt>;
  }
}

declare module "./raw_surface.js" {
  interface SignalHistory {
    worker_branch_basis(branchId: bigint): WorkerBranchBasisReceipt;
    fork_branch(request: WorkerForkBranchRequest): WorkerForkBranchReceipt;
    apply_transaction_to_branch(request: WorkerApplyTransactionToBranchRequest): WorkerApplyTransactionToBranchReceipt;
    retire_branch(request: WorkerRetireBranchRequest): WorkerRetireBranchReceipt;
    retire_branches(request: WorkerRetireBranchesRequest): WorkerRetireBranchesReceipt;
    closeout_effect_branch(request: WorkerCloseoutEffectBranchRequest): WorkerCloseoutEffectBranchReceipt;
  }
  interface SignalWorkerRuntime {
    workerBranchBasis(branchId: bigint): WorkerBranchBasisReceipt;
    forkBranch(request: WorkerForkBranchRequest): WorkerForkBranchReceipt;
    applyTransactionToBranch(request: WorkerApplyTransactionToBranchRequest): WorkerApplyTransactionToBranchReceipt;
    retireBranch(request: WorkerRetireBranchRequest): WorkerRetireBranchReceipt;
    retireBranches(request: WorkerRetireBranchesRequest): WorkerRetireBranchesReceipt;
    closeoutEffectBranch(request: WorkerCloseoutEffectBranchRequest): WorkerCloseoutEffectBranchReceipt;
  }
}
