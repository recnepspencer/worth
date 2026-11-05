import init, { SignalWorkerRuntime } from "../../../raw_surface.js";
import { applyWorkerAcknowledgedTransaction, readWorkerAcknowledgementState, readWorkerImportState } from "./worker_runtime_committed_state.js";
import { createWorkerLocalTruthRuntime } from "../../local_truth/protocol/worker_local_truth_runtime.js";
import {
  isWorkerRuntimeWasmBootstrapMessage,
} from "./worker_runtime_wasm_bootstrap.js";

const earlyBrowserMessages = [];
let browserMessageHandler = null;
let browserBootstrapResolver = null;

if (typeof globalThis.addEventListener === "function") {
  globalThis.addEventListener("message", (event) => {
    if (isWorkerRuntimeWasmBootstrapMessage(event.data)) {
      if (browserBootstrapResolver) {
        const resolve = browserBootstrapResolver;
        browserBootstrapResolver = null;
        resolve(event.data);
        return;
      }
      earlyBrowserMessages.unshift(event.data);
      return;
    }
    if (browserMessageHandler) {
      browserMessageHandler(event.data);
      return;
    }
    earlyBrowserMessages.push(event.data);
  });
}

const bootstrap = await receiveWasmBootstrap();
await init(bootstrap.wasmUrl ?? undefined);

const runtime = new SignalWorkerRuntime();
const localTruthRuntime = createWorkerLocalTruthRuntime(runtime);
const port = await resolveWorkerPort();

port.listen(async (message) => {
  if (!message || typeof message !== "object") {
    return;
  }
  if (isWorkerRuntimeWasmBootstrapMessage(message)) {
    return;
  }
  const { id, method, args = [] } = message;
  try {
    const value = method === "localTruthCommand"
      ? localTruthRuntime.command(args[0])
      : resolveRuntimeMethod(runtime, method, args);
    port.postMessage({
      id,
      ok: true,
      value: await Promise.resolve(value),
    });
  } catch (error) {
    port.postMessage({
      id,
      ok: false,
      error: serializeError(error),
    });
  }
});

async function receiveWasmBootstrap() {
  const queued = takeQueuedBootstrapMessage();
  if (queued) {
    return queued;
  }
  const nodeBootstrap = await receiveNodeWasmBootstrap();
  if (nodeBootstrap) {
    return nodeBootstrap;
  }
  return await new Promise((resolve) => {
    browserBootstrapResolver = resolve;
    const raced = takeQueuedBootstrapMessage();
    if (raced && browserBootstrapResolver === resolve) {
      browserBootstrapResolver = null;
      resolve(raced);
    }
  });
}

function takeQueuedBootstrapMessage() {
  const index = earlyBrowserMessages.findIndex((message) =>
    isWorkerRuntimeWasmBootstrapMessage(message)
  );
  if (index < 0) {
    return null;
  }
  return earlyBrowserMessages.splice(index, 1)[0];
}

async function receiveNodeWasmBootstrap() {
  if (typeof globalThis.process !== "object") {
    return null;
  }
  try {
    const workerThreads = await import("node:worker_threads");
    if (!workerThreads.parentPort) {
      return null;
    }
    return await new Promise((resolve) => {
      const onMessage = (message) => {
        if (!isWorkerRuntimeWasmBootstrapMessage(message)) {
          earlyBrowserMessages.push(message);
          return;
        }
        workerThreads.parentPort.off("message", onMessage);
        resolve(message);
      };
      workerThreads.parentPort.on("message", onMessage);
    });
  } catch {
    return null;
  }
}

function serializeError(error) {
  if (error instanceof Error) {
    return {
      name: error.name,
      message: error.message,
      stack: error.stack ?? null,
    };
  }
  if (error && typeof error === "object") {
    return {
      name: typeof error.name === "string" ? error.name : "WorkerRuntimeBridgeError",
      message: typeof error.message === "string" ? error.message : JSON.stringify(error),
      code: typeof error.code === "string" ? error.code : null,
      stack: null,
    };
  }
  return {
    name: "WorkerRuntimeBridgeError",
    message: String(error),
    stack: null,
  };
}

