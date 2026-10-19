# Workflow succession output affinity

## What This Feature Is

A supplier-authored identity match between an output captured by a completed
workflow and an existing subject selected by its successor operation. Use it
when several workflow chains can share one item: matching the item is not enough
to know which round, revision, or episode the new work continues.

## Why You Use It

A completed review of Round A must not reopen whichever round happens to be
current now. If another independent chain certified Round B, renewal from A
must refuse, even though both chains belong to the same item and A has no child.

## Stable Entry Points

Use the existing body builder from `worth_query_decl::facade::application_schema`:

- `succeed_workflow::<Spec, Binding, SubjectRole>()` establishes native succession.
- `preserve_related_output::<Binding, SelectedRole>(relation)` captures selection.
- `require_predecessor_output::<Binding, SelectedRole, Producer, ProducedRole>()`
  declares the identity equality.

Both roles require ExactlyOne cardinality and the same entity marker.
SelectedRole must be Preserve; ProducedRole belongs to the named producer's
output contract. Cold declaration readmission checks the actual descriptors.
Vocabulary installation requires that producer to be a registered guarded
workflow operation. No second host registration or output-ID input is needed.

## Core Mental Model

The producer retains an actual entity identity when its guarded operation
performs. Native history, not a host receipt cache or today's business pointer,
owns that evidence. The successor captures its existing selection through its
own authored body at the native decision basis.

The selector means **latest performed occurrence of the named producer binding**,
ordered by validated workflow occurrence. It does not mean final operation,
largest entity ID, or latest workflow anywhere. It chooses the occurrence
before reading its role: missing or legacy context there refuses rather than
falling back to an earlier usable occurrence.

The match proves identity equality only. The supplier's graph determines what
that producer certified and how it relates to parent closure. Later field edits
or deletion do not rewrite historical identity; freshness of signatures and
business eligibility remain separately authored requirements.

## How It Executes

Native succession admits the capability and completed predecessor, reconstructs
its supported pinned definition and exact history, then carries the declared
producer output into the existing succession authority. Binding the successor
compares it with the sealed candidate's Preserve output. Neither the host nor a
post-commit lookup chooses that entity.

Matching and selection dependencies travel with the existing native read set.
A competing publication stales the complete candidate; no losing business row
or native successor is published. An available but unequal target produces
`WorkflowPredecessorOutputMismatch`; unavailable historical correspondence
produces `WorkflowPredecessorOutputUnavailable`, inside the ordinary typed
workflow settlement denial.

These are fresh-commit refusals. Exact retained idempotent replay resolves first,
including after the current selection changes or the predecessor has a child.
Changed input/key is not exact replay.

## Small Example

Given declared roles and a CurrentRound relation:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// Schema markers and output contracts are declared by the application.
let body = RenewRoundOperation::reference().body()
    .succeed_workflow::<ReviewWorkflow, RenewBinding, ExistingItem>()?
    .preserve_related_output::<RenewBinding, SelectedRound>(CurrentRound::reference())?
    .require_predecessor_output::<
        RenewBinding, SelectedRound, CompleteRoundBinding, ClosedRound
    >()?;
```

ExistingItem names the admitted scope. SelectedRound and ClosedRound both name
the round entity. The declaration carries correspondence, never an entity ID.

## Real Example

Complete the same body with a real unsigned replacement:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
// Continue the preceding application-specific builder.
let body = body
    .create_in_scope(ApprovalRound::reference(), "approval-round")?
    .initialize_input(RoundIdentity::reference(), "identity")?
    .initialize_optional_from_related(
        CurrentRound::reference(), RoundDescription::reference(), RoundDescription::reference(),
    )?
    .initialize_from_related(
        CurrentRound::reference(), RoundAssignee::reference(), RoundAssignee::reference(),
    )?
    .link_from_scope(RoundHistory::reference(), "history")?
    .replace_from_scope(CurrentRound::reference(), "current")?
    .create_output::<RenewBinding, RenewedRound>()?
    .finish()?;
```

The match, copy initializers and selection replacement share the native selected
preimage. Historical signatures stay on their actual old round; this creation
does not copy approval grants or native workflow history. The existing
prepare/bind succession lane publishes the body and fresh instance together.

## How It Relates To Other Features

Use related signed-evidence requirements at certification to prove the actual
selected round is signed/current. Use this clause at succession to identify the
same historical round. Neither replaces the other. Retained output metadata
has its existing settlement/history budgets and immutable audit references.

## Inspection And Debugging

The clause participates in canonical schema and program meaning. Inspect the
body's succession output match for selected role, producer binding and exact
producer role descriptor. Portable body profile 23 carries it; older profiles
refuse its tag. Ordinary succession without a match keeps its prior encoding.
Decoded descriptions grant no execution authority.

## Anti-Patterns

Do not pass a chosen closing-round ID, compare display strings in the host,
reinterpret a preserved root as a round, or use an ordinary operation receipt
instead of a performed workflow occurrence. Do not label the selector as proof
of business closure without the authored graph establishing that relationship.

## Current Limits

One fixed ExactlyOne match is supported per succession clause. Optional roles,
role families and multiple matches are not part of this surface. The feature
does not automatically decide destinations, business-current policy, actor
policy, or cross-release adoption.

It does not reacquire a completed instance command locator after checkpoint
restoration, serialize live authority, or establish Store crash durability.
Those remain separate boundaries; ForeignOwner checks are not bypassable.

## Related Docs

- [Completed-chain succession](workflows.md#completed-chain-succession)
- [Retained operation correspondence](workflows.md#retained-operation-correspondence)
- [Scope-bound operation bodies](../authoring/scope-field-operation-bodies.md)
