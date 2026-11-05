# Document history over declared owned models

Governing increment to milestone 9.18.2, public issues #53–#57.
Ownership ruling: D1113, 2026-11-03. D1159 integration order: #55, #53,
#57 under D1119, then #56. The implemented #54 checkpoint is retained.

## Boundary review

Ownership is installed meaning over stored full-generation relation endpoints.
Only relations explicitly enrolled in a history definition establish membership;
transitive traversal never follows arbitrary adjacency. A model owns itself.
Native owns structural truth and candidate invariant execution. Query owns
installed enrollment, fresh admission, document selection and recorded steps.
Decl/Host remain the consumer facades. No consumer callback or claimed list of
entity IDs can establish ownership. Existing source-scope publication custody
remains with its original owner.

The adversarial case is a child with equal-looking names under another model,
or a relation change that creates two different root owners through different
paths. Membership must use the evaluated basis, and exclusivity must be checked
at every candidate publication, including an ordinary operation outside history.
Changing an ancestor can give an unchanged descendant two owners; checking only
the changed edge's immediate target is insufficient. Traversal and retained
scratch are admitted under the original request before work/allocation.

Existing equality checks occur at declaration, installation, enrollment,
candidate execution and receipt capture. Preserve the existing same-model path;
an owned-scope path must carry the admitted scope correspondence through those
same owners. A receipt from another scope cannot replace it. Current READ,
operation authorization and head guards remain independent requirements.

## Ownership and navigation contract

History definitions declare typed ownership relations. Installation resolves
them against actual schema relations and verifies each enrolled operation scope
is reachable from the model type through that declared graph. An automatic
Native commit-boundary invariant enforces at most one model root for every
affected entity across installed ownership domains. Sharing several paths from
the same root does not create a second owner. Refusals identify the actual
full-generation subject. Unadmitted owner identities are not disclosed. Missing material and stopped
requests refuse rather than prove absence.

Recording proves scope membership at the current head. Navigation proves it at
the selected snapshot. Checkout atomically restores the union of current and
selected owned closures: retire current-only entities, restore selected-only
entities, restore relations, and preserve every publication's custody. Relation
moves are ordinary recorded edits. The cascade-custody restoration and correction
recording remain gated on the other lane landing; their safety checks cannot be
relaxed meanwhile.

## #55 implementation plan

1. Add ownership relation meaning beneath declaration `application_history` and
   a typed `.owns(relation)` builder. Keep `.operation`'s existing exact-scope
   contract and add `.owned_operation` for scopes requiring recorded proof.
   Installation validates the complete relation graph and operation membership.
   Archive encoding preserves bytes for definitions without ownership and adds
   a distinct version for owned definitions.
2. Add Native declared model-ownership observation and automatic invariant under
   `branch/model_ownership/`: one graph algorithm, exact-basis and candidate read
   adapters, and invariant registration. This is structural evidence, not READ
   or mutation authority. Candidate validation examines affected descendants
   and their declared ancestors, deduplicating common roots.
3. Install that invariant from Query's invariant installation owner using the
   existing mandatory custom-invariant mechanism. Do not require an application
   to remember a second registration. Record admission obtains the actual
   operation scope's model owner from Native. Carry exact admitted scope into
   reservation, candidate restriction and receipt capture.
4. Remove publication-only equality restrictions that prevent reaching the
   installed owned-operation admission. Preserve original ordinary preparation,
   no-source requirement, current authorization, idempotency and candidate
   lifecycle. Group membership consumes the same proof.
5. Add generic multi-entity courts in the existing Decl/Host court binary:
   direct and transitive child edits; foreign and recreated child refusal;
   undeclared reachability; two-owner refusal through different paths; lawful
   relation move; old selected model visible before adoption. Mutation probes
   must convict each new authority check. Archive tests protect original bytes.

