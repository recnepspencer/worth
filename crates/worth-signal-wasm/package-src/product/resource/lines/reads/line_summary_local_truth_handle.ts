import { createBindingLocalResourceReadHandle } from '../../views/binding_local_read_handle.js';
import { createResourceViewHandle } from '../../views/view_handle.js';
import { readLineSummary, readReactiveLineSummary, captureLineSummaryInputs } from './line_summary_read.js';
import { readLineBindingState } from '../state/line_binding_state.js';
import { requireActiveLine } from '../actions/line_activity_guard.js';

export function createLineSummaryLocalTruthHandle(materialization) {
  let previousState;
  let snapshot;
  function readSnapshot() {
    requireActiveLine(materialization, 'summarySignal');
    const state = readLineBindingState(materialization.binding);
    if (state !== previousState) {
      snapshot = readLineSummary(materialization, { includeExplainability: false });
      previousState = state;
    }
    return snapshot;
  }
  const underlying = createResourceViewHandle(materialization.lineScope.computed(
    () => readReactiveLineSummary(materialization), { debugName: 'resourceLineSummary' },
  ));
  return createBindingLocalResourceReadHandle(underlying, {
    current() { captureLineSummaryInputs(materialization); return readSnapshot(); },
    peek: readSnapshot,
    release: () => underlying.free(),
  });
}
