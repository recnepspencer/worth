import { canonicalStringify, deepFreeze } from "../support/canonical.js";
import {
  normalizePublicBindings,
  validateInitialBindings,
} from "../signals_local_truth_factory.js";
import { requireDeclaredSchema } from "../schema/schema_declaration.js";

let nextRegistrationId = 0;

export function createWorkerLocalTruthFactory(rootSession) {
  return function localTruth(options) {
    const schema = requireDeclaredSchema(options?.schema, "signals.localTruth");
    const authorityId = options?.authorityId;
    if (typeof authorityId !== "string" || authorityId === "") {
      throw new TypeError("signals.localTruth authorityId must be a non-empty string");
    }
    nextRegistrationId += 1;
    const registrationId = `worker-local-truth:${nextRegistrationId}`;
    const bridge = rootSession.bridge();
    const baseEnvelope = { authorityId, registrationId };
    const bridgeCounters = { roundTrips: 0, serializedBreadth: 0 };
    let nextSequence = 1;
    const bindings = normalizePublicBindings(options.bindings);
    validateInitialBindings(options.initialEntities, bindings);
    const ready = rootSession.settleAuthoredPublications().then(() => {
      const envelope = {
        ...baseEnvelope,
        sequence: 0,
        operation: "create",
        request: {
          ...options,
          schema,
          bindings,
        },
      };
      recordBridgeCommand(bridgeCounters, envelope);
      return bridge.localTruthCommand(envelope);
    });
    let commandTail = ready.then(() => undefined, () => undefined);
    const command = (operation, request = null) => {
      const sequence = nextSequence;
      nextSequence += 1;
      const execute = async () => {
        await ready;
        const envelope = { ...baseEnvelope, sequence, operation, request };
        recordBridgeCommand(bridgeCounters, envelope);
        const value = await bridge.localTruthCommand(envelope);
        return operation === "inspect"
          ? deepFreeze({ ...value, bridgeCounters: { ...bridgeCounters } })
          : value;
      };
      const result = commandTail.then(execute);
      commandTail = result.then(() => undefined, () => undefined);
      return result;
    };
    return deepFreeze({
      kind: "typescriptInMemoryLocalTruth",
      schema,
      ready: () => ready,
      inspect: () => command("inspect"),
      branch: (branchId = "branch:main") => command("branch", branchId),
      commit: (request) => command("commit", request),
      forkBranch: (request) => command("forkBranch", request),
      checkpoint: (branchId) => command("checkpoint", branchId),
      history: (branchId) => command("history", branchId),
      historicalSnapshot: (request) => command("historicalSnapshot", request),
      derivation: (branchId = "branch:main") => command("derivation", branchId),
      destroyDerivation: (branchId) => command("destroyDerivation", branchId),
      rebuildDerivation: (branchId) => command("rebuildDerivation", branchId),
      terminate: () => command("terminate"),
    });
  };
}

function recordBridgeCommand(counters, envelope) {
  counters.roundTrips += 1;
  counters.serializedBreadth += canonicalStringify(envelope).length;
}
