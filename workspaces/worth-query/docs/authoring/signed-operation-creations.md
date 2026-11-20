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
- `ApplicationSigningGraphSubjectProvenance`, its v3 binding, and
  [`capture_from_path`](./graph-certified-subjects.md) for related source facts
  and optional local captures;
- `creation.initialize_from_present_scope(optional_source, required_snapshot)`;
- `creation.capture_from_scope(source, snapshot, ApplicationOperationBodyCaptureRole)`
  and `capture_from_present_scope` for explicitly classified v2 captures;
- `operation.requirement_operands().principal_identity::<Binding>(name)`.
- `creation.supersede_on_subject_change(history, current, consequences, record, maximum_evidence)`
  for native dependency-scoped permanent supersession.
- `ApplicationSigningSupersession` and `ApplicationSigningSupersessionBinding`
  for the immutable, descriptive consequence payload.
- `completed_body.signed_evidence::<Binding>()` derives a typed descriptive
  producer reference; `operation.requirement_operands().signed_evidence(name, &reference)`
  binds its current-evidence existence as a Boolean in the normal requirement evaluator.
- `operation.requirement_operands().signed_evidence_from_related(name, relation, &reference)`
  selects one outgoing related subject before using that same evaluator. The
  relation's target type must be the producer binding's scope type.

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

For an approval capability whose installed control operation also declares a
signed creation, use `prepare_workflow_approval_with_body` instead. It returns
`WorkflowApprovalBodyPreparation::Prepared(signing_request)` or
`Settled(native_progress)`. `Settled` is a resolved native outcome, not necessarily
success: inspect its `WorkflowProgressOutcome`, including denials. The sign-only `prepare_workflow_approval` refuses a
declared body rather than silently skipping it.

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
The same candidate can advance bookkeeping with a
[computed scope assignment](./computed-body-values.md). Capturing CurrentRevision
as `AuditContext` records its observed pre-sign value, while assigning the native
`next-revision` result publishes the next value to the root. The normal signing
intent authenticates both effects. A concurrent revision-only change stales that
candidate; later changes to audit context alone do not supersede settled evidence.
This does not permit a computed initializer inside the signed artifact: such an
initializer lacks an explicit subject/audit capture classification and is refused.
V2 and v3 reject unclassified scope captures. Classified captures require certified signing;
they cannot be silently interpreted by an unsigned creation or the legacy profile.

The footprint alone is not a supersession event or an effectiveness decision.
Opt into `supersede_on_subject_change` on the signed creation to declare the native
consequence contract once. Query derives its publication closure for installed
source writers; the host supplies neither changed-field lists nor consequence effects.

History and current evidence are different native relations. Signing appends the
history link and adds current membership. A real change to a row's own `Subject`
dependencies atomically creates its immutable supersession record, links that
record to the evidence and removes its current membership. The source edit and
consequence share one publication. History and signed contents remain untouched.
No-op/unrelated edits preserve evidence, and restoring a previous value never
removes a supersession or revives the old row. `AuditContext` changes alone do not
supersede it. The declared limit bounds current fan-out, not lifetime history.

A later business requirement can consume this current attestation without
converting it into a workflow approval grant. Derive its reference from the
completed signing body once, then register that same body. The reference pins
the binding, operation, scope, native provenance, purpose, lifecycle and complete
canonical producer-body meaning. Cold readmission rejects a reference from a
different body even when its operation and purpose labels match.
The Boolean means at least one matching current row. Other healthy attestations
do not satisfy it; coexistence does not make a valid match ambiguous. Query also
requires the row's sealed canonical program revision to equal the native active
revision observed at the consumer's snapshot. Cross-revision compatibility is
not inferred. Revision mismatch is nonmatching evidence, not a supersession event.

