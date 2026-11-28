# Causal audit reads from a Home session

An audit or "why is this record in this state?" view needs four answers:
what the record looked like at an earlier commit, what changed and when, who
advanced its workflow, and what an operation wrote. The Host session answers
each in one call through the governed request owners described in
[record and aspect history](record-and-aspect-history.md),
[causal inspection](cross-runtime-causal-inspection.md) and
[workflows](../foundations/workflows.md). The calls add no authority: every
read admits the session principal afresh and refuses instead of returning less
than the principal may see.

All names below are on `worth_query_host::facade::audit` unless noted.

## Pin an earlier commit

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled. The compiled journey is
// crates/worth-query-court-decl-host-integration/tests/worth_query_court_decl_host_integration/
//   host/application_home/workflow_durability/desktop_sessions/audit.rs
let then = session.snapshot_at_commit(&receipt, NonZeroUsize::new(256).unwrap())?;
let rows = then.query(intent)?;              // reads the pinned commit
then.with_request(|at| at.read_workflow_history::<Spec, Subject, Audit>(..));
```

| Call | Selects |
|---|---|
| `session.snapshot_at_commit(&receipt, work)` | The commit a live receipt of the session's branch committed |
| `session.snapshot_at_recorded_commit(durable_commit, work)` | A durable commit coordinate kept in an audit log, including before a Directory reopen |
| `session.snapshot_at_original_receipt(&recovered, work)` | The commit an original recovered receipt names |

Each returns an ordinary `WorthQuerySessionSnapshot`. Selection walks at most
`work` entries of the session branch's product history. A commit of another
branch, or one beyond the bound, refuses with
`WorthQuerySessionHistoricalSnapshotDenial::History(CommitUnavailable)`. Reads
through the snapshot still admit the principal at the head, so a pin never
widens who may read.

## Read a record's governed change log

`session.record_history::<History>(request)` (or the same call on a snapshot)
captures the subjects of the history's `SubjectRead`, opens the history and
pages it to completion or to `limits.maximum_pages` in one advancement:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
let log = snapshot.record_history::<GovernedChanges>(WorthQueryRecordHistoryRequest {
    subject: SubjectInput { identity: id.clone() },          // the SubjectRead intent
    read: WorthQueryGovernedReadAccess::new(PayloadCapability::reference(),
        PayloadRead::reference(), grant_input.clone()),
    metadata: WorthQueryGovernedReadAccess::new(MetadataCapability::reference(),
        MetadataRead::reference(), grant_input),
    limits,                                                  // capture, page, maximum_pages
})?;
for change in log.changes() {
    show(change.commit(), change.version(), change.structural_change(), change.changed_aspects());
}
```

`steps()` keeps the walk order: each `ProductCommit { commit, root_import }`
the walk visits, then the `Change`s it carried, newest first. `completeness()`
is `PageLimitReached` when pages remain. Both capabilities are admitted for the
session principal at capture, at opening and for every page; a revoked grant
refuses the next page. A principal without the read grant refuses with
`WorthQueryGovernedHistoryDenial::Request(..)`; a capability that does not
satisfy the history's metadata contract refuses with `History(..)`.

The World walk needs custody of each product parent it steps through, and a
Home has no retention setting that keeps them: a product commit nobody holds is
retired once newer commits supersede it. An audit view that must walk back
across commits retains custody of each of them as it lands: keep the session
snapshot taken after each commit (`session.snapshot()`), or the commit's
receipt, for as long as the history must stay readable. Pinning only an old
commit does not keep the commits between it and the head. A walk that reaches
a retired parent refuses whole; `denial.is_retired_history_gap()` is `true` for
exactly that refusal, and its `user_summary()` says older history is no longer
kept, distinct from work, capacity, deadline, branch and continuation stops.

Record history records no actor. Actor attribution is a workflow fact.

## Read a workflow instance's audit trail

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
let trail = snapshot.workflow_audit::<Spec, SubjectRead, AuditRead>(
    home.vocabulary(),
    WorthQueryWorkflowAuditRequest { target, subject, audit, page_width: 32,
        indexes: WorthQueryWorkflowIndexRead::PublishedOnly, maximum_pages },
)?;
for transition in trail.transitions() {
    if let WorthQueryWorkflowHistoryAudit::Visible { advancer, executor, proposer, approver } =
        transition.audit() { /* actor entities */ }
}
```

This follows `read_workflow_history` continuations at the snapshot's commit.
Each page admits the subject read and the audit read afresh. `Withheld` means
the audit read did not admit this principal, never that a role was absent.

## Combine both at one commit

`snapshot.causal_timeline::<History, _, SubjectRead, AuditRead>(record,
vocabulary, workflow)` reads both streams at the same pinned commit and returns
`WorthQueryCausalTimeline { commit, record, workflow }`. `session.causal_timeline`
pins the session's current commit first. The two streams stay separate: record
history carries commit and version evidence but no actor, and workflow history
carries actors but no per-transition commit, so the timeline never joins them by
inference. Either stream's refusal refuses the whole timeline.

## Explain what an operation wrote

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
let provenance = session.source_provenance::<SourceDisclosure>(
    &receipt,
    WorthQuerySourceProvenanceRequest { metadata, diagnostic },
)?;
if let Some(field) = provenance.field("retention_days") {
    (field.effect(), field.value());
}
```

