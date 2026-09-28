# History And Replay

This guide covers the bridge's trust and auditability story.

The bridge is not supposed to be magical.
It is supposed to be inspectable and replay-safe.

That means it should be able to answer questions like:

- what truth basis produced this result?
- what happened on this branch earlier?
- if I replay this later, do I recover the same bridge meaning?

## Why History Matters

History is not just an archive feature.
It is part of the bridge contract.

The bridge must preserve enough canonical information to make these workflows
possible:

- historical truth-view evaluation
- branch-local auditability
- offline diagnosis
- replay-safe comparison between live execution and retained evidence

## Historical Evaluation

The first history-shaped workflow is explicit truth-view evaluation:

```rust
use worth_runtime_bridge::facade::{
    BridgeTruthViewEvaluationRequest, RelationalBridgeSnapshotIdentityParts,
    TruthBranchIdentity, TruthCommitIdentity, TruthSnapshotIdentity,
};

let branch_eval = bridge.evaluate(
    BridgeTruthViewEvaluationRequest::for_branch_head(
        TruthBranchIdentity::from_relational_branch_id("main"),
    ),
)?;

let snapshot_eval = bridge.evaluate(
    BridgeTruthViewEvaluationRequest::for_branch_snapshot(
        TruthBranchIdentity::from_relational_branch_id("pricing-main"),
        TruthSnapshotIdentity::from_relational_snapshot(
            RelationalBridgeSnapshotIdentityParts::new(snapshot_id, version_id),
        ),
    ),
)?;

let historical_eval = bridge.evaluate(
    BridgeTruthViewEvaluationRequest::for_historical_commit(
        TruthBranchIdentity::from_relational_branch_id("main"),
        TruthCommitIdentity::from_relational_commit_id(commit_id),
    ),
)?;
```

This is the public way to say:

- evaluate against a specific point in truth history

## Replay Is About Meaning, Not Just Output

Replay is not merely:

- "did we end up with the same final value?"

Replay must preserve:

- causal meaning
- truth-view meaning
- branch identity meaning
- failure meaning

If final output matches but the bridge cannot recover the same causal story,
that is not a trustworthy replay.

## Why Replay Belongs In Bridge Docs

The bridge sits between authoritative truth and derived computation.
That position makes replay especially important.

Users need confidence that:

- retained bridge evidence is sufficient
- diagnostics are not lying by omission
- offline analysis can reconstruct what mattered

That is why replay belongs in the public trust story rather than only in test
internals.

## History In Speculative Work

History matters just as much for preview flows.

A good bridge should be able to tell you:

- what basis the speculative branch forked from
- what remained branch-local
- whether discard left zero authoritative residue
- whether promotion later crossed the authority boundary correctly

Those are history questions, not only runtime questions.

## Diagnostics And History

For ordinary inspection, start with the diagnostics door:

```rust
let diagnostics = bridge.diagnostics();

let last = diagnostics.explain_last();
let evaluation = diagnostics.explain_last_evaluation();
let session = diagnostics.explain_last_session();
let promotion = diagnostics.explain_last_promotion();
```

The point is that history should be reachable through job-shaped explanation
before you ever need deeper artifact-level reconstruction.

## Offline Diagnosis

Milestone 13 raises the standard here.
The bridge should eventually support bundle-sufficient diagnosis where a result
can be judged from canonical retained artifacts alone.

That means history and replay are not just implementation conveniences.
They are proof obligations.

The bridge should be able to support questions like:

- did routing meaning drift?
- did branch comparison meaning drift?
- did a discard really stay non-authoritative?
- did a promotion keep the same authority story after replay?

## Rule Of Thumb

If the job is:

- inspect a branch head
- inspect a historical commit
- compare retained evidence to live meaning
- validate that replay keeps the same bridge story

you are in history-and-replay territory.

Start with:

- `bridge.evaluate(...)`
- `bridge.diagnostics().explain_*()`

Then go deeper only when you need specialist replay or certification surfaces.
