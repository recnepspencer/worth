```
use worth_relational::facade::history::{CanonicalCommitEnvelope, SharedCanonicalCommitEnvelope};
fn inspect(value: &SharedCanonicalCommitEnvelope) {
    let shared = value.clone();
    let body: &CanonicalCommitEnvelope = shared.as_ref();
    assert_eq!(body, &**value);
}
```

```compile_fail
use worth_relational::facade::history::SharedCanonicalCommitEnvelope;
fn cannot_retain_a_weak_allocation(value: &SharedCanonicalCommitEnvelope) {
    let _ = std::sync::Arc::downgrade(value);
}
```

```compile_fail
use worth_relational::facade::history::SharedCanonicalCommitEnvelope;
fn cannot_extract_the_owner(value: SharedCanonicalCommitEnvelope) {
    let _ = value.into_arc();
}
```
