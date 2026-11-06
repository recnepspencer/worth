import { version as reactVersion } from "react";
import { createSignals } from "worth-signals-wasm";
import { createReactSignalsStore } from "worth-signals-wasm/react";
import wasmUrl from "worth-signals-wasm/wasm?url";
import workerUrl from "worth-signals-wasm/worker?worker&url";
import { disconnectedControl, mountRace, sharedSubscribers, signalDelivery,
  transactionDelivery } from "./signal_delivery.jsx";
import { formDelivery, resourceDelivery } from "./resource_delivery.jsx";
import { deniedNavigation, routerDelivery } from "./router_delivery.jsx";
import { distribution } from "./observation.jsx";

const NativeWorker = window.Worker;
let activeTrace;
class TracedWorker extends NativeWorker {
  pending = new Map();
  constructor(...args) {
    super(...args);
    activeTrace.workers += 1;
    this.addEventListener("message", ({ data }) => {
      const request = this.pending.get(data?.id);
      if (!request) return;
      request.elapsedMs = performance.now() - request.started;
      this.pending.delete(data.id);
    });
  }
  postMessage(message, ...args) {
    if (message?.method) {
      const request = { method: message.method, started: performance.now(),
        signalCount: message.args?.[0]?.signalIds?.length ?? null };
      activeTrace.requests.push(request);
      this.pending.set(message.id, request);
    }
    return super.postMessage(message, ...args);
  }
}

document.querySelector("#run").addEventListener("click", async () => {
  document.querySelector("#run").disabled = true;
  document.querySelector("#status").textContent = "Running";
  const results = [];
  const browserErrors = [];
  const recordError = event => browserErrors.push(String(event.reason?.stack ?? event.reason ?? event.message));
  window.addEventListener("error", recordError);
  window.addEventListener("unhandledrejection", recordError);
  const environment = {
    react: reactVersion, mode: import.meta.env.DEV ? "development-strict" : "production",
    browser: navigator.userAgent, hardwareThreads: navigator.hardwareConcurrency,
    visibility: document.visibilityState, boundary: "action to DOM mutation; command acknowledgement separately",
  };
  document.querySelector("#environment").textContent = JSON.stringify(environment);
  const deployments = new URLSearchParams(location.search).get("order") === "worker"
    ? ["workerFirst", "mainThreadCompatibility"] : ["mainThreadCompatibility", "workerFirst"];
  const cases = [
    ["single-node sequential and burst", (s, store, trace) => signalDelivery(s, store, trace, 1)],
    ["64-node sequential and burst", (s, store, trace) => signalDelivery(s, store, trace, 64)],
    ["render-to-subscribe race", mountRace],
    ["shared subscribers and remount", sharedSubscribers],
    ["transaction", transactionDelivery],
    ["resource lifecycle", resourceDelivery],
    ["router and bound resource", routerDelivery],
    ["router denied and missing navigation", deniedNavigation],
    ["form source and draft", formDelivery],
    ["disconnected negative control", disconnectedControl],
  ];
  window.Worker = TracedWorker;
  try {
    for (const deployment of deployments) {
      for (const [name, run] of cases) {
        const errorStart = browserErrors.length;
        let signals, store;
        activeTrace = { workers: 0, requests: [] };
        const result = { deployment, name };
        try {
          const begin = performance.now();
          signals = await createSignals({ deployment,
            assets: deployment === "workerFirst" ? { wasmUrl, workerUrl } : { wasmUrl } });
          result.startupMs = performance.now() - begin;
          store = createReactSignalsStore(signals);
          Object.assign(result, await run(signals, store, activeTrace));
          if ((deployment === "workerFirst") !== (activeTrace.workers > 0)) {
            throw new Error("requested deployment did not match actual Worker construction");
          }
          result.pass = true;
        } catch (error) {
          result.pass = false;
          result.error = String(error?.stack ?? error);
        } finally {
          store?.dispose();
          try { await signals?.terminate(); }
          catch (error) { result.pass = false; result.teardownError = String(error); }
        }
        result.workersCreated = activeTrace.workers;
        result.browserErrors = browserErrors.slice(errorStart);
        if (result.browserErrors.length) result.pass = false;
        result.requestCounts = Object.fromEntries([...new Set(activeTrace.requests.map(r => r.method))]
          .map(method => [method, activeTrace.requests.filter(r => r.method === method).length]));
        result.largestSignalReadBatch = Math.max(0, ...activeTrace.requests.map(r => r.signalCount ?? 0));
        result.rpcLatency = Object.fromEntries(Object.keys(result.requestCounts).map(method => [method,
          distribution(activeTrace.requests.filter(r => r.method === method && r.elapsedMs !== undefined)
            .map(r => r.elapsedMs)),
        ]));
        results.push(result);
        document.querySelector("#results").textContent = JSON.stringify({ environment, results }, null, 2);
      }
    }
  } finally {
    window.Worker = NativeWorker;
    window.removeEventListener("error", recordError);
    window.removeEventListener("unhandledrejection", recordError);
    const pass = results.length === deployments.length * cases.length
      && results.every(r => r.pass) && browserErrors.length === 0;
    document.querySelector("#status").textContent = pass ? "PASS" : "FAIL";
    await fetch("/__results", { method: "POST", body: JSON.stringify({ environment, results, browserErrors, pass }) });
    document.querySelector("#run").disabled = false;
  }
});
