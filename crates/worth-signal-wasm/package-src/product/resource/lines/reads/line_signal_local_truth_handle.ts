import { createBindingLocalResourceReadHandle } from '../../views/binding_local_read_handle.js';
import { readLineValue } from './line_value_read.js';
import { requireActiveLine } from '../actions/line_activity_guard.js';

/** Local tip truth must also enroll the authoritative input in callback reads. */
export function createLineSignalLocalTruthHandle(materialization) {
  return createBindingLocalResourceReadHandle(materialization.binding.readableValueSignal, {
    current() {
      requireActiveLine(materialization, 'signal');
      materialization.binding.valueSignal();
      return readLineValue(materialization);
    },
    peek() { requireActiveLine(materialization, 'signal'); return readLineValue(materialization); },
  });
}
