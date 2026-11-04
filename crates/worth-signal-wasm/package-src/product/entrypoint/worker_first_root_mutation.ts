import { applyAcknowledgedWorkerMutation } from "./worker_first_mutation_acknowledgement.js";
import {
  stampTransactionOpsWithTipEpochs,
  tipWritesFromTransactionOps,
} from "./sessions/support/authored/worker_first_host_tip_commit.js";
import { mergeWorkerFirstPatchValue } from "./sessions/support/authored/worker_first_authored_input_state.js";

export function createWorkerFirstRootMutation(deps) {
  const pendingMutations = new Set();
  let operationTail = Promise.resolve();
  const trackMutation = (operation) => {
    const mutation = operationTail.then(operation, operation);
    operationTail = mutation.catch(() => {});
    const tracked = Promise.resolve(mutation).finally(() => {
      pendingMutations.delete(tracked);
    });
    pendingMutations.add(tracked);
    return tracked;
  };
  return Object.freeze({
    applyImportMutation(controller, transactionOps, outputIds) {
      const tipCommit = commitTipsForOps(deps, transactionOps);
      return trackMutation(async () => {
        try {
          return await applyImportMutation(deps, controller, transactionOps, outputIds);
        } catch (error) {
          tipCommit?.rollback();
          throw error;
        }
      });
    },
    applyActiveTransaction(transactionOps) {
      const tipCommit = commitTipsForOps(deps, transactionOps);
      return trackMutation(async () => {
        try {
          return await applyActiveTransaction(deps, transactionOps);
        } catch (error) {
          tipCommit?.rollback();
          throw error;
        }
      });
    },
    applyActiveInputMutation(id, mutation) {
      // Expand reset → set(initial) and patch → merged set so tipWrites carry
      // complete values (kind:"reset"/patch fragments painted only after worker).
      const transactionOps = [buildActiveInputMutationOperation(deps, id, mutation)];
      const tipCommit = commitTipsForOps(deps, transactionOps);
      return trackMutation(async () => {
        try {
          return await applyActiveInputMutation(deps, id, mutation, transactionOps);
        } catch (error) {
          tipCommit?.rollback();
          throw error;
        }
      });
    },
    applyAuthoredInputMutation(id, mutation) {
      // Tip advance lives in beginAuthoredInputMutation; dependent projection +
      // observer notify go through the same notifyHostTipIds seam used by
      // commitHostTipAndNotify (import/graph/resource tipWrites).
      deps.requireActive("signals.inputAsync");
      const pendingMutation = deps.authoredRuntime.beginAuthoredInputMutation(id, mutation);
      commitHostTipNotify(deps, [id]);
      return trackMutation(async () => {
        try {
          await deps.ready();
          return await applyWorkerOwnedTransaction(
            deps,
            pendingMutation.transactionOps,
          );
        } catch (error) {
          if (pendingMutation.rollback()) {
            commitHostTipNotify(deps, [id]);
          }
          throw error;
        }
      });
    },
    /**
     * Tips already advanced via commitHostTipAndNotify — queue one worker apply
     * without re-tipping / re-notifying.
     */
    applyCommittedTipWorkerBatch(tipWrites) {
      const writes = Array.isArray(tipWrites) ? tipWrites : [];
      if (writes.length === 0) {
        return Promise.resolve(null);
      }
      const transactionOps = writes.map((write) => {
        if (!write || typeof write.id !== "string") {
          throw new TypeError(
            "worker-first applyCommittedTipWorkerBatch requires { id, value } tip writes",
          );
        }
        return {
          kind: "set",
          id: write.id,
          value: write.value,
          epochAtWrite: write.epochAtWrite,
        };
      });
      return trackMutation(() => applyWorkerOwnedTransaction(deps, transactionOps));
    },
    async settlePendingMutations() {
      while (pendingMutations.size > 0) {
        await Promise.all([...pendingMutations]);
      }
    },
  });
}

async function applyImportMutation(deps, controller, transactionOps, outputIds) {
  await deps.ready();
  deps.requireActive("importedGraph.apply");
  deps.requireControllerActive(controller, "importedGraph.apply");
  return applyAcknowledgedWorkerMutation(deps, transactionOps);
}

