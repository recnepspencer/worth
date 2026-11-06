import { createLineSummaryLocalTruthHandle } from "./reads/line_summary_local_truth_handle.js";
import { createLineObservationBindings } from "./reads/line_observation_bindings.js";
import { requireActiveLine } from "./actions/line_activity_guard.js";
import { invalidateSingleLine } from "./actions/line_invalidate.js";
import { refreshLine } from "./actions/line_refresh.js";
import { revalidateLine } from "./actions/line_revalidate.js";
import { releaseLine } from "./actions/line_release.js";
import { readLineDiagnosticsSummary } from "./reads/line_diagnostics_summary_read.js";
import { readLineFreshness } from "./reads/line_freshness_read.js";
import { readLineStatus } from "./reads/line_status_read.js";
import { readLineSummary } from "./reads/line_summary_read.js";
import { readLineValue } from "./reads/line_value_read.js";
import { createLineSignalLocalTruthHandle } from "./reads/line_signal_local_truth_handle.js";
import { createLineView } from "./line_view_factory.js";
import { requireCurrentMaterialization } from "./state/line_handle_helpers.js";
import { readLineBindingState } from "./state/line_binding_state.js";
import { createTimedOutLineStatus } from "./state/line_status_value.js";

function isPartialConfirmationKind(confirmationKind) {
  return (
    confirmationKind === "partialCanonicalTruth"
    || confirmationKind === "refetchRequired"
    || confirmationKind === "deliveryAwaited"
  );
}

function readAwaitSettlementResult(lineBacking) {
  const materialization = requireCurrentMaterialization(lineBacking);
  requireActiveLine(materialization, "awaitSettlement");
  const status = readLineStatus(materialization);
  if (status.kind === "pending") {
    return null;
  }
  const summary = readLineSummary(materialization);
  const freshness = readLineFreshness(materialization);
  const diagnosticsSummary = readLineDiagnosticsSummary(materialization);
  const diagnostics = readLineBindingState(materialization.binding).diagnostics;
  const mutationResponse = "lastMutationResponsePlan" in diagnostics
    ? diagnostics.lastMutationResponsePlan
    : null;
  if (status.kind === "fulfilled") {
    const confirmationKind = mutationResponse?.confirmation.kind ?? null;
    return Object.freeze({
      resultKind: isPartialConfirmationKind(confirmationKind)
        ? "partial"
        : "fulfilled",
      status,
      value: readLineValue(materialization),
      summary,
      freshness,
      diagnosticsSummary,
      mutationResponse,
      confirmationKind,
    });
  }
  return Object.freeze({
    resultKind: status.kind,
    status,
    summary,
    freshness,
    diagnosticsSummary,
    mutationResponse,
    confirmationKind: null,
  });
}

// The waiter's deadline elapsed while the line is still pending. This is the
// waiter's result, not the line's: the line keeps its pending status and
// settles later like any other pending reload. `status` reports the pending
// operation and continuity as timed out so callers branch on `resultKind`
// exactly as they do for a policy timeout.
function readAwaitSettlementTimedOutResult(lineBacking) {
  const materialization = requireCurrentMaterialization(lineBacking);
  requireActiveLine(materialization, "awaitSettlement");
  const pending = readLineStatus(materialization);
  const diagnostics = readLineBindingState(materialization.binding).diagnostics;
  return Object.freeze({
    resultKind: "timedOut",
    status: createTimedOutLineStatus(
      pending.operation,
      pending.continuity === "preservedVisibleValue",
    ),
    summary: readLineSummary(materialization),
    freshness: readLineFreshness(materialization),
    diagnosticsSummary: readLineDiagnosticsSummary(materialization),
    mutationResponse: "lastMutationResponsePlan" in diagnostics
      ? diagnostics.lastMutationResponsePlan
      : null,
    confirmationKind: null,
  });
}

