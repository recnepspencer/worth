import assert from "node:assert/strict";
import test from "node:test";

import { createRealRequestRuntime } from "../runtime_fixture/real_request_runtime.mjs";

test("resource effect closeout matrices certify profile capability rows", async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { effects } = runtime.resource;
    const branchNative = effects.closeoutMatrix(effects.branchNative());
    const pessimistic = effects.closeoutMatrix(effects.pessimistic());

    assert.equal(branchNative.profileName, "branchNative");
    assert.equal(pessimistic.profileName, "pessimistic");
    assert.deepEqual(branchNative.proofLanes, [
      "runtime",
      "typeSurface",
      "diagnosticsHistory",
      "performance",
    ]);
    assert.ok(Object.isFrozen(branchNative));
    assert.ok(Object.isFrozen(branchNative.rows));
    assert.ok(branchNative.rows.every((row) => Object.isFrozen(row)));
    assert.deepEqual(projectCapabilityRows(branchNative), {
      localPatch: "admitted",
      deliveryPatch: "admitted",
      optimisticWrite: "admitted",
      confirmation: "serverCanonical",
      failureRollback: "branchRestoreOrInverse",
      branchRestore: "branchRestoreOrInverse",
      broadReplacement: "admitted",
      diagnosticsHistory: "admitted",
    });
    assert.deepEqual(projectCapabilityRows(pessimistic), {
      localPatch: "admitted",
      deliveryPatch: "admitted",
      optimisticWrite: "unsupported",
      confirmation: "serverCanonical",
      failureRollback: "unavailable",
      branchRestore: "unsupported",
      broadReplacement: "admitted",
      diagnosticsHistory: "admitted",
    });
    assert.deepEqual(
      branchNative.rows.map((row) => projectProofColumns(row)),
      branchNative.rows.map(() => ({
        runtimeProof: true,
        typeSurfaceProof: true,
        diagnosticsHistoryProof: true,
        performanceProof: true,
      })),
    );
    assert.equal(
      branchNative.rows.every((row) =>
        Object.values(row.evidence).every((references) => references.length > 0)
      ),
      true,
    );
    assert.ok(Object.isFrozen(projectEvidenceFor(branchNative, "localPatch")));
    assert.ok(
      Object.isFrozen(projectEvidenceFor(branchNative, "localPatch").runtimeTests),
    );
    assert.throws(
      () =>
        effects.closeoutMatrix({
          name: "fake",
          optimism: "branchSpeculative",
          confirmation: "serverCanonical",
          rollback: "branchRestoreOrInverse",
          rebase: "unavailable",
          preimage: "compactInverse",
        }),
      /requires a profile created with resource\.effects\.\*\(\)/,
    );
  } finally {
    await runtime.cleanup();
  }
});

function projectCapabilityRows(matrix) {
  return Object.fromEntries(
    matrix.rows.map((row) => [row.effectFamily, row.capability]),
  );
}

function projectProofColumns(row) {
  return {
    runtimeProof: row.runtimeProof,
    typeSurfaceProof: row.typeSurfaceProof,
    diagnosticsHistoryProof: row.diagnosticsHistoryProof,
    performanceProof: row.performanceProof,
  };
}

function projectEvidenceFor(matrix, effectFamily) {
  return matrix.rows.find((row) => row.effectFamily === effectFamily).evidence;
}
