import { clearPendingRawInput, denyFieldWriteIfBlocked } from './write_support.js';
import { parseFailureArtifact } from "../validation/artifacts.js";
import { applyFieldInteraction, normalizeInteractionInputSource } from "../interaction/controller_bindings.js";
import { FormDeclarationError } from "../form_errors.js";
import {
  deletePath,
  readPath,
  writePath,
} from "../values/value_paths.js";
import { cloneFormValue, stableValueDigest } from "../values/value_semantics.js";
import { compareSemanticValues } from "../values/semantic_equality.js";

export function createFieldHandle(field, form, state) {
  let handle;
  const baseHandle = {
    id: field.id,
    path: field.path,
    locus() {
      return Object.freeze({
        field: field.id,
        path: field.path,
        segments: Object.freeze([...field.segments]),
      });
    },
    sourceValue() {
      return cloneFormValue(readPath(form.source(), field.segments));
    },
    draftValue() {
      return cloneFormValue(readPath(form.draft(), field.segments));
    },
    effectiveValue() {
      return cloneFormValue(readPath(form.effective(), field.segments));
    },
    value() {
      return handle.effectiveValue();
    },
    set(value) {
      denyFieldWriteIfBlocked(field, form, "edit");
      const previousDraft = form.draft();
      clearPendingRawInput(state, field.id, "clearedBySet");
      state.parseFailures.delete(field.id);
      state.writeDraft(writePath(form.draft(), field.segments, value));
      state.recordDraftWrite(field.id, "setValue", previousDraft, value);
      return handle;
    },
    clearDraft() {
      denyFieldWriteIfBlocked(field, form, "patch");
      const previousDraft = form.draft();
      clearPendingRawInput(state, field.id, "clearedByDraftReset");
      state.parseFailures.delete(field.id);
      state.writeDraft(deletePath(form.draft(), field.segments));
      state.recordDraftWrite(field.id, "clearDraft", previousDraft, null);
      return handle;
    },
    input(rawValue, options = {}) {
      denyFieldWriteIfBlocked(field, form, "edit");
      state.parseFailures.delete(field.id);
      const source = normalizeInteractionInputSource(options.source);
      state.rawInputs.set(field.id, {
        field: field.id,
        rawValue: cloneFormValue(rawValue),
        source,
        committed: false,
      });
      state.recordRawInput(field.id, "reported", rawValue, source);
      applyFieldInteraction(state.interactions, field, {
        kind: "input",
        source,
        rawValue,
      });
      if (options.commit === true) {
        return handle.commitInput();
      }
      return handle;
    },
    compose(rawValue) {
      denyFieldWriteIfBlocked(field, form, "edit");
      state.parseFailures.delete(field.id);
      state.rawInputs.set(field.id, {
        field: field.id,
        rawValue: cloneFormValue(rawValue),
        source: "composition",
        committed: false,
      });
      state.recordRawInput(field.id, "compositionReported", rawValue, "composition");
      applyFieldInteraction(state.interactions, field, {
        kind: "compositionStart",
        rawValue,
      });
      return handle;
    },
    commitInput(parser) {
      denyFieldWriteIfBlocked(field, form, "patch");
      const pending = state.rawInputs.get(field.id);
      if (!pending) {
        return handle;
      }
      const parse = parser ?? field.parse;
      let parsedValue;
      try {
        parsedValue = parse ? parse(pending.rawValue) : pending.rawValue;
      } catch (error) {
        state.parseFailures.set(field.id, parseFailureArtifact(field, error, pending.rawValue));
        clearPendingRawInput(state, field.id, "parseFailed", String(error?.message ?? error));
        applyFieldInteraction(state.interactions, field, {
          kind: "compositionCancel",
        });
        return handle;
      }
      state.parseFailures.delete(field.id);
      clearPendingRawInput(state, field.id, "committed");
      applyFieldInteraction(state.interactions, field, {
        kind: "compositionCommit",
      });
      const previousDraft = form.draft();
      state.writeDraft(writePath(form.draft(), field.segments, parsedValue));
      state.recordDraftWrite(field.id, "commitInput", previousDraft, parsedValue);
      return handle;
    },
    touch() {
      applyFieldInteraction(state.interactions, field, {
        kind: "touch",
        source: "imperative",
      });
      return handle;
    },
    visit() {
      applyFieldInteraction(state.interactions, field, {
        kind: "visit",
        source: "imperative",
      });
      return handle;
    },
    focus() {
      applyFieldInteraction(state.interactions, field, {
        kind: "focus",
        source: "imperative",
      });
      return handle;
    },
    blur() {
      applyFieldInteraction(state.interactions, field, {
        kind: "blur",
        source: "imperative",
      });
      return handle;
    },
    dirty() {
      const sourceValue = readPath(form.source(), field.segments);
      const effectiveValue = readPath(form.effective(), field.segments);
      const comparison = compareSemanticValues(sourceValue, effectiveValue);
      return Object.freeze({
        field: field.id,
        path: field.path,
        isDirty: !comparison.equal,
        semanticDirty: !comparison.equal,
        equality: comparison.counters,
      });
    },
    diagnostics() {
      return fieldDiagnostics(field, form, state, handle);
    },
  };
  if (field.family === "repeated") {
    handle = Object.freeze({
      ...baseHandle,
    addItem(item) {
      denyFieldWriteIfBlocked(field, form, "edit");
      const current = requireArrayValue(handle.effectiveValue(), field, "addItem");
      const nextItemId = collectionItemId(field, item);
      if (current.some((entry) => collectionItemId(field, entry) === nextItemId)) {
        throw new FormDeclarationError("repeated field item identity must be unique", {
          field: field.id,
          itemId: nextItemId,
        });
      }
      const previousDraft = form.draft();
      state.writeDraft(writePath(form.draft(), field.segments, [...current, cloneFormValue(item)]));
      state.recordDraftWrite(field.id, "addItem", previousDraft, item);
      return handle;
    },
    removeItem(itemId) {
      denyFieldWriteIfBlocked(field, form, "edit");
      const current = requireArrayValue(handle.effectiveValue(), field, "removeItem");
      const removedItemId = normalizeIdentityValue(itemId, "repeated field remove identity", {
        field: field.id,
      });
      if (!current.some((entry) => collectionItemId(field, entry) === removedItemId)) {
        throw new FormDeclarationError("repeated field remove target was not found", {
          field: field.id,
          itemId: removedItemId,
        });
      }
      const previousDraft = form.draft();
      state.writeDraft(writePath(
        form.draft(),
        field.segments,
        current.filter((entry) => collectionItemId(field, entry) !== removedItemId),
      ));
      state.recordDraftWrite(field.id, "removeItem", previousDraft, null, removedItemId);
      return handle;
    },
    replaceItem(itemId, nextItem) {
      denyFieldWriteIfBlocked(field, form, "edit");
      const current = requireArrayValue(handle.effectiveValue(), field, "replaceItem");
      const expectedItemId = normalizeIdentityValue(itemId, "repeated field replacement identity", {
        field: field.id,
      });
      const nextItemId = collectionItemId(field, nextItem);
      if (nextItemId !== expectedItemId) {
        throw new FormDeclarationError("repeated field replacement must preserve item identity", {
          field: field.id,
          expectedItemId,
          nextItemId,
        });
      }
      if (!current.some((entry) => collectionItemId(field, entry) === expectedItemId)) {
        throw new FormDeclarationError("repeated field replacement target was not found", {
          field: field.id,
          itemId: expectedItemId,
        });
      }
      const previousDraft = form.draft();
      state.writeDraft(writePath(
        form.draft(),
        field.segments,
        current.map((entry) => (
          collectionItemId(field, entry) === expectedItemId ? cloneFormValue(nextItem) : entry
        )),
      ));
      state.recordDraftWrite(field.id, "replaceItem", previousDraft, nextItem, expectedItemId);
      return handle;
    },
    moveItem(itemId, beforeItemId = null) {
      denyFieldWriteIfBlocked(field, form, "edit");
      const current = requireArrayValue(handle.effectiveValue(), field, "moveItem");
      const movingItemId = normalizeIdentityValue(itemId, "repeated field move identity", {
        field: field.id,
      });
      const movingIndex = current.findIndex((entry) => collectionItemId(field, entry) === movingItemId);
      if (movingIndex < 0) {
        throw new FormDeclarationError("repeated field move target was not found", {
          field: field.id,
          itemId: movingItemId,
        });
      }
      const moving = current[movingIndex];
      const withoutMoving = current.filter((_, index) => index !== movingIndex);
      const insertionIndex = beforeItemId === null
        ? withoutMoving.length
        : withoutMoving.findIndex((entry) => collectionItemId(field, entry) === normalizeIdentityValue(
            beforeItemId,
            "repeated field move insertion identity",
            { field: field.id },
          ));
      if (insertionIndex < 0) {
        throw new FormDeclarationError("repeated field move insertion target was not found", {
          field: field.id,
          beforeItemId,
        });
      }
      const next = withoutMoving.slice();
      next.splice(insertionIndex, 0, moving);
      const previousDraft = form.draft();
      state.writeDraft(writePath(form.draft(), field.segments, next));
      state.recordDraftWrite(field.id, "moveItem", previousDraft, moving, beforeItemId ?? null);
      return handle;
    },
    collectionIdentity() {
      const value = requireArrayValue(handle.effectiveValue(), field, "collectionIdentity");
      return Object.freeze({
        field: field.id,
        posture: field.collectionIdentity.posture,
        items: Object.freeze(value.map((entry) => Object.freeze({
          itemId: collectionItemId(field, entry),
          digest: stableValueDigest(entry),
        }))),
      });
    },
    });
    return handle;
  }
  if (field.family === "attachment" || field.family === "evidence") {
    handle = Object.freeze({
      ...baseHandle,
    attachmentIdentity(value = handle.effectiveValue()) {
      if (value === undefined || value === null) {
        return null;
      }
      const attachmentDigest = attachmentId(field, value);
      return Object.freeze({
        field: field.id,
        attachmentDigest,
        metadata: field.attachment.metadata,
        posture: field.attachment.posture,
        valueDigest: stableValueDigest(value),
      });
    },
    });
    return handle;
  }
  handle = Object.freeze(baseHandle);
  return handle;
}

