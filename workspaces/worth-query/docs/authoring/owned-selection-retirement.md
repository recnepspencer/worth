# Retire Owned Selections When an Input Changes

## What This Feature Is

An operation body can retire current selections beneath its persistent holder
rows when the body's source-field input changes. The declared path is
`scope → holders → optional selected snapshot`. Query observes the old source,
compares the already admitted input, and publishes the source write, selection
unlinks and snapshot deletions in one native candidate. The scope and holders
survive. There is no host cleanup loop or later repair mutation.

## Why You Use It

Use this for replaceable dependent snapshots, such as selected lots beneath an
NCR's affected-lot rows. A different company makes those selections obsolete;
reselecting the same company must preserve them. Keep the rows themselves,
including their independently authored assignments.

Do not use this to delete historical revisions, approvals or audit evidence.
Deleting a snapshot is a domain decision, not a synonym for signature
supersession or replacing a current pointer while retaining history.

## Stable Entry Points

Declare through `worth_query_decl::facade::application_schema`:

```rust,ignore
body.retire_owned_selections_on_input_change(
    SourceField::reference(), "source_member",
    OwnedRows::reference(), CurrentSelection::reference(),
    std::num::NonZeroUsize::new(256).unwrap(),
)?
```

Finish and install through the ordinary operation-body funnel. There is no
special host handler, cleanup endpoint, or caller-supplied list of deleted IDs.
`ApplicationOperationBodyOwnedSelectionRetirement` is inspectable descriptive
meaning; decoding it grants no permission or native target custody.

## Core Mental Model

The rule must match an exact `assign_input` or `set_optional_input` in the same
completed body: field, member name, scalar family and binding all correspond.
Query reuses that source write's original prepared scalar slot. It does not
serialize the input a second time or fetch the provider again. An absent optional
source differs from the required incoming scalar. Same-value writes observe the
source comparison but do not read or retire any holder selections.

The outer relation has at most one owner per holder
(`target_max == Some(1)`). The inner relation has at most one outgoing edge per
holder and at most one incoming edge per selected snapshot (`source_max` and
`target_max` both `Some(1)`). These are declared integrity constraints, including
duplicate edges, not inferred ownership from a business identifier or context.

The selected entity kind must differ from the scope and every holder kind used
by this body's rules. Duplicate relation paths are refused. If different rules
still select the same actual snapshot, execution refuses the whole candidate.

## How It Executes

After ordinary authorization and input admission, Query compares the source at
the native decision observation. If changed, it observes complete outer
membership and every inner adjacency, including empty selections, plus endpoint
liveness. It retains the actual native mutation targets with request-accounted
memory. Candidate construction consumes those targets through the existing
unlink/delete owner; it does not look up business IDs again.

A later holder membership change, selection replacement or endpoint lifecycle
change can stale publication. Nothing from the losing candidate commits—not the
source write, cleanup or co-created audit. Exact logical-request replay recovers
the original commit before evaluating today's source or selections.

The nonzero holder bound limits prepared retirement plans and effects. Outer
traversal also obeys the existing request/projection work and memory budgets;
this is not an independently optimized paginated discovery API. Exceeding the
bound refuses before applying a prefix. Installation requires declared Delete
and Unlink authority; the native owner grows actual effect storage fallibly,
without supplier-authored candidate cardinality forecasts.

## Small Example

```rust,ignore
let body = SelectSource::reference().body()
    .assign_input(SourceField::reference(), "source")?
    .retire_owned_selections_on_input_change(
        SourceField::reference(), "source",
        OwnedRows::reference(), CurrentSelection::reference(),
        std::num::NonZeroUsize::new(32).unwrap(),
    )?
    .finish()?;
```

The operation needs typed Read capabilities on scope, holder, selected kind,
source field and both relations; Unlink on the inner relation; Delete on the
selected kind; and the ordinary source Write capability. Registration derives
the corresponding installed read and program membership. Cold readmission checks
the full correspondence. Those declarations do not replace request authorization.

## Real Example

For a company selection with a provider-admitted company value:

```rust,ignore
let body = SelectCompany::reference().body()
    .expect(Status::reference(), NcrStatus::Active)?
    .expect(Stage::reference(), NcrStage::Initiation)?
    .set_optional_input(Company::reference(), "company")?
    // Include all admitted provenance/revision writes in this same body.
    .retire_owned_selections_on_input_change(
        Company::reference(), "company",
        NcrAffectedLots::reference(), AffectedLotSelection::reference(),
        std::num::NonZeroUsize::new(256).unwrap(),
    )?
    .create_in_scope(ChangeAudit::reference(), "audit")?
    .initialize_input(AuditCompany::reference(), "company")?
    .link_from_scope(NcrAudits::reference(), "audit-owner")?
    .finish()?;
```

This is an illustrative domain declaration, not a claim that all NCR product
joins are already integrated. Provider capture/revision admission remains at the
external-input port. Provider freshness is not atomic with graph publication.
Retain the exact original input, source and logical key through the owning action.
Company is optional, so first selection from absence is a change. The unsigned
audit shown records the new value in the same publication; it does not fabricate
an absent old company as empty text, or promise an optional preimage initializer
that this example does not declare. It is not a signature. Same-company input still performs the body's other writes
and audit creation, but preserves selections.

## How It Relates To Other Features

`replace_from_scope` replaces one current pointer with an actual new creation
while retaining its predecessor. This rule instead removes obsolete dependent
selections across bounded persistent rows and deletes their selected snapshots.
Use separate history relations where the domain retains history.

Existing native signature lifecycle laws still own dependency-scoped
supersession. This consequence does not add a signature invalidation loop or
clear assignments. Native requirements and authorization remain independently
enforced. A requirement assessment is not a cleanup preview or execution permit.

## Inspection And Debugging

Inspect `body.retirements()` for its source, paths and bound. The meaning
participates in schema/program identity; canonical body v21 and archive body v31
apply only when rules are present. Existing bodies keep prior encoding paths.
Inspect current state through governed queries after commit. Native capacity,
overlap, resource and stale outcomes must remain typed rather than being mapped
to an ordinary domain rejection or silently retried with another key.

## Anti-Patterns

Do not precompute selected IDs in a host, delete only the first page, apply a
partial cleanup, re-fetch provider data on replay, infer ownership from labels,
or increment a signature counter. Do not use this destructive profile for
records whose retention policy requires preserving their historical snapshots.

## Current Limits

This profile supports unsigned source consequences only. Combining it with a
signed creation is rejected during body completion, cold admission and native
installation. Unsigned audit creation is supported. Arbitrary-depth traversal,
conditional source Clear, Option-valued source replacement and parent-path input
comparisons are not supplied by this method. A scope-field guard is not a
comparison with a field on a related parent.

## Related Docs

- [Scope-bound operation bodies](./scope-field-operation-bodies.md)
- [Graph-certified subjects](./graph-certified-subjects.md)
- [Owning application front door](../foundations/ordinary-application-front-door.md)
