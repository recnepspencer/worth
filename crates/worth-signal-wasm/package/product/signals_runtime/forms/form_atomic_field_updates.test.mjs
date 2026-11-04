import assert from 'node:assert/strict';
import test from 'node:test';
import { createRealResourceSignals } from '../../resource_runtime/runtime_fixture/real_resource_signals.mjs';

test('form field batches publish coordinated values once and clear drafts without replacing canonical siblings', async () => {
  const runtime = await createRealResourceSignals();
  try {
    const { signals } = runtime;
    const source = signals.input({ title: 'Before', provenance: 'system', sibling: 'One' });
    const form = signals.form({ source, fields: ({ field }) => ({
      title: field('title'), provenance: field('provenance'), sibling: field('sibling'),
    }) });
    const summary = form.summarySignal();
    summary();
    const seen = [];
    const watch = signals.effect(summary.id, () => seen.push(summary().effective));
    form.setFields({ title: 'After', provenance: 'human' }, { touch: ['title'] });
    await new Promise(resolve => setTimeout(resolve, 0));
    assert.deepEqual(seen, [{ title: 'After', provenance: 'human', sibling: 'One' }]);
    assert.equal(form.interaction().fields.find(field => field.field === 'title').touched, true);
    source.set({ title: 'Before', provenance: 'system', sibling: 'Two' });
    assert.deepEqual(form.effective(), { title: 'After', provenance: 'human', sibling: 'Two' });
    form.setFields({}, { clearDraft: ['title', 'provenance'] });
    assert.deepEqual(form.draft(), {});
    assert.equal(form.dirty().isDirty, false);
    watch.free(); summary.free(); source.free();
  } finally { await runtime.cleanup(); }
});

test('invalid, duplicate, and overlapping bulk field requests leave all draft truth unchanged', async () => {
  const runtime = await createRealResourceSignals();
  try {
    const { signals } = runtime;
    const source = signals.input({ title: 'Before', document: { note: 'Original' } });
    const form = signals.form({ source, fields: ({ field }) => ({
      title: field('title'), document: field('document'), note: field('document.note'),
    }) });
    for (const [values, options] of [
      [{ title: 'Forbidden', unknown: true }, {}],
      [{ title: 'Forbidden' }, { clearDraft: ['title'] }],
      [{ title: 'Forbidden' }, { touch: ['unknown'] }],
      [{ document: { note: 'Forbidden' }, note: 'Ambiguous' }, {}],
    ]) {
      assert.throws(() => form.setFields(values, options), /declared field ids|declared touch field ids|overlapping field paths/);
      assert.deepEqual(form.draft(), {});
      assert.deepEqual(form.effective(), source());
    }
    const cyclic = {}; cyclic.self = cyclic;
    assert.throws(() => form.setFields({ title: 'Forbidden', document: cyclic }));
    assert.deepEqual(form.draft(), {});
    source.free();
  } finally { await runtime.cleanup(); }
});
