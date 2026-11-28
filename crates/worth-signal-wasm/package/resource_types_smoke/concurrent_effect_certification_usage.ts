import {
  sealConcurrentResourceEffectBranchDagCertificationRun,
  type ConcurrentResourceEffectCertificationEvidence,
} from "../index.js";

const verified = (evidence: readonly string[]) => ({
  verified: true as const,
  evidence,
});

const certificationEvidence = {
  layerProof: {
    nativeCore: verified(["native"]),
    workerBoundary: verified(["worker"]),
    resourceProduct: verified(["resource"]),
    formsIntegration: verified(["forms"]),
    demoFive: verified(["demo-five"]),
    documentation: verified(["docs"]),
  },
  scenarioMatrix: {
    generatedScenarioCount: 12,
    minimumEffectCount: 10,
    siblings: true,
    singleDependencies: true,
    multiDependencies: true,
    sameLocusConflicts: true,
    retries: true,
    responsePermutations: true,
  },
  parity: {
    matched: true,
    workerFirst: true,
    mainThreadCompatibility: true,
    digest: "parity",
  },
  performanceEnvelope: {
    fixedAffectedBreadth: true,
    populations: [4, 12, 24],
    counterDigest: "counters",
  },
  residueReport: {
    liveSettledBranches: 0,
    openEffects: 0,
    pendingAdmissions: 0,
    dependencyIndexKeys: 0,
    locusIndexKeys: 0,
  },
  crashRestore: {
    recoveredWithoutDuplicateCommit: true,
    phases: [
      "responseRecorded",
      "projectionRefresh",
      "branchRetirement",
    ],
  },
  docsProof: {
    example: true,
    claims: true,
    links: true,
    evidence: ["documentation-test"],
  },
} as const satisfies ConcurrentResourceEffectCertificationEvidence;

const certificationRun =
  sealConcurrentResourceEffectBranchDagCertificationRun(certificationEvidence);

void certificationRun.evidenceDigest;
void certificationRun.evidence.scenarioMatrix.generatedScenarioCount;