function releaseRuntimeWatch(handle) {
  if (handle && typeof handle.free === "function") {
    handle.free();
    return;
  }
  if (handle && typeof handle[Symbol.dispose] === "function") {
    handle[Symbol.dispose]();
  }
}

async function drainWorkerFirstAuthoredWork(lineBacking) {
  const materialization = requireCurrentMaterialization(lineBacking);
  const settleAuthoredWork = materialization.lineScope?.settleAuthoredWork;
  if (typeof settleAuthoredWork !== "function") {
    return;
  }
  await settleAuthoredWork.call(materialization.lineScope);
}

export function shouldDrainAuthoredWork(options) {
  return options?.drainAuthoredWork === true;
}

function awaitLineSettlement(lineBacking, activeWaiterFailures, options = {}) {
  const settled = readAwaitSettlementResult(lineBacking);
  if (settled !== null) {
    if (!shouldDrainAuthoredWork(options)) {
      return Promise.resolve(settled);
    }
    return drainWorkerFirstAuthoredWork(lineBacking).then(() => settled);
  }
  return new Promise((resolve, reject) => {
    let finished = false;
    let watchHandle = null;
    let watchedStatusSignalId = null;
    let timeoutHandle = null;

    function cleanup() {
      activeWaiterFailures.delete(fail);
      releaseRuntimeWatch(watchHandle);
      watchHandle = null;
      watchedStatusSignalId = null;
      if (timeoutHandle !== null) {
        clearTimeout(timeoutHandle);
        timeoutHandle = null;
      }
    }

    function fail(error) {
      if (finished) {
        return;
      }
      finished = true;
      cleanup();
      reject(error);
    }

    function finish(result) {
      if (finished) {
        return;
      }
      finished = true;
      cleanup();
      if (!shouldDrainAuthoredWork(options)) {
        resolve(result);
        return;
      }
      drainWorkerFirstAuthoredWork(lineBacking).then(
        () => resolve(result),
        (error) => reject(error),
      );
    }

    // A deadline is a deadline: the timed-out result never waits on authored
    // work, even when `drainAuthoredWork` was requested for the settled case.
    function finishTimedOut() {
      if (finished) {
        return;
      }
      finished = true;
      cleanup();
      resolve(readAwaitSettlementTimedOutResult(lineBacking));
    }

    function observeDeadline() {
      if (finished) {
        return;
      }
      try {
        const settled = readAwaitSettlementResult(lineBacking);
        if (settled !== null) {
          finish(settled);
          return;
        }
        finishTimedOut();
      } catch (error) {
        fail(error);
      }
    }

    function bindCurrentStatusSignal() {
      const materialization = requireCurrentMaterialization(lineBacking);
      requireActiveLine(materialization, "awaitSettlement");
      const nextStatusSignalId = materialization.binding.statusSignal.id;
      if (nextStatusSignalId === watchedStatusSignalId) {
        return;
      }
      if (typeof materialization.binding.statusSignal.watch !== "function") {
        throw new TypeError(
          "resource line awaitSettlement requires watch-capable status truth",
        );
      }
      releaseRuntimeWatch(watchHandle);
      watchedStatusSignalId = nextStatusSignalId;
      watchHandle = materialization.binding.statusSignal.watch(
        () => {
          queueMicrotask(observeSettlement);
        },
      );
    }

    function observeSettlement() {
      if (finished) {
        return;
      }
      try {
        const next = readAwaitSettlementResult(lineBacking);
        if (next !== null) {
          finish(next);
          return;
        }
        bindCurrentStatusSignal();
      } catch (error) {
        fail(error);
      }
    }

    try {
      activeWaiterFailures.add(fail);
      bindCurrentStatusSignal();
      if (typeof options.timeoutMs === "number" && options.timeoutMs >= 0) {
        timeoutHandle = setTimeout(observeDeadline, options.timeoutMs);
      }
      observeSettlement();
    } catch (error) {
      fail(error);
    }
  });
}

