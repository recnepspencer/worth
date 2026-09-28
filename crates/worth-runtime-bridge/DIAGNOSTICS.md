# Diagnostics

Diagnostics are a first-class part of `worth-runtime-bridge`.

They are not just support output.
They are part of the bridge's product promise:

- deterministic routing
- explicit truth-view basis
- explicit preview versus authoritative boundaries
- replayable evidence
- certification trust surfaces

For normal work, start from one door:

```rust
let diagnostics = bridge.diagnostics();
```

## The Everyday Questions

The diagnostics surface should answer ordinary questions directly:

- what did the bridge do last?
- why did this route happen?
- what truth view produced this evaluation?
- what happened in this speculative session?
- what got promoted?

That is why the standard path exposes explanation helpers instead of forcing
you to start with raw record lookup.

## Start With `explain_last`

When you just want orientation, begin here:

```rust
let explanation = bridge.diagnostics().explain_last();
```

This is the fastest way to answer:

- "what just happened?"

It should be the first move when you are exploring behavior or debugging an
unexpected result.

## Explain Routing

Use this when you need to understand a truth-to-compute invalidation outcome.

```rust
let route = bridge.route(TruthCommitIdentity::from_relational_commit_id(commit_id))?;

let last_route = bridge.diagnostics().explain_last_route();
let named_route = bridge.diagnostics().explain_route(route.id());
```

Routing diagnostics are where you confirm things like:

- which truth scope was observed
- which signal targets were selected
- whether fanout happened
- whether specificity beat fallback

## Explain Evaluation

Use this when the important question is not "was something routed?" but "what
truth view was actually evaluated?"

```rust
let evaluation = bridge.evaluate_current(route.target())?;

let last_evaluation = bridge.diagnostics().explain_last_evaluation();
let named_evaluation = bridge.diagnostics().explain_evaluation(evaluation.id());
```

Evaluation diagnostics are where you confirm things like:

- branch head versus snapshot versus historical basis
- which truth selector was in force
- whether the evaluation was tied to current or explicit history

## Explain Speculation

Use this when you are running preview or simulation flows.

```rust
let session = bridge.speculate(spec_request)?;

let last_session = bridge.diagnostics().explain_last_session();
let named_session = bridge.diagnostics().explain_session(session.id());
```

Speculation diagnostics are where you confirm:

- preview session identity
- comparison-to-main basis
- discard versus promote outcome
- branch-local isolation

## Explain Promotion

Use this when a speculative session crosses into authoritative territory.

```rust
let promoted = session.promote()?;

let last_promotion = bridge.diagnostics().explain_last_promotion();
let named_promotion = bridge.diagnostics().explain_promotion(promoted.id());
```

Promotion diagnostics are where the bridge should make authority boundaries
obvious instead of implicit.

## Diagnostics In The Standard Path

The intended daily flow is:

```rust
let route = bridge.route(change)?;
let evaluation = bridge.evaluate_current(route.target())?;

let diagnostics = bridge.diagnostics();
let route_explanation = diagnostics.explain_last_route();
let evaluation_explanation = diagnostics.explain_last_evaluation();
```

Or for speculation:

```rust
let session = bridge.speculate(spec_request)?;
let comparison = session.compare_to_main();

let diagnostics = bridge.diagnostics();
let session_explanation = diagnostics.explain_last_session();
```

The important thing is not the exact return type.
The important thing is the ergonomics:

- diagnostics should stay attached to jobs

## When To Leave The Everyday Diagnostics Door

Stay with `bridge.diagnostics()` for:

- ordinary troubleshooting
- branch comparison readback
- route and evaluation explanation
- preview lifecycle inspection
- normal trust checks

Drop lower only when you are intentionally doing:

- replay and canonical proof work
- certification harness authoring
- deep record-family forensics
- protocol-level parity analysis

Those are legitimate jobs.
They just are not the first door.

## Diagnostics As Trust Surface

The bridge is supposed to be auditable, not magical.

That means diagnostics should help users establish:

- what changed
- what the bridge believed
- which branch or snapshot basis was used
- whether a preview stayed isolated
- whether a promotion crossed the authority boundary correctly
- whether replay and retained evidence agree with the live result

This is also why diagnostics belong in the first-read docs layer rather than
being buried in specialist reference.

## Rule Of Thumb

If you are trying to understand bridge behavior and your first instinct is to
search raw record families, pause and start here instead:

```rust
let diagnostics = bridge.diagnostics();
```

That is the intended front door for explanation.

## Common Pitfalls

- Treating diagnostics as optional support output instead of part of the bridge
  contract.
- Jumping straight to raw retained records before asking the job-shaped
  explanation question first.
- Assuming richer diagnostics are allowed to change bridge meaning. They should
  only change retained richness.
- Explaining bridge behavior with host folklore when the diagnostics door should
  already be able to answer it.
