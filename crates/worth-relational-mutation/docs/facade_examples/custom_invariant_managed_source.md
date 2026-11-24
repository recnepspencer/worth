Evaluation's admitted read, at preparation: the prepared object must
belong to this exact invocation, audience and observation. Private:
a rule's preparation reaches only the exact delta, never the payload or
unchanged rows.

```compile_fail
use worth_relational::facade::runtime::{
    CustomInvariantScopePlanner, PreparedManagedIndexSourceRead,
};
fn full(planner: &CustomInvariantScopePlanner<'_>, prepared: &PreparedManagedIndexSourceRead) {
    let _ = planner.read_managed_index_source(prepared);
}
```

The same shape compiles through the delta loan:

```
use worth_relational::facade::runtime::{
    CustomInvariantScopePlanner, PreparedManagedIndexSourceRead,
};
fn delta(planner: &CustomInvariantScopePlanner<'_>, prepared: &PreparedManagedIndexSourceRead) {
    let _ = planner.lend_managed_index_source_changes(prepared, false, |rows| rows.len());
}
```
