import assert from "node:assert/strict";
import { writeFile } from "node:fs/promises";
import path from "node:path";
import { execFileAsync } from "./verify-worth-signals-wasm-package-support.mjs";
import { buildRuntimeSmokeSource } from "./verify-worth-signals-wasm-runtime-smoke-source.mjs";

export async function runRuntimeSmoke(tempDir, packageName) {
  const smokeRuntimePath = path.join(tempDir, "smoke.mjs");
  const source = buildRuntimeSmokeSource(packageName);
  await writeFile(smokeRuntimePath, source, "utf8");
  const { stdout } = await execFileAsync("node", [smokeRuntimePath], { cwd: tempDir });
  const result = JSON.parse(stdout.trim());

  assert.equal(result.hasInit, true);
  assert.equal(result.hasCreateSignals, true);
  assert.equal(result.reactKeys.includes("createReactSignalsStore"), true);
  assert.equal(result.reactKeys.includes("useSignalValue"), true);
  assert.equal(result.doubled, 4);
  assert.equal(result.visibleLabel, "hidden");
  assert.equal(result.viewportLabel, "1440x900");
  assert.equal(result.onlineLabel, "offline");
  assert.equal(result.clockLabel, 7);
  assert.equal(result.persistenceLabel, 2);
  assert.equal(result.visibilityState, "hidden");
  assert.deepEqual(result.localDraftValue, {
    title: "Ready to ship",
    done: true,
    status: "queued",
  });
  assert.equal(result.nameValue, "Ada");
  assert.notEqual(result.nameOpaqueId, "name");
  assert.equal(result.firstShippingOptionId, "ground");
  assert.equal(result.firstShippingOptionDebugName, "firstShippingOption");
  assert.equal(result.preservedShippingAfterSourceChangeId, "air");
  assert.equal(result.preservedShippingAfterRelinkId, "air");
  assert.equal(result.preservedShippingAfterResetId, "air");
  assert.equal(result.firstShippingAfterResetId, "sea");
  assert.equal(result.firstShippingAfterRelinkId, "sea");
  assert.equal(result.preservedShippingAfterFallbackRelinkId, "sea");
  assert.equal(result.linkedSelectionAfterGraphResetId, "ready");
  assert.equal(result.linkedRevisionAfterSecondGraphResetId, "review");
  assert.equal(result.namingGraphInputId, result.nameOpaqueId);
  assert.equal(result.namingGraphOutputId, "naming.publicDisplayName");
  assert.notEqual(result.displayLabelOpaqueId, "displayLabel");
  assert.equal(result.namingGraphDescriptor.sourceId, result.displayLabelOpaqueId);
  assert.equal(result.namingGraphDescriptor.outputName, "publicDisplayName");
  assert.equal(result.namingGraphCompatibilityOutputId, "naming.publicDisplayName");
  assert.deepEqual(
    result.requirednessInputDescriptors.map((descriptor) => ({
      inputName: descriptor.inputName,
      authority: descriptor.authority,
      requiredness: descriptor.requiredness,
    })),
    [
      { inputName: "serverValue", authority: "readOnly", requiredness: "required" },
      { inputName: "draftValue", authority: "writable", requiredness: "optional" },
    ],
  );
  assert.equal(result.requirednessServerAuthority.requiredness, "required");
  assert.equal(result.requirednessDraftAuthority.requiredness, "optional");
  assert.equal(result.itemDetailGraphId, "itemDetail");
  assert.equal(result.itemDetailGraphInputNames.includes("serverItemData"), true);
  assert.equal(result.itemDetailGraphOutputNames.includes("submitReadiness"), true);
  assert.equal(result.itemDetailGraphInputKeys.includes("serverItemData"), true);
  assert.equal(result.itemDetailGraphReadKeys.includes("submitReadiness"), true);
  assert.equal(result.itemDetailGraphOperationalWriteId, "itemDetail.editSession.serverItemData");
  assert.equal(result.itemDetailGraphOperationalPatchId, "itemDetail.editSession.draftEdits");
  assert.equal(result.itemDetailGraphOperationalAuthority, "writable");
  assert.equal(result.itemDetailGraphOperationalSupportsPatch, true);
  assert.equal(result.itemDetailGraphOperationalServerState, "done");
  assert.equal(result.itemDetailGraphOperationalDraftTitle, "Ready to ship");
  assert.equal(result.itemDetailGraphOperationalDraftReviewState, "approved");
  assert.deepEqual(result.itemDetailGraphResetDraftKeys, []);
  assert.equal(result.itemDetailGraphResetServerValue, null);
  assert.deepEqual(result.row0ScopePath, ["rows", "rows.row-0"]);
  assert.equal(typeof result.row0ScopeParent, "string");
  assert.equal(result.row0SignalCanonicalId, "rows.row-0.count");
  assert.equal(result.row0SignalGraphId, null);
  assert.equal(result.row0SignalRootScopeId, "rows");
  assert.equal(result.row1SignalCanonicalId, "rows.row-1.count");
  assert.equal(result.row0HandleId, "rows.row-0.count");
  assert.equal(result.row1HandleId, "rows.row-1.count");
  assert.equal(result.itemDetailGraphCompatibilityInputId, "itemDetail.editSession.serverItemData");
  assert.equal(result.itemDetailGraphCompatibilityContractInputId, "itemDetail.editSession.serverItemData");
  assert.equal(result.itemDetailGraphCompatibilityOutputId, "itemDetail.submitReadiness");
  assert.equal(result.itemDetailGraphWhyId, "itemDetail.submitReadiness");
  assert.equal(result.itemDetailGraphInputWhyId, "itemDetail.editSession.serverItemData");
  assert.equal(result.itemDetailGraphReplayFrameCount >= 1, true);
  assert.equal(result.itemDetailGraphInputReplayFrameCount >= 1, true);
  assert.deepEqual(
    result.itemDetailGraphDependencyInputNames.sort(),
    ["draftEdits", "serverItemData"],
  );
  assert.deepEqual(
    result.itemDetailGraphDependencySourceIds.sort(),
    [
      "itemDetail.editSession.draftEdits",
      "itemDetail.editSession.serverItemData",
    ],
  );
  assert.equal(result.itemDetailGraphContractSummaryOutputCount, 3);
  assert.deepEqual(
    result.itemDetailGraphContractDeltaAddedOutputs,
    ["effectiveItemData", "dirtyState", "submitReadiness"],
  );
  assert.equal(result.importedItemDetailGraphOutputId, "itemDetail.submitReadiness");
  assert.equal(result.importedItemDetailGraphReadiness, null);
  assert.equal(result.importedItemDetailGraphInputCount, null);
  assert.equal(result.importedItemDetailGraphContractInputId, "itemDetail.editSession.serverItemData");
  assert.equal(result.importedItemDetailGraphHistoryChanged, false);
  assert.equal(result.importedItemDetailGraphHistoryRestoreMode, "SameRuntimeExact");
  assert.equal(result.itemDetailGraphImportPortableMode, "Denied");
  assert.equal(result.itemDetailGraphImportHydrateMode, "Deferred");
  assert.equal(result.importedItemDetailGraphExactRestoreMode, "SameRuntimeExact");
  assert.equal(result.pageModalPageInputId, "itemWorkspace.page.serverItemData");
  assert.equal(result.pageModalModalInputId, "itemWorkspace.modal.serverItemData");
  assert.equal(result.pageModalPageOutputId, "itemWorkspace.pageEffectiveItemData");
  assert.equal(result.pageModalModalOutputId, "itemWorkspace.modalEffectiveItemData");
  assert.equal(result.taskEditorGraphInputId, "taskEditor.form.serverValue");
  assert.equal(result.taskEditorGraphRouteParamsId, "taskEditor.resource.routeParams");
  assert.equal(result.taskEditorGraphOutputId, "taskEditor.submitAvailability");
  assert.equal(result.taskEditorGraphPatchId, "taskEditor.form.draftValue");
  assert.equal(result.taskEditorGraphPatchedStatus, "published");
  assert.equal(result.taskEditorGraphRouteParamTaskId, "task-8");
  assert.equal(result.taskEditorGraphInputWhyId, "taskEditor.resource.routeParams");
  assert.equal(result.taskEditorGraphOutputWhyId, "taskEditor.submitAvailability");
  assert.equal(result.taskEditorGraphInputReplayFrameCount >= 1, true);
  assert.equal(result.taskEditorGraphOutputReplayFrameCount >= 1, true);
  assert.equal(result.taskEditorGraphCompatibilityInputId, "taskEditor.resource.routeParams");
  assert.equal(result.taskEditorGraphCompatibilityOutputId, "taskEditor.submitAvailability");
  assert.equal(result.authorityGraphReadOnlyAuthority, "readOnly");
  assert.equal(result.authorityGraphImportedAuthority, "imported");
  assert.equal(result.authorityGraphWritablePatchId, "taskAuthority.authority.draftValue");
  assert.equal(result.authorityGraphInputId, "taskAuthority.authority.serverValue");
  assert.equal(result.authorityGraphOutputId, "taskAuthority.effectiveValue");
  assert.equal(result.authorityGraphDraftTitle, "Queued");
  assert.equal(result.authorityGraphTaskId, "task-7");
  assert.equal(result.authorityGraphInputWhyId, "taskAuthority.authority.externalParams");
  assert.equal(result.authorityGraphOutputReplayFrameCount >= 1, true);
  assert.equal(result.authorityGraphCompatibilityInputId, "taskAuthority.authority.externalParams");
}
