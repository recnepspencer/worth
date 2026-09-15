import { createRuntimeLineIdentity } from "../../identity/runtime_line_identity.js";
import { createLineMaterializationRecord } from "../../lowering/line_materialization_record.js";
import { createLineReloadRecord } from "../../lowering/line_reload_record.js";
import { createLineHandle } from "../../lines/line_handle.js";
import { createPatchCapableLineHandle } from "../../lines/line_patch_capable_handle.js";
import { recordLineHistoryEntry } from "../../lines/history/record_line_history_entry.js";
import { createLineHistoryState } from "../../lines/history/line_history_state.js";
import { createLineLifecycleState } from "../../lines/state/line_lifecycle_state.js";
import { createLineBackingRef } from "../../lines/state/line_backing_ref.js";
import { createLineRegistryEntry } from "../../lines/state/line_registry_entry.js";
import { createLineDeliveryState } from "../../lines/state/line_delivery_state.js";
import { createIdentityMigratedDiagnostics } from "../../lines/state/line_identity_migration_diagnostics.js";
import {
  patchLineBindingState,
  readLineBindingState,
  replaceLineBindingState,
} from "../../lines/state/line_binding_state.js";
import { createLineRequestState } from "../../requests/line_request_state.js";
import { recordMutationResponsePlanIfPresent } from "../../mutation/resource_mutation_response_execution.js";
import { createBinding } from "./materialized_family_binding.js";
import { createResolvedRequestDescriptor } from "./resolved_request_descriptor.js";
import { createResourceEffectBranchDag } from "../../effects/runtime/resource_effect_branch_dag.js";