function fieldDiagnostics(field, form, state, handle) {
  return Object.freeze({
    locus: handle.locus(),
    dirty: handle.dirty(),
    pendingRawInput: state.rawInputs.has(field.id),
    parseFailure: state.parseFailures.get(field.id) ?? null,
    interaction: form.interaction().fields.find((entry) => entry.field === field.id) ?? null,
    writePosture: form.fieldWritePosture(field.id),
    inputAdapter: field.inputAdapter,
    ...(field.family === "repeated" ? { collectionIdentity: handle.collectionIdentity() } : {}),
    ...(field.family === "repeated" && field.resourceLocus !== null ? { resourceLocus: field.resourceLocus } : {}),
    ...(field.family === "attachment" || field.family === "evidence"
      ? { attachment: handle.attachmentIdentity() }
      : {}),
  });
}

function requireArrayValue(value, field, operation) {
  if (!Array.isArray(value)) {
    throw new FormDeclarationError(`repeated field ${operation} requires an array value`, {
      field: field.id,
    });
  }
  return value;
}

function collectionItemId(field, item) {
  if (field.collectionIdentity.kind === "resolver") {
    return normalizeIdentityValue(
      field.collectionIdentity.resolver(item),
      "repeated field resolver returned an invalid item identity",
      { field: field.id },
    );
  }
  const identity = readPath(item, [field.collectionIdentity.field]);
  if (!isUsableIdentity(identity)) {
    throw new FormDeclarationError("repeated field item is missing declared item identity", {
      field: field.id,
      itemIdentity: field.collectionIdentity.field,
    });
  }
  return String(identity);
}

function attachmentId(field, attachment) {
  if (field.attachment.identityKind === "resolver") {
    return normalizeIdentityValue(
      field.attachment.identityResolver(attachment),
      "attachment field resolver returned an invalid attachment identity",
      { field: field.id },
    );
  }
  const identity = readPath(attachment, [field.attachment.identityField]);
  if (!isUsableIdentity(identity)) {
    throw new FormDeclarationError("attachment field value is missing declared attachment identity", {
      field: field.id,
      attachmentIdentity: field.attachment.identityField,
    });
  }
  return String(identity);
}

function normalizeIdentityValue(identity, message, details) {
  if (!isUsableIdentity(identity)) {
    throw new FormDeclarationError(message, details);
  }
  return String(identity);
}

function isUsableIdentity(identity) {
  return identity !== undefined && identity !== null && String(identity).length > 0;
}
