import { invalidateLine } from "./line_invalidation_execution.js";
import { refreshLine } from "./line_refresh.js";

// Every live materialization in the namespace: registered backings whose
// current line has not been released. Cost: O(registered lines); `current()`
// rematerializes a backing whose epoch moved, exactly as a direct line read
// would.
function readLiveRuntimeMaterializations(resourceLineEpoch) {
  const materializations = [];
  for (const lineBacking of resourceLineEpoch.lineBackings()) {
    const materialization = lineBacking.current();
    if (materialization === null || materialization.lifecycle.isReleased()) {
      continue;
    }
    materializations.push(materialization);
  }
  return materializations;
}

// Marks every materialized line in the namespace stale. Visible values stay;
// each line reports `manualRuntimeInvalidateAll` / `runtimeAll` so a reader
// can tell a runtime-wide sweep from a family or line invalidation. Returns
// the number of lines marked.
function invalidateAllRuntimeLines(resourceLineEpoch) {
  let invalidatedLineCount = 0;
  for (const materialization of readLiveRuntimeMaterializations(resourceLineEpoch)) {
    invalidateLine(materialization, "manualRuntimeInvalidateAll", "runtimeAll");
    invalidatedLineCount += 1;
  }
  return invalidatedLineCount;
}

// Starts a refresh on every materialized line in the namespace, with the
// same semantics as `line.refresh()` on each (a pending reload is superseded).
// Returns the number of lines refreshed.
function refreshAllRuntimeLines(resourceLineEpoch) {
  let refreshedLineCount = 0;
  for (const materialization of readLiveRuntimeMaterializations(resourceLineEpoch)) {
    refreshLine(materialization);
    refreshedLineCount += 1;
  }
  return refreshedLineCount;
}

export { invalidateAllRuntimeLines, refreshAllRuntimeLines };