This is `request.inspect_source::<SourceDisclosure>(&receipt).why_performed()
.materialize(source_effects()).execute_governed(..)` with both capabilities
admitted for the session principal immediately before the inspection consumes
them, and the inspection runs under the session's execution policy. The receipt
must belong to the session's branch: a write on another branch, such as an
unadopted preview, refuses with `WorthQuerySourceProvenanceDenial::ForeignBranch`;
explain it from a session `on_branch(preview.branch())`. `source_effects_outcome()`
says why effects were not materialized (for example
`UnsupportedSourceProjectionShape` when the projection reaches past the
disclosed record's own fields).

Field effects are materialized for the record the source disclosure scopes,
and field diagnostics admit only a disclosure query rooted at that record: no
root path, orbit or component selection, continuation or live mode. Any other
shape refuses with `Inspection(..)` after both capabilities are admitted, and
`denial.is_unsupported_field_shape()` is `true` (its `user_summary()` is
`Unsupported`, never a permission or capacity refusal).

The common case is a creation scoped by its parent. Installation requires a
source disclosure to be scoped like its operation, so a Site-scoped enrollment
that creates a Dossier can only disclose the Dossier through a root path
(`ApplicationQueryRootPath::from(Site).reverse(DossierSite)`). Field values for
that shape would need two things inspection does not have: evaluating the
disclosure query at the original commit under fresh governance to learn which
child records it discloses (the receipt retains the scope and parameters, not
the selected children), and a diagnostic contract that covers root-path reads.
Using every child the commit wrote instead would disclose records the query's
parameters exclude, so the shape refuses rather than guess.

The governed receipt itself stays readable:
`request.inspect_source::<D>(&receipt).why_performed().reference_only()
.execute_governed(metadata)` admits the metadata capability and returns the
source commit, and `record_history` over a SubjectRead of the child lists the
aspects that commit changed. A `#[derive(WorthQueryAction)]` declares its
source disclosure with `inspection_disclosure = SourceDisclosure`
([court](../../crates/worth-query-court-decl-host-macros/tests/worth_query_court_decl_host_macros/standing_grant_dx/provenance_court.rs)).

## Find what depends on an entity type and what references a record

`session.entity_impact(Entity::reference())` reads installed meaning only:

- `incoming()`: relations targeting the entity, each with the installed queries
  that read it in reverse (`reverse_reads()`; empty means no governed
  referenced-by read exists for that relation). A query is listed exactly when
  `referenced_by` would admit it: scoped on the entity, every root path starting
  with a reverse step into it;
- `outgoing()`: relations the entity references;
- `dependencies()`: each capability (`GrantEntity`, `ScopeEntity`,
  `ResourceHead`, `Traversal`, `TargetRelation`) and ability policy
  (`ScopeEntity`, `PrincipalEntity`, `Traversal`) whose authority depends on it.

`session.referenced_by(intent)`, `referenced_by_governed(intent, access)` and
`snapshot.referenced_by(intent)` read the records referencing the intent's scoped
record. WORTH declares disclosure per query, not per relation, so the intent must
be an installed query whose every root path starts by stepping backward from its
scope entity along an incoming relation (`ApplicationQueryRootPath::from(X).reverse(Rel)`).
Any other query refuses with `NotAnIncomingReference` before admission; the
read itself keeps the query's own principal, scope, disclosure and capability
admission.

## Refusals for operators

Every refusal above is typed, and each implements
`WorthQueryUserFacingRefusal` (`worth_query_host::facade`): `user_summary()`
returns a `WorthQueryRefusalCategory` and one sentence safe to show, so an
audit UI never prints `{denial:?}`. Log the typed refusal for developers.

| Refusal | Typed classifiers |
|---|---|
| `WorthQueryGovernedHistoryDenial` (record history) | `is_retired_history_gap()` |
| `WorthQueryWorkflowObservationDenial` (workflow audit) | match its variants |
| `WorthQueryCausalTimelineDenial` | `is_retired_history_gap()` (record stream) |
| `WorthQuerySourceProvenanceDenial` | `is_unsupported_field_shape()`, `ForeignBranch`; inspection stops classify by their exact resource and owner cause (an expired session renders `SessionExpired`) |
| `WorthQueryReferencedByDenial` | `NotAnIncomingReference { query }` |
| `WorthQuerySessionHistoricalSnapshotDenial` | `History(CommitUnavailable)` |
| `WorthQueryEligibilityDenial` (eligibility, rosters, authority inventory) | `Observation(RosterNotDisclosed { .. })` and the other variants |

A declaration or caller defect (an uninstalled capability, a query that reads
no incoming link) renders `Internal`; a withheld disclosure renders
`PermissionDenied`; a retired history gap or an unknown earlier commit renders
`NotFound`.

## Workflow Homes

`WorthQueryWorkflowApplicationHome` forwards the plain Home's
`correction_history(_with_scope)`, `undo`/`redo` (and `_with_key`,
`_authored`), `fork_preview(_with_scope)`, `continue_preview_recovery`,
`adopt_preview`, `close_branch`,
 `catch_up_temporal(_observing)` and
`pump_temporal(_observing)` with the same signatures. `discard_preview(preview)`
replaces `preview.discard(&home)`, because a workflow Home does not lend its
plain Home. Every session call above works on its sessions and, through
`with_workflow`, on an owned workflow session's call worker.
