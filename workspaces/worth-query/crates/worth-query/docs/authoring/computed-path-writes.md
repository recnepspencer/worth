# Computed Graph-Target Writes

## What This Feature Is

An authored operation can assign a computed value to an existing related entity,
selected from its admitted scope through an exact native graph path. The related
write, scope edits, audit creation and other declared effects publish together.

## Why You Use It

Use this when editing a child must also update a parent's business revision.
Keep the parent value in its authoritative field: neither a copied input revision
nor a second host mutation supplies atomic parent/child behavior.

## Stable Entry Points

Through `worth_query_decl::facade::application_schema`:

- `body.assign_value_at_path(path, field, &value)?`;
- `ApplicationOperationBodyValue::admit(...)` for the existing checked calculation;
- `ApplicationOperationBodyPathValueAssignment` for inert transported meaning.

Use `application_query::ApplicationQueryRootPath` for identity, forward and reverse
paths. This profile supports required, writable, unitless UInt64 destinations
using the value's exact readable, bounded-admission codec. It does not provide
arbitrary input assignments, optional writes or filtered target selection.

## Core Mental Model

The operation's scope remains the path start. Each hop must select exactly one
distinct live endpoint; equal values do not make different identities equivalent.
The path declaration and decoded assignment grant no permission. The installed
operation still requires its normal authenticated principal, abilities and policy.

Query retains the selected mutation target and observes the destination field,
even when the computed value reads other facts. It never looks up that target
again during candidate construction. Scope and path destinations must be distinct:
two paths resolving to the same entity/field are refused, not applied last-wins.

## How It Executes

Native declaration derives the exact path reads and terminal-field write.
Installation checks the destination codec and declared write authority. At the
decision observation Query evaluates the shared value, validates all its sinks,
selects each path target and retains its dependencies and memory custody.

The existing candidate owner writes the prepared value to the retained target.
Path membership, endpoint liveness, destination field and expression source facts
remain publication obligations. A race stales the whole candidate, including
scope edits and audit. Exact committed replay recovers the original outcome before
reevaluating today's graph. Keep the original input, source and logical key.

## Small Example

Given an already authored `next` value and a required revision field:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
let body = EditRow::reference().body()
    .assign_value_at_path(
        ApplicationQueryRootPath::from(Row::reference())
            .reverse(ParentRows::reference()),
        ParentRevision::reference(), &next,
    )?
    .finish()?;
```

No dummy scope write is needed. Declare operation read membership for the path's
entities and field and write membership for the destination, as for other typed
body calls. The completed declaration derives actual read/write contracts.

## Real Example

A genuine row-scoped query can disclose the selected parent's revision. Bind that
exact query to the existing computed-value mechanism, then reuse its result:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
let op = EditRow::reference();
let operands = op.requirement_operands()
    .query_context::<EditRowBinding, _, _, _, _>("context", parent_revision_query())?;
let expression = worth_foundational::expression_api::expressions()
    .reading_within(ApplicationOperationBodyExpression::PROFILE)
    .parse("context.revision + uint64(1)")?;
let next = ApplicationOperationBodyValue::<
    Schema, EditRow, U64ApplicationValueBinding,
>::admit("next-revision", operands, expression)?;
let body = op.body()
    .assign_value_at_path(parent_path(), ParentRevision::reference(), &next)?
    .assign_input(RowValue::reference(), "value")?
    .create_in_scope(Audit::reference(), "audit")?
    .initialize_input(AuditIdentity::reference(), "row")?
    .initialize_value(AuditRevision::reference(), &next)?
    .finish()?;
```

The query is parameter-free and ExactlyOne, with its root selected from the row
through the parent path. Execute it with `execute_with_expression_context()` and
pass its actual row-affine source to `expect_source`. The parent assignment and
audit consume one checked result. Overflow denies before candidate publication.
The host does not parse the result DTO to calculate a new revision.

A signed creation can share that same body. Capture the observed parent revision
as `AuditContext`, not as a certified Subject that the candidate itself rewrites:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled. Complete the declared reads, writes and signing binding.
let op = SignRow::reference();
// Declare `next` for this operation and SignBinding, using the query-context pattern above.
let body = op.body()
    .assign_value_at_path(parent_path(), ParentRevision::reference(), &next)?
    .create_in_scope(Verification::reference(), "verification")?
    .capture_from_scope(
        RowValue::reference(), SignedValue::reference(),
        ApplicationOperationBodyCaptureRole::Subject,
    )?
    .capture_from_path(
        parent_path(), ParentRevision::reference(), SignedRevision::reference(),
        ApplicationOperationBodyCaptureRole::AuditContext,
    )?
    .signed_by::<SignBinding, _, _, _, _, _, _, _>(
        SignedActor::reference(), GraphProvenance::reference(), "verification",
    )?
    .finish()?;
```

Use the v3 graph-subject provenance binding for the path capture. The signed
snapshot contains the revision preimage (for example 1), while the same commit
writes the computed next revision (2). The actual native authentication intent
covers the whole candidate and retained observation, not just the snapshot.
Prepare, authenticate, sign and commit through the existing signing owner; an
unpublished signature or stale candidate publishes no bookkeeping half.

## How It Relates To Other Features

Use [path-input guards](./path-input-guards.md) to compare a parent's Company with
captured provider input; that comparison does not write the parent. Use ordinary
`assign_value` for scope writes and `initialize_value` for created fields.
Existing signature-impact installation includes path-written fields; native
dependency/lifecycle rules still determine affected evidence and their bounds.
Path writes can accompany signed creation. Both certified profiles reject writes
or deletion touching a captured Subject's actual entity/field, including when
different paths alias it. Obvious same-path conflicts fail declaration/readmission;
dynamic aliases fail native signing preparation with `CandidateMismatch`.
AuditContext is authenticated preimage data, not a continued-effectiveness
dependency. This rule applies even without a supersession lifecycle declaration.
Computed initializers inside the signed artifact remain unsupported; capture
its classified values from native observations instead.

## Inspection And Debugging

Missing or ambiguous path endpoints produce native execution denials. Duplicate
actual destinations produce `ConflictingEffectStep`. Missing declared reads,
writes, invalid field presence or codec correspondence fail fresh admission or
installation. Authored paths participate in canonical schema/program identity
and portable body v33; older bodies keep their existing format/identity paths.

## Anti-Patterns

Do not calculate the parent's new value in the host, pass an arbitrary target ID,
split the writes into separate commits, use first/latest selection, or re-query
the target after decision. Do not normalize or retry a stale lifecycle command
automatically. Read descriptions and receipt identities grant no mutation authority.

## Current Limits

Guard-free identity/forward/reverse paths only; every hop is a single selection.
Required unitless UInt64 computed writes only. No optional replacement, dynamic
collection mutation or general cross-entity effect language. Signed snapshots
capture observed preimages, not computed postimages or caller-built authority.
Expression profiles and explicit request, decision and signature-impact limits remain.
This proves native atomic graph writes, not distributed provider transactions,
Store restart durability or a complete application endpoint.

## Related Docs

- [Shared computed values](./computed-body-values.md)
- [Native query expression contexts](./native-query-expression-contexts.md)
- [Scope-bound operation bodies](./scope-field-operation-bodies.md)
