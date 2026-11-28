import assert from "node:assert/strict";
import test from "node:test";

import { createRealRequestRuntime } from "../../../../resource_runtime/runtime_fixture/real_request_runtime.mjs";
import { withSignals } from "../../action_execution_test_helpers.mjs";
import {
  createCommittedVisibleSelection,
  createHistory,
  createRequest,
  createVerificationPackage,
} from "../fixtures/resource_line_fixture_shared.mjs";
import { createDetailPatchLineFixture } from "../fixtures/resource_line_fixture.mjs";

test("signals.form reports preserved remote resource drift when no local draft is active", async () => {
  await withSignals((signals) => {
    const line = createMutableRemoteResourceLineFixture({
      initialValue: { title: "Loaded" },
    });
    const form = signals.form({
      source: signals.form.source.resourceLine(line, { id: "resource-drift-preserved" }),
      fields: ({ field }) => ({
        title: field("title"),
      }),
    });

    form.resourceDrift();
    line.applyRemoteValue({ title: "Remote change" });

    const report = form.resourceDrift();
    assert.equal(report.summary.status, "preserved");
    assert.equal(report.current?.hadLocalDraft, false);
    assert.deepEqual(form.source(), { title: "Remote change" });
    assert.deepEqual(form.effective(), { title: "Remote change" });
    assert.equal(report.history.length, 1);
    assert.equal(form.diagnostics().resourceDrift.digest, report.digest);
    assert.equal(form.verification().digests.resourceDriftDigest, report.digest);
    assert.equal(form.verification().performanceEnvelope.resourceDrift.observedChanges, 1);
  });
});

test("signals.form reports rebased remote resource drift when local draft truth remains preserved", async () => {
  await withSignals((signals) => {
    const line = createMutableRemoteResourceLineFixture({
      initialValue: { title: "Loaded" },
    });
    const form = signals.form({
      source: signals.form.source.resourceLine(line, { id: "resource-drift-rebased" }),
      fields: ({ field }) => ({
        title: field("title"),
      }),
    });

    form.resourceDrift();
    form.fields.title.set("Local draft");
    line.applyRemoteValue({ title: "Remote change" });

    const report = form.resourceDrift();
    assert.equal(report.summary.status, "rebased");
    assert.equal(report.current?.hadLocalDraft, true);
    assert.deepEqual(form.source(), { title: "Remote change" });
    assert.deepEqual(form.draft(), { title: "Local draft" });
    assert.deepEqual(form.effective(), { title: "Local draft" });
    assert.equal(report.current?.blockers.length, 0);
    assert.equal(typeof form.verification().digests.resourceDriftHistoryDigest, "string");
  });
});

test("signals.form does not overclaim rebased remote drift without admitted merged branch proof", async () => {
  await withSignals((signals) => {
    const line = createMutableRemoteResourceLineFixture({
      initialValue: { title: "Loaded" },
    });
    const form = signals.form({
      source: signals.form.source.resourceLine(line, { id: "resource-drift-preserved-draft" }),
      fields: ({ field }) => ({
        title: field("title"),
      }),
    });

    form.resourceDrift();
    form.fields.title.set("Local draft");
    line.applyRemoteValue(
      { title: "Remote change" },
      {
        visibleSelection: createCommittedVisibleSelection(
          "resource line kept committed remote truth without admitted merge proof",
        ),
      },
    );

    const report = form.resourceDrift();
    assert.equal(report.summary.status, "preserved");
    assert.equal(report.current?.hadLocalDraft, true);
    assert.match(report.current?.reason ?? "", /without admitted resource branch rebase proof/);
  });
});

test("signals.form reports blocked remote resource drift when source schema compatibility cannot admit the local draft", async () => {
  await withSignals((signals) => {
    let schemaVersion = "v1";
    const line = createMutableRemoteResourceLineFixture({
      initialValue: { title: "Loaded" },
    });
    const form = signals.form({
      source: {
        value: signals.form.source.resourceLine(line, { id: "resource-drift-blocked" }),
        schemaVersion: () => schemaVersion,
      },
      fields: ({ field }) => ({
        title: field("title"),
      }),
    });

    form.resourceDrift();
    form.fields.title.set("Local draft");
    schemaVersion = "v2";
    line.applyRemoteValue({ title: "Remote v2" });

    const report = form.resourceDrift();
    assert.equal(report.summary.status, "blocked");
    assert.equal(report.current?.sourceCompatibilityPosture, "unavailable");
    assert.equal(report.current?.blockers[0]?.kind, "schema:drift");
    assert.equal(
      form.readiness().blockers.some((blocker) => blocker.kind === "schema:drift"),
      true,
    );
  });
});

