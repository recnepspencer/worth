import { buildActiveImportContext } from "./sessions/support/worker_first_root_import_context.js";
import { preserveImportTipsAcrossContextReplace } from "./sessions/support/authored/worker_first_host_tip_commit.js";

/** Reuse the receipt's readback until a real host callback produces another write. */
export async function applyAcknowledgedWorkerMutation(deps, transactionOps) {
  const originalContext = deps.activeImportContext();
  const controller = deps.activeImportController();
  const signalIds = deps.authoredRuntime.acknowledgementReadbackIds();
  const importSignalIds = originalContext === null ? null : [...new Set([
    ...originalContext.definition.inputDescriptors.map(entry => entry.sourceId),
    ...originalContext.definition.descriptors.flatMap(entry => [entry.publishedId, entry.sourceId]),
  ])];
  let latest;
  let readbackById;
  let hostEpoch;
  const hostRefreshes = deps.authoredRuntime.hostDependencyReadback();
  const readbackRequest = () => ({ signalIds, importSignalIds,
    deliverObservation: deps.observations.hasSubscriptions(),
    importOutputIds: originalContext?.definition.descriptors.map(entry => entry.publishedId) ?? null });
  async function apply(ops) {
    hostEpoch = hostRefreshes.epoch();
    const packet = await deps.bridge.applyTransactionAcknowledged({ ...readbackRequest(), transactionOps: ops });
    await adopt(packet, ops);
    return packet.transaction;
  }
  async function adopt(packet, ops) {
    latest = packet;
    readbackById = new Map(latest.signalReadbackPacket.signals.map(signal => [signal.id, signal]));
    if (controller !== null) deps.requireControllerActive(controller, "importedGraph.apply");
    if (originalContext !== null) {
      const context = deps.activeImportContext();
      const next = await buildActiveImportContext(deps.bridge,
        context.definition, context.snapshot, latest.importState);
      if (controller !== null) deps.requireControllerActive(controller, "importedGraph.apply");
      preserveImportTipsAcrossContextReplace(context, next, ops);
      deps.setActiveImportContext(next);
    }
    deps.adoptBranchCache(latest.currentBranch, latest.branches);
  }
  await deps.observations.syncLifecycle(deps.bridge);
  const transaction = await apply(transactionOps);
  deps.authoredRuntime.applyCommittedInputs(transactionOps);
  await deps.authoredRuntime.refreshReadables(transactionOps.map(op => op.id), [], {
    async readSignals({ signalIds: ids }) {
      if (hostRefreshes.isCurrent(hostEpoch) && ids.every(id => readbackById.has(id))) {
        return { signals: ids.map(id => readbackById.get(id)) };
      }
      await hostRefreshes.settle();
      hostEpoch = hostRefreshes.epoch();
      await adopt(await deps.bridge.readAcknowledgementState(readbackRequest()), []);
      if (ids.every(id => readbackById.has(id))) return { signals: ids.map(id => readbackById.get(id)) };
      return deps.bridge.readSignals({ signalIds: ids });
    },
    applyTransaction: apply,
  });
  if (originalContext !== null) {
    await deps.observations.replaceContext(deps.bridge, deps.activeImportContext(), latest.deliveryPacket);
    await controller?.refreshFromRootRuntime?.();
    if (controller !== null) deps.requireControllerActive(controller, "importedGraph.apply");
  } else {
    deps.observations.deliverCurrent(latest.deliveryPacket);
  }
  deps.adoptDiagnostics(latest.diagnostics);
  return transaction.runSummary;
}