function createMaterializedLine(
  canonicalParamIdentity,
  linesByCanonicalKey,
  familyIdentity,
  familyRecord,
  familyPatchRecord,
  familyScope,
  resourceLineEpoch,
  nextLineCounter,
  initialBindingFactory = null,
  effectProjectionCoordinator,
) {
  let currentCanonicalParamIdentity = canonicalParamIdentity;
  let currentCanonicalKey = canonicalParamIdentity.canonicalKey;
  let registryEntry = null;
  let sharedRequestState = null;
  const sharedLifecycleHistory = createLineHistoryState();

  const lineBacking = createLineBackingRef(
    resourceLineEpoch,
    () => createReplacementMaterialization(),
    snapshotContinuity,
    restoreContinuity,
  );
  resourceLineEpoch.register(lineBacking);
  const releaseCurrentLine = createCurrentLineRelease(
    lineBacking,
    resourceLineEpoch,
    linesByCanonicalKey,
    () => currentCanonicalKey,
  );

  function snapshotContinuity(materialization) {
    try {
      const state = readLineBindingState(materialization.binding);
      return Object.freeze({
        status: state.status,
        freshness: state.freshness,
        diagnostics: state.diagnostics,
      });
    } catch {
      return null;
    }
  }

  function restoreContinuity(materialization, continuitySnapshot) {
    if (continuitySnapshot === null) {
      return;
    }
    patchLineBindingState(materialization.binding, {
      status: continuitySnapshot.status,
      freshness: continuitySnapshot.freshness,
      diagnostics: continuitySnapshot.diagnostics,
    });
  }

  function createReplacementMaterialization(
    requestDescriptorOverride = null,
    requestStateOverride = sharedRequestState,
  ) {
    return createConcreteMaterialization(
      currentCanonicalParamIdentity,
      familyIdentity,
      familyRecord,
      familyPatchRecord,
      familyScope,
      resourceLineEpoch,
      nextLineCounter,
      releaseCurrentLine,
      rematerializeLine,
      migrateLineIdentity,
      sharedLifecycleHistory,
      requestStateOverride,
      requestDescriptorOverride,
      null,
      effectProjectionCoordinator,
    );
  }

  function rematerializeLine({
    requestDescriptorOverride = null,
    requestStateOverride = sharedRequestState,
    invalidateNamespace = false,
  } = {}) {
    if (invalidateNamespace) {
      resourceLineEpoch.advanceVersion();
    }
    return lineBacking.forceRematerialize(
      () =>
        createReplacementMaterialization(
          requestDescriptorOverride,
          requestStateOverride,
        ),
    );
  }

  function migrateLineIdentity(nextCanonicalParamIdentity) {
    const currentMaterialization = lineBacking.current();
    const nextCanonicalKey = nextCanonicalParamIdentity.canonicalKey;
    if (nextCanonicalKey === currentCanonicalKey) {
      return Object.freeze({
        kind: "noop",
        previousCanonicalKey: currentCanonicalKey,
        nextCanonicalKey,
        previousRuntimeLineId: currentMaterialization.lineIdentity.runtimeLineId,
        nextRuntimeLineId: currentMaterialization.lineIdentity.runtimeLineId,
        basisId: currentMaterialization.requestState.currentBasisId(),
        requestPath: currentMaterialization.requestState.readDescriptor().target.requestPath,
      });
    }
    const occupied = linesByCanonicalKey.get(nextCanonicalKey);
    if (occupied !== undefined && occupied !== registryEntry) {
      return Object.freeze({
        kind: "unavailable",
        reason: "destinationOccupied",
        detail:
          `identity migration cannot claim ${nextCanonicalKey} because ${familyIdentity.kind} ${familyIdentity.familyId} already has a resident line for that canonical key`,
      });
    }
    const currentBasisId = currentMaterialization.requestState.currentBasisId();
    const nextRequestDescriptor = createMigratedRequestDescriptor(
      familyIdentity,
      familyRecord,
      nextCanonicalParamIdentity,
      currentBasisId,
    );
    const previousCanonicalKey = currentCanonicalKey;
    const previousRuntimeLineId = currentMaterialization.lineIdentity.runtimeLineId;
    const previousState = readLineBindingState(currentMaterialization.binding);
    const previousVisibleValue = previousState.value;
    const previousCanonicalValue = previousState.canonicalValue;
    currentCanonicalParamIdentity = nextCanonicalParamIdentity;
    currentCanonicalKey = nextCanonicalKey;
    linesByCanonicalKey.delete(previousCanonicalKey);
    linesByCanonicalKey.set(nextCanonicalKey, registryEntry);
    const migratedMaterialization = rematerializeLine({
      requestDescriptorOverride: nextRequestDescriptor,
      requestStateOverride: null,
    });
    if (previousVisibleValue !== null) {
      patchLineBindingState(migratedMaterialization.binding, {
        value: previousVisibleValue,
        canonicalValue: previousCanonicalValue,
      });
    }
    sharedRequestState = migratedMaterialization.requestState;
    const identityMigration = Object.freeze({
      previousCanonicalKey,
      nextCanonicalKey,
      previousRuntimeLineId,
      nextRuntimeLineId: migratedMaterialization.lineIdentity.runtimeLineId,
      basisId: migratedMaterialization.requestState.currentBasisId(),
      requestPath:
        migratedMaterialization.requestState.readDescriptor().target.requestPath,
    });
    patchLineBindingState(migratedMaterialization.binding, {
      diagnostics: createIdentityMigratedDiagnostics(
        readLineBindingState(migratedMaterialization.binding).diagnostics,
        identityMigration,
      ),
    });
    recordLineHistoryEntry(
      migratedMaterialization.lifecycleHistory,
      migratedMaterialization.binding,
      "identityMigrated",
      Object.freeze({
        identityMigration,
      }),
    );
    return Object.freeze({
      kind: "migrated",
      previousCanonicalKey,
      nextCanonicalKey,
      previousRuntimeLineId,
      nextRuntimeLineId: migratedMaterialization.lineIdentity.runtimeLineId,
      basisId: migratedMaterialization.requestState.currentBasisId(),
      requestPath: migratedMaterialization.requestState.readDescriptor().target.requestPath,
    });
  }

  const materialization = lineBacking.forceRematerialize(() =>
    createConcreteMaterialization(
      currentCanonicalParamIdentity,
      familyIdentity,
      familyRecord,
      familyPatchRecord,
      familyScope,
      resourceLineEpoch,
      nextLineCounter,
      releaseCurrentLine,
      rematerializeLine,
      migrateLineIdentity,
      sharedLifecycleHistory,
      sharedRequestState,
      null,
      initialBindingFactory,
      effectProjectionCoordinator,
    ));
  sharedRequestState = materialization.requestState;
  recordLineHistoryEntry(
    materialization.lifecycleHistory,
    materialization.binding,
    "materialized",
  );
  const diagnostics = readLineBindingState(materialization.binding).diagnostics;
  recordMutationResponsePlanIfPresent(
    materialization.lifecycleHistory,
    materialization.binding,
    "lastMutationResponsePlan" in diagnostics
      ? diagnostics.lastMutationResponsePlan
      : null,
  );
  const handle = createMaterializedLineHandle(lineBacking);
  registryEntry = createLineRegistryEntry(lineBacking, handle);
  return registryEntry;
}

