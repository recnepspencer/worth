# worth-query-replay

`worth-query-replay` is the cert-band audience facade for Query certification
authority. It re-exports owner types from the Query authority crates without
wrapping them or creating a second executor, comparator or certification
model.

Ordinary application and host code must use `worth-query-decl` or
`worth-query-host`. Only cert-band crates may depend on this crate. That
boundary keeps certification-only measurement authority out of ordinary
execution lanes while preserving exact type identity with its owner,
`worth-query-execution`.

Use the blessed surface:

```rust
use worth_query_replay::facade::{
    WorthQueryCertificationCostObservation,
    WorthQueryCertificationCostRuntimeExt,
    WorthQueryCertificationCostScope,
};
```

`WorthQueryCertificationCostRuntimeExt` captures a
`WorthQueryCertificationCostScope` on a product branch before measured work and
observes a `WorthQueryCertificationCostObservation` after it: storage cost,
application attempt work, output producer attempts, and the product world's
retention and history before and after. The evidence is read-only and grants
nothing; cost courts use it to state growth classes against real host
applications.
