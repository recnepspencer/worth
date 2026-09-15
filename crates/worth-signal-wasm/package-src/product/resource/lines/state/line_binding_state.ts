function freezeLineState(state, canonicalRevision) {
  return Object.freeze({
    value: state.value,
    canonicalValue: state.canonicalValue,
    canonicalRevision,
    processing: state.processing,
    upload: state.upload,
    download: state.download,
    status: state.status,
    freshness: state.freshness,
    diagnostics: state.diagnostics,
  });
}

function createLineBindingState(initialState) {
  return {
    current: freezeLineState(initialState, 0),
  };
}

function readLineBindingState(binding) {
  return binding.state.current;
}

function replaceLineBindingState(binding, nextState) {
  const previousState = binding.state.current;
  const canonicalRevision = previousState.canonicalValue === nextState.canonicalValue
    ? previousState.canonicalRevision
    : previousState.canonicalRevision + 1;
  const frozenNextState = freezeLineState(nextState, canonicalRevision);
  binding.state.current = frozenNextState;
  publishLineBindingState(binding, frozenNextState, previousState);
  return frozenNextState;
}

function patchLineBindingState(binding, patch) {
  return replaceLineBindingState(binding, {
    ...binding.state.current,
    ...patch,
  });
}

function publishLineBindingState(binding, nextState, previousState = null) {
  const tipWrites = [];
  const statusChanged = previousState === null
    || previousState.status !== nextState.status;
  const enteringPending = statusChanged && nextState.status.kind === "pending";

  function stage(signal, value, changed) {
    if (!changed) {
      return;
    }
    tipWrites.push({ signal, value });
  }

  stage(
    binding.statusSignal,
    nextState.status,
    enteringPending,
  );
  stage(
    binding.valueSignal,
    nextState.value,
    previousState === null || previousState.value !== nextState.value,
  );
  stage(
    binding.canonicalValueSignal,
    nextState.canonicalValue,
    previousState === null || previousState.canonicalValue !== nextState.canonicalValue,
  );
  stage(
    binding.processingSignal,
    nextState.processing,
    previousState === null || previousState.processing !== nextState.processing,
  );
  stage(
    binding.uploadSignal,
    nextState.upload,
    previousState === null || previousState.upload !== nextState.upload,
  );
  stage(
    binding.downloadSignal,
    nextState.download,
    previousState === null || previousState.download !== nextState.download,
  );
  stage(
    binding.freshnessSignal,
    nextState.freshness,
    previousState === null || previousState.freshness !== nextState.freshness,
  );
  stage(
    binding.diagnosticsSignal,
    nextState.diagnostics,
    previousState === null || previousState.diagnostics !== nextState.diagnostics,
  );
  stage(
    binding.statusSignal,
    nextState.status,
    statusChanged && !enteringPending,
  );

  if (tipWrites.length === 0) {
    return;
  }

  const batchWrites = tipWrites.map(({ signal, value }) => ({
    id: signal.id,
    value,
  }));

  // One tip epoch + one worker batch — never N independent signal.set()s.
  const lineScope = binding.lineScope;
  if (
    typeof lineScope?.commitHostTipAndNotify !== "function"
    || typeof lineScope.applyCommittedTipWorkerBatch !== "function"
  ) {
    throw new TypeError(
      "worker-first line binding publish requires lineScope.commitHostTipAndNotify "
        + "and lineScope.applyCommittedTipWorkerBatch (tip batch path); "
        + "silent notify-only or N× signal.set fallbacks are not permitted",
    );
  }
  const tipEpoch = lineScope.commitHostTipAndNotify(batchWrites);
  const stampedWrites = batchWrites.map((write) => ({
    ...write,
    epochAtWrite: tipEpoch.epochById?.get?.(write.id),
  }));
  void lineScope.applyCommittedTipWorkerBatch(stampedWrites);

  // The readable computed is the runtime's copy of the line's value: graph
  // outputs, watches and history (replay frames, lineage) all key on it. It
  // is evaluated with every published value so the runtime holds the truth
  // the line claims, instead of waiting for a reader that may never come and
  // leaving history empty. `get()` (not `peek()`: peek returns the cached
  // value without evaluating) is frame-free here: the tip commit above is a
  // mutation, which is denied inside any computed callback frame, so no
  // frame can be open at this point on either deployment.
  if (tipWrites.some((write) => write.signal === binding.valueSignal)) {
    binding.readableValueSignal.get();
  }
}

export {
  createLineBindingState,
  patchLineBindingState,
  publishLineBindingState,
  readLineBindingState,
  replaceLineBindingState,
};
