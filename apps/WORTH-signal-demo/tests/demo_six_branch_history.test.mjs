import assert from "node:assert/strict";
import test from "node:test";
import { Worker as NodeWorker } from "node:worker_threads";

import { createGearScenario } from "../src/local-truth-gear/gear_scenario.ts";

test("Demo 6 preserves successive caller-supplied slider values without substitution", async () => {
  await withGearScenario(async (scenario) => {
    await scenario.forkDesignBranch();

    for (const thickness of [0.22, 0.48, 0.94, 1.18]) {
      const view = await scenario.commitBranchPatch("main", { thickness });
      assert.equal(view.main.thickness, thickness);
      assert.equal(view.design.thickness, 0.58);
    }

    for (const teeth of [11, 17, 29, 35]) {
      const view = await scenario.commitBranchPatch("design", { teeth });
      assert.equal(view.design.teeth, teeth);
      assert.equal(view.main.teeth, 18);
    }
  });
});

test("Demo 6 inspects any retained commit without moving its live head", async () => {
  await withGearScenario(async (scenario) => {
    await scenario.forkDesignBranch();
    await scenario.commitBranchPatch("main", { thickness: 0.42 });
    const live = await scenario.commitBranchPatch("main", { thickness: 0.88 });
    const earlier = live.history.find(({ title }) => title === "thickness → 0.42");
    const liveHead = live.history.find(({ title }) => title === "thickness → 0.88");
    assert.ok(earlier);
    assert.ok(liveHead?.isLiveHead);

    const inspected = await scenario.selectHistoryCommit(earlier.branchId, earlier.id);
    assert.equal(inspected.main.thickness, 0.42);
    assert.equal(inspected.historySelection.commitId, earlier.id);
    assert.equal(inspected.history.find(({ id }) => id === liveHead.id).isLiveHead, true);
    await assert.rejects(
      () => scenario.commitBranchPatch("main", { thickness: 0.5 }),
      /Select a live history head/u,
    );

    const resumed = await scenario.selectHistoryCommit(liveHead.branchId, liveHead.id);
    assert.equal(resumed.historySelection, null);
    assert.equal(resumed.main.thickness, 0.88);
  });
});

test("Demo 6 inspects the design branch without moving either live head", async () => {
  await withGearScenario(async (scenario) => {
    await scenario.forkDesignBranch();
    await scenario.commitBranchPatch("design", { teeth: 26 });
    const live = await scenario.commitBranchPatch("design", { teeth: 31 });
    const earlierDesignCommit = live.history.find(({ title }) => title === "teeth → 26");
    assert.ok(earlierDesignCommit);

    const inspected = await scenario.selectHistoryCommit(
      earlierDesignCommit.branchId,
      earlierDesignCommit.id,
    );
    assert.equal(inspected.design.teeth, 26);
    assert.equal(inspected.main.teeth, 18);
    assert.equal(inspected.historySelection.commitId, earlierDesignCommit.id);
    assert.equal(inspected.history[0].kind, "mutation");
    assert.equal(inspected.history[0].isLiveHead, true);
  });
});

async function withGearScenario(run) {
  const previousWorker = globalThis.Worker;
  globalThis.Worker = NodeWorker;
  const scenario = await createGearScenario();
  try {
    await run(scenario);
  } finally {
    await scenario.terminate();
    globalThis.Worker = previousWorker;
  }
}
