import * as React from "react";
import { createRoot } from "react-dom/client";

export function check(condition, message) {
  if (!condition) throw new Error(message);
}

export function distribution(samples) {
  if (samples.length === 0) return { count: 0, p50Ms: null, p95Ms: null, worstMs: null };
  const sorted = [...samples].sort((a, b) => a - b);
  const at = p => Number(sorted[Math.min(sorted.length - 1, Math.ceil(sorted.length * p) - 1)].toFixed(3));
  return { count: sorted.length, p50Ms: at(0.5), p95Ms: at(0.95), worstMs: at(1) };
}

// Assertions observe only DOM mutations. No act(), polling reads, forced
// rerenders, authored-work drains, or worker-message repair can deliver a value.
export async function mountView(store, useView, initial, options = {}) {
  const container = document.createElement("div");
  document.querySelector("#views").append(container);
  let controls;
  let closed = false;
  let attached;
  const subscriptionReady = new Promise(resolve => { attached = resolve; });
  const observedStore = {
    ...store,
    subscribeSignal(signal, listener) {
      const stop = options.disconnect ? () => {} : store.subscribeSignal(signal, listener);
      attached();
      return stop;
    },
  };
  function View() {
    const view = useView(observedStore);
    React.useLayoutEffect(() => { controls = view.controls; });
    React.useLayoutEffect(() => { options.atLayout?.(); }, []);
    return <output>{JSON.stringify(view.value)}</output>;
  }
  const root = createRoot(container);
  root.render(<React.StrictMode><View /></React.StrictMode>);
  function waitForValue(expected, timeoutMs = 3000) {
    return new Promise((resolve, reject) => {
      const serialized = JSON.stringify(expected);
      let timer;
      const observer = new MutationObserver(inspect);
      function inspect() {
        if (container.querySelector("output")?.textContent !== serialized) return;
        clearTimeout(timer);
        observer.disconnect();
        resolve(performance.now());
      }
      observer.observe(container, { subtree: true, characterData: true, childList: true });
      timer = setTimeout(() => {
        observer.disconnect();
        reject(new Error(`DOM stayed at ${container.textContent}; expected ${serialized}`));
      }, timeoutMs);
      inspect();
    });
  }
  try {
    if (!options.atLayout) await waitForValue(initial);
    if (options.signalSubscription !== false) await subscriptionReady;
  } catch (error) {
    root.unmount();
    container.remove();
    throw error;
  }
  return {
    waitForValue,
    controls: () => controls,
    readDOM: () => container.textContent,
    async remainWhile(expected, pending) {
      const expectedText = JSON.stringify(expected);
      let rejectChange;
      const changed = new Promise((_, reject) => { rejectChange = reject; });
      const observer = new MutationObserver(() => {
        if (container.textContent !== expectedText) {
          rejectChange(new Error(`committed worker response regressed the visible tip: ${container.textContent}`));
        }
      });
      check(container.textContent === expectedText, "burst did not reach its final value");
      observer.observe(container, { subtree: true, characterData: true, childList: true });
      try {
        await Promise.race([changed, Promise.resolve(pending).then(
          () => new Promise(resolve => setTimeout(resolve, 0)),
        )]);
        check(container.textContent === expectedText, "command completion left stale DOM");
      } finally { observer.disconnect(); }
    },
    stayAt(expected, durationMs = 150) {
      return new Promise((resolve, reject) => {
        const expectedText = JSON.stringify(expected);
        const observer = new MutationObserver(() => {
          if (container.textContent === expectedText) return;
          clearTimeout(timer);
          observer.disconnect();
          reject(new Error(`stale completion replaced current view: ${container.textContent}`));
        });
        check(container.textContent === expectedText, "stability assertion began at wrong value");
        observer.observe(container, { subtree: true, characterData: true, childList: true });
        const timer = setTimeout(() => { observer.disconnect(); resolve(); }, durationMs);
      });
    },
    close() {
      if (closed) return;
      closed = true;
      root.unmount();
      container.remove();
    },
  };
}