Destination topology keeps declaration meaning, installation validation, Native
structural observation, candidate invariant execution and Query document custody
separate. #53 extends the Native ownership owner with union selection; it does
not create a separate ownership definition or reachability algorithm. #54
extends the existing private-candidate group owner. #57 extends existing output
profiles and retains separate tree custody.

## Verification and coordination

Run focused owner/public courts, formatting, dirty Rust line caps, function
scrutiny, boundary-check and agent-context checks; broaden to archive and
consumer compilation where affected. Reproduce alleged master debt on clean
master before classifying it. No new test runner or binary. Files stay <=400
lines. Existing single-record behavior and wire bytes remain unchanged.

Before each slice, inspect cascade42's committed diff and dirty status. Never
edit its protected files. Both consumer documentation entry files are currently
protected, so their final updates await that lane's release. Commit locally;
do not push. The first READY TO MERGE handoff requires all of #55/#54/#57 and
their required evidence, not this plan alone.

## #54 boundary review

Native's group reservation already owns a runtime Vec of separately admitted
members, including their individual operation indices, scopes and idempotency
meaning. Its candidate, capture and adoption APIs accept the actual member
binding at the consuming boundary. The publication root is the restrictive
owner: its member enum and result Vec fix one I/A pair. Each typed member must
retain the original Mode-selected operation, completion, terminal or disposal
owner. Dynamic dispatch may select that existing owner; it must not reconstruct
admission, receipts, completion evidence or consumer callbacks.

The common group lifecycle owns the single reservation, fork, capture count,
selection and abort cleanup. Per-member state owns construct/prepare/bind/run,
receipt extraction, fresh final admission, and disposal through its original
Mode. Results retain their original resource backing through owned iteration.
Homogeneous storage should keep its present allocation and typed results;
heterogeneous storage adds bounded boxes and typed result projections. Both
must consume the same lifecycle owner. Membership freezes before the first
construction attempt, including a refused attempt.

## #54 implementation plan

Generalize the existing group root over a sealed member-storage type instead of
one intent/action pair. Move original per-member transitions into the existing
Member owner, behind a private object-safe driver parameterized by S/H/P. Keep
all candidate and journal policy in the existing group execution. Typed storage
retains Vec<Member<I,A>> and typed completions; heterogeneous storage retains
runtime-length boxed original members and typed projections of their original
completions. No consumer can implement the driver or supply evidence.

Retain group_edits as the homogeneous construction entrance. Add gesture(program,
request,key) and push::<Intent,Action>(request,intent,member_key) on the owning
heterogeneous root. A runtime loop can choose a different action for each push.
Installed member/result maxima and execution memory are checked before input
capture or storage growth. The first advance freezes the list; empty lists and
later pushes refuse typed. Advancing, completing, finishing and aborting consume
the same private candidate and metadata owners as homogeneous groups.

Qualify with a generic mixed model/member gesture, variable runtime lengths,
old selected values between member completions, one recorded step, a denied
member with full abort, idempotency changed-list refusal and lawful replay.
Existing group courts, mutation probes, cost courts, doctests and mandatory
repository gates must still pass. #53 supplies owned-graph navigation separately;
#54 must preserve the existing redo-selection policy.

## Remaining canonical-custody boundary

The relation-only implementation is a work-in-progress checkpoint. It cannot be
qualified or adopted while source-scope-owned publication membership is absent.
For example, a generated entity under A's source scope must not become B's member
merely because B gains a declared ownership edge. The same-basis owner and its
transitive source custody must come from #39/#42's canonical ownership owner.
Its closure implementation and export file are protected by cascade42. Do not
add another ledger, scan unrelated history, or treat relation-only exclusivity
as the complete installed one-owner invariant. The checkpoint is not merge-ready.

The required handoff is a read-only observation of recorded publication custody
at an admitted exact basis, with full-generation subjects, transitive source-scope
parents, charged work and missing-material refusals. D1114 explicitly defers this
handoff until root announces the cascade batch's landing. Do not request a new
door from that lane. Consumer capability-index and main-guide updates also await
their protected-file release.