function createMutableRemoteResourceLineFixture({ initialValue, effectProfile = null }) {
  let value = { ...initialValue };
  let version = 1;
  let currentStatus = Object.freeze({ kind: "fulfilled", operation: "initialLoad" });
  let currentVisibleSelection = createCommittedVisibleSelection(
    "resource line is showing committed server truth",
  );
  const request = createRequest(effectProfile);
  const freshness = Object.freeze({ kind: "fresh" });

  return Object.freeze({
    value: () => ({ ...value }),
    descriptor: () => ({
      family: { kind: "detail", familyId: "task" },
      canonicalParams: { params: { id: "t1" }, canonicalKey: "id=t1" },
      runtimeLineId: "task:t1",
      scopeId: "workspace",
    }),
    request: () => request,
    summary: () => createSummary({
      currentStatus,
      freshness,
      version,
      visibleSelection: currentVisibleSelection,
      request,
    }),
    diagnosticsSummary() {
      return this.summary().diagnostics;
    },
    status: () => currentStatus,
    freshness: () => freshness,
    mutationResponse: () => null,
    history() {
      return createHistory(createVerificationPackage({
        request,
        status: currentStatus,
        freshness,
        visibleSelection: currentVisibleSelection,
        patchCount: 0,
        lastEffect: null,
        mutationResponse: null,
      }));
    },
    applyRemoteValue(nextValue, options = {}) {
      value = { ...nextValue };
      version += 1;
      currentStatus = Object.freeze({ kind: "fulfilled", operation: "refresh" });
      currentVisibleSelection = options.visibleSelection ?? Object.freeze({
        kind: "merged",
        source: "refresh",
        effectId: null,
        branchId: 0,
        snapshotId: null,
        basisId: "basis-2",
        detail: "resource line visible truth advanced after a remote refresh",
      });
    },
  });
}

function createSummary({ currentStatus, freshness, version, visibleSelection, request }) {
  return {
    current: {
      status: currentStatus,
      freshness,
      hasVisibleValue: true,
      visibleValueVersion: version,
      visibleSelection,
    },
    request,
    processing: { kind: "ready", completionKind: "none", jobId: null, message: null },
    upload: { kind: "ready", transportKind: "none", uploadId: null, descriptor: null, finalizeRequired: false, awaitingProcessing: false, message: null },
    download: { count: 0, readyCount: 0, unavailableCount: 0, incompatibleCount: 0, descriptors: [] },
    diagnostics: {
      current: {
        status: currentStatus,
        freshness,
        hasVisibleValue: true,
        visibleValueVersion: version,
        visibleSelection,
      },
      activity: {
        lastOperation: currentStatus.operation,
        lastOutcome: currentStatus.kind,
        pendingOperation: null,
        continuity: "preserveVisibleValue",
        freshnessPolicy: "stable",
      },
      counts: {
        refreshCount: currentStatus.operation === "refresh" ? 1 : 0,
        revalidateCount: 0,
        retryAttemptCount: 0,
        rejectionCount: 0,
        timeoutCount: 0,
        supersessionCount: 0,
        invalidationCount: 0,
        patchCount: 0,
        deliveryCount: 0,
        basisAdvanceCount: 1,
      },
      latest: {
        basisCurrentId: visibleSelection.basisId ?? "basis-1",
        effect: null,
        errorMessage: null,
      },
      request: {
        method: request.method,
        effects: request.effects,
      },
      processing: { kind: "ready", completionKind: "none", jobId: null, message: null },
      upload: { kind: "ready", transportKind: "none", uploadId: null, descriptor: null, finalizeRequired: false, awaitingProcessing: false, message: null },
      download: { count: 0, readyCount: 0, unavailableCount: 0, incompatibleCount: 0, descriptors: [] },
      explainability: { available: false, reason: "not requested" },
    },
    explainability: { available: false, reason: "not requested" },
  };
}
