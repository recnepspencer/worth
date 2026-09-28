# Branch-Local Candidates And Staged Branches

## What This Feature Is

This feature gives you the shared vocabulary for branch-local work before any
authority transition happens.

It lets you describe:

- which branch the work belongs to
- which candidate identity it belongs to
- which fork basis it started from
- which observation, fork-observation, or comparison basis shaped it
- whether the branch-local state is still a candidate or already staged

Use this when you need a branch-local transition surface that is explicit,
typed, and mechanically non-authoritative.

## Why You Use It

Use this surface when you need to:

- stage work on a branch without calling it a commit
- keep branch-local identity and fork basis visible
- attach observation and comparison basis explicitly before merge planning
  starts
- discard branch-local work with typed non-authoritative closeout evidence

This is the right lane for preview, intent, or staged branch work. It is not
the right lane for authority, receipts, or committed history.

## Stable Entry Points

- `foundational_branch_candidate()`
- `FoundationalBranchCandidateBuilder`
- `FoundationalBranchCandidateArtifact<T>`
- `FoundationalStagedBranchArtifact<T>`
- `FoundationalBranchId`
- `FoundationalBranchCandidateId`
- `FoundationalBranchCandidateForkBasis`
- `FoundationalBranchCandidateObservationBasis`
- `FoundationalBranchCandidateForkObservationBasis`
- `FoundationalBranchCandidateComparisonBasis`

## Core Mental Model

There are two real branch-local states:

- candidate
- staged

A candidate means "this branch-local work exists and is named."

A staged artifact means "this branch-local work is now ready to feed a merge
planner."

Neither one is authority.

That is the core law of this feature. If you need a committed transition, you
must leave this lane and move into committed authority later.

These Milestone 5 candidate forms also do not identify the current target of a
live owner reference. `BoundaryEpoch` and `EquivalenceBasisId` are descriptive
candidate axes; they cannot be converted into the exact branch-reference
grammar. Relational or Signal must supply a
`FoundationalBranchReferenceObservation<TargetBasis>` and then issue its own
admitted authority. See [Exact Branch References](./branch-references.md).

## How It Executes

You build a candidate with the builder:

1. choose the branch
2. choose the candidate id
3. declare the fork basis
4. declare the observation basis
5. optionally add fork-observation and comparison basis
6. stage the payload

From there you can:

- inspect the branch-local artifact
- convert the candidate into a staged artifact
- discard the candidate with explicit zero-residue closeout evidence

## Small Example

```rust
use worth_foundational::{
    foundational_branch_candidate, BoundaryEpoch, BoundaryHandle, EquivalenceBasisId,
    FoundationalBranchCandidateForkBasis, FoundationalBranchCandidateId,
    FoundationalBranchCandidateObservationBasis, FoundationalBranchId,
};

let candidate = foundational_branch_candidate()
    .on_branch(FoundationalBranchId::new("feature/catalog").expect("branch id"))
    .with_candidate_id(FoundationalBranchCandidateId::new(BoundaryHandle::new(7)))
    .from_fork_basis(FoundationalBranchCandidateForkBasis::new(
        FoundationalBranchId::new("main").expect("branch id"),
        BoundaryEpoch::new(401),
    ))
    .under_observation_basis(FoundationalBranchCandidateObservationBasis::new(
        EquivalenceBasisId::new(11),
        BoundaryEpoch::new(402),
    ))
    .stage("catalog-update")?;
```

## Real Example

Use the staged artifact when you are about to enter merge planning:

```rust
use worth_foundational::{
    foundational_branch_candidate, BoundaryEpoch, BoundaryHandle, EquivalenceBasisId,
    FoundationalBranchCandidateComparisonBasis, FoundationalBranchCandidateForkBasis,
    FoundationalBranchCandidateForkObservationBasis, FoundationalBranchCandidateId,
    FoundationalBranchCandidateObservationBasis, FoundationalBranchId,
};

let staged = foundational_branch_candidate()
    .on_branch(FoundationalBranchId::new("feature/catalog").expect("branch id"))
    .with_candidate_id(FoundationalBranchCandidateId::new(BoundaryHandle::new(7)))
    .from_fork_basis(FoundationalBranchCandidateForkBasis::new(
        FoundationalBranchId::new("main").expect("branch id"),
        BoundaryEpoch::new(401),
    ))
    .under_observation_basis(FoundationalBranchCandidateObservationBasis::new(
        EquivalenceBasisId::new(11),
        BoundaryEpoch::new(402),
    ))
    .under_fork_observation_basis(FoundationalBranchCandidateForkObservationBasis::new(
        EquivalenceBasisId::new(12),
        BoundaryEpoch::new(403),
    ))
    .against_comparison_basis(FoundationalBranchCandidateComparisonBasis::new(
        EquivalenceBasisId::new(13),
        FoundationalBranchId::new("main").expect("branch id"),
    ))
    .stage("catalog-update")?
    .staged();

let branch = staged.branch_id();
let comparison_basis = staged.comparison_basis();
let payload = staged.payload();

let _ = (branch, comparison_basis, payload);
```

## How It Relates To Other Features

- [Merge Planning And Verdicts](./merge-planning-and-verdicts.md) consumes
  staged branch artifacts.
- [Commit Receipts, Discard, And Transition Bundles](./commit-receipts-discard-and-transition-bundles.md)
  covers the explicit discard surface that candidates can emit.

## Inspection And Debugging

Check these first:

- `branch_local_state_kind()`
- `branch_id()`
- `candidate_id()`
- `fork_basis()`
- `observation_basis()`
- `fork_observation_basis()`
- `comparison_basis()`

If the branch-local artifact is being compared against another branch, the
comparison basis should stay explicit here instead of first appearing during
merge admission.

If you discard the work, inspect:

- `closeout_cause()`
- `non_authoritative_residue_report()`
- `summary()`

If the next step wants a commit or receipt, you are in the wrong lane. Branch
local state never becomes authority by accident.

## Anti-Patterns

- Do not treat a candidate as if it were a committed transition.
- Do not hide fork basis, fork-observation basis, or comparison basis in
  payload metadata.
- Do not skip staged state and jump straight from branch-local work to receipt
  issuance.
- Do not use discard as if it were a commit-like receipt.
- Do not upgrade an epoch/equivalence candidate into an operational owner basis
  through an alias, wrapper, or compatibility constructor.

## Current Limits

- This lane does not perform merge planning.
- This lane does not carry proof-bearing authority.
- This lane is descriptive and branch-local by design.

## Related Docs

- [Exact Branch References](./branch-references.md)
- [Merge Planning And Verdicts](./merge-planning-and-verdicts.md)
- [Commit Receipts, Discard, And Transition Bundles](./commit-receipts-discard-and-transition-bundles.md)