## D1114 observation-court boundary and plan

The exact-basis observer consumes owner-admitted immutable branch roots. Its
structural reader must demand authenticated Head metadata before deciding that
an entity or ownership edge is absent. Recovery's external-member fixture can
keep the actual entity and relation partitions cold, and its storage port can
refuse the required member. This proves a real observation boundary without
inventing structural facts or supplying an ownership answer from a fixture.
Native returns the discovered owner; the existing public operation admission
compares it with the document model and owns the foreign-document refusal.

Add courts to the existing Relational certification court: a generic model
and member graph with endpoints and relations in separate partitions, successful
cold ownership observation, unavailable ownership material naming its partition,
and work/memory refusals with an unchanged selected root. Exercise the existing
public foreign-member court with an independently resolved full-generation
subject and unchanged history step as well as unchanged payload. Mutate each
new guard to establish that its court fails. Run focused Native/public courts,
formatting, dirty line caps, boundary-check and agent-context.

## D1114 navigation boundary under review

An owned selection needs the union of two independently evaluated closures,
each reached only through installed ownership relations at its original basis.
Current-only entities retire and selected-only entities restore. The existing
single-entity selection cannot represent lifecycle changes or relations in its
canonical authority and must remain unchanged. No graph selection may masquerade
as that authority or copy historical metadata over current provenance.

The eventual publication handoff must carry one prepared graph restoration and
one explicit typed `requires cascade custody` seam until the batch lands.
D1117 below resolves the relation-boundary question raised in this review:
outgoing relations and incoming declared ownership edges belong to the document;
ordinary foreign incoming relations remain outside. Observation courts do not
depend on that ruling.

### Entity-boundary preparation slice

The existing exact-basis structural reader already owns authenticated metadata
demand, full-generation adjacency and charged traversal. Extend that owner to
prepare the two entity closures in one request, retaining both original admitted
bases and all result-memory backing. Expose current, selected and union views
without a caller-supplied membership list. Require the same concrete model root
kind at both bases and preserve missing/foreign-basis and request causes. The
union is an allocation-free view over the retained sets.

This is selection material, not a legal mutation authority or an applied graph
restoration. The graph publication owner must later consume it together with the
relation boundary and publication custody. Keep its implementation in Native's
`branch/model_ownership/selection.rs`; consolidate basis-reader construction in
`basis.rs` so membership and selection cannot disagree about cold absence. Add
an exact-basis ownership-move court in the generic Native fixture. It
must distinguish current-only and selected-only membership, exclude a separate
model, retain the historical answer after further edits, refuse root-kind
substitution, and expose traversal-budget bypass mutations. Full relation and
publication restoration remain unclaimed until their boundaries are resolved.

### Court-placement correction

Boundary-check rejected the first placement under Native `src`: constructing
`RelationalRuntime` and execution requests there creates a new source court.
This is scoped implementation debt, not master debt. Move the runtime courts and
their fixture into the existing `worth-court-relational-certification` binary,
under its registered `relational/relational_certification` owner. Use public
Native branch transactions and external checkpoint recovery; introduce no test
door, baseline exemption, test runner or Cargo target. Public recovery correctly
rejects a v1 root in a v2-configured runtime, so it cannot construct the schema
mismatch scenario. Isolate that private admission rule as a pure unit test using
actual commitments from registries with changed contracts and revisions but
unchanged kind identities. It constructs no runtime root or admitted basis.
Preserve all eight test names (seven courts and one unit), and repeat mutation
probes against these final placements before qualification.

## #57 boundary findings (D1119 supersedes the settlement plan)

Each producer family's records belong to the owned closure. Graph navigation
restores the selected versions, original source observations and output witnesses
atomically with authored state; it does not rerun producers to reconstruct them.
Every family must then be qualified through the existing required-set currentness
owner at the same final endpoint, including the restored Native current ownership
roster. Verification checks existence, producing build, source facts and witnesses.
A failed verification may run only the affected original producer, with its
reason visible on the owner. Keep its source, root and disposal custody only for
that fallback. There is no per-tree re-settlement design or second producer engine.

