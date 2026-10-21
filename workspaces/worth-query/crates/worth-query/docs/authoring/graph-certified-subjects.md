# Graph-Certified Signature Subjects

## What This Feature Is

A signed body can certify facts on related native entities without copying those
facts into its mutable root. `capture_from_path` selects the source through an
authored query path and initializes the immutable snapshot. Query derives both
the recorded correspondence and dependency-scoped supersession from that declaration.

## Why You Use It

Use this when several signable rows depend on shared parent facts while keeping
their own assignments and optional notes. A shared edit affects the rows that
actually certified that source; a local note affects only its own row.

This is business attestation, not a replacement for workflow approval authority,
read policy, assignment requirements, or authorization of the source editor.

## Stable Entry Points

Use `worth_query_decl::facade::application_schema` together with
`application_query::ApplicationQueryRootPath`:

- `creation.capture_from_path(path, source_field, snapshot_field, role)`;
- `ApplicationSigningGraphSubjectProvenance` and its native `...Binding` for v3;
- the existing `signed_by`, `supersede_on_subject_change`, and `signed_evidence`;
- `creation.bound_subject_impacts(NonZeroUsize)` after declaring the lifecycle.

Install and execute through the ordinary signed-operation entry. There is no
graph-specific signing service, invalidation callback, or dependency-list input.

## Core Mental Model

The source field stays authoritative on its original entity. The body declares
how to find it, which immutable field captures it, and whether it is `Subject`
content or only `AuditContext`. The native owner records the actual selected
identities, field presence/revision, and path-hop observations when signing.

V3 is a distinct persisted codec. It does not reinterpret v1 authentication or
v2 scope-only correspondence. Decoding provenance grants no source, signing,
approval, or mutation authority. Native current-evidence consumption validates
the installed producer contract and lifecycle, rather than guessing a codec.

The identity path is useful for optional local fields. An optional source and
optional target preserve absence; a required target requires a source value.
A missing selected entity is unavailable, not optional field absence. A required
source field cannot be certified as absent. Query requires an actual native
absence observation, including when its containing aspect has never existed.

## How It Executes

1. The installed body resolves each hop to exactly one distinct live identity,
   using the existing native query-path vocabulary and decision observers.
2. The sealer checks the captured value or absence against the native observation.
   The authenticated intent includes the complete preparation read set, not just
   the durable certified footprint.
3. Publication consumes the genuine authentication event and fills the deferred
   v3 provenance slot together with the immutable snapshot and current/history links.
4. An ordinary source mutation follows the inverse of the declared paths at its
   retained preimage. Query intersects actual source identity and field locus with
   each current row's footprint. It atomically appends supersession and removes
   current membership with the source edit. No history scan or host-authored
   consequence list is needed.

Selection is content too: replacing a selected parent supersedes dependent
evidence even when the replacement has identical field values. Replacing an edge
while preserving the same distinct endpoint does not supersede settled evidence.
It can still stale an unpublished signature because preparation observed a
different native adjacency. The same final-endpoint comparison protects minting
against changes to its own certified selection within that candidate.

## Small Example

Inside an otherwise complete signed creation, capture an optional local note:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled. Names stand for your declared fields and operation.
let creation = creation.capture_from_path(
    ApplicationQueryRootPath::from(Row::reference()),
    RowNote::reference(),
    SignedNote::reference(),
    ApplicationOperationBodyCaptureRole::Subject,
)?;
```

Both fields use the same scalar binding and unit. Declare `SignedNote` optional
and read-only. No note is required to sign solely because this capture exists;
author any business readiness requirement explicitly.

## Real Example

A row can capture quantity from its lot and an optional row-local note. This
lot/row mapping is illustrative, not a shipped NCR declaration:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled. Complete the surrounding schema, reads and body.
let creation = creation
    .capture_from_path(
        ApplicationQueryRootPath::from(Row::reference())
            .forward(RowLot::reference()),
        LotQuantity::reference(),
        SignedQuantity::reference(),
        ApplicationOperationBodyCaptureRole::Subject,
    )?
    .capture_from_path(
        ApplicationQueryRootPath::from(Row::reference()),
        RowNote::reference(),
        SignedNote::reference(),
        ApplicationOperationBodyCaptureRole::Subject,
    )?
    .signed_by::<SignBinding, _, _, _, _, _, _, _>(
        SignedActor::reference(), Provenance::reference(), "affected-lot-verification",
    )?
    .supersede_on_subject_change(
        History::reference(), Current::reference(), Supersessions::reference(),
        SupersessionRecord::reference(), NonZeroUsize::new(32).unwrap(),
    )?
    .bound_subject_impacts(NonZeroUsize::new(128).unwrap())?;
```

