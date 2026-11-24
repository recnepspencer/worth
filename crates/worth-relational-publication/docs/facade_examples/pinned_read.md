Only the native owner can construct this capability.
```compile_fail,E0451
use worth_relational::facade::runtime::RelationalPinnedRead;
fn forged() -> RelationalPinnedRead {
    RelationalPinnedRead { snapshot: panic!(), ..panic!() }
}
```
```
use worth_relational::facade::{runtime::RelationalRuntime, snapshots::SnapshotHandle};
use worth_execution::ExecutionRequest;
fn owned(runtime: &RelationalRuntime, snapshot: &SnapshotHandle, request: &ExecutionRequest<'_, '_>) {
    let _ = runtime.read_truth().pin_read_capability(snapshot, &[], request);
}
```
Observation-backed pins have no snapshot query context.
```compile_fail,E0599
use worth_relational::facade::runtime::RelationalPinnedRead;
fn context(read: &RelationalPinnedRead<()>) { read.query_context(); }
```
No runtime borrow can be extracted from a pinned read.
```compile_fail,E0599
use worth_relational::facade::runtime::{RelationalPinnedRead, RelationalRuntime};
fn runtime(read: &RelationalPinnedRead) -> &RelationalRuntime { read.runtime() }
```
```
use worth_relational::facade::runtime::RelationalPinnedRead;
use worth_relational::facade::snapshots::SnapshotHandle;
fn snapshot(read: &RelationalPinnedRead) -> &SnapshotHandle { read.snapshot() }
```