The count court uses at least two families. Undo and redo under unchanged builds
must contact no producers. Changing one producing build must contact exactly
that producer. Reintroducing regeneration and omitting one family's verification
must each fail the courts. These counts prove the fast path, not a public cost
contract. Existing ScopeEntityRecord behavior remains unchanged.

D1117 remains the complete foreign-edge policy. Product links are pinned;
implement no additional foreign incoming edge machinery.

## D1117 relation boundary and implementation slice

Root selects outgoing relations from owned sources and incoming declared
ownership edges. Ordinary incoming relations from outside sources remain owned
by those sources. Restoring a current-only entity's retirement must refuse if
one of those live foreign relations survives; the diagnostic retains its exact
relation, source owner and actual installed endpoint-deletion contracts.

Boundary review: Native's immutable Head reader has full-generation relation
metadata and charged adjacency, and the retained bases carry authoritative
schema contracts. The existing single-entity commit authority cannot represent
this graph and must remain unchanged. Native's mutation/publication dispatch,
canonical-envelope variants and transaction preparation facade are still in the
cascade exclusion set. They cannot be extended or bypassed in this slice.
Therefore the outside-owner handoff can prepare and validate the declared
structural selection, but cannot publish it before that batch releases these
owners. Publication custody must still contribute its authoritative closure.

Prepare current relation material from the union of current and selected entity
closures: all outgoing relations, plus incoming ownership edges declared anywhere
in the installed ownership registry. This includes a moved-away entity's current
ownership edge. Desired historical relations come from the selected closure
only; current-only entities retire rather than gaining another document's old
payload. Historical outgoing relations whose current source moved outside the
union refuse with that current owner. A historical outside endpoint must still
be live at its full generation; checkout cannot restore its payload by accident.

Implementation order:

1. Extend the same Native basis reader with paid full-generation relation reads.
   Keep membership traversal and its existing bill unchanged. Add a graph
   selection owner retaining the entity boundary, both relation maps and funding.
   No caller can supply a mutation roster or turn it into commit authority.
2. Qualify foreign incoming retirement against actual current facts. Diagnostics
   borrow the current basis's installed deletion contracts, without synthesizing
   invariant receipts or copying a second schema policy. A foreign relation with
   no explicit deletion contract still cannot be severed under D1117.
3. Cache combined installed ownership meaning once in Query's existing layout
   owner. Navigation consumes the Native declared structural selection after its
   existing READ/NAV/program/Signal checks, then returns one explicit typed
   `RequiresCascadeCustody` refusal before candidate creation. It must never
   fall through to single-entity publication.
4. Add generic public Native courts for outgoing reference selection, untouched
   foreign incoming edges, typed retirement blocking, ownership moves across
   declared relation types, and corrupted cold material. Add a public history
   court proving the single custody refusal and unchanged step/payload. Qualify
   each new guard with mutation probes, then run owner/public courts, formatting,
   dirty caps, boundary-check and agent-context.

The applied-checkout versions of these courts and atomic Native graph publication
remain unqualified until the protected dispatch and canonical authority can be
extended after landing. This is one explicit custody seam, not an entity-only
fallback or an alternate mutation engine. #55 custody, #57 and #56 continue to
wait for root's landing notice.

### Explicit domain carriage

Issue #53 asks for a navigation domain rather than a reinterpretation of the
single-record name. Add `OwnedModelGraph` to the existing declaration enum and
canonical/archive owners. It selects the declared graph path, including when the
closure currently contains only the model. `ScopeEntityRecord` navigation keeps
its existing empty-ownership boundary. Graph-without-ownership still uses archive
version 4; tag 3 is admitted only there. Existing domain tags 1/2 and all version
1–3 encodings remain unchanged. Prove roundtrip, canonical distinction, old-version
refusal, and preservation of existing literal-byte courts. This is within #53's
explicit domain request, with no change to D1117's ownership policy.

