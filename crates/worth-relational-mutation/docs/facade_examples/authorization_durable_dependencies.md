Issued/decoded native description with shared deep-storage custody.
Context-free deserialization cannot construct this owning value.

```compile_fail
use worth_relational::facade::authorization::RelationalAuthorizationDurableDependencies;
fn unpaid(bytes: &[u8]) -> RelationalAuthorizationDurableDependencies {
    rmp_serde::from_slice(bytes).unwrap()
}
```

```
use worth_relational::facade::authorization::{RelationalAuthorizationDurableDependencies,
    RelationalAuthorizationDependencyDenial};
fn admitted(execution: &worth_execution::ExecutionRequest<'_, '_>, wire: &str)
    -> Result<RelationalAuthorizationDurableDependencies, RelationalAuthorizationDependencyDenial> {
    RelationalAuthorizationDurableDependencies::from_wire(execution, wire)
}
```
