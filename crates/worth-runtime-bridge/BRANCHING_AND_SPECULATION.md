# Branching And Speculation

This guide covers the bridge's preview and branch-local coordination story.

The bridge must be able to answer a very practical question:

- "what happens if we change truth on a branch without contaminating main?"

That is the role of speculation.

## Why Speculation Exists

Speculation is how the bridge supports:

- isolated preview work
- split-screen comparison against main
- discard with zero authoritative residue
- explicit promotion into authoritative truth

In Milestone 13 terms, this is the Rust-only equivalent of:

- simulate supply-chain shock
- compare main versus speculative prices
- discard or execute pricing strategy

## The Session Mental Model

The standard path treats speculation as a session:

```rust
let session = bridge.speculate(spec_request)?;
```

That shape matters.
Speculation should feel like entering a scoped mode, not manually threading
preview ids through unrelated top-level operations.

The session owns:

- comparison to main
- discard
- promote

## Compare To Main

Once a session is open, the first question is usually:

- "how does this branch differ from main?"

That starts here:

```rust
let comparison = session.compare_to_main();
```

The comparison handle lets you materialize explicit truth-view requests for both
sides:

```rust
use worth_runtime_bridge::facade::TruthBranchIdentity;

let main_eval = bridge.evaluate(
    comparison.main_evaluation_request(TruthBranchIdentity::new("main")),
)?;

let speculative_eval = bridge.evaluate(
    comparison.speculative_evaluation_request(),
)?;
```

This is the bridge equivalent of a split-screen comparison.

## Isolation Is The Product Requirement

The hard requirement is not just that speculation exists.
It is that branch-local work stays branch-local.

That means:

- speculative truth must not leak into main
- main-branch updates must not silently retarget speculative basis
- both branches must remain comparable against a known fork story

The bridge should preserve that even under interleaved main-branch churn.

## Example Shape

The pricing-shock reference workload uses speculation like this:

1. main continues accepting live cost changes
2. a speculative branch forks
3. the speculative branch applies `rubber +300%`
4. both branch outcomes are evaluated side by side
5. the session is either discarded or promoted

The bridge is responsible for the boundary behavior, not the pricing semantics.

## Discard

Discard is the "walk away cleanly" path:

```rust
let discarded = session.discard()?;
```

The important product rule is:

- discard is positive evidence, not hopeful absence

The bridge should be able to prove that preview work stayed non-authoritative
and left no authoritative residue behind.

## Promote

Promotion is the "this branch becomes authoritative" path:

```rust
let promoted = session.promote()?;
```

Promotion should always feel explicit.
The bridge should never make preview and commit blur into one ambient mode.

## Diagnostics For Speculation

Diagnostics are part of the speculation workflow, not a separate afterthought:

```rust
let session = bridge.speculate(spec_request)?;

let diagnostics = bridge.diagnostics();
let last_session = diagnostics.explain_last_session();
let named_session = diagnostics.explain_session(session.id());
```

These are the ordinary questions diagnostics should answer:

- which preview session is this?
- what was compared to main?
- was it discarded or promoted?
- what branch-local basis was in effect?

## What The Bridge Must Preserve

Speculation is correct only if all of these stay true:

- branch-local identity remains distinct
- comparison basis is explicit
- discard remains non-authoritative
- promotion crosses an explicit authority boundary
- replay preserves the same session meaning

That is why speculation belongs in the bridge itself rather than being treated
as host-local folklore.

## Everyday Rule Of Thumb

If the job is:

- open preview
- compare to main
- inspect both sides
- discard or promote

stay with:

- `bridge.speculate(...)`
- `session.compare_to_main()`
- `session.discard(...)`
- `session.promote(...)`
- `bridge.diagnostics().explain_session(...)`

That is the intended public story for speculation.

## Common Pitfalls

- Thinking speculation is ambient mode instead of a session-shaped workflow.
- Comparing to main informally instead of using the explicit comparison handle.
- Treating discard as "probably cleaned up" instead of as a positive
  zero-residue story.
- forgetting that main-branch churn during preview is part of the real bridge
  workload, not an edge case.
