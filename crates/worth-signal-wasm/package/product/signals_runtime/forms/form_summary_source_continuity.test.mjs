import assert from 'node:assert/strict';
import test from 'node:test';
import { createRealResourceSignals } from '../../resource_runtime/runtime_fixture/real_resource_signals.mjs';

test('form summaries retain live source dependencies after a draft and acknowledge canonical truth', async () => {
  const runtime = await createRealResourceSignals();
  try {
    const { signals } = runtime;
    const canonical = signals.input({ title: 'First', other: 'One' });
    const source = signals.computed(() => canonical());
    const form = signals.form({ source, fields: ({ field }) => ({ title: field('title'), other: field('other') }) });
    const summary = form.summarySignal();
    const other = signals.computed(() => summary().effective.other);
    assert.equal(other(), 'One');
    await form.fields.title.set('Draft');
    canonical.set({ title: 'First', other: 'Two' });
    assert.equal(other(), 'Two');
    assert.deepEqual(summary().effective, form.effective());
    assert.deepEqual(summary().source, source());
    assert.deepEqual(summary(), summary(), 'reads do not change the summary');
    await form.fields.title.clearDraft();
    canonical.set({ title: 'Saved', other: 'Three' });
    assert.equal(other(), 'Three');
    assert.deepEqual(summary().effective, { title: 'Saved', other: 'Three' });
    other.free(); summary.free(); source.free(); canonical.free();
  } finally { await runtime.cleanup(); }
});
