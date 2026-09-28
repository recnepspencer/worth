# Daily Workflows

`worth-runtime-bridge` should feel like a framework product in day-to-day use.

That means most bridge work should collapse into a short set of repeatable
jobs:

- build
- route
- evaluate
- speculate
- discard or promote
- inspect

This guide is the practical cookbook for those jobs.

## Use The Facade First

For ordinary bridge work, start here:

```rust
use worth_runtime_bridge::facade::*;
```

Reach for the deeper explicit-control parts of that facade only when you need
non-default behavior.
Reach for the retained-record and replay parts of that same facade only when
you are intentionally doing proof-bearing protocol work or certification.

## Workflow 1: Build A Runtime

Use this when you are wiring the bridge into a host application or harness.

```rust
use worth_foundational::facade::{AspectKey, FieldKey, ScalarAspectType};
use worth_runtime_bridge::facade::{
    BridgeMappingId, BridgeMappingRegistration, CoarseRoutingMode, MappingSelector,
    RuntimeBridge, SignalInvalidationScope, SnapshotReadContract, TruthPatchScope,
};

let bridge = RuntimeBridge::builder()
    .with_truth_source(relational_source.clone())
    .with_truth_branch_head_source(relational_source)
    .with_compute_sink(signal_sink)
    .register_mapping(BridgeMappingRegistration::new(
        BridgeMappingId::from_stable_name("pricing:catalog"),
        TruthPatchScope::for_entity_field(
            MappingSelector::exact("item:widget"),
            AspectKey::new("cost").expect("valid aspect key"),
            FieldKey::new("usd".to_owned()).expect("valid field key"),
        ),
        SnapshotReadContract::scalar(
            AspectKey::new("cost").expect("valid aspect key"),
            ScalarAspectType::String,
        ),
        SignalInvalidationScope::from_stable_name("order:total"),
        CoarseRoutingMode::Direct,
    ))
    .build()?;
```

The ordinary setup memory should be:

- bind truth
- bind branch-head reads
- bind compute
- register mappings
- build

## Workflow 2: Route A Truth Change

Use this when new authoritative truth arrives and you want the bridge to fan it
into compute invalidation.

```rust
let route = bridge.route(TruthCommitIdentity::from_relational_commit_id(commit_id))?;
```

For standard usage, `route(...)` is the front door.
You should not have to manually spell out planning or delivery phases to route
a normal change.

Useful follow-up:

```rust
let route_explanation = bridge.diagnostics().explain_last_route();
```

## Workflow 3: Evaluate The Current Result

Use this when you want the default bridge-visible result for the target that
was just routed.

```rust
let evaluation = bridge.evaluate_current(route.target())?;
```

This is the normal answer to:

- "what does the compute side see now?"

Useful follow-up:

```rust
let evaluation_explanation = bridge.diagnostics().explain_last_evaluation();
```

## Workflow 4: Evaluate An Explicit Truth View

Use this when you need a branch head, snapshot, or historical commit instead of
the default current view.

```rust
use worth_runtime_bridge::facade::{BridgeTruthViewEvaluationRequest, TruthBranchIdentity};

let main_eval = bridge.evaluate(
    BridgeTruthViewEvaluationRequest::for_branch_head(
        TruthBranchIdentity::from_relational_branch_id("main"),
    ),
)?;
```

Use this path when the truth basis is part of the job, not just an internal
detail.

## Workflow 5: Open A Speculative Session

Use this when you want an isolated preview branch that does not contaminate
main.

```rust
let session = bridge.speculate(spec_request)?;
let comparison = session.compare_to_main();
```

The session model is important.
Speculation should feel like entering a scoped mode, not manually threading ids
through unrelated runtime calls.

If you want both sides as explicit evaluations:

```rust
use worth_runtime_bridge::facade::TruthBranchIdentity;

let main_eval = bridge.evaluate(
    comparison.main_evaluation_request(
        TruthBranchIdentity::from_relational_branch_id("main"),
    ),
)?;
let speculative_eval = bridge.evaluate(
    comparison.speculative_evaluation_request(),
)?;
```

## Workflow 6: Discard A Simulation

Use this when the speculative run should disappear without authoritative
residue.

```rust
let discarded = session.discard(vec![
    worth_runtime_bridge::facade::BridgePreviewResidueClass::PreviewExecutionRetained,
    worth_runtime_bridge::facade::BridgePreviewResidueClass::ReplayRetainedNonAuthoritative,
])?;
```

This is the standard "try it, inspect it, walk away cleanly" path.

Useful follow-up:

```rust
let discard_explanation = bridge.diagnostics().explain_last_session();
```

## Workflow 7: Promote A Simulation

Use this when the speculative branch should become authoritative truth.

```rust
let promoted = session.promote()?;
```

Promotion should always feel explicit.
The bridge should make the authority boundary visible rather than silently
blurring preview and commit.

Useful follow-up:

```rust
let promotion_explanation = bridge.diagnostics().explain_last_promotion();
```

## Workflow 8: Start With Diagnostics

Use this when something feels off or when you need evidence instead of a guess.

```rust
let diagnostics = bridge.diagnostics();

let last = diagnostics.explain_last();
let route = diagnostics.explain_last_route();
let evaluation = diagnostics.explain_last_evaluation();
let session = diagnostics.explain_last_session();
let promotion = diagnostics.explain_last_promotion();
```

The bridge should not make you spelunk record families just to understand what
happened in normal work.

## Everyday Rule Of Thumb

If ordinary code starts depending on:

- `validate_*`
- `admit_*`
- `lower_*`
- `canonicalize_*`
- `replay_*`

you have probably drifted out of the intended daily path.

That does not mean those surfaces are wrong.
It means the job may belong in `advanced` or `specialist`, not the ordinary
workflow memory model.
