import assert from 'node:assert/strict';
import test from 'node:test';
import { createRealRequestRuntime } from '../runtime_fixture/real_request_runtime.mjs';

test('resource signals propagate fulfillment, refresh, and errors to computed consumers', async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    let settle;
    const family = signals.api({}).url('/users/:id').detail({
      load: () => new Promise((resolve, reject) => { settle = { resolve, reject }; }),
    });
    const line = family.line({ id: 'a' });
    const value = line.signal();
    const summary = line.summarySignal();
    const name = signals.computed(() => value()?.name ?? 'loading');
    const status = signals.computed(() => summary().current.status.kind);
    const peeked = signals.computed(() => value.peek()?.name ?? 'loading');
    assert.equal(name(), 'loading');
    assert.equal(status(), 'pending');
    assert.equal(peeked(), 'loading');
    settle.resolve({ id: 'a', name: 'First' });
    await line.awaitSettlement({ timeoutMs: 5000 });
    assert.equal(name(), 'First');
    assert.equal(status(), 'fulfilled');
    assert.equal(peeked(), 'loading', 'peek must remain untracked');
    assert.equal(summary(), summary(), 'React snapshots must retain identity between writes');
    line.refresh();
    assert.equal(status(), 'pending');
    settle.resolve({ id: 'a', name: 'Second' });
    await line.awaitSettlement({ timeoutMs: 5000 });
    assert.equal(name(), 'Second');
    line.refresh();
    settle.reject(new Error('offline'));
    await line.awaitSettlement({ timeoutMs: 5000 });
    assert.equal(status(), 'rejected');
    assert.equal(name(), 'Second', 'a failed refresh preserves visible truth');
    name.free(); status.free(); peeked.free();
    summary.free();
    assert.throws(() => summary(), /cannot be used after/);
    assert.throws(() => summary.peek(), /cannot be used after/);
    line.free();
    assert.throws(() => value(), /cannot be used after/);
    assert.throws(() => value.peek(), /cannot be used after/);
  } finally { await runtime.cleanup(); }
});

test('computed consumers can select a newly materialized resource without crossing a derived frontier', async () => {
  const runtime = await createRealRequestRuntime();
  try {
    const { signals } = runtime;
    const family = signals.api({}).url('/users/:id').detail({
      load: async ({ id }) => ({ id, name: id.toUpperCase() }),
    });
    const lines = new Map();
    const selected = signals.input('a');
    const select = (id) => {
      const line = family.line({ id });
      lines.set(id, { line, value: line.signal(), summary: line.summarySignal() });
      selected.set(id);
      return line;
    };
    const first = select('a');
    const descriptor = signals.computed(() => {
      const { value, summary } = lines.get(selected());
      return `${value()?.name ?? '?'}:${summary().current.status.kind}`;
    });
    assert.equal(descriptor(), '?:pending');
    await first.awaitSettlement({ timeoutMs: 5000 });
    assert.equal(descriptor(), 'A:fulfilled');
    const second = select('b');
    assert.equal(descriptor(), '?:pending');
    await second.awaitSettlement({ timeoutMs: 5000 });
    assert.equal(descriptor(), 'B:fulfilled');
    selected.set('a');
    assert.equal(descriptor(), 'A:fulfilled');
    descriptor.free(); selected.free();
    for (const { line } of lines.values()) line.free();
  } finally { await runtime.cleanup(); }
});
