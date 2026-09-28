# Transition Strategy And Basis Semantics

## What This Feature Is

This feature gives merges and committed transitions a shared language for the
things that shaped their meaning:

- strategy identity
- strategy family and version
- strategy ownership
- strategy descriptor digest
- strategy contract basis
- strategy basis
- transition basis identity, family, and version
- merge-base selection basis
- correspondence basis
- remap basis
- branch-basis drift

This is the part of Milestone 5 that stops complex merge and commit semantics
from becoming runtime-private folklore.

## Why You Use It

Use this surface when you need to:

- preserve which strategy shaped a merge or commit
- make strategy identity durable enough for replay or certification
- keep basis-bearing matching and remap semantics visible
- explain stale-basis or target-advanced situations explicitly

If a transition is strategy-bearing or basis-bearing, this is how you keep that
truth readable at the boundary.

## Stable Entry Points

Strategy and basis types are exposed from `transitions::merges`:

- `FoundationalTransitionStrategyIdentity`
- `FoundationalTransitionStrategyId`
- `FoundationalTransitionStrategyFamily`
- `FoundationalTransitionStrategySemanticName`
- `FoundationalTransitionStrategyVersion`
- `FoundationalTransitionStrategyOwnershipClass`
- `FoundationalTransitionStrategyDescriptorDigest`
- `FoundationalTransitionStrategyContractBasis`
- `FoundationalStrategyBasis`
- `FoundationalTransitionBasisIdentity`
- `FoundationalTransitionBasisFamily`
- `FoundationalTransitionBasisVersion`
- `FoundationalMergeBasis`
- `FoundationalMergeBaseSelectionBasis`
- `FoundationalTransitionCorrespondenceBasis`
- `FoundationalTransitionRemapBasis`
- `FoundationalBranchBasisDrift`
- `FoundationalBranchBasisDriftKind`

The ownership and drift enums are part of the contract too:

- strategy ownership: `RuntimeBuiltIn`, `CustomRegistered`,
  `CompatibilityLowered`
- drift kinds: `TargetAdvanced`, `SourceAdvanced`,
  `MergeBasisInvalidated`, `ParentBasisUnavailable`

## Core Mental Model

These types answer two different questions:

- Which strategy shaped this transition?
- Under what basis did we decide that shape was valid?

Strategy identity is about the semantic merge or commit approach.

Basis is about the truth surface that the approach depended on.

Those are related, but not interchangeable. If you collapse them together, the
consumer can no longer tell whether a replay difference came from strategy
change, basis change, or both.

## How It Executes

You attach strategy and basis meaning during merge planning:

- strategy identity says what strategy was used
- descriptor digest and contract basis make that strategy durable and
  replay-safe
- merge basis and merge-base selection basis explain the merge truth surface
- correspondence and remap basis explain matching or remapping semantics
- drift reports explain when the basis no longer matches the live state

That information then flows forward into the merge verdict, committed
authority artifact, receipt provenance rows, and canonical basis lowering.

## Small Example

```rust
use worth_foundational::foundational_merge;

let candidate = foundational_merge(staged_branch)
    .into_target_branch(main_branch)
    .with_intent(intent)
    .with_structural_summary(summary)
    .with_merge_basis(merge_basis)
    .with_merge_base_selection_basis(merge_base_selection_basis)
    .under_strategy(strategy_identity)
    .with_strategy_descriptor_digest(strategy_descriptor_digest)
    .with_strategy_contract_basis(strategy_contract_basis)
    .with_strategy_basis(strategy_basis)
    .plan()?;
```

## Real Example

Once a verdict exists, you can inspect the full shaping story directly:

```rust
let verdict = candidate.admit_as_advisory()?;

let strategy = verdict.strategy_identity();
let descriptor_digest = verdict.strategy_descriptor_digest();
let contract_basis = verdict.strategy_contract_basis();
let strategy_basis = verdict.strategy_basis();
let merge_basis = verdict.merge_basis();
let merge_base_selection_basis = verdict.merge_base_selection_basis();
let correspondence_basis = verdict.correspondence_basis();
let remap_basis = verdict.remap_basis();

let _ = (
    strategy,
    descriptor_digest,
    contract_basis,
    strategy_basis,
    merge_basis,
    merge_base_selection_basis,
    correspondence_basis,
    remap_basis,
);
```

## How It Relates To Other Features

- [Merge Planning And Verdicts](./merge-planning-and-verdicts.md) is where
  these semantics are first attached.
- [Committed Authority Transitions](./committed-authority-transitions.md)
  carries this information forward into proof-bearing authority.
- [Transition Canonical Basis, Locators, And Current-Basis](./transition-canonical-basis-locators-and-current-basis.md)
  lowers these semantics into canonical and current-basis surfaces.

## Inspection And Debugging

Inspect these first:

- `strategy_identity()`
- `strategy_descriptor_digest()`
- `strategy_contract_basis()`
- `strategy_basis()`
- `merge_basis()`
- `merge_base_selection_basis()`
- `correspondence_basis()`
- `remap_basis()`
- `branch_basis_drift()`

If a runtime says two transitions were "the same merge" but these values do not
match, they were not the same transition in the Milestone 5 sense.

## Anti-Patterns

- Do not hide strategy identity behind a runtime-local plugin name.
- Do not reduce basis-bearing semantics to one free-form comment.
- Do not treat correspondence or remap basis as optional trivia if they
  materially shaped the verdict.
- Do not invent a second strategy identity surface in receipts or provenance
  rows.

## Current Limits

- This layer defines shared vocabulary, not a strategy registry or hook
  executor.
- It does not implement correspondence engines, remap engines, or domain-specific
  solvers.

## Related Docs

- [Merge Planning And Verdicts](./merge-planning-and-verdicts.md)
- [Committed Authority Transitions](./committed-authority-transitions.md)