### D1117 qualification checkpoint

Restored-source checks pass: 15 Native ownership/graph courts, the pure Native
schema-meaning unit, 125 public history/cost courts (two existing ignored), 14
archive history tests, the canonical-domain identity unit, 42 external consumer
history tests, and 30 history doctests (14 positive and 16 compile-fail).
Boundary-check, agent-context, scoped formatting, dirty Rust line caps and
whitespace checks pass. All 109 changed Rust files across the complete branch
and dirty set are at most 400 lines.

Twelve failed-test mutation probes cover graph boundary guards and actual owner
attribution, global declarations, union membership, historical basis choice,
cold material demand, retained funding, shared work metering, outgoing relation
projection, domain wire/canonical carriage, forbidden record fallback, and Query
propagation of Native refusals. Probes reject compilation failures as evidence;
all source snapshots were restored before the positive checks.

These are preparation and refusal proofs. They do not qualify applied checkout,
canonical graph mutation, complete publication custody or atomic restoration.
Those still require protected cascade owners. No READY handoff is permitted yet.


## D1159 landing boundary review and D1174 storage requirement

Merged public master fb0870356f39e1e991adf6ca0d47987a8e558a7e with merge
commit 005f98df3a. This releases the earlier cascade-path exclusion and supersedes
the historical landing waits above. The execution test split is retained. Constraint courts use the
new testkit and existing receiver-owned door; the receiver now derives the
history operation scope from the genuinely resolved entity. Exact caller changes
come from scripts/test_topology/src_court_doors.py callers, not hand edits.

The landed current-publication roster is the sole current custody reader. Its
one-hop selection and independent position presence must supply #55
membership/exclusivity and #53 closure selection. Typed retired-closure evidence
stays separate from published roles and grants no current entity or producer
custody. Current facts are not reconstructed from canonical history. Navigation reaching owned publications
requires storage registered through facade::schema::register_current_publication_storage;
absence must preserve typed CurrentOwnershipStorageMissing. CHK18 remains current;
no Native format increment is authorized here. D1149/#59's accepted coherent
checkpoint residual remains outside this lane.

Adversarial constraint: adding a declared link must not give a source-owned
publication member a second model owner, including when the publication itself
is first installed in that candidate. A current-head-only check is insufficient
for that prospective fact. Snapshot invariants precede publication installation
and Native current-ownership completion in authority/commit/pipeline/execution.rs
and publication_execution/preparation.rs. The publication owner must carry the
actual candidate's custody to exclusivity, with ordinary invariant receipts and
failure types intact. Do not create a second prospective ownership ledger.

D1119 point 3 exposes a separate, precise restoration owner gap:

- mvcc/publication/application_publication/restored_producer_prior.rs readmits
  only authored-retirement restoration, then resolves the prior through the
  actual retirement's old-to-new entity mapping.
- history/data/canonical_commit_envelope/required_output_origin/authored_restoration.rs
  requires an actual authored-source retirement and newly resolved creations.
  It cannot mean arbitrary selection of an older live publication at its original
  full-generation identities.

The required owner change is a genuine Native historical owned-publication
selection handoff: fresh commit provenance, retained producer/source observations
and witnesses, current roster and independent position facts restored together,
and later Preserve/currentness recognizing the original producer antecedent.
It must be composed by the ordinary graph candidate/publication path. It cannot
be emulated by a synthetic retirement, copied metadata or producer regeneration.
D1119 explicitly requires reporting this exact cascade owner rather than working
around it. No #55/#53/#57/#56 completion claim is made at this merge checkpoint.

After that handoff is available, the implementation order remains #55 custody
and one-owner courts, #53 atomic restoration, #57 all-family verification and
zero-contact courts, then #56 correction steps. Every issue gets focused courts,
mutation evidence, and its own report section before a READY TO MERGE handoff.