function resolveRuntimeMethod(runtime, method, args) {
  switch (method) {
    case "applyTransactionAcknowledged":
      return applyWorkerAcknowledgedTransaction(runtime, args[0]);
    case "readAcknowledgementState":
      return readWorkerAcknowledgementState(runtime, args[0]);
    case "readImportState":
      return readWorkerImportState(runtime, args[0]);
    case "publishPortableGraph":
      return resolvePublishPortableGraph(runtime, args[0]);
    case "applyTransaction":
      return resolveApplyTransaction(runtime, args[0]);
    case "admitHostCapabilityIngress":
      return resolveHostCapabilityIngress(runtime, args[0]);
    case "admitBrowserHistoryIngress":
      return resolveBrowserHistoryIngress(runtime, args[0]);
    case "readSignals":
      return resolveReadSignals(runtime, args[0]);
    case "currentBranch":
      return resolveCurrentBranch(runtime);
    case "branches":
      return resolveBranches(runtime);
    case "branchSnapshotId":
      return invokeRequiredRuntimeMethod(runtime, "branchSnapshotId", args);
    case "createBranch":
      return resolveCreateBranch(runtime, args[0]);
    case "workerBranchBasis":
    case "forkBranch":
    case "applyTransactionToBranch":
    case "retireBranch":
    case "retireBranches":
    case "closeoutEffectBranch":
      return invokeRequiredRuntimeMethod(runtime, method, args);
    case "switchBranch":
      return resolveSwitchBranch(runtime, args[0]);
    default:
      if (typeof runtime[method] === "function") {
        return runtime[method](...args);
      }
      throw new TypeError(`unsupported worker runtime method ${method}`);
  }
}

function resolvePublishPortableGraph(runtime, publication) {
  return invokeRequiredRuntimeMethod(runtime, "publishPortableGraph", [publication]);
}

function resolveApplyTransaction(runtime, transactionOps) {
  return invokeRequiredRuntimeMethod(runtime, "applyTransaction", [transactionOps]);
}

function resolveHostCapabilityIngress(runtime, batch) {
  return invokeRequiredRuntimeMethod(runtime, "admitHostCapabilityIngress", [batch]);
}

function resolveBrowserHistoryIngress(runtime, ingress) {
  return invokeRequiredRuntimeMethod(runtime, "admitBrowserHistoryIngress", [ingress]);
}

function resolveReadSignals(runtime, request) {
  return invokeRequiredRuntimeMethod(runtime, "readSignals", [request]);
}

function resolveCurrentBranch(runtime) {
  return invokeRequiredRuntimeMethod(runtime, "currentBranch", []);
}

function resolveBranches(runtime) {
  return invokeRequiredRuntimeMethod(runtime, "branches", []);
}

function resolveCreateBranch(runtime, name) {
  return invokeRequiredRuntimeMethod(runtime, "createBranch", [name]);
}

function resolveSwitchBranch(runtime, branchId) {
  return invokeRequiredRuntimeMethod(runtime, "switchBranch", [branchId]);
}

function invokeRequiredRuntimeMethod(runtime, method, args) {
  if (typeof runtime[method] !== "function") {
    throw new TypeError(
      `worker runtime method ${method} is unavailable; worker-first execution does not fall back to JavaScript authority`,
    );
  }
  return runtime[method](...args);
}

async function resolveWorkerPort() {
  const nodePort = await resolveNodeWorkerPort();
  if (nodePort) {
    return nodePort;
  }
  return {
    listen(handler) {
      browserMessageHandler = handler;
      for (const message of earlyBrowserMessages.splice(0)) {
        handler(message);
      }
    },
    postMessage(message) {
      globalThis.postMessage(message);
    },
  };
}

async function resolveNodeWorkerPort() {
  if (typeof globalThis.process !== "object") {
    return null;
  }
  try {
    const workerThreads = await import("node:worker_threads");
    if (!workerThreads.parentPort) {
      return null;
    }
    return {
      listen(handler) {
        workerThreads.parentPort.on("message", handler);
        for (const message of earlyBrowserMessages.splice(0)) {
          handler(message);
        }
      },
      postMessage(message) {
        workerThreads.parentPort.postMessage(message);
      },
    };
  } catch {
    return null;
  }
}
