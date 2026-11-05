import { FormDeclarationError } from '../form_errors.js';
import { cloneFormValue, deletePath, isPlainObject, stableValueDigest, writePath } from '../values/value_paths.js';
import { clearPendingRawInput, denyFieldWriteIfBlocked } from './write_support.js';
import { applyFieldInteraction } from '../interaction/controller_bindings.js';

/** Prepare every path/value/admission before committing one native draft revision. */
export function applyFormFieldUpdates(form, declarations, state, values, options = {}) {
  if (!isPlainObject(values) || !Array.isArray(options.clearDraft ?? []) || !Array.isArray(options.touch ?? [])) {
    throw new FormDeclarationError('form.setFields requires values and optional clearDraft field ids');
  }
  const writes = Object.entries(values).map(([id, value]) => ({ id, value, clear: false }));
  writes.push(...(options.clearDraft ?? []).map(id => ({ id, value: null, clear: true })));
  const seen = new Set();
  const paths = [];
  const byId = new Map(declarations.map(field => [field.id, field]));
  const touched = [...new Set(options.touch ?? [])].map(id => {
    const field = byId.get(id);
    if (!field) throw new FormDeclarationError('form.setFields requires declared touch field ids', { fieldId: id });
    return field;
  });
  const previousDraft = form.draft();
  let nextDraft = previousDraft;
  const prepared = writes.map(write => {
    const field = byId.get(write.id);
    if (!field || seen.has(write.id)) throw new FormDeclarationError('form.setFields requires unique declared field ids', { fieldId: write.id });
    if (paths.some(path => path.every((segment, i) => field.segments[i] === segment)
      || field.segments.every((segment, i) => path[i] === segment))) {
      throw new FormDeclarationError('form.setFields cannot write overlapping field paths', { fieldId: write.id });
    }
    paths.push(field.segments);
    seen.add(write.id);
    denyFieldWriteIfBlocked(field, form, write.clear ? 'patch' : 'edit');
    const value = cloneFormValue(write.value);
    nextDraft = write.clear ? deletePath(nextDraft, field.segments) : writePath(nextDraft, field.segments, value);
    return { ...write, field, value };
  });
  if (!prepared.length && !touched.length) return;
  stableValueDigest(nextDraft);
  state.writeDraft(nextDraft);
  for (const write of prepared) {
    clearPendingRawInput(state, write.id, write.clear ? 'clearedByDraftReset' : 'clearedBySet');
    state.parseFailures.delete(write.id);
    state.recordDraftWrite(write.id, write.clear ? 'clearDraft' : 'setValue', previousDraft, write.value);
  }
  for (const field of touched) applyFieldInteraction(state.interactions, field, { kind: 'touch' });
}
