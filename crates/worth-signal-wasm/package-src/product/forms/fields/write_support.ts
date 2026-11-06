export function denyFieldWriteIfBlocked(field, form, capability) {
  const posture = form.fieldWritePosture(field.id, capability);
  if (posture.canWrite) return;
  const error = new TypeError(posture.reason);
  error.name = 'FormFieldWriteDenied';
  error.details = posture;
  throw error;
}

export function clearPendingRawInput(state, fieldId, operation, reason = null) {
  const pending = state.rawInputs.get(fieldId);
  if (!pending) return;
  state.rawInputs.delete(fieldId);
  state.recordRawInput(fieldId, operation, pending.rawValue, pending.source ?? null, reason);
}
