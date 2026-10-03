import { freezeObject } from "../graph_support.js";

export function createWorkerFirstRootObservationManager(options) {
  let currentContext = null;
  let nextObserverId = 1;
  let nextHandleId = 1;
  const observers = new Map();
  const handleIds = new WeakMap();
  const subscriptions = new Map();

  return {
    watch(bridge, target, callback) {
      if (typeof callback !== "function") {
        throw new TypeError("worker-first root watch(...) requires a callback");
      }
      const handle = createObserverHandle(bridge, target, callback, false);
      void syncLifecycle(bridge).then(() => notifyObservers(currentContext, currentContext, null));
      return handle;
    },
    effect(bridge, target, callback) {
      if (typeof callback !== "function") {
        throw new TypeError("worker-first root effect(...) requires a callback");
      }
      const handle = createObserverHandle(bridge, target, callback, true);
      void syncLifecycle(bridge).then(() => notifyObservers(currentContext, currentContext, null));
      return handle;
    },
    nuke(bridge, handle) {
      const handleId = handleIds.get(handle);
      if (handleId === undefined) {
        return false;
      }
      const didDispose = disposeObserver(handleId);
      if (didDispose) {
        void syncLifecycle(bridge);
      }
      return didDispose;
    },
    async syncLifecycle(bridge) {
      await syncLifecycle(bridge);
    },
    async replaceContext(bridge, nextContext, deliveryPacket = null) {
      const previousContext = currentContext;
      currentContext = nextContext;
      await syncLifecycle(bridge);
      notifyObservers(previousContext, nextContext, deliveryPacket);
    },
    deliverCurrent(deliveryPacket = null) {
      notifyObservers(currentContext, currentContext, deliveryPacket);
    },
    /**
     * Force watch/effect pulses for tip-ingress signal ids (changed + projected).
     * Tip notify uses this once per epoch; worker confirm uses deliverCurrent
     * and skips equal values so tipped ids are not re-pulsed.
     */
    deliverSignalIds(signalIds) {
      if (!Array.isArray(signalIds) || signalIds.length === 0) {
        return;
      }
      notifyForcedSignalIds(currentContext, signalIds);
    },
    async clearContext(bridge) {
      currentContext = null;
      await syncLifecycle(bridge);
    },
    async clearObservers(bridge) {
      observers.clear();
      currentContext = null;
      await syncLifecycle(bridge);
    },
  };

  function createObserverHandle(bridge, target, callback, effectOnly) {
    const signalId = normalizeObservedTargetId(target);
    if (!hasObservedSignal(signalId, currentContext)) {
      throw new TypeError(
        `worker-first root ${effectOnly ? "effect" : "watch"}(...) requires a signal from the active imported graph`,
      );
    }
    const observerId = nextObserverId;
    nextObserverId += 1;
    const handleId = nextHandleId;
    nextHandleId += 1;
    const handle = freezeObject({
      free() {
        if (disposeObserver(handleId)) {
          void syncLifecycle(bridge);
        }
      },
      [Symbol.dispose]() {
        if (disposeObserver(handleId)) {
          void syncLifecycle(bridge);
        }
      },
    });
    handleIds.set(handle, handleId);
    observers.set(handleId, {
      observerId,
      handleId,
      signalId,
      effectOnly,
      callback,
      previousValue: readObservedSignal(signalId, currentContext),
    });
    return handle;
  }

  function disposeObserver(handleId) {
    if (!observers.has(handleId)) {
      return false;
    }
    observers.delete(handleId);
    return true;
  }

  async function syncLifecycle(bridge) {
    const desiredSignalIds = collectDesiredSignalIds();
    for (const signalId of desiredSignalIds) {
      if (subscriptions.has(signalId)) {
        continue;
      }
      const lifecycle = await bridge.attachObservationDelivery({ signalId });
      subscriptions.set(signalId, {
        lifecycleSubscriptionId: lifecycle.lifecycleSubscriptionId,
      });
    }
    for (const [signalId, subscription] of subscriptions) {
      if (desiredSignalIds.has(signalId)) {
        continue;
      }
      await bridge.detachObservationDelivery({
        lifecycleSubscriptionId: subscription.lifecycleSubscriptionId,
      });
      subscriptions.delete(signalId);
    }
  }

  function collectDesiredSignalIds() {
    const desiredSignalIds = new Set();
    if (currentContext === null) {
      return desiredSignalIds;
    }
    for (const observer of observers.values()) {
      if (
        currentContext?.publishedOutputIds.has(observer.signalId)
        || options.hasAuthoredSignal(observer.signalId)
      ) {
        desiredSignalIds.add(observer.signalId);
      }
    }
    return desiredSignalIds;
  }

  function notifyObservers(previousContext, nextContext, deliveryPacket) {
    for (const observer of observers.values()) {
      if (!hasObservedSignal(observer.signalId, nextContext)) {
        continue;
      }
      const nextValue = readObservedSignal(observer.signalId, nextContext);
      const deliveryEvent = findWorkerDeliveryEvent(deliveryPacket, observer.signalId);
      if (deliveryEvent) {
        // Tip already painted equal values — worker confirmation must not re-pulse.
        if (deepEqualObservationValue(observer.previousValue, nextValue)) {
          observer.previousValue = nextValue;
          continue;
        }
        observer.previousValue = nextValue;
        if (deliveryEvent.outcome !== "Delivered") {
          continue;
        }
        if (observer.effectOnly) {
          observer.callback();
          continue;
        }
        observer.callback(
          freezeObject({
            observerId: observer.observerId,
            handleId: observer.handleId,
            signalId: observer.signalId,
            branchId: 0,
            policy: deliveryEvent.policy,
            visited: deliveryEvent.visited,
            recomputed: deliveryEvent.recomputed,
            meaningfulChange: deliveryEvent.meaningful_change,
            triggerMatched: deliveryEvent.trigger_matched,
          }),
        );
        continue;
      }
      const previousValue = (
        !options.hasAuthoredSignal(observer.signalId)
        && previousContext?.signalValueById.has(observer.signalId)
      )
        ? previousContext.signalValueById.get(observer.signalId)
        : observer.previousValue;
      if (deepEqualObservationValue(previousValue, nextValue)) {
        observer.previousValue = nextValue;
        continue;
      }
      observer.previousValue = nextValue;
      if (observer.effectOnly) {
        observer.callback();
        continue;
      }
      observer.callback(
        freezeObject({
          observerId: observer.observerId,
          handleId: observer.handleId,
          signalId: observer.signalId,
          branchId: 0,
          policy: null,
          visited: true,
          recomputed: true,
          meaningfulChange: true,
          triggerMatched: true,
        }),
      );
    }
  }

  function hasObservedSignal(signalId, context) {
    return context?.signalValueById.has(signalId)
      || options.hasAuthoredSignal(signalId);
  }

  function readObservedSignal(signalId, context) {
    if (context?.signalValueById.has(signalId)) {
      return context.signalValueById.get(signalId);
    }
    return options.readAuthoredSignal(signalId);
  }

  function notifyForcedSignalIds(context, signalIds) {
    const idSet = new Set(signalIds);
    for (const observer of observers.values()) {
      if (!idSet.has(observer.signalId) || !hasObservedSignal(observer.signalId, context)) {
        continue;
      }
      const nextValue = readObservedSignal(observer.signalId, context);
      observer.previousValue = nextValue;
      if (observer.effectOnly) {
        observer.callback();
        continue;
      }
      observer.callback(
        freezeObject({
          observerId: observer.observerId,
          handleId: observer.handleId,
          signalId: observer.signalId,
          branchId: 0,
          policy: null,
          visited: true,
          recomputed: true,
          meaningfulChange: true,
          triggerMatched: true,
        }),
      );
    }
  }
}

