declare const concurrentResourceEffectBranchDagCertificationRunBrand: unique symbol;

export interface ConcurrentResourceEffectCertificationEvidence {
  readonly layerProof: Readonly<Record<
    | "nativeCore"
    | "workerBoundary"
    | "resourceProduct"
    | "formsIntegration"
    | "demoFive"
    | "documentation",
    {
      readonly verified: true;
      readonly evidence: readonly string[];
    }
  >>;
  readonly scenarioMatrix: {
    readonly generatedScenarioCount: number;
    readonly minimumEffectCount: number;
    readonly siblings: true;
    readonly singleDependencies: true;
    readonly multiDependencies: true;
    readonly sameLocusConflicts: true;
    readonly retries: true;
    readonly responsePermutations: true;
  };
  readonly parity: {
    readonly matched: true;
    readonly workerFirst: true;
    readonly mainThreadCompatibility: true;
    readonly digest: string;
  };
  readonly performanceEnvelope: {
    readonly fixedAffectedBreadth: true;
    readonly populations: readonly number[];
    readonly counterDigest: string;
  };
  readonly residueReport: {
    readonly liveSettledBranches: 0;
    readonly openEffects: 0;
    readonly pendingAdmissions: 0;
    readonly dependencyIndexKeys: 0;
    readonly locusIndexKeys: 0;
  };
  readonly crashRestore: {
    readonly recoveredWithoutDuplicateCommit: true;
    readonly phases: readonly (
      | "responseRecorded"
      | "projectionRefresh"
      | "branchRetirement"
    )[];
  };
  readonly docsProof: {
    readonly example: true;
    readonly claims: true;
    readonly links: true;
    readonly evidence: readonly string[];
  };
}

export interface ConcurrentResourceEffectBranchDagCertificationRun {
  readonly version: "concurrent-resource-effect-branch-dag-certification-v1";
  readonly status: "sealed";
  readonly evidence: ConcurrentResourceEffectCertificationEvidence;
  readonly evidenceDigest: string;
  readonly [concurrentResourceEffectBranchDagCertificationRunBrand]:
    "concurrentResourceEffectBranchDagCertificationRun";
}

export function sealConcurrentResourceEffectBranchDagCertificationRun(
  evidence: ConcurrentResourceEffectCertificationEvidence,
): ConcurrentResourceEffectBranchDagCertificationRun;
