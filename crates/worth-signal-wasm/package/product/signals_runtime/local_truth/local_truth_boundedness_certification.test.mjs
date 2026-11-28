import assert from "node:assert/strict";
import test from "node:test";

import { loadSignalsModule } from "../module_loading/load_signals_module.mjs";

test("32 branches, 128 entities, and 64 aspects remain sparse and checkpoint-bounded", async () => {
  const loaded = await loadSignalsModule();
  const { declareLocalTruthSchema } = await loaded.importProductModule(
    "local_truth/schema/schema_declaration.js",
  );
  const { createLocalTruthAuthority } = await loaded.importProductModule(
    "local_truth/authority/local_truth_authority.js",
  );
  const aspectIds = Array.from({ length: 64 }, (_, index) => `aspect${index}`);
  const schema = declareLocalTruthSchema({
    id: "certification.maxima",
    aspects: aspectIds.map((field) => ({
      id: field,
      field,
      valueType: "number",
      equivalence: { kind: "exact" },
      costClass: "constant",
    })),
  });
  const initialValue = Object.fromEntries(aspectIds.map((id) => [id, 0]));
  const initialEntities = Object.fromEntries(
    Array.from({ length: 128 }, (_, index) => [`entity${index}`, initialValue]),
  );
  const truth = createLocalTruthAuthority({
    authorityId: "certification-maxima",
    schema,
    initialEntities,
  });
  const main = required(await truth.branch());
  const branches = [main];
  for (let index = 1; index < 32; index += 1) {
    branches.push(required(await truth.forkBranch({
      parentBranchId: main.id,
      expectedParentBasis: main.basis,
      name: `branch-${index}`,
    })));
  }
  for (let index = 1; index < branches.length; index += 1) {
    await commit(truth, branches[index], `seed-${index}`, `entity${index}`, `aspect${index}`, index);
  }

  for (const branch of (await truth.inspect()).branches.filter((entry) => !entry.retired)) {
    assert.equal((await truth.checkpoint(branch.id)).posture, "success");
  }
  const inspection = await truth.inspect();
  assert.equal(inspection.branches.filter((branch) => !branch.retired).length, 32);
  assert.equal(inspection.counters.compactions, 1);
  for (const branch of inspection.branches.filter((entry) => !entry.retired)) {
    const history = required(await truth.history(branch.id));
    assert.equal(history.commits.length, 0);
    assert.equal(history.checkpoint.headCommitId, branch.headCommitId);
  }
});

async function commit(truth, branch, requestId, entityId, aspectId, value) {
  const current = required(await truth.branch(branch.id));
  return required(await truth.commit({
    requestId,
    branchId: branch.id,
    expectedBasis: current.basis,
    operations: [{ entityId, aspectId, value }],
  }));
}

function required(outcome) {
  assert.ok(outcome.posture === "success" || outcome.posture === "advisory", outcome.message);
  return outcome.value;
}
