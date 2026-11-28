declare const WorthSignalResourceEffectProfileBrand: unique symbol;

export type ResourceEffectProfileName =
  | "branchNative"
  | "serverCanonical"
  | "pessimistic"
  | "deliveryAuthoritative"
  | "nonReversible"
  | "sensitive"
  | string;

export type ResourceEffectOptimism = "branchSpeculative" | "none";
export type ResourceEffectConfirmation =
  | "exact"
  | "serverCanonical"
  | "acceptedPendingDelivery";
export type ResourceEffectRollback =
  | "branchRestore"
  | "branchRestoreOrInverse"
  | "unavailable";
export type ResourceEffectRebase = "unavailable";
export type ResourceEffectPreimage =
  | "none"
  | "compactInverse"
  | "digestOnly"
  | "retainedFragment";
export type ResourceEffectCloseoutFamily =
  | "localPatch"
  | "deliveryPatch"
  | "optimisticWrite"
  | "confirmation"
  | "failureRollback"
  | "branchRestore"
  | "broadReplacement"
  | "diagnosticsHistory";
export type ResourceEffectCloseoutProofLane =
  | "runtime"
  | "typeSurface"
  | "diagnosticsHistory"
  | "performance";
export type ResourceEffectCloseoutCapability =
  | "admitted"
  | "unsupported"
  | ResourceEffectConfirmation
  | ResourceEffectRollback
  | ResourceEffectRebase;

export interface ResourceEffectProfile {
  readonly name: ResourceEffectProfileName;
  readonly optimism: ResourceEffectOptimism;
  readonly confirmation: ResourceEffectConfirmation;
  readonly rollback: ResourceEffectRollback;
  readonly rebase: ResourceEffectRebase;
  readonly preimage: ResourceEffectPreimage;
  readonly [WorthSignalResourceEffectProfileBrand]: "resourceEffectProfile";
}

export interface ResourceEffectProfileOptions {
  readonly name: string;
  readonly optimism: ResourceEffectOptimism;
  readonly confirmation: ResourceEffectConfirmation;
  readonly rollback: ResourceEffectRollback;
  readonly rebase: ResourceEffectRebase;
  readonly preimage: ResourceEffectPreimage;
}

export interface ResourceEffectCloseoutMatrixRow {
  readonly effectFamily: ResourceEffectCloseoutFamily;
  readonly capability: ResourceEffectCloseoutCapability;
  readonly runtimeProof: true;
  readonly typeSurfaceProof: true;
  readonly diagnosticsHistoryProof: true;
  readonly performanceProof: true;
  readonly evidence: ResourceEffectCloseoutEvidence;
}

export interface ResourceEffectCloseoutEvidence {
  readonly runtimeTests: readonly string[];
  readonly typeSurface: readonly string[];
  readonly diagnosticsHistory: readonly string[];
  readonly performance: readonly string[];
}

export interface ResourceEffectCloseoutMatrix {
  readonly profileName: ResourceEffectProfileName;
  readonly proofLanes: readonly ResourceEffectCloseoutProofLane[];
  readonly rows: readonly ResourceEffectCloseoutMatrixRow[];
}

export interface ResourceEffects {
  branchNative(): ResourceEffectProfile;
  serverCanonical(): ResourceEffectProfile;
  pessimistic(): ResourceEffectProfile;
  deliveryAuthoritative(): ResourceEffectProfile;
  nonReversible(): ResourceEffectProfile;
  sensitive(): ResourceEffectProfile;
  custom(options: ResourceEffectProfileOptions): ResourceEffectProfile;
  closeoutMatrix(profile: ResourceEffectProfile): ResourceEffectCloseoutMatrix;
}

export const resourceEffects: ResourceEffects;
