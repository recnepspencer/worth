function normalizeWorkerUrl(workerUrl) {
  if (workerUrl instanceof URL) {
    return workerUrl;
  }
  if (typeof workerUrl === "string") {
    // Absolute URLs ignore the base; relative bundler paths resolve against this module.
    return new URL(workerUrl, import.meta.url);
  }
  throw new TypeError("createWorkerRuntimeBridge workerUrl must be a string or URL");
}

function normalizeOptionalWasmUrl(wasmUrl) {
  if (wasmUrl === undefined || wasmUrl === null) {
    return null;
  }
  if (wasmUrl instanceof URL) {
    return wasmUrl.href;
  }
  if (typeof wasmUrl === "string") {
    if (wasmUrl.length === 0) {
      throw new TypeError("createWorkerRuntimeBridge wasmUrl must not be an empty string");
    }
    try {
      return new URL(wasmUrl).href;
    } catch {
      return wasmUrl;
    }
  }
  throw new TypeError("createWorkerRuntimeBridge wasmUrl must be a string or URL");
}

function normalizeWorkerBranchId(branchId, operation) {
  if (typeof branchId === "bigint") {
    if (branchId < 0n) {
      throw new RangeError(`${operation} expects a non-negative branch id`);
    }
    return branchId;
  }
  if (!Number.isSafeInteger(branchId) || branchId < 0) {
    throw new TypeError(`${operation} expects a non-negative safe integer branch id`);
  }
  return BigInt(branchId);
}


function normalizeWorkerPreviewBranchId(branchId, operation) {
  if (typeof branchId === "bigint") {
    if (branchId < 0n) {
      throw new RangeError(`${operation} expects a non-negative branch id`);
    }
    if (branchId > BigInt(Number.MAX_SAFE_INTEGER)) {
      throw new RangeError(
        `${operation} exceeds the safe integer range supported by merge preview requests`,
      );
    }
    return Number(branchId);
  }
  if (!Number.isSafeInteger(branchId) || branchId < 0) {
    throw new TypeError(`${operation} expects a non-negative safe integer branch id`);
  }
  return branchId;
}

function deserializeError(error) {
  const normalized = error && typeof error === "object" ? error : {};
  const message = typeof normalized.message === "string"
    ? normalized.message
    : trySerializeUnknownError(error);
  const reconstructed = new Error(message);
  reconstructed.name = typeof normalized.name === "string"
    ? normalized.name
    : "WorkerRuntimeBridgeError";
  if (typeof normalized.code === "string") {
    reconstructed.code = normalized.code;
  }
  if (typeof normalized.stack === "string") {
    reconstructed.stack = normalized.stack;
  }
  return reconstructed;
}

function trySerializeUnknownError(error) {
  try {
    return JSON.stringify(error);
  } catch {
    return String(error);
  }
}

function attachMessageListener(worker, listener) {
  if (typeof worker.addEventListener === "function") {
    worker.addEventListener("message", (event) => listener(event.data));
    return;
  }
  if (typeof worker.on === "function") {
    worker.on("message", listener);
    return;
  }
  worker.onmessage = (event) => listener(event.data);
}

function attachErrorListener(worker, listener) {
  if (typeof worker.addEventListener === "function") {
    worker.addEventListener("error", listener);
    return;
  }
  if (typeof worker.on === "function") {
    worker.on("error", listener);
    return;
  }
  worker.onerror = listener;
}

export {
  attachErrorListener,
  attachMessageListener,
  deserializeError,
  normalizeOptionalWasmUrl,
  normalizeWorkerBranchId,
  normalizeWorkerUrl,
};