function createMaterializedLineHandle(lineBacking) {
  const baseHandle = createLineHandle(lineBacking);
  if (!lineBacking.current().patch.broadReplace) {
    return baseHandle;
  }
  return createPatchCapableLineHandle(baseHandle, lineBacking);
}

function createConcreteMaterialization(
  canonicalParamIdentity,
  familyIdentity,
  familyRecord,
  familyPatchRecord,
  familyScope,
  resourceLineEpoch,
  nextLineCounter,
  release,
  rematerialize,
  migrateIdentity,
  lifecycleHistory,
  requestStateOverride,
  requestDescriptorOverride = null,
  bindingFactoryOverride = null,
  effectProjectionCoordinator,
) {
  const lineCounter = nextLineCounter();
  const lineScope = familyScope.scope(`line${lineCounter}`);
  const lineIdentity = createRuntimeLineIdentity(
    familyIdentity,
    canonicalParamIdentity,
    `${familyIdentity.familyId}.line${lineCounter}`,
    lineScope.scopeId,
    familyRecord.compatibility ?? null,
  );
  const requestDescriptor =
    requestDescriptorOverride
    ?? createResolvedRequestDescriptor(
      lineIdentity,
      familyRecord,
      canonicalParamIdentity.params,
    );
  const lifecycle = createLineLifecycleState();
  const requestState =
    requestStateOverride ?? createLineRequestState(requestDescriptor);
  const bindingFactory = bindingFactoryOverride ?? createBinding;
  return createLineMaterializationRecord(
    lineIdentity,
    requestDescriptor,
    requestState,
    bindingFactory(
      canonicalParamIdentity.params,
      lineScope,
      familyRecord.identity.kind,
      familyRecord.declaration.load,
      familyRecord.policy,
      requestDescriptor,
      lineIdentity,
      familyRecord.declaration.mutationResponse ?? null,
      lifecycle,
      lifecycleHistory,
    ),
    familyScope.history(),
    lifecycleHistory,
    createLineDeliveryState(),
    familyPatchRecord,
    lineScope,
    lifecycle,
    createLineReloadRecord(
      canonicalParamIdentity.params,
      familyRecord.identity.kind,
      familyRecord.declaration.load,
      familyRecord.policy,
      requestState,
      familyRecord.declaration.mutationResponse ?? null,
    ),
    release,
    rematerialize,
    migrateIdentity,
    resourceLineEpoch,
    (record) => createResourceEffectBranchDag(
      record,
      effectProjectionCoordinator,
    ),
    effectProjectionCoordinator,
  );
}

function createCurrentLineRelease(
  lineBacking,
  resourceLineEpoch,
  linesByCanonicalKey,
  readCanonicalKey,
) {
  let released = false;
  return () => {
    if (released) {
      return;
    }
    released = true;
    linesByCanonicalKey.delete(readCanonicalKey());
    resourceLineEpoch.unregister(lineBacking);
    const materialization = lineBacking.current();
    if (materialization !== null) {
      disposeLineMaterialization(materialization);
    }
    for (const retiredMaterialization of lineBacking.retired()) {
      disposeLineMaterialization(retiredMaterialization);
    }
  };
}

function createMigratedRequestDescriptor(
  familyIdentity,
  familyRecord,
  canonicalParamIdentity,
  basisId,
) {
  const descriptor = createResolvedRequestDescriptor(
    Object.freeze({
      family: familyIdentity,
      canonicalParams: canonicalParamIdentity,
    }),
    familyRecord,
    canonicalParamIdentity.params,
  );
  return Object.freeze({
    ...descriptor,
    context: Object.freeze({
      ...descriptor.context,
      basisId,
    }),
  });
}

function disposeLineMaterialization(materialization) {
  materialization.unregisterEffectProjection();
  materialization.lifecycle.markReleased();
  materialization.lifecycle.releaseOwnedViews();
  materialization.binding.valueSignal.free();
  materialization.binding.canonicalValueSignal.free();
  materialization.binding.readableValueSignal.free();
  materialization.binding.processingSignal.free();
  materialization.binding.uploadSignal.free();
  materialization.binding.downloadSignal.free();
  materialization.binding.statusSignal.free();
  materialization.binding.freshnessSignal.free();
  materialization.binding.diagnosticsSignal.free();
}

export { createMaterializedLine };