The operation's declared reads must cover the root, each path entity and relation,
and each source field. Paths can also use reverse hops and several connected hops.
Cold readmission checks the complete correspondence, not only the terminal name.

The compiled downstream deviation-request approval-round consumer demonstrates reverse
parent capture, two-hop tenant capture, optional local notes, real signed-evidence
consumption, and ordinary source edits. This is infrastructure usage evidence;
it does not establish a particular NCR application's business parity.

## How It Relates To Other Features

Scope-only v2 captures remain available. V3 can combine classified scope captures
with path captures in one native signed creation. `AuditContext` is authenticated
but does not control settled effectiveness. Native computed scope bookkeeping
can accompany either profile; computed signed-artifact initializers remain refused.
Computed path writes can also accompany signing: a captured parent revision may
be AuditContext while its next value publishes in the same native candidate.
The native sealer rejects any same-candidate write to an actual certified Subject
locus, including a distinct path that aliases it. No lifecycle declaration is
needed for that preimage-integrity check. See [computed graph-target writes](./computed-path-writes.md).

Current-evidence requirements use the existing typed producer reference. The
related-subject requirement selector is distinct from the capture path: the
former chooses which subject must have evidence; the latter chooses that
subject's certified source facts.

## Inspection And Debugging

V3 `dependencies()` exposes source/captured loci, actual source coordinates,
binding identity, and observed presence/revision. `selections()` exposes each
capture's ordered native identity chain. These describe the signed observation;
do not compare their stamps with today's stamps to decide effectiveness.

Inspect current membership and immutable supersession records for effectiveness.
An ordinary no-op may move a native stamp while preserving current membership.
Restoring an old value never removes a supersession or resurrects old evidence.

## Anti-Patterns

- Copying shared source truth onto every mutable row to imitate scope-only capture.
- Passing dependency lists, source coordinates, or replacement evidence from React.
- Invalidating every signature on any document edit, or merely on stage navigation.
- Checking only field names instead of actual source identities and declared paths.
- Turning decoded provenance or a successful requirement observation into authority.

## Current Limits

The profile accepts connected, guard-free paths with one distinct live endpoint
per hop, including zero hops. Missing or ambiguous selections fail closed. It
does not aggregate a collection, pick a first/latest member, or weaken a guarded path.

`bound_subject_impacts` bounds the deduplicated union of affected signable subjects
per lifecycle rule, not a fresh allowance per path. `maximum_evidence` separately
bounds current evidence per subject. Exhaustion refuses the entire mutation; it
never truncates the consequences. Fund the resulting candidate envelope in the host.
Without an explicit subject bound, native body writers retain the one-subject
default; existing controller writers retain their candidate-cardinality fallback.

Path captures or an explicit subject bound use body record v29 and canonical body
meaning v19. Earlier records retain their meaning. Publish a new immutable
package identity rather than rewriting a released package or incrementing a schema
compatibility tuple as a build counter. Signed source-field changes within the
same candidate are refused; audit-context bookkeeping remains distinct.

## Related Docs

- [Signed Operation Creations](./signed-operation-creations.md)
- [Shared Computed Body Values](./computed-body-values.md)
- [Workflows](../foundations/workflows.md)
