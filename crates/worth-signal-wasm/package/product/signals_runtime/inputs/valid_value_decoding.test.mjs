import assert from "node:assert/strict";
import test from "node:test";

import { loadSignalsModule } from "../module_loading/load_signals_module.mjs";

// Valid JSON crosses into WASM through one self-describing decode. A decoder
// that tries alternatives builds a JavaScript Error per rejected shape even
// when the value is accepted, so the oracle here is the Error constructor
// itself, observed on the real artifact in the main-thread deployment.

async function withCompatibilitySignals(run) {
  const signalsModule = await loadSignalsModule({ rawSurface: "real" });
  let signals = null;
  try {
    signals = await signalsModule.createSignals({ deployment: "mainThreadCompatibility" });
    await run(signals);
  } finally {
    if (signals) {
      await signals.terminate();
    }
    await signalsModule.cleanup();
  }
}

async function constructedErrors(run) {
  const OriginalError = globalThis.Error;
  const messages = [];
  globalThis.Error = new Proxy(OriginalError, {
    apply(target, receiver, args) {
      messages.push(String(args[0]));
      return Reflect.apply(target, receiver, args);
    },
    construct(target, args, newTarget) {
      messages.push(String(args[0]));
      return Reflect.construct(target, args, newTarget);
    },
  });
  try {
    await run();
  } finally {
    globalThis.Error = OriginalError;
  }
  return messages;
}

function rows(count, revision) {
  return {
    revision,
    rows: Array.from({ length: count }, (_, index) => ({
      id: `row-${index}`,
      enabled: index % 2 === 0,
      label: "Example",
      score: index / 4,
      nested: { choices: ["one", "two"], empty: null },
    })),
  };
}

function nested(depth) {
  let value = "leaf";
  for (let level = 0; level < depth; level++) {
    value = { level, next: [value, null, true] };
  }
  return value;
}

const unicode = {
  empty: "",
  accented: "café déjà vu",
  emoji: "😀👍🏽",
  cjk: "漢字かなカナ",
  combining: "é",
  quoted: "line\nbreak \"quoted\" \\ slash",
};

const numbers = [0, 1, -1.5, 0.1, 1e21, 1e-7, Number.MAX_SAFE_INTEGER, -Number.MAX_SAFE_INTEGER];

test("valid JSON values cross the WASM boundary without constructing a single Error", async () => {
  await withCompatibilitySignals(async (signals) => {
    for (const count of [1, 10, 100, 1000]) {
      const first = rows(count, 0);
      const errors = await constructedErrors(async () => {
        const source = signals.input(first, { debugName: `decode.rows.${count}` });
        const projected = signals.computed(() => source(), { debugName: `decode.projected.${count}` });
        assert.deepEqual(projected(), first);

        const changed = rows(count, 1);
        await source.set(changed);
        assert.deepEqual(projected(), changed);

        await source.set(structuredClone(changed));
        assert.deepEqual(projected(), changed);

        const committed = rows(count, 2);
        await signals.transaction((tx) => {
          tx.set(source, committed);
        });
        assert.deepEqual(projected(), committed);

        for (let read = 0; read < 20; read++) {
          assert.deepEqual(projected(), committed);
        }
      });
      assert.deepEqual(errors, [], `${count} rows constructed Errors while decoding valid JSON`);
    }

    const shapes = { deep: nested(32), unicode, numbers, mixed: [[], {}, [[]], { a: [{}] }] };
    const errors = await constructedErrors(async () => {
      const source = signals.input(shapes, { debugName: "decode.shapes" });
      const projected = signals.computed(() => source(), { debugName: "decode.shapes.projected" });
      assert.deepEqual(projected(), shapes);
      const changed = { ...shapes, deep: nested(8) };
      await source.set(changed);
      assert.deepEqual(projected(), changed);
    });
    assert.deepEqual(errors, [], "nested, Unicode and numeric shapes constructed Errors while decoding");
  });
});

test("every JSON scalar and container round-trips exactly through input, computed and set", async () => {
  await withCompatibilitySignals(async (signals) => {
    const values = [null, true, false, ...numbers, ...Object.values(unicode), [], {}, [null, [1, ["two"]]],
      { b: 1, a: { d: [], c: null } }, nested(32)];
    const source = signals.input(null, { debugName: "roundTrip.source" });
    const projected = signals.computed(() => source(), { debugName: "roundTrip.projected" });
    for (const value of values) {
      await source.set(value);
      assert.deepEqual(source(), value);
      assert.deepEqual(projected(), value);
    }
  });
});

test("function and symbol writes are refused and leave the committed value in place", async () => {
  await withCompatibilitySignals(async (signals) => {
    const committed = rows(3, 7);
    const source = signals.input(committed, { debugName: "refused.source" });
    const projected = signals.computed(() => source(), { debugName: "refused.projected" });
    for (const value of [() => 1, Symbol("not-json"), { nested: () => 1 }, [Symbol("not-json")]]) {
      await assert.rejects(async () => {
        await source.set(value);
      });
      assert.deepEqual(source(), committed);
      assert.deepEqual(projected(), committed);
    }
  });
});
