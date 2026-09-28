# Declaration Entry Readiness

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

Declaration entry readiness is the Query-owned family-level support and
readiness projection over the declaration-entry seam ledger.

It turns one declaration family plus one admitted operating world into one
readiness report that keeps envelope, relational, bridge, and signal seam
rows aligned with the same retained seam inventory.

For temporal-capable families, readiness now also projects whether later
declaration-entry seams would require unsupported temporal runtime support or
an invalid preview or historical truth basis before those seams are attempted.
For async-capable families, readiness now also projects whether later
declaration-entry seams would require unsupported async runtime support or an
invalid source, lifecycle, or preview/historical basis posture before those
seams are attempted.

## Why You Use It

- inspect declaration-entry seam posture before doing more expensive work
- keep envelope, relational, bridge, and signal readiness rows synchronized
  with one shared seam-ledger inventory
- distinguish admitted, deferred, unsupported, and invalid-context seam rows
  without relying on trial and error
- compare family-level readiness with the narrower relational, bridge, and
  signal support helpers

## Stable Entry Points

- `WorthQueryDeclarationEntryReadinessReport`
- `WorthQueryDeclarationEntryReadinessRequest`
- `WorthQueryDeclarationEntryRetainedSubjectInput`
- `WorthQueryDeclarationEntryReadinessRow`
- `WorthQueryDeclarationEntryReadinessStatus`
- `WorthQueryDeclarationEntryContributionEvidence`
- `WorthQueryDeclarationEntryContributionEvidenceSet`
- `WorthQueryDeclarationEntryContributionComposition`
- `WorthQueryDeclarationEntryContributionCompositionError`
- `WorthQueryDeclarationEntryCrossingInventory`
- `WorthQueryDeclarationEntryCrossingRow`
- `WorthQueryDeclarationEntryCrossingSurface`
- `WorthQueryInstalledDomainDeclarationContext::declaration_entry_readiness::<I>()`
- `WorthQueryInstalledDomainDeclarationContext::try_declaration_entry_readiness::<I>(...)`
- `WorthQueryInstalledDomainDeclarationContext::declaration_entry_crossing_inventory::<I>()`

## API Reference

Admitted-handle readiness:

- `declaration_entry_readiness::<I>() -> WorthQueryDeclarationEntryReadinessReport<D, I>`
- `try_declaration_entry_readiness::<I>(request) -> Result<WorthQueryDeclarationEntryReadinessReport<D, I>, WorthQueryDeclarationEntryContributionCompositionError<D, I>>`
- `declaration_entry_crossing_inventory::<I>() -> WorthQueryDeclarationEntryCrossingInventory<D, I>`

Readiness request construction:

- `WorthQueryDeclarationEntryReadinessRequest::base()`
- `WorthQueryDeclarationEntryReadinessRequest::with_contribution_evidence(...)`
- `WorthQueryDeclarationEntryReadinessRequest::for_retained_subject(...)`
- `WorthQueryDeclarationEntryReadinessRequest::with_admitted_plan_scope(...)`
- `WorthQueryDeclarationEntryReadinessRequest::with_lower_runtime_boundary_scope(...)`

Readiness report inspection:

- `declaration_family_key()`
- `rows()`
- `contribution_composition()`
- `readiness_digest()`

Readiness row inspection:

- `crossing_row()`
- `status()`
- `reason()`
- `envelope_aspect_publication()`
- `relational_authority_summary()`
- `bridge_authority_summary()`
- `signal_authority_summary()`
- `readiness_digest()`

Readiness statuses:

- `Admitted`
- `Deferred`
- `Unsupported`
- `InvalidBasis`

## Core Mental Model

Readiness is the family-level seam answer to:

"Which declaration-entry crossings are structurally available for this family
inside this admitted world?"

That means:

1. the crossing inventory describes the seam rows
2. the readiness report adds family-level admitted/deferred/unsupported
   posture to those same rows
3. the narrower relational, bridge, and signal support helpers should agree
   with the matching readiness rows, including status instead of only sharing a
   descriptive reason string
4. optional declaration-scoped contribution evidence may be attached
   explicitly, but readiness remains the source of declaration-entry seam truth
5. when contribution evidence should be reconciled against one concrete
   retained declaration crossing, attach a retained seam subject explicitly so
   readiness can fail closed on wrong-handle, wrong-world, or mismatched
   declaration digest posture
6. retained-subject-aware readiness now carries the retained envelope
   publication plus relational, bridge, and signal authority summaries on the
   matching rows, but it still does not pretend those lower-authority surfaces
   executed successfully
7. if a retained envelope publication is missing, conflicting, or only
   partially maps the required lower-authority slice, the matching readiness
   row no longer stays `Admitted` just because the family and config posture
   were broadly available
8. bridge rows now also fail closed when the retained envelope publication is
   broadly available but the narrower mapped continuation slice is only
   partial, missing, or conflicting
9. stronger admitted-plan-bound and lower-runtime-bound contribution categories
   require matching retained downstream proof explicitly; readiness does not
   infer that proof from family posture alone
10. the narrower relational, bridge, and signal support helpers remain
   declaration-entry seam projections; declaration-entry readiness is the
   composed public support surface when contribution evidence matters
11. temporal-capable families can now surface `Deferred` or `InvalidBasis`
   bridge and signal rows before route planning when their later seam posture
   would require unsupported temporal runtime support or an invalid preview or
   historical truth basis
