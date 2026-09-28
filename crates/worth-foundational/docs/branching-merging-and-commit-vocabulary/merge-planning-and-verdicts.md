# Merge Planning And Verdicts

## What This Feature Is

This feature turns staged branch work into a merge candidate and then into a
typed merge admission outcome or merge verdict.

It gives you:

- explicit target branch selection
- structural summary before admission
- explicit merge basis and merge-base selection basis
- explicit branch-basis drift instead of ambient "stale merge" folklore
- typed verdict kinds instead of success/failure folklore
- `TransitionOutcome` for denied, deferred, stale, rebind-required, and failed
  paths

## Why You Use It

Use this surface when you need to:

- plan a merge before any authority crossing
- keep merge breadth, strategy, basis, and conflict loci visible
- distinguish accepted, advisory, conflict, superseded, denied, stale,
  deferred, rebind-required, and failed outcomes

This is the planning and admission lane. It is still non-authoritative.

## Stable Entry Points

- `foundational_merge(...)`
- `FoundationalMergeBuilder<T>`
- `FoundationalMergeCandidate<T>`
- `FoundationalMergeVerdict<T>`
- `FoundationalMergeVerdictKind`
- `FoundationalMergeStructuralSummary`
- `FoundationalMergeConflictLocus`
- `FoundationalMergeAdmissionOutcome`

## Core Mental Model

There are three important layers:

- staged branch input
- merge candidate
- merge admission outcome

A merge candidate means "the merge has been planned honestly."

A merge verdict means "the merge has been admitted into a typed outcome."

Neither one is committed authority yet.

That is why the non-success paths use `worth-proof::TransitionOutcome`. The
merge lane needs real topology without pretending it already crossed authority.

## How It Executes

You start from a staged branch artifact and then:

1. choose the target branch
2. declare merge intent
3. attach a structural summary
4. attach merge basis and merge-base selection basis
5. attach strategy identity and related strategy evidence
6. optionally attach correspondence or remap basis
7. call `plan()`
8. admit the candidate into a verdict or a structured non-success outcome

The non-success topology matters:

- `Denied` means the merge was not admitted on semantic grounds
- `Deferred` means the merge is intentionally waiting on more work or timing
- `Stale` means the basis story drifted and the caller must rebuild against a
  newer truth surface
- `RebindRequired` means stronger basis or authority context is required before
  continuing
- `Failed` means the planner or admission machinery could not produce an honest
  verdict

## Small Example

```rust
use worth_foundational::{foundational_merge, FoundationalMergeIntent};

let candidate = foundational_merge(staged_branch)
    .into_target_branch(main_branch)
    .with_intent(FoundationalMergeIntent::ReconcileIntoTarget)
    .with_structural_summary(summary)
    .with_merge_basis(merge_basis)
    .with_merge_base_selection_basis(merge_base_selection_basis)
    .under_strategy(strategy_identity)
    .with_strategy_descriptor_digest(strategy_digest)
    .with_strategy_contract_basis(strategy_contract_basis)
    .with_strategy_basis(strategy_basis)
    .plan()?;
```

## Real Example

Use the admitted outcome path when you need explicit merge topology:

```rust
use worth_foundational::FoundationalMergeConflictLocus;
use worth_proof::TransitionOutcome;

let outcome = candidate.admit_as_conflict(vec![
    FoundationalMergeConflictLocus::new(
        "document-section",
        "source:section-7",
        "target:section-7",
    ),
]);

match outcome {
    TransitionOutcome::Success(verdict) => {
        let kind = verdict.kind();
        let summary = verdict.structural_summary();
        let conflicts = verdict.conflict_loci();
        let _ = (kind, summary, conflicts);
    }
    TransitionOutcome::Denied(denial) => {
        let _ = denial;
    }
    TransitionOutcome::Stale(drift) => {
        let _ = drift;
    }
    TransitionOutcome::Deferred(wait) => {
        let _ = wait;
    }
    TransitionOutcome::RebindRequired(rebind) => {
        let _ = rebind;
    }
    TransitionOutcome::Failed(failure) => {
        let _ = failure;
    }
}
```

## How It Relates To Other Features

- [Branch-Local Candidates And Staged Branches](./branch-local-candidates-and-staged-branches.md)
  supplies the staged branch input.
- [Transition Strategy And Basis Semantics](./transition-strategy-and-basis-semantics.md)
  explains the strategy and basis surfaces that this lane carries.
- [Committed Authority Transitions](./committed-authority-transitions.md)
  is the next lane after a commit-eligible verdict exists.

## Inspection And Debugging

Check these first:

- `structural_summary()`
- `merge_basis()`
- `merge_base_selection_basis()`
- `branch_basis_drift()`
- `conflict_loci()`
- `kind()`

If admission fails before a verdict exists, inspect the `TransitionOutcome`
category first. If `plan()` fails, inspect the construction denial first.

## Anti-Patterns

- Do not hide merge structure inside payload metadata.
- Do not flatten merge topology into `bool` or `Result<(), String>`.
- Do not commit directly from a staged branch without going through merge
  planning and admission.
- Do not treat a conflict verdict or stale outcome as if it were commit-eligible.

## Current Limits

- This lane does not cross authority.
- This lane does not issue receipts.
- This lane assumes the caller already has a staged branch artifact.
- This lane preserves strategy and basis meaning, but it does not register or
  execute strategies.

## Related Docs

- [Transition Strategy And Basis Semantics](./transition-strategy-and-basis-semantics.md)
- [Committed Authority Transitions](./committed-authority-transitions.md)