When the consumer's root and the signable subject differ, use
`signed_evidence_from_related` rather than supplying a subject ID or evidence
path in React or the host. The declared outgoing relation selects exactly one
distinct live subject. No subject produces Boolean `false` (and `Unmet` for a
bare `verified` requirement); the authored expression determines status. Multiple subjects produce
typed `AmbiguousRelation`, even if only one is signed. Duplicate edges to the
same subject do not multiply it. A dangling endpoint is unavailable, not absent.
The selected subject's actual native identity—not the root's identity—is checked
against provenance. Evidence is never unioned across related subjects.

A real round can therefore be a separate signable subject. Selecting another
subject does not itself supersede the old subject's signatures. That alone is
not a complete round lifecycle: the supplier must still author round creation,
current-round replacement, history retention and when signing/editing is allowed.
An incoming relationship count does not replace capability policy, assignment,
parent isolation or business-open guards. Do not increment a certified mutable
round counter and call that historical-round preservation.

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

For a lifecycle declaration, signing also observes current membership and its
capacity before authentication, including absence. The v2 intent binds this full
admission observation separately from certified subject coverage. An unused event
cannot be reused after membership movement even when the certified content matches.
Source-edit consequences are observed on the retained decision basis before
publication-basis readmission. Concurrent new evidence makes that edit stale;
neither its source write nor its consequence publishes. Refusals never publish
one half of the batch. Exact retained replay precedes consequence discovery.

Missing proof, stale dependencies or owner refusal publish no partial snapshot.

Current-evidence requirements use the same bounded native collection and scalar
observers as other body operands. Adjacency, endpoint liveness and provenance
reads are retained for publication; native program readmission and Product CAS
fence activation movement. Unavailable or malformed authoritative observations
remain typed denials, not a fabricated false finding. Consumer guidance and
execution use the same expression. Guidance grants neither occurrence authority
nor permission. A prepared completion becomes stale if a concurrent dependency
edit supersedes the evidence. Acquire a fresh native operation need after a spent
or stale attempt; never automatically reuse its move-only authority.
Do not compare saved dependency revisions with today's field stamps to decide
validity: a no-op write may move a native stamp while preserving Current membership.
An unrelated source edit need not stale the candidate. Exact logical replay
resolves before asking for another event. Deferred and indeterminate outcomes
retain their usual native recovery meanings; do not rerun effects on them.

