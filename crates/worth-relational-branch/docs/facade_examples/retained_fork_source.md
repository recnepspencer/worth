A retained source cannot enter the current-source fork receiver:
```compile_fail,E0308
use worth_relational::facade::branch::{AdmittedRelationalRetainedForkSource, RelationalForkPort};
use worth_relational::facade::history::BranchId;
fn current(port: &RelationalForkPort, request: &worth_execution::ExecutionRequest<'_, '_>, target: BranchId, source: AdmittedRelationalRetainedForkSource) {
    let _ = port.fork_branch(request, target, source);
}
```
Transfer it through its actual retained-source receiver:
```
use worth_relational::facade::branch::{AdmittedRelationalRetainedForkSource, RelationalForkPort};
use worth_relational::facade::history::BranchId;
fn retained(port: &RelationalForkPort, request: &worth_execution::ExecutionRequest<'_, '_>, target: BranchId, source: AdmittedRelationalRetainedForkSource) {
    let _ = port.fork_retained_branch(request, target, source);
}
```
