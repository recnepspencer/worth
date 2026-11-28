const CERTIFICATION_RUN = Symbol(
  "WORTHSignal.concurrentResourceEffectBranchDagCertificationRun",
);

function sealConcurrentResourceEffectBranchDagCertificationRun(evidence) {
  requireCompleteEvidence(evidence);
  const normalizedEvidence = deeplyFreeze(cloneEvidence(evidence));
  return Object.freeze({
    [CERTIFICATION_RUN]: "concurrentResourceEffectBranchDagCertificationRun",
    version: "concurrent-resource-effect-branch-dag-certification-v1",
    status: "sealed",
    evidence: normalizedEvidence,
    evidenceDigest: canonicalDigest(normalizedEvidence),
  });
}

function requireCompleteEvidence(evidence) {
  requireEvidenceObject(evidence, "certification evidence");
  const layers = requireEvidenceObject(evidence.layerProof, "layer proof");
  for (const layer of [
    "nativeCore",
    "workerBoundary",
    "resourceProduct",
    "formsIntegration",
    "demoFive",
    "documentation",
  ]) {
    const proof = requireEvidenceObject(layers[layer], `${layer} proof`);
    if (proof.verified !== true || !Array.isArray(proof.evidence) || proof.evidence.length === 0) {
      throw incomplete(`${layer} requires named verification evidence`);
    }
  }
  const matrix = requireEvidenceObject(evidence.scenarioMatrix, "scenario matrix");
  if (matrix.generatedScenarioCount < 12 || matrix.minimumEffectCount < 10) {
    throw incomplete("scenario matrix requires at least twelve ten-effect runs");
  }
  requireTrueFields(matrix, [
    "siblings",
    "singleDependencies",
    "multiDependencies",
    "sameLocusConflicts",
    "retries",
    "responsePermutations",
  ]);
  requireExactMembers(matrix.dependencyPolicies, [
    "independent",
    "cancelOnDependencyRejection",
  ], "dependency policies");
  if (!Array.isArray(matrix.seeds) || matrix.seeds.length !== matrix.generatedScenarioCount) {
    throw incomplete("scenario matrix requires every generated seed");
  }
  if (
    !Array.isArray(matrix.effectCounts)
    || matrix.effectCounts.some((count) => count < matrix.minimumEffectCount)
  ) {
    throw incomplete("scenario matrix effect counts do not meet minimum breadth");
  }
  const parity = requireEvidenceObject(evidence.parity, "deployment parity");
  requireTrueFields(parity, ["matched", "workerFirst", "mainThreadCompatibility"]);
  requireNonEmptyString(parity.digest, "parity digest");
  requireNonEmptyString(parity.denialDigest, "denial parity digest");
  const performance = requireEvidenceObject(
    evidence.performanceEnvelope,
    "performance envelope",
  );
  requireTrueFields(performance, ["fixedAffectedBreadth"]);
  if (!Array.isArray(performance.populations) || performance.populations.length < 3) {
    throw incomplete("performance envelope requires at least three populations");
  }
  const counters = requireEvidenceObject(
    performance.exactCounters,
    "performance exact counters",
  );
  for (const field of [
    "openEffectLookupCount",
    "dependencyTraversalCount",
    "affectedEffectCount",
    "affectedLocusCount",
    "reconstructionCount",
    "fallbackBreadth",
  ]) {
    if (!Number.isInteger(counters[field]) || counters[field] < 0) {
      throw incomplete(`${field} must be an exact non-negative counter`);
    }
  }
  const residue = requireEvidenceObject(evidence.residueReport, "residue report");
  for (const field of [
    "liveSettledBranches",
    "openEffects",
    "pendingAdmissions",
    "dependencyIndexKeys",
    "locusIndexKeys",
  ]) {
    if (residue[field] !== 0) throw incomplete(`${field} must be zero`);
  }
  const crash = requireEvidenceObject(evidence.crashRestore, "crash restore");
  requireTrueFields(crash, ["recoveredWithoutDuplicateCommit"]);
  const requiredPhases = new Set([
    "responseRecorded",
    "projectionRefresh",
    "branchRetirement",
    "admissionProjectionCleanup",
    "rejectionNativeRetirement",
  ]);
  for (const phase of crash.phases ?? []) requiredPhases.delete(phase);
  if (requiredPhases.size > 0) {
    throw incomplete(`crash restore is missing: ${[...requiredPhases].join(", ")}`);
  }
  requireTrueFields(evidence.docsProof, ["example", "claims", "links"]);
  if (!Array.isArray(evidence.docsProof.evidence) || evidence.docsProof.evidence.length === 0) {
    throw incomplete("documentation requires named verification evidence");
  }
}

function requireExactMembers(value, expected, label) {
  if (!Array.isArray(value)) throw incomplete(`${label} must be an array`);
  const actual = [...new Set(value)].sort();
  const required = [...expected].sort();
  if (JSON.stringify(actual) !== JSON.stringify(required)) {
    throw incomplete(`${label} must include ${required.join(", ")}`);
  }
}

function requireNonEmptyString(value, label) {
  if (typeof value !== "string" || value.length === 0) {
    throw incomplete(`${label} is required`);
  }
}

function requireEvidenceObject(value, label) {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw incomplete(`${label} must be an object`);
  }
  return value;
}

function requireTrueFields(value, fields) {
  const evidence = requireEvidenceObject(value, fields.join("/"));
  for (const field of fields) {
    if (evidence[field] !== true) throw incomplete(`${field} proof is required`);
  }
}

function incomplete(detail) {
  const error = new TypeError(`concurrent effect certification incomplete: ${detail}`);
  error.name = "ConcurrentResourceEffectCertificationDenial";
  error.code = "incompleteEvidence";
  return error;
}

function cloneEvidence(value) {
  if (Array.isArray(value)) return value.map(cloneEvidence);
  if (!value || typeof value !== "object") return value;
  return Object.fromEntries(
    Object.entries(value).map(([key, entry]) => [key, cloneEvidence(entry)]),
  );
}

function deeplyFreeze(value) {
  if (!value || typeof value !== "object") return value;
  for (const entry of Object.values(value)) deeplyFreeze(entry);
  return Object.freeze(value);
}

function canonicalDigest(value) {
  return JSON.stringify(canonicalize(value));
}

function canonicalize(value) {
  if (Array.isArray(value)) return value.map(canonicalize);
  if (!value || typeof value !== "object") return value;
  return Object.fromEntries(
    Object.keys(value).sort().map((key) => [key, canonicalize(value[key])]),
  );
}

export { sealConcurrentResourceEffectBranchDagCertificationRun };