function findWorkerDeliveryEvent(deliveryPacket, signalId) {
  const boundaryEvents = deliveryPacket?.observation?.observation?.boundary_events;
  if (!Array.isArray(boundaryEvents)) {
    return null;
  }
  return boundaryEvents.find((event) =>
    event?.observed_nodes?.nodes?.includes(signalId)
    || event?.matched_nodes?.nodes?.includes(signalId),
  ) ?? null;
}

function normalizeObservedTargetId(target) {
  if (typeof target === "string" && target.length > 0) {
    return target;
  }
  if (
    target &&
    typeof target === "object" &&
    typeof target.id === "string" &&
    target.id.length > 0
  ) {
    return target.id;
  }
  if (
    typeof target === "function" &&
    typeof target.id === "string" &&
    target.id.length > 0
  ) {
    return target.id;
  }
  throw new TypeError(
    "worker-first root observations require an active imported-graph signal handle or canonical signal id",
  );
}

function deepEqualObservationValue(left, right) {
  if (Object.is(left, right)) {
    return true;
  }
  if (typeof left !== typeof right) {
    return false;
  }
  if (left === null || right === null) {
    return false;
  }
  if (Array.isArray(left) || Array.isArray(right)) {
    if (!Array.isArray(left) || !Array.isArray(right) || left.length !== right.length) {
      return false;
    }
    for (let index = 0; index < left.length; index += 1) {
      if (!deepEqualObservationValue(left[index], right[index])) {
        return false;
      }
    }
    return true;
  }
  if (typeof left !== "object") {
    return false;
  }
  const leftKeys = Object.keys(left);
  const rightKeys = Object.keys(right);
  if (leftKeys.length !== rightKeys.length) {
    return false;
  }
  for (const key of leftKeys) {
    if (!Object.prototype.hasOwnProperty.call(right, key)) {
      return false;
    }
    if (!deepEqualObservationValue(left[key], right[key])) {
      return false;
    }
  }
  return true;
}
