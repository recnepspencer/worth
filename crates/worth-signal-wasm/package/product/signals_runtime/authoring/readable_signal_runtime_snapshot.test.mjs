import assert from "node:assert/strict";
import test from "node:test";
import { loadSignalsModule } from "../module_loading/load_signals_module.mjs";

test("runtime callback snapshots survive release of their borrowed FFI handle", async () => {
  const { importProductModule } = await loadSignalsModule();
  const { wrapReadableSignal } = await importProductModule("handles.js");
  const { withComputedCallbackFrame } = await importProductModule("callback_frames.js");
  let released = false;
  const raw = {
    get id() {
      assert.equal(released, false, "identity must not re-enter a released FFI wrapper");
      return "retained-native-source";
    },
    get() {
      assert.equal(released, false, "ordinary reads still require a live FFI handle");
      return 1;
    },
    free() { released = true; },
  };
  const runtime = {};
  const input = wrapReadableSignal(raw, runtime);
  const callback = withComputedCallbackFrame(runtime, () => input() + input.get());
  assert.equal(callback().value, 2);
  input.free();
  const key = "__WorthSignalActiveRuntimeCallbackReads";
  const previous = globalThis[key];
  try {
    globalThis[key] = { "retained-native-source": 7 };
    const result = callback();
    assert.equal(result.value, 14);
    assert.deepEqual(result.reads, ["retained-native-source"]);
    assert.equal(input.id, "retained-native-source");
  } finally {
    if (previous === undefined) delete globalThis[key];
    else globalThis[key] = previous;
  }
  assert.throws(() => input.get(), /ordinary reads still require a live FFI handle/);
});


test("a native recipe can reuse a released input through its runtime-owned snapshot", async () => {
  const { createSignals } = await loadSignalsModule({ rawSurface: "real" });
  const signals = await createSignals({ deployment: "mainThreadCompatibility" });
  try {
    const retained = signals.input(7);
    const trigger = signals.input(1);
    const sum = signals.computed(() => retained() + trigger());
    assert.equal(sum(), 8);
    retained.free();
    trigger.set(2);
    assert.equal(sum(), 9);
  } finally {
    await signals.terminate();
  }
});
