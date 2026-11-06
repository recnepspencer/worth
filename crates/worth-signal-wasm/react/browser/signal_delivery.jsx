import { useSignalValue } from "worth-signals-wasm/react";
import { check, distribution, mountView } from "./observation.jsx";

export async function signalDelivery(signals, store, trace, nodeCount) {
  const input = await signals.inputAsync(0);
  let output = input;
  for (let index = 0; index < nodeCount; index += 1) {
    const previous = output;
    output = await signals.computedAsync({
      reads: [previous.id],
      expr: { kind: "sum", args: [{ kind: "read", id: previous.id }, { kind: "value", value: 1 }] },
      identity: { kind: "exact" },
    });
  }
  const view = await mountView(store, s => ({ value: useSignalValue(output, s) }), nodeCount);
  const domLatency = [], acknowledgement = [], requests = [];
  try {
    for (let value = 1; value <= 20; value += 1) {
      const before = trace.requests.length;
      const start = performance.now();
      const observed = view.waitForValue(value + nodeCount);
      let acknowledged;
      const command = Promise.resolve(input.set(value)).then(() => { acknowledged = performance.now(); });
      const committed = await observed;
      await command;
      domLatency.push(committed - start);
      acknowledgement.push(acknowledged - start);
      requests.push(trace.requests.length - before);
    }
    const observed = view.waitForValue(50 + nodeCount);
    const writes = [];
    for (let value = 21; value <= 50; value += 1) writes.push(input.set(value));
    await observed;
    await view.remainWhile(50 + nodeCount, Promise.all(writes));
    return {
      nodeCount, sequentialWrites: 20, burstWrites: 30,
      actionToDOM: distribution(domLatency), actionToAcknowledgement: distribution(acknowledgement),
      requestsPerWrite: { minimum: Math.min(...requests), maximum: Math.max(...requests) },
    };
  } finally { view.close(); }
}

export async function mountRace(signals, store) {
  const input = await signals.inputAsync(0);
  let pending;
  const view = await mountView(store, s => ({ value: useSignalValue(input, s) }), 0, {
    atLayout: () => { pending = input.set(1); },
  });
  try {
    await view.waitForValue(1);
    await pending;
    return { renderToSubscribeDelivered: true };
  } finally { view.close(); }
}

export async function sharedSubscribers(signals, store) {
  const input = await signals.inputAsync(0);
  const first = await mountView(store, s => ({ value: useSignalValue(input, s) }), 0);
  let second;
  try {
    second = await mountView(store, s => ({ value: useSignalValue(input, s) }), 0);
    check(store.performanceSummary().activeRuntimeWatchHandleCount === 1, "subscribers must share one watch");
    const both = Promise.all([first.waitForValue(1), second.waitForValue(1)]);
    const pending = input.set(1);
    await both;
    await pending;
    first.close();
    const next = second.waitForValue(2);
    await input.set(2);
    await next;
  } finally { first.close(); second?.close(); }
  check(store.performanceSummary().activeReactSubscriberCount === 0, "unmount leaked subscribers");
  await input.set(3);
  const remounted = await mountView(store, s => ({ value: useSignalValue(input, s) }), 3);
  remounted.close();
  return { sharedWatch: true, siblingAfterUnmount: true, remountFresh: true };
}

export async function transactionDelivery(signals, store) {
  const left = await signals.inputAsync(0);
  const right = await signals.inputAsync(0);
  const sum = await signals.computedAsync(() => left() + right());
  const view = await mountView(store, s => ({ value: useSignalValue(sum, s) }), 0);
  try {
    const observed = view.waitForValue(30);
    const pending = signals.transaction(tx => { tx.set(left, 10); tx.set(right, 20); });
    await observed;
    await pending;
    return { coordinatedWritesDelivered: true };
  } finally { view.close(); }
}

export async function disconnectedControl(signals, store) {
  const input = await signals.inputAsync(0);
  const view = await mountView(store, s => ({ value: useSignalValue(input, s) }), 0, { disconnect: true });
  try {
    const lost = view.waitForValue(1, 150).then(() => false, () => true);
    await input.set(1);
    check(await lost && view.readDOM() === "0", "DOM oracle repaired a disconnected subscription");
    return { lostSubscriptionDetected: true };
  } finally { view.close(); }
}
