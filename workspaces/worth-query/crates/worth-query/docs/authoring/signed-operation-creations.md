# Signed Operation Creations

## What This Feature Is

A signed creation publishes an immutable business snapshot with an actor and
native authentication provenance in the same commit. It can run before a workflow
approval frontier. It does not manufacture workflow approval authority.

## Why You Use It

Use it when a person may attest to complete information while the document is
still being gathered—for example, an assigned verifier signing a ready Draft.
Saving incomplete source information remains a different ordinary operation.

## Stable Entry Points

Declare through `worth_query_decl::facade::application_schema`:

- `creation.signed_by::<Binding, _, _, _, _, _, _, _>(actor, provenance, purpose)`;
- `ApplicationSigningProvenance` and `ApplicationSigningProvenanceBinding`;
- `ApplicationSigningSubjectProvenance` and
  `ApplicationSigningSubjectProvenanceBinding` for the v2 certified footprint;
- `creation.initialize_from_present_scope(optional_source, required_snapshot)`;
- `creation.capture_from_scope(source, snapshot, ApplicationOperationBodyCaptureRole)`
  and `capture_from_present_scope` for explicitly classified v2 captures;
- `operation.requirement_operands().principal_identity::<Binding>(name)`.

Install the body through the usual contribution's `operation_body::<Binding>()`.
Install a genuine authentication-event owner using
`worth_query_host::facade::admission::authentication_event`, then register its
`signing_owner()` with `program.install_signing_owner(revision, owner)`.
Retaining a workflow specification registers its owner for that same revision.
Another owner cannot replace it; retaining the same owner again is harmless.

Prepare through the ordinary application entry's `prepare_in_program`, or
`prepare_capability_in_program` for a capability binding. The retained candidate
exposes `authentication_intent()`, accepts `sign(&event)`, then `commit(...)`.
These methods come from `worth_query_host::facade::application_entry`.

## Core Mental Model

The supplier authors the snapshot fields, purpose, requirements and assignment
rule once. Query owns the actor initializer and deferred provenance slot. Neither
appears as a caller input. Required snapshot fields must all be initialized, and
every field on the signed entity must be read-only. Other unsigned creators,
updates and deletes cannot be an alternate way to manufacture or rewrite it.

The intent binds the actual prepared effects, installed operation meaning, exact
program revision, logical key, input, original authenticated principal, snapshot
values and tracked native source revisions. A value changed and restored is not
the original observation. The event is issued and consumed by the fixed owner,
not by a verifier selected by the person submitting the request.

Provenance is descriptive persisted data. Deserializing it cannot reconstruct an
authentication event, a consumed proof, a workflow approval or commit authority.

V2 also records the exact source entity, source field, captured snapshot field,
value binding and native revision for each explicitly classified `Subject` capture. Query derives
that association from the same installed body; the caller supplies no dependency
list. Repeated use of one source retains each distinct capture. Principal/permission
and lookup facts are not added to this certified footprint. They still participate
in full authorization and candidate-currentness checks.

Choose `AuditContext` for copied context that must be authenticated but does not
determine continued signature validity, such as the document revision or stage
at signing. Both roles use the same tracked initializer and stale checks; audit
values remain in the immutable signed snapshot but not in `dependencies()`.
V2 rejects unclassified scope captures. Classified captures require v2 signing;
they cannot be silently interpreted by an unsigned creation or the legacy profile.

This persisted correspondence is not a supersession event or an effectiveness
decision. A no-op, a dependency edit, and a later restored value need the separate
authoritative business consequence contract; comparing these revisions or digests
in the host cannot implement it.

## How It Executes

1. Ordinary authorization, capability policy and body requirements admit the
   request. Guidance uses the same requirements but grants no mutation permit.
2. The native body builds one unpublished candidate and reserves its deferred
   provenance slot. Query seals its signing intent at that retained observation.
3. The installed owner authenticates the original principal for that intent.
   `sign` consumes the genuine event without rerunning the body.
4. Commit rechecks native source revisions, current authority, program occurrence,
   immutable effects and the owner's expiry/revocation decisions. It fills the
   reserved slot and publishes snapshot, actor, provenance, output and links together.

Missing proof, stale dependencies or owner refusal publish no partial snapshot.
An unrelated source edit need not stale the candidate. Exact logical replay
resolves before asking for another event. Deferred and indeterminate outcomes
retain their usual native recovery meanings; do not rerun effects on them.

## Small Example

With the application's typed immutable snapshot fields declared:

```rust,ignore
let body = Verify::reference().body()
    .in_scope(Document::reference())?
    .create_in_scope(Verification::reference(), "verification")?
    .initialize_from_scope(Description::reference(), VerifiedDescription::reference())?
    .signed_by::<VerifyBinding, _, _, _, _, _, _, _>(
        VerifiedActor::reference(), SigningProvenance::reference(), "verification",
    )?
    .finish()?;
```

