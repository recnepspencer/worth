import assert from "node:assert/strict";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { stripTypeScriptTypes } from "node:module";
import test from "node:test";
import { fileURLToPath } from "node:url";

const reactDir = path.dirname(fileURLToPath(import.meta.url));

async function loadActionsModule() {
  const tempDir = await mkdtemp(path.join(tmpdir(), "worth-signal-react-form-actions-"));
  try {
    const source = await readFile(path.join(reactDir, "signals_form_actions.ts"), "utf8");
    await writeFile(
      path.join(tempDir, "signals_form_actions.js"),
      stripTypeScriptTypes(source, { mode: "transform" }),
      "utf8",
    );
    const moduleUrl = new URL(
      `file:///${path.join(tempDir, "signals_form_actions.js").replace(/\\/g, "/")}`,
    );
    const loaded = await import(moduleUrl.href);
    return { ...loaded, cleanup: () => rm(tempDir, { recursive: true, force: true }) };
  } catch (error) {
    await rm(tempDir, { recursive: true, force: true });
    throw error;
  }
}

function createCountingController() {
  const calls = { actionPlan: [], debugAction: [], executeAction: [] };
  const controller = {
    actionPlan(actionId) {
      calls.actionPlan.push(actionId);
      return { id: actionId, status: "accepted", readiness: { canRun: true } };
    },
    debugAction(actionId) {
      calls.debugAction.push(actionId);
      return { action: actionId, pending: false, latestExecution: { resultKind: "fulfilled" } };
    },
    executeAction(actionId) {
      calls.executeAction.push(actionId);
      return { action: actionId, resultKind: "pending" };
    },
  };
  return { controller, calls };
}

test("lazy action bindings build only the actions a render reads, once each", async () => {
  const { createLazyActionBindings, cleanup } = await loadActionsModule();
  try {
    const { controller, calls } = createCountingController();
    const actions = createLazyActionBindings(controller, ["approve", "archive", "reject"]);

    assert.deepEqual(Object.keys(actions), ["approve", "archive", "reject"]);
    assert.ok(Object.isFrozen(actions));
    assert.deepEqual(calls.debugAction, []);
    assert.deepEqual(calls.actionPlan, []);

    const approve = actions.approve;
    assert.equal(approve, actions.approve);
    assert.deepEqual(calls.debugAction, ["approve"]);
    assert.deepEqual(calls.actionPlan, ["approve"]);
    assert.equal(approve.disabled, false);
    assert.equal(approve.pending, false);
    assert.equal(approve.resultKind, "fulfilled");

    assert.deepEqual(approve.execute(), { action: "approve", resultKind: "pending" });
    assert.deepEqual(calls.executeAction, ["approve"]);
    assert.deepEqual(calls.debugAction, ["approve"]);

    // Enumerating (for example spreading into props) builds the rest, once.
    const entries = Object.entries(actions);
    assert.equal(entries.length, 3);
    assert.deepEqual(calls.debugAction, ["approve", "archive", "reject"]);
    assert.equal(actions.reject.plan.id, "reject");
    assert.deepEqual(calls.debugAction, ["approve", "archive", "reject"]);
  } finally {
    await cleanup();
  }
});

test("readActionBinding disables an action that is pending or whose plan is not runnable", async () => {
  const { readActionBinding, cleanup } = await loadActionsModule();
  try {
    const pendingController = {
      actionPlan: () => ({ status: "accepted", readiness: { canRun: true } }),
      debugAction: () => ({ pending: true, latestExecution: { resultKind: "pending" } }),
      executeAction: () => null,
    };
    const pending = readActionBinding(pendingController, "approve");
    assert.equal(pending.disabled, true);
    assert.equal(pending.resultKind, "pending");

    const deniedController = {
      actionPlan: () => ({ status: "denied", readiness: { canRun: false } }),
      debugAction: () => ({ pending: false, latestExecution: null }),
      executeAction: () => null,
    };
    const denied = readActionBinding(deniedController, "approve");
    assert.equal(denied.disabled, true);
    assert.equal(denied.resultKind, null);
  } finally {
    await cleanup();
  }
});