12. async-capable families can now surface `Deferred`, `Unsupported`, or
    `InvalidBasis` bridge and signal rows before route planning when their
    later seam posture would require unsupported async runtime support, an
    unsupported source/lifecycle regime, or an invalid preview or historical
    basis

Readiness is not a substitute for retained inspection of a concrete declaration
artifact. It answers family-level seam posture, not concrete legality or
progression outcomes for one already-authored declaration.

Readiness is also not the remediation surface for one concrete stopped run. If
an ordinary, checked, or proof-visible declaration-entry lane already stopped,
use typed-stop remediation guidance instead of inferring a response from
family-level readiness rows.

Readiness also is not an orchestration transcript surface. It answers which
declaration-entry seam rows are structurally available for a family in one
admitted world. It does not describe which stages one specific orchestration
run crossed.

## Small Example

```rust
let readiness = handle.declaration_entry_readiness::<ReclassifyBoundaryLoop>();

for row in readiness.rows() {
    let _ = row.crossing_row().entrypoint_key();
    let _ = row.status();
    let _ = row.reason();
}
```

## Real Example

```rust
let request = WorthQueryDeclarationEntryReadinessRequest::base()
    .with_contribution_evidence(evidence);

let readiness = handle.try_declaration_entry_readiness::<ReclassifyBoundaryLoop>(request)?;

let _ = readiness.contribution_composition();
let _ = readiness.readiness_digest();
```

```rust
let request = WorthQueryDeclarationEntryReadinessRequest::base()
    .with_contribution_evidence(evidence)
    .for_retained_subject(
        WorthQueryDeclarationEntryRetainedSubjectInput::envelope_checked(checked_envelope),
    )
    .with_admitted_plan_scope(admitted_plan);

let readiness = handle.try_declaration_entry_readiness::<AttachFaceMaterialAssignment>(request)?;

let _ = readiness.contribution_composition();
let _ = readiness.readiness_digest();
let _ = readiness
    .rows()
    .iter()
    .find(|row| row.crossing_row().bridge_continuation_family().is_some())
    .and_then(|row| row.bridge_authority_summary());
```

```rust
let request = WorthQueryDeclarationEntryReadinessRequest::base()
    .with_contribution_evidence(evidence)
    .for_retained_subject(
        WorthQueryDeclarationEntryRetainedSubjectInput::envelope_checked(checked_envelope),
    );

let readiness = handle.try_declaration_entry_readiness::<PublishRevisionWithToleranceProfile>(request)?;

let _ = readiness.contribution_composition();
let _ = readiness.readiness_digest();
```

## Inspection And Debugging

Use readiness when you need to know:

- which seam rows exist for this family
- which lower owner crate each row belongs to
- whether a seam row is admitted, deferred, unsupported, or invalid here
- which envelope-published slice and authority-specific mismatch posture the
  retained subject currently implies for each row
- whether a row became unsupported because the retained semantic slice was
  missing or conflicting, rather than because the broad family/config support
  was absent
- whether a bridge row became unsupported because the retained envelope slice
  existed broadly but did not map cleanly into the narrower continuation slice
- whether the narrower support helpers still match the shared seam ledger
- whether a temporal-capable family is blocked because the temporal runtime
  support row is still gated for sibling-root DX, because the declaration
  family itself remains deferred, or because a preview or historical truth
  basis is structurally invalid for that temporal seam
- whether an async-capable family is blocked because the async-resource runtime
  support row is still gated for sibling-root DX, because the retained async
  declaration uses an unsupported source or lifecycle regime, or because a
  preview or historical basis is structurally invalid for that async seam
- whether explicit declaration-scoped contribution evidence should travel with
  the family-level readiness answer
- whether attached contribution evidence actually matches one retained
  declaration-entry subject before composition is admitted
- whether stronger admitted-plan-bound or lower-runtime-bound contribution
  targets were backed by matching retained downstream proof rather than family
  posture alone

## Anti-Patterns

- do not treat readiness as proof that one concrete declaration already
  crossed a lower seam
- do not use readiness to replace route, receipt, envelope, or signal
  inspection on a retained declaration artifact
- do not use readiness to explain one proof-visible orchestration transcript
- do not assume `declaration_entry_readiness::<I>()` discovers domain
  contributions automatically; use the explicit request path when you need
  composition

## Current Limits

- readiness is family-level seam posture, not concrete declaration legality
- temporal readiness is still a family-level seam projection; concrete
  declaration-specific temporal legality remains a separate surface
- async readiness is still a family-level seam projection; concrete
  declaration-specific async legality remains a separate surface
- composed contribution evidence is an overlay on the readiness report, not a
  rewrite of the crossing inventory or readiness rows
- retained-subject-aware readiness strengthens contribution reconciliation, but
  it still does not turn readiness into a concrete declaration inspection
- declaration-bound admission, support, and explanation evidence compose
  directly
- admitted-plan-bound evidence composes only when both a retained
  declaration-entry subject and matching retained admitted-plan proof are
  attached
- lower-runtime-bound explanation and consequence evidence compose only when both
  a retained declaration-entry subject and matching retained lower-runtime
  boundary proof are attached
- it does not replace retained route, receipt, envelope, or signal inspection
- later orchestration and recovery remain separate surfaces

## Related Docs

- [Declaration Entry Orchestration](./declaration-entry-orchestration.md)
- [Declaration Entry Inspection](./declaration-entry-inspection.md)
- [Declaration Signal Compatibility](./declaration-signal-compatibility.md)
- Runtime-Installed Domain Handles
- [Typed Stops And Remediation Guidance](./typed-stops-and-remediation-guidance.md)