function createLineExecution(lineHandle, lineBacking, activeWaiterFailures, options = {}) {
  const freeOnSettle = options.freeOnSettle ?? true;
  let settlementPromise = null;

  return Object.freeze({
    line: lineHandle,
    settled(settlementOptions) {
      if (settlementPromise !== null) {
        return settlementPromise;
      }
      settlementPromise = awaitLineSettlement(
        lineBacking,
        activeWaiterFailures,
        settlementOptions,
      ).finally(() => {
        if (freeOnSettle) {
          lineHandle.free();
        }
      });
      return settlementPromise;
    },
    free() {
      lineHandle.free();
    },
    [Symbol.dispose]() {
      lineHandle[Symbol.dispose]();
    },
  });
}

function createLineHandle(lineBacking) {
  const activeWaiterFailures = new Set();
  let summarySignalHandle = null;
  let valueSignalHandle = null;

  function cancelActiveAwaiters(reason) {
    for (const fail of [...activeWaiterFailures]) {
      fail(reason);
    }
  }

  function ensureSummarySignalHandle() {
    if (summarySignalHandle !== null) {
      return summarySignalHandle;
    }
    const materialization = requireCurrentMaterialization(lineBacking);
    const handle = createLineSummaryLocalTruthHandle(materialization);
    materialization.lifecycle.addOwnedView(handle);
    summarySignalHandle = handle;
    return handle;
  }

  function ensureValueSignalHandle() {
    if (valueSignalHandle !== null) {
      return valueSignalHandle;
    }
    const materialization = requireCurrentMaterialization(lineBacking);
    valueSignalHandle = createLineSignalLocalTruthHandle(materialization);
    return valueSignalHandle;
  }

  return Object.freeze({
    ...createLineObservationBindings(lineBacking, { value: ensureValueSignalHandle, summary: ensureSummarySignalHandle }),
    mutationResponse() {
      const materialization = requireCurrentMaterialization(lineBacking);
      requireActiveLine(materialization, "mutationResponse");
      const diagnostics = readLineBindingState(materialization.binding).diagnostics;
      return "lastMutationResponsePlan" in diagnostics
        ? diagnostics.lastMutationResponsePlan
        : null;
    },
    free() {
      cancelActiveAwaiters(
        new Error("resource line awaitSettlement was cancelled because line.free() released the line"),
      );
      releaseLine(requireCurrentMaterialization(lineBacking));
    },
    invalidate() {
      const materialization = requireCurrentMaterialization(lineBacking);
      requireActiveLine(materialization, "invalidate");
      return invalidateSingleLine(materialization);
    },
    refresh() {
      const materialization = requireCurrentMaterialization(lineBacking);
      requireActiveLine(materialization, "refresh");
      return refreshLine(materialization);
    },
    revalidate() {
      const materialization = requireCurrentMaterialization(lineBacking);
      requireActiveLine(materialization, "revalidate");
      return revalidateLine(materialization);
    },
    awaitSettlement(options) {
      return awaitLineSettlement(lineBacking, activeWaiterFailures, options);
    },
    execute(options) {
      return createLineExecution(this, lineBacking, activeWaiterFailures, options);
    },
    [Symbol.dispose]() {
      cancelActiveAwaiters(
        new Error("resource line awaitSettlement was cancelled because line.free() released the line"),
      );
      releaseLine(requireCurrentMaterialization(lineBacking));
    },
    status() {
      const materialization = requireCurrentMaterialization(lineBacking);
      requireActiveLine(materialization, "status");
      return readLineStatus(materialization);
    },
    freshness() {
      const materialization = requireCurrentMaterialization(lineBacking);
      requireActiveLine(materialization, "freshness");
      return readLineFreshness(materialization);
    },
    view(project) {
      const materialization = requireCurrentMaterialization(lineBacking);
      requireActiveLine(materialization, "view");
      return createLineView(lineBacking, project);
    },
  });
}

export { createLineHandle };