Related selection also seals root/subject liveness and the exact root-to-subject
adjacency in that same native read set. Changing selection after preparation
makes publication stale, including when a second subject is added. A successful
guarded receipt describes the checked observation; it is not a permanent grant
or permission to keep using a subsequently changed selection.

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
The binding requires an application program and declared authority for the
creation and every initializer, including actor and provenance.

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
    .supersede_on_subject_change(
        VerificationHistory::reference(), VerificationCurrent::reference(),
        VerificationSupersessions::reference(), SupersessionRecord::reference(),
        std::num::NonZeroUsize::new(32).expect("bounded current evidence"),
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

To require that early verification on a later operation:

```rust,ignore
let verification = completed_verify_body.signed_evidence::<VerifyBinding>()?;
// Register completed_verify_body itself, not a newly reconstructed body.
let operation = CompleteReview::reference();
let operands = operation.requirement_operands().signed_evidence("verified", &verification)?;
let expression = worth_foundational::expression_api::expressions()
    .reading_within(ApplicationOperationBodyRequirementExpression::PROFILE)
    .parse("verified")?;
let requirement = ApplicationOperationBodyRequirement::admit(
    "current-verification", operands, None, expression, "verified",
)?;
let completion = operation.body()
    .expect(CurrentStage::reference(), Stage::Review)?
    .requirement(requirement)?
    .assign(CurrentStage::reference(), Stage::Approved)?;
// Add the normal native audit creation, finish and register this body.
```

The operation still declares its own ordinary permission, capability policy,
effect authority and any occurrence requirement. The supplier example's
information-gathering EnterReview remains unsigned; a distinct later
CompleteReview requires current verification and an exact operation need.
A genuine acknowledgement can share the evidence kind and lifecycle but have a
different purpose and certified footprint. It neither satisfies this reference
nor becomes superseded by an edit outside its own `Subject` dependencies.

For a document whose current approval round is a different entity, replace only
the operand declaration with the typed outgoing selector:

```rust,ignore
// CurrentRound: Document -> ApprovalRound; SignRoundBinding is scoped to ApprovalRound.
let round_evidence = completed_sign_round_body.signed_evidence::<SignRoundBinding>()?;
let operands = CompleteDocument::reference().requirement_operands()
    .signed_evidence_from_related("verified", CurrentRound::reference(), &round_evidence)?;
// Admit the same Boolean requirement and attach it to the Document-scoped body.
// Register completed_sign_round_body itself and declare CurrentRound read membership.
```

The actual supplier example also declares a tenant-scoped guarded check over
`TenantDrafts` and the completed `VerifyBinding` producer. Its preservation-only
body returns the checked tenant without inventing a business transition. It is
intentionally an exact-one check: a tenant with two drafts is ambiguous, not
"whichever draft has a signature." It does not implement current-round replacement.

With v2, the governed result's `provenance.dependencies()` exposes the installed
source-to-snapshot correspondence. `provenance.authentication()` exposes the
authentication metadata. These are inspection data, not reusable authority.

Declare distinct same-context, retain-for-audit relations: document→evidence for
history and current, and evidence→supersession for consequences. The consequence
entity contains exactly one required, unitless, read-only field bound to
`ApplicationSigningSupersessionBinding`. Ordinary operations cannot create, edit
or delete that record or mutate protected associations; only the signed body's
own history/current links and native consequence owner can do so. A signed body
cannot simultaneously write the `Subject` values whose preimages it certifies.
Installation derives the native consequence publication touch closure. Execution
checks original request authority, observes dependency changes and grows actual
effect storage fallibly. Only explicit authored subject-impact limits restrict
the number of affected subjects; no candidate-cardinality forecast supplies a fallback.

## How It Relates To Other Features

### One approval action with a business attestation

An approval frontier can require a fresh signature and publish an immutable
business attestation through one action. Declare the attestation on that
approval capability's `WORKFLOW_CONTROL` operation, install its compiled body
with `operation_body::<Binding>()`, and register the capability through
the workflow vocabulary's `.approval::<Capability, Operation, Input>()`.
The binding uses the usual native body decision and denial types, unit result,
and its real output contract. Author/start/advance roles cannot accept this
body; neither can an ordinary workflow `.operation()` registration.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
match request.prepare_workflow_approval_with_body(
    workflow, instance, &required_approval, &proposal, decision,
)? {
    WorkflowApprovalBodyPreparation::Settled(progress) => consume_progress(progress),
    WorkflowApprovalBodyPreparation::Prepared(signing) => {
        let intent = signing.authentication_intent().expect("declared signed body").clone();
        let event = installed_owner.authenticate(credential, principal, intent, scope).await?;
        consume_progress(signing.sign(&event)?.execute())
    }
}
```

Query retains the original approval decision and its snapshot, evaluates the
installed body on that same snapshot, appends the native approval effects, and
seals the combined candidate once. One genuine event authenticates that exact
candidate; it is not consumed separately for each half. Publication rechecks
both body and native approval dependencies and publishes both or neither.
The principal, proposal, decision, native evidence and capability checks remain
mandatory. No host coordinator or signature-to-grant mapping is involved.

For example, a manager-review body captures the document description as
`Subject`, the signing stage as `AuditContext`, declares a typed created output,
and uses the ordinary supersession lifecycle. Its approval node still consumes
the authored assessment evidence and grants only its native approval meaning.
Read the attestation through a governed query or its actual transition receipt's
`outputs_of::<Contract>()`; neither observation becomes reusable authority.
Another early-verification producer stays independent and cannot supply this
approval merely because it also contains a signature.

Exact settled replay returns the original native progress before evaluating
today's body requirements or requesting another event. Changed logical intent
is not replay. A stale fresh attempt never publishes one half; retain native
deferred/indeterminate results rather than automatically rerunning it.
Both Approve and Reject execute the declared body: name its business meaning
accordingly (for example, "reviewed", not an unconditional "approved").

This join supports the existing bounded scalar signed-creation body, not
partitioned-computation-producing profiles, enrolled birth, succession, external
effects, or arbitrary additional native output/emission profiles. Unsupported
composition is refused rather than dropping carried state. Original request
interruption, canonicalization failures, admission and publication's exact
candidate recheck remain enforced without supplier resource forecasts.
This does not add Store/process durability guarantees.

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

Read supersession history through an ordinary governed query. Its payload records
exact source/evidence/actor coordinates, operation, logical request digest and
changed dependency loci. It does not claim UTC time or grant approval authority.
Typed lifecycle denials distinguish unavailable observations, evidence capacity,
representation capacity and forgery. Observation failures retain their typed
source cause; inspect `signing_lifecycle_observation_failure()` when applicable.

## Anti-Patterns

Do not accept actor, signed flags, provenance JSON or precomputed coverage as
signing authority. Do not select a new verifier per request, reexecute the body
after authentication, auto-retry a stale signature, or mint fake workflow approvals.
Do not seed signed business records through bootstrap or checkpoint migration.
Program-adoption migration also refuses signed creations explicitly; it cannot
discard their signing state and publish them as ordinary migration effects.
Do not treat an evictable invalidation index or current-value/digest comparison as
permanent supersession. Do not scan or rewrite all historical evidence on each edit.
Do not restate the producer's evidence paths or purpose in the host, default a
missing match to another attestation, or turn decoded provenance into approval
authority. Do not substitute current-value equality for Current lifecycle membership.

## Current Limits

This body form admits one signed creation per candidate. It uses the existing
bounded scalar body and canonicalization support; unsupported encoding, resource
or compiler profiles fail closed. Unitless UInt64 computed scope bookkeeping may
accompany signing, but `initialize_value` on a signed creation remains unsupported.
The lifecycle's Subject source-write exclusion is unchanged. Old portable body versions remain unchanged;
the original signed form and principal operand use body record v13. Explicit
capture role and source presence use body record v14; earlier versions cannot
reinterpret its new opcode. The v2 provenance binding has a distinct codec
identity; it does not reinterpret v1 records or require a build-counter
schema revision. Unsupported codecs fail installation rather than falling back.
The original scope-only lifecycle declaration uses body record v15 without changing
v1–v14 and uses v2 provenance. The [graph-certified profile](./graph-certified-subjects.md)
uses v3 provenance and body v29 for path captures or a subject-impact bound.
Fan-out/capacity exhaustion refuses the whole edit, never truncates.
Effectful program-adoption migration for a lifecycle schema is explicitly
unsupported until it has a native consequence join; it cannot bypass this law.

Checkpoint restoration preserves signed data and durable idempotency facts, not
live event or receipt authority. The current-evidence requirement uses body v16;
v1–v15 records are unchanged and cannot silently acquire the new operand. It
requires certified v2/v3 provenance and the declared native lifecycle, and matches only the
same canonical active program revision. Related-subject requirements use body
v18; earlier opcodes/records keep their existing meaning. This selector supports
one outgoing hop, not arbitrary paths, input-supplied IDs or first/latest selection.
Native round creation/replacement and Reopen are not supplied by this selector.
Automatic conversion of early evidence into kernel approval, regulatory
signature manifestation and UTC evidence are not supplied by this feature.

## Related Docs

- [Executable Scope-Bound Operation Bodies](./scope-field-operation-bodies.md)
- [Shared Computed Body Values](./computed-body-values.md)
- [Graph-Certified Signature Subjects](./graph-certified-subjects.md)
- [Ordinary Application Front Door](../foundations/ordinary-application-front-door.md)
- [Programs and Adoption](../foundations/programs-and-adoption.md)