`VerifiedActor` uses the binding's exact principal identity codec. The required,
unitless, read-only provenance field uses `ApplicationSigningProvenanceBinding`
for authentication-only v1, or `ApplicationSigningSubjectProvenanceBinding` for
the certified-subject v2 record. The example above uses the v1 initializer;
with v2, use explicit capture roles as below. Both use the same `signed_by` call. Ordinary
supplier codecs cannot implement its sealed native provenance-binding contract.
The binding requires an application program and admits the creation plus every
initializer, including actor and provenance, in its normal candidate ceilings.

## Real Example

A ready Draft can add a nonempty-description requirement and an assigned-person
equality requirement. Bind the stored assignment with `field(...)` and the actual
admitted identity with `principal_identity::<VerifyBinding>(...)`; combine them
in the usual typed Foundation equality expression. Capture description,
assignment and round into the immutable row. If description is optional source
data, use `capture_from_present_scope` rather than weakening the existing
required-source initializer. Link the new row to the document and declare its
typed created output using the usual body methods.

```rust,ignore
creation
    .capture_from_present_scope(
        Description::reference(), VerifiedDescription::reference(),
        ApplicationOperationBodyCaptureRole::Subject,
    )?
    .capture_from_scope(
        CurrentStage::reference(), VerifiedStage::reference(),
        ApplicationOperationBodyCaptureRole::AuditContext,
    )?
    .signed_by::<VerifyBinding, _, _, _, _, _, _, _>(
        VerifiedActor::reference(), V2Provenance::reference(), "verification",
    )?
    .finish()?;
```

The caller performs the actual authentication handoff:

```rust,ignore
match request.prepare_capability_in_program(program, allocation)? {
    Preparation::Settled(outcome) => consume_native_outcome(outcome),
    Preparation::Prepared(mut prepared) => {
        let intent = prepared.authentication_intent().expect("signed body").clone();
        let event = installed_owner.authenticate(credential, principal, intent, scope).await?;
        prepared.sign(&event)?;
        consume_native_outcome(prepared.commit(allocation))
    }
}
```

Keep the same prepared handle and logical request key. A mismatched event is not
consumed; denial does not permit replacing the request's original signer.

With v2, the governed result's `provenance.dependencies()` exposes the installed
source-to-snapshot correspondence. `provenance.authentication()` exposes the
authentication metadata. These are inspection data, not reusable authority.

## How It Relates To Other Features

Workflow operation authority and workflow approval remain separate native
capabilities. A signature does not advance a workflow or satisfy an approval
automatically. Unsigned principal attribution is still appropriate for a plain
transition audit. Governed queries read the committed snapshot and provenance;
their finite result envelope must accommodate that history.

## Inspection And Debugging

Inspect native requirement findings, typed preparation/authentication/commit
denials and the governed persisted history. Provenance names the operation,
program revision, binding, external signer, purpose, verifier configuration,
subject/signer coordinates, intent and coverage digests, event reference and
named-clock observations. Clock coordinates are not UTC timestamps.
V2 coverage includes the certified correspondence and immutable snapshot; v1
keeps its original coverage semantics. Coverage authenticates audit context too:
it is not a future validity predicate. Only the declared certified dependencies
describe that footprint. The two typed codecs reject each other's
bytes: a v1 record is never silently interpreted as an empty v2 footprint.

## Anti-Patterns

Do not accept actor, signed flags, provenance JSON or precomputed coverage as
signing authority. Do not select a new verifier per request, reexecute the body
after authentication, auto-retry a stale signature, or mint fake workflow approvals.
Do not seed signed business records through bootstrap or checkpoint migration.
Program-adoption migration also refuses signed creations explicitly; it cannot
discard their signing state and publish them as ordinary migration effects.

## Current Limits

This body form admits one signed creation per candidate. It uses the existing
bounded scalar body and canonicalization support; unsupported encoding, resource
or compiler profiles fail closed. Old portable body versions remain unchanged;
the original signed form and principal operand use body record v13. Explicit
capture role and source presence use body record v14; earlier versions cannot
reinterpret its new opcode. The v2 provenance binding has a distinct codec
identity; it does not reinterpret v1 records or require a build-counter
schema revision. Unsupported codecs fail installation rather than falling back.

Checkpoint restoration preserves signed data and durable idempotency facts, not
live event or receipt authority. Permanent business supersession, approval-round
selection, automatic consumption of early evidence by later approval, regulatory
signature manifestation and UTC evidence are not supplied by this feature.

## Related Docs

- [Executable Scope-Bound Operation Bodies](./scope-field-operation-bodies.md)
- [Ordinary Application Front Door](../foundations/ordinary-application-front-door.md)
- [Programs and Adoption](../foundations/programs-and-adoption.md)