async function applyActiveTransaction(deps, transactionOps) {
  await deps.ready();
  deps.requireActive("transactionAsync");
  if (!Array.isArray(transactionOps)) {
    throw new TypeError("worker-first root transactionAsync(...) requires transactionOps as an array");
  }
  for (const op of transactionOps) {
    if (!op || typeof op !== "object" || typeof op.id !== "string") {
      throw new TypeError(
        "worker-first root transactionAsync(...) encountered an invalid input mutation operation",
      );
    }
    if (!deps.hasMutableInputId(op.id)) {
      throw new TypeError(
        `worker-first root transactionAsync(...) can mutate only currently available worker-first inputs; \`${op.id}\` is not currently available`,
      );
    }
  }
  return applyWorkerOwnedTransaction(deps, transactionOps);
}

async function applyActiveInputMutation(deps, id, mutation, transactionOps) {
  await deps.ready();
  deps.requireActive("signals.spec.input");
  const activeImportContext = deps.currentImportContext();
  const inputDescriptor = activeImportContext.definition.inputDescriptors.find(
    (entry) => entry.sourceId === id,
  );
  if (!inputDescriptor) {
    throw new TypeError(
      `worker-first signals.spec.input(...) binds only to input ids from the active imported graph; \`${id}\` is not currently available`,
    );
  }
  const ops = transactionOps ?? [buildActiveInputMutationOperation(deps, id, mutation)];
  return applyImportMutation(
    deps,
    deps.activeImportController(),
    ops,
    activeImportContext.definition.descriptors.map((entry) => entry.publishedId),
  );
}

async function applyWorkerOwnedTransaction(deps, transactionOps) {
  await deps.authoredRuntime.settlePendingPublications();
  requireAuthoredTransactionOpsPublicationReady(deps.authoredRuntime, transactionOps);
  const authoredOpIds = extractChangedSignalIds(transactionOps);
  await deps.authoredRuntime.ensureAuthoredInputsPresentOnWorker(authoredOpIds);
  return applyAcknowledgedWorkerMutation(deps, transactionOps);
}

function requireAuthoredTransactionOpsPublicationReady(authoredRuntime, transactionOps) {
  if (!Array.isArray(transactionOps)) {
    return;
  }
  for (const op of transactionOps) {
    if (!op || typeof op.id !== "string" || !authoredRuntime.hasAuthoredSignalId(op.id)) {
      continue;
    }
    authoredRuntime.requireAuthoredInputPublicationReady(op.id);
  }
}

function buildActiveInputMutationOperation(deps, id, mutation) {
  if (!mutation || typeof mutation !== "object") {
    throw new TypeError("worker-first signals.spec.input mutation requires an operation object");
  }
  switch (mutation.kind) {
    case "set":
      return { kind: "set", id, value: mutation.value };
    case "reset":
      return { kind: "set", id, value: readImportSnapshotSourceValue(deps, id) };
    case "patch":
      return {
        kind: "set",
        id,
        value: mergeWorkerFirstPatchValue(readImportTipValue(deps, id), mutation.value),
      };
    default:
      throw new TypeError("worker-first signals.spec.input mutation kind is unsupported");
  }
}

function readImportTipValue(deps, id) {
  const context = typeof deps.activeImportContext === "function"
    ? deps.activeImportContext()
    : null;
  if (!context?.signalValueById?.has(id)) {
    throw new TypeError(
      `worker-first signals.spec.input patch requires a current tip value for \`${id}\``,
    );
  }
  return context.signalValueById.get(id);
}

function readImportSnapshotSourceValue(deps, id) {
  const context = typeof deps.activeImportContext === "function"
    ? deps.activeImportContext()
    : null;
  const sources = context?.snapshot?.snapshotEnvelope?.state?.sources;
  if (!Array.isArray(sources)) {
    throw new TypeError(
      `worker-first signals.spec.input reset requires an active imported snapshot for \`${id}\``,
    );
  }
  const source = sources.find((entry) => entry && entry.id === id);
  if (!source) {
    throw new TypeError(
      `worker-first signals.spec.input reset cannot recover initial value for \`${id}\``,
    );
  }
  return source.value;
}

function extractChangedSignalIds(transactionOps) {
  if (!Array.isArray(transactionOps)) {
    return [];
  }
  return transactionOps
    .filter((op) => op && typeof op.id === "string")
    .map((op) => op.id);
}

function commitTipsForOps(deps, transactionOps) {
  const tipWrites = tipWritesFromTransactionOps(transactionOps);
  if (tipWrites.length === 0) {
    return null;
  }
  const tipCommit = deps.authoredRuntime.commitHostTipAndNotify(
    deps.observations,
    () => deps.activeImportContext(),
    tipWrites,
  );
  stampTransactionOpsWithTipEpochs(transactionOps, tipCommit.epochById);
  return tipCommit;
}

function commitHostTipNotify(deps, changedIds) {
  deps.authoredRuntime.notifyHostTipIds(deps.observations, changedIds);
}
