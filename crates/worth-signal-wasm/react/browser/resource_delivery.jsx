import { useResourceView, useSignalsForm } from "worth-signals-wasm/react";
import { check, distribution, mountView } from "./observation.jsx";

export function delayedDetails(signals) {
  const pending = new Map();
  const family = signals.api({}).url("/delivery/:id")
    .response(signals.resource.response.detail()({ name: "name" }))
    .detail({ load: ({ id }) => new Promise((resolve, reject) => { pending.set(id, { resolve, reject }); }) });
  return {
    family,
    complete(id, value) {
      check(pending.has(id), `no pending load for ${id}`);
      pending.get(id).resolve({ id, name: value });
    },
    reject(id) { pending.get(id).reject(new Error("deliberate offline response")); },
  };
}

export function useDetailView(line, store) {
  const view = useResourceView(line, store);
  return { state: view.contentState, status: view.status?.kind ?? null, name: view.value?.name ?? null };
}

export async function resourceDelivery(signals, store) {
  const loads = delayedDetails(signals);
  const line = loads.family.line({ id: "one" });
  const view = await mountView(store, s => ({ value: useDetailView(line, s) }),
    { state: "loading", status: "pending", name: null });
  const latency = [];
  try {
    const ready = view.waitForValue({ state: "ready", status: "fulfilled", name: "First" });
    const start = performance.now();
    loads.complete("one", "First");
    latency.push(await ready - start);
    const refreshing = view.waitForValue({ state: "refreshing", status: "pending", name: "First" });
    line.refresh();
    await refreshing;
    const rejected = view.waitForValue({ state: "error", status: "rejected", name: "First" });
    loads.reject("one");
    await rejected;
    const recovered = view.waitForValue({ state: "ready", status: "fulfilled", name: "Recovered" });
    line.refresh();
    loads.complete("one", "Recovered");
    await recovered;
    const patched = view.waitForValue({ state: "ready", status: "fulfilled", name: "Patched" });
    const pending = line.patch(loads.family.patch.field({ field: "name", value: "Patched" }));
    await patched;
    await pending;
    return { delayedLoad: true, refreshContinuity: true, rejectedRefresh: true, recovery: true,
      patch: true, completionToDOM: distribution(latency) };
  } finally {
    view.close();
    await signals.settleAuthoredWork();
    line.free();
  }
}

export async function formDelivery(signals, store) {
  const source = await signals.inputAsync({ title: "Server", other: "One" });
  const declaration = { source,
    fields: ({ field }) => ({ title: field("title"), other: field("other") }) };
  const view = await mountView(store, s => {
    const editor = useSignalsForm(declaration, s);
    return { value: `${editor.effective.title}:${editor.effective.other}`, controls: editor };
  }, "Server:One");
  try {
    const draft = view.waitForValue("Draft:One");
    view.controls().field("title").onChange("Draft");
    await draft;
    const updatedSource = view.waitForValue("Draft:Two");
    const pending = source.set({ title: "Server", other: "Two" });
    await updatedSource;
    await pending;
    const cleared = view.waitForValue("Server:Two");
    view.controls().controller.setFields({}, { clearDraft: ["title"] });
    await cleared;
    return { draftEdit: true, sourceContinuity: true, clearDraft: true };
  } finally { view.close(); }
}
