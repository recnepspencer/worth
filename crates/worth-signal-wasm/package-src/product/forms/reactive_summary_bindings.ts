export function createReactiveFormBindings(signalNamespace, formId, formRef) {
  let currentRevision = 0;
  const formScope = signalNamespace.scope(`${formId}:reactive`);
  const mutationRevision = formScope.input(0, {
    debugName: `${formId}.mutationRevision`,
  });
  let summarySignal = null;

  function noteMutation() {
    return mutationRevision.set(++currentRevision);
  }

  function summarySignalHandle() {
    return summarySignal;
  }

  return Object.freeze({
    initialize() {
      summarySignal = formScope.computed(
        () => { mutationRevision(); return readSummary(formRef()); },
        { debugName: `${formId}.summary` },
      );
    },
    noteMutation,
    summarySignalHandle,
    wrapFieldHandle(handle) {
      return createWrappedMutationFacade(
        handle,
        FIELD_MUTATION_METHODS,
        noteMutation,
        { awaitSummaryPublish: true },
      );
    },
    wrapControllerMutations(form) {
      wrapMutationMethods(form, FORM_MUTATION_METHODS, noteMutation, {
        // Action/validation receipts stay sync objects (resultKind/operationId).
        // Summary publish is still kicked; await settleAuthoredWork for worker drain.
        awaitSummaryPublish: false,
      });
      return form;
    },
  });
}

function readSummary(form) {
  return Object.freeze({
    source: form.source(),
    draft: form.draft(),
    effective: form.effective(),
    dirty: form.dirty(),
    patchPlan: form.patchPlan(),
    readiness: form.readiness(),
    visibleMessages: form.visibleMessages(),
  });
}

const FIELD_MUTATION_METHODS = Object.freeze([
  "set",
  "clearDraft",
  "input",
  "compose",
  "commitInput",
  "touch",
  "visit",
  "focus",
  "blur",
  "addItem",
  "removeItem",
  "replaceItem",
  "moveItem",
]);

const FORM_MUTATION_METHODS = Object.freeze([
  'setFields',
  "attemptAction",
  "executeAction",
  "fulfillAction",
  "rejectAction",
  "cancelAction",
  "timeoutAction",
  "retryAction",
  "startAsyncValidation",
  "fulfillAsyncValidation",
  "rejectAsyncValidation",
  "cancelAsyncValidation",
  "timeoutAsyncValidation",
  "reportRouteAuthority",
  "bindRouteAuthority",
  "clearRouteAuthority",
  "reportFieldInteraction",
  "reportSubmitIntent",
  "clearSubmitIntent",
  "reset",
  "rollbackLastResourceEffect",
  "replayExactResourceSource",
  "restoreExactResourceSource",
  "acknowledgePresentation",
  "timeoutPresentation",
  "clearPresentationLane",
]);

function wrapMutationMethods(target, methodNames, noteMutation, options) {
  for (const methodName of methodNames) {
    wrapMutationMethod(target, methodName, noteMutation,
      methodName === 'setFields' ? { ...options, awaitSummaryPublish: true } : options);
  }
}

function wrapMutationMethod(target, methodName, noteMutation, options) {
  const original = target[methodName];
  if (typeof original !== "function") {
    return;
  }
  const awaitSummaryPublish = options?.awaitSummaryPublish === true;
  Object.defineProperty(target, methodName, {
    enumerable: false,
    configurable: true,
    writable: true,
    value(...args) {
      const result = original.apply(this, args);
      if (result && typeof result.then === "function") {
        return result.then(async (resolved) => {
          await Promise.resolve(noteMutation());
          return resolved;
        });
      }
      const noted = noteMutation();
      if (noted && typeof noted.then === "function") {
        if (awaitSummaryPublish) {
          // Field mutations: return thenable so callers can await worker publish.
          return noted.then(() => result);
        }
        // Controller receipts stay sync; surface publish failure without swallowing.
        noted.catch((error) => {
          if (typeof globalThis.reportError === "function") {
            globalThis.reportError(error);
          }
        });
      }
      return result;
    },
  });
}

function createWrappedMutationFacade(target, methodNames, noteMutation, options) {
  const wrapped = {
    ...target,
  };
  wrapMutationMethods(wrapped, methodNames, noteMutation, options);
  return Object.isFrozen(target) ? Object.freeze(wrapped) : wrapped;
}
