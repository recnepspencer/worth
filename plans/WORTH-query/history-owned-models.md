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
selected owned closures, with D1188's recorded transfer boundary: retire
current-only entities only when the selected basis has no live selected owner.
Restore live transferred members at their original IDs and selected incoming
ownership edges; keep the outside owner's other state current. Restore relations
and preserve every publication's custody. Relation moves are ordinary recorded
edits. The cascade owners have landed; custody integration and correction
recording remain unfinished work in this lane.

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
ownership edge. D1188 supersedes the earlier current-only retirement rule:
desired historical relations include the selected closure and the selected
incoming declared ownership edges of live transferred members. Only the transfer
member's version enters restoration; the outside owner's other records do not. Historical outgoing relations whose current source moved outside the
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

## D1178 Native historical-selection origin

The D1159 owner stop is resolved: this lane owns the Native custody changes.
HistoricalSelection is additive within CHK18; no Native version increment.

Boundary brief: the selected basis supplies immutable publication ownership and
original full-generation record identity. The current basis supplies the guarded
candidate's predecessor and foreign-custody conflicts. The slot lifecycle owner
selects retained versions under their original IDs, with fresh version metadata;
it does not allocate identities or issue create intents. Native publication and
position completion remain independent writers. Their result passes the existing
pairing check. Query carries original source observations/witnesses into the
existing currentness owner; it must not mint settlement from descriptive IDs.

The adversarial case is a selection that correctly names a producer but includes
an unrelated record, or restores only roster facts while route presence stays
current. Both must refuse before publication. A no-storage store cannot make an
owned publication disappear into an empty current roster. Unknown additive origin
tags remain typed decoder errors; old CHK18 records retain their exact encoding.

Implementation plan, in authority order:

1. Add a distinct historical-output selection admission in worth-proof and a
   Native-prepared selection capsule under mvcc/publication/application_publication.
   Only an exact selected-basis roster can supply its publications and members.
2. Add HistoricalSelection beside the existing canonical output origins, binding
   the selected publication commit/generation and original producer antecedent.
   Validate route, payload and complete owned record coverage in the canonical
   installer. Keep authored retirement/restoration meaning unchanged.
3. Extend the slot lifecycle and existing semantic selection plan to restore the
   declared graph plus source-owned publications without ordinary copy/create.
   Restore current roster/claims through publication completion and presence
   through the position owner in that same candidate; validate their pairing.
4. Integrate #55 membership and prospective exclusivity with that same Native
   custody owner; then remove #53's explicit refusal only when atomic restoration
   is complete. Add #57 per-family currentness verification and fallbacks on the
   restored facts, then #56 correction step composition.
5. Court original IDs/payloads, foreign coverage, pairing mismatch, storage absence,
   old CHK18 opening, unknown origin refusal and cold selection reopen. Public
   undo/redo with two families must contact zero producers; changed build contacts
   only the affected family. Mutate presence, foreign admission, verification and
   regeneration. Record fixtures through their existing recorders only.

Each issue batch runs complete root and Query workspace tests against an isolated
clean-master checkout. #62 compile blocks, #64 frozen invoices and D1169's named
snapshot/doctest failures are accepted only when reproduced identically there.
Owner/public tests, boundary-check, agent-context, formatting and scoped caps
remain required. No issue is ready merely because this prerequisite plan exists.

D1178 pairing review refinement: the working-state image may be sparse. Pairing
must therefore consume the complete prepared root after both writers finish,
through the existing Native current-custody reader. That reader now distinguishes
an admitted-basis observation from a candidate-root observation; the latter has
no admitted descriptor and cannot be reused to issue current publication custody.
No alternate roster decoder or pairing policy is introduced.

D1178 exclusivity review refinement: the declared-relation rule executes before
prospective publication facts exist. The model-ownership registration therefore
retains its immutable ownership meaning for a final candidate-root judgment. Its
rule adapter is private, so installation cannot omit that judgment by constructing
a generic registration directly. The same traversal consumes the same Native
roster reader after both writers complete. This stores no parallel custody facts;
conflicts retain the existing typed custom-invariant refusal and true subject.

D1178 payload preparation uses the retained head's existing symbol-admission
receiver before proposal symbol rounds, matching retained-entity selection. Slot
selection derives both full identity and original birth from the retained slot;
equal kind/generation cannot authorize copying a different slot. Suspended slots
re-establish field-revision storage before ordinary delta stamping.

D1178 invariant-scope review refinement: a graph constituent is not an empty
authored-command plan. Its recorded selected kinds/endpoints and full targets
must enter packet scope, custom applicability/touches and the existing relation
scope accumulator. A restored outgoing edge must be judged against live outside
edges; target cardinality supplies the refusal court and mutation. No proxy
create intents or duplicate invariant evaluator are introduced.

D1178 next custody slice boundary: current-only publication routes are selected
absence, not authored retirements or producer runs. The current and selected
Native rosters already authenticate that distinction. A graph-wide preparation
must read both once under the caller's checkpoint, retain the exact original
publication for each selected route, and retain current custody for a route absent
at the selected root. The latter may retire only the graph's current-only records.
Publication completion removes that route's roster/claims, while position
completion independently removes its presence. Their shared reader checks the
complete candidate; no canonical-history absence can repair missing storage.
The position selector records the historical absence as an additive selection
coordinate so cold route selection cannot fall through to the displaced birth.
Courts will select a pre-publication snapshot, then reselect the published
snapshot, checking original output identity and equal cold/warm current facts.
The existing scalar preparation remains a publication selection; graph-wide
preparation supplies the complete heterogeneous route set without N roster reads.

Currentness boundary still to implement: ordinary field/aspect revisions name
fresh writes, including restored writes. Original recorded observations and
witnesses must stay immutable. Native must carry the selected-version antecedent
at its revision owner; Query's existing verifier must consume that evidence,
rather than rewinding ordinary revision stamps or asserting equality in history.
The healthy fast path and changed-build fallback need public two-family courts.

The milestone's opening scope decision and phase 7 defer durable history
reopening to Store integration. Graph selection therefore retains the existing
memory-only selection profile: an attached medium refuses before contact, and
raw canonical-tail replay refuses with its named unsupported graph-selection
profile before installing the selection. Completed Native checkpoints still
carry and reopen the selected root and both current-fact writers. This avoids
silently journaling a selection that lacks recovery source custody; it adds no
parallel replay or Store history protocol.

D1178 graph-write admission boundary refinement: preparing a descriptive graph
with caller-supplied meaning must not authorize a write under an ad hoc ownership
relation list. Native graph admission and preparation must match the retained
global recipe to the actual installed model-ownership registration, and match
the document recipe to declared domains in that registration. A caller cannot
turn an ordinary reference into ownership or omit another document's ownership
edges. The descriptive observation door remains descriptive. Retain both exact
meanings with funded graph preparation; canonicalize domain sets at construction,
then compare their exact typed values under the request meter. The public courts
must install the real one-owner invariant before publishing, and negative courts
must refuse uninstalled, broadened and incomplete global meanings before effect.
The Query installer also registers that actual meaning for an explicit
OwnedModelGraph domain with no ownership edges yet. Its root-only graph is still
a declared navigation domain. ScopeEntityRecord installations without ownership
keep their existing registration behavior.

The full Query comparison exposed a causal worker-only bootstrap regression:
prospective model exclusivity entered a Native scope even on installations with
no ownership declaration. The frozen invariant registry now carries the index
of its exact tagged ownership registration. Both graph write admission and the
final candidate check consume that original registration in O(1), and an
installation without it enters no extra work scope. This is compiled
applicability over immutable installation meaning, not another ownership fact
reader. The existing worker-only recorded parameter-inverse court is the
regression oracle; do not accept its branch-only failure as master debt.

D1178 route-absence implementation checkpoint: graph-wide preparation reads the
Native current and selected rosters once each, under one checkpoint. Union
entities are filtered by genuine full-generation point liveness at each basis
before entering that reader: a current-only record may not have existed in the
selected partition at all. The filter derives no ownership and never converts
missing cold material to absence. Both current-fact writers independently remove
absent routes; the existing reader checks their complete candidate result.
Warm and cold route selection stop at an explicit historical-absence coordinate,
so an old publication cannot leak through ancestry. The disposition retains the
actual displaced publication/version and fresh selection commit; it makes no
producer-run or fresh-identity claim.

Two-family Native courts select the pre-publication snapshot, cold-open the
absence, and reselect the original publications at their issued IDs. A separate
court removes a current-only source together with its route, avoiding a second
presence deletion in source cleanup. These are Native atomic/disposition courts,
not the required public zero-producer-contact count court.

Codec-retired closures remain a distinct typed posture. The roster supplies only
their diagnostic publication/owner coordinate to this path; it does not retag
retirement evidence as published roles. The graph batch and coverage gate refuse
such a selection until its disposition restoration is composed at the existing
Native codec-retirement owner. This is unfinished work, not a route-absence
fallback. Whole-output derived-codec retirement invalidates the output generation
and requires target-producer rebuild; ordinary physical rematerialization is a
different owner. Preserve that distinction during the upcoming currentness join.

D1178 public graph cutover boundary and plan: navigation currently performs real
Native graph observation, then unconditionally refuses and later prepares only
the model's single record. The next slice removes that split: retain a borrow of
the immutable installed document recipe in the attempt, prepare the current and
selected union at the actual C-derived candidate, and send it through Native's
owned-graph receiver and existing World publication/adoption. Eligibility also
prepares and drops the real graph candidate on C, so custody and integrity
refusals name their actual Native subject before a private branch is created.
The single-record branch remains byte-for-byte unchanged. No Query ownership
reader or authored delete/create translation is introduced. Native's coverage
gate refuses any reached publication until its immutable Query observations can
be transported; that precise publication refusal replaces the unconditional
model-only seam. Public generic courts will prove checkout and redo restore
member values at original IDs; the existing foreign-incoming refusal remains.
Run the owned-model and legacy navigation courts, Native ownership courts,
boundary/context, formatting and scoped line caps. This slice does not claim
publication currentness or zero-producer-contact qualification.

## D1188 — selected ownership of transferred members

Root resolves the transfer round-trip gap: current-only membership is not a
retirement instruction when the selected basis records a live owner elsewhere.
Native restores that union member at its original identity and restores only its
selected incoming declared ownership edges. The other owner's own entity,
other members, authored fields and publications are not restoration targets.
Unowned or absent current-only members retain the prior retirement behavior.

Boundary review: `selection.rs` owns the two actual closures; keep them truthful.
`graph_selection` owns the separate restoration set, derived from stored selected
full-generation facts through the installed ownership observer. A transferred
member can belong to that set without belonging to the selected document's
closure. Its incoming declared edges enter the selected relation set; its
outgoing relations do not enter merely because it transferred. The candidate
must contain a live selected edge source. Missing sources refuse with the actual
member, selected owner and relation. Native's ordinary custom/relation invariant
scope judges the outside owner too, and final one-owner validation remains the
single exclusivity owner. Publication selections still use D1178's exact custody
and both current-fact writers; incomplete publication coverage refuses typed.
No Query ownership reader, extra identity, or historical operation replay is used.

Plan: add a funded restoration boundary under `branch/model_ownership`, carry it
through canonical descriptive targets, symbol admission and the existing slot
selection receiver, and exclude retained transfers from retirement. Use
`execution::request_stage::checkpoint` throughout affected graph stages so worker
requests poll without requiring serial accounting. Add generic Native round-trip,
three-owner chain, unavailable-owner, second-owner and outside-owner cardinality
courts, including worker-only selection. Kill the old-retirement mutation. Then
qualify and commit the preserved public graph cutover on this behavior. Update
the consumer guide/index without claiming unfinished output currentness. Before
READY, fetch/merge the pending triage batch and run complete root/Query tests
against that exact clean master. `origin/master` is still fb0870356f at this review.

D1188 inverse-recording refinement: the strict A/B/C chain court exposed loss of
E after it left both of A's actual closures. Canonical graph navigation now
records the transferred participant IDs. Bounded, charged navigation ancestry
selects the latest actual graph change for this model, carrying those subjects
through ordinary intervening commits. This is inverse-change provenance, not
ownership evidence. Both bases' exact declared incoming relations are read
again; an outside participant is activated only when those facts differ.
No unrelated outside payload is enrolled, and unavailable or nonlinear transfer
evidence refuses typed. This preserves B→C→B navigation after A→B without
reconstructing a parallel current-ownership ledger. The actual closure getters
continue to describe only recorded current/selected ownership.

D1188 next publication join boundary/plan: moving an authored producer source
moves its already-owned output subtree. The source and those outputs are in the
transfer restoration set even when they are outside the selected document's
actual closure. D1178 coverage must therefore consume that proven restoration
set, not the narrower closure getters. It must still require every publication
member and owned relation and deny a partial/foreign publication. Native's
existing roster reader supplies any publication-owned semantic relations; only
those relations join a transferred source's selected edges. Other ordinary
outside relations and B's unrelated publications remain current. Add an executed
source/output transfer round trip that compares B's unrelated publication's exact
coordinate and members across all legs, with the existing independent roster and
presence pairing. No observation rewriting, regeneration, or Query currentness
claim belongs in this Native custody slice.

Publication-join refinement: publication versions may contain different member
IDs. For a restored source, the selected Native source-custody closure supplies
its selected publication members, including members no longer live now; the
current source-custody closure supplies displaced members to retire. This is a
custody walk only, through the same Native roster reader, never arbitrary
outgoing references or another document's other source scopes. Record these
cross-document participants for the next inverse as well. Full publication
coverage still gates admission. Add a real producer succession that retires its
old member and creates a replacement before navigation: selecting the older
transferred source must restore the old ID and retire the replacement.

## D1178 immutable currentness evidence: boundary and next slice

The adversarial case is an unchanged selected producer whose source and output
slots receive fresh Native revisions during selection. Comparing its original
facts directly rejects a healthy version; rewriting those facts or running its
producer would erase the original evidence. Conversely, accepting equal values
across an arbitrary ABA or allowing later changed fields through is unsound.

Boundary review: the Native canonical OwnedGraphSelection names its exact
selected basis and full-generation restored members. The committed restoration
root supplies the fresh baseline. Native's existing exact projection reads own
field/aspect revision semantics and storage denials. Query's original RecordedOutput,
source facts, performed origin, upstream evidence and sealed witness remain
immutable; the existing full verifier continues to judge them and the installed
build. No descriptive history step, copied value or caller-supplied member list
may substitute for that canonical selection. External entities continue through
ordinary current reads. A field changed after restoration cannot use the bridge,
even if its value later returns to the selected value.

Implement the Native read-evidence prerequisite first, under the exact projection
owner. Admit only two genuine observations whose committed graph-selection
artifact names that exact selected basis. Retain the original roots and artifact,
with bounded request work and funded backing. For a canonical restored member,
compare the final field/aspect revision to the fresh restoration baseline before
comparing the original expected revision to the selected basis. Unrestored
members cannot enter that comparison. This read evidence grants no mutation,
producer settlement or authorization. Use request_stage checkpoints throughout.

Court genuine source changes and selection, original immutable field/aspect
stamps, later relevant and unrelated field changes, ABA, an outside entity,
wrong selected basis, ordinary non-selection commit, foreign runtime, bounded
requests, and worker-only execution. Mutate the membership, baseline and exact
selected-basis checks. Then integrate this evidence at the existing Query full
verifier together with exact selected Product lineage transport; that subsequent
cutover must preserve the branch/source fence for every ordinary verification
and prove two-family zero-contact undo/redo before claiming #57.

D1196 read-evidence checkpoint: `RelationalHistoricalSelectionRead` is
implemented and courted at the Native projection owner. It is constructed only
by `VisibilityReadContext::read_historical_selection` from two same-runtime
observations whose restored commit is an `OwnedGraphSelection` envelope naming
exactly the other observation's canonical basis. It retains both exact roots and
the envelope under funded memory and request-stage checkpoints. A member comparison
requires the entity's canonical selected kind in the committed selection and in all
three views, then the final revision equal to the restoration baseline, then the
selected-basis revision equal to the original expectation. Courts
(`history_ownership::historical_read_evidence`): bridged original field and
aspect revisions after a genuine edit and selection (direct comparison would
reject); unrelated field/aspect edits keep the proof; a later relevant edit and
ABA break it; an edited outside entity cannot use it; a wrong selected basis, an
ordinary commit and foreign runtimes/views refuse typed; work and memory bounds
refuse as request denials; worker-only requests poll without accounting.
Mutations killed: admit an unrestored member, skip the field baseline, skip the
aspect baseline, skip the exact selected-basis check. Removing the
older-than-restoration version guard survives as an equivalent mutant: the
baseline comparison already rejects every such view. This grants no mutation or
producer settlement; the Query full-verifier integration (#57) is next.

## D1196 #57 join: verified boundary and slice order

Public fixture `certification::document_history::owned_outputs` (7df5047373):
an OwnedModelGraph Model sourcing two SourceScopeCascade families. Courts fix
today's boundary. Checkout over the model's owned publications refuses at
`prepare` with Native `OwnedPublicationRequiresCascade { publication, owner:
model }`, with no effect or producer contact, because Query navigation still
passes no output custody (`navigation/progression.rs` and `eligibility.rs`
call `prepare_owned_graph_selection(.., None)`). Settling producers on the
document branch outside history moves its head (`HeadDiverged`). In-history
output settlement therefore belongs to the recorded step
(`RecordingCompletion::RequiredOutputs`, one tree today). Multi-family
recording is part of #57, not a workaround.

Verified Query currentness owner: demand-time producer selection
(`application_contribution/producer/demand/selection/native_selection.rs`).
It reads lineage candidates, then output liveness, the producing build
(`RequiredOutputProducingImplementation`), settlement verification
(`output_reuse::retained_output_settlement_is_verified`, the full verifier
with its branch/source fence), and the recorded facts
(`compare_retained_output_dependencies`, which first requires
`identity_current`: the runtime source idempotency identity). Then it checks
the sealed witness (`compare_retained_output_witness`). Reuse makes no
producer contact; any failure selects Preserve. The Native-restored prior
transport (`selection/native_restoration.rs`) runs only when lineage has no
candidate. It records a fact-less prior (`output_lineage/restoration/prior_output.rs`),
which can only select Preserve.

Each gate fails on a healthy restored output for a distinct reason. The
lineage candidate is the current head's row, not the selected step's. The
source identity commits to the current basis. Recorded field/aspect revision
facts name the selected version, while restoration stamps fresh revisions.
The witness aspects are restamped. The settlement row's read basis lies on the
selected step's branch coordinate. Slice order, each with courts and mutants:

1. Native exposes whether an installed publisher owns source scopes. Query
   graph navigation prepares `prepare_historical_output_selections` and
   `admit_historical_output_selection` only then, so storage absence still
   refuses typed. It passes `RequiredOutputPublication::HistoricalSelection` in
   eligibility and progression through one helper. In the same slice, at the
   performed navigation commit, the selected step's exact lineage rows for each
   restored route are transported to the new product coordinate. The transport
   carries their facts, settlement identity, witness, implementation and
   correspondence. Its `required_publication` is the committed
   HistoricalSelection publication. It is never a fact-less prior, and never
   the current head's row. Without this transport, a later Preserve would name
   a stale predecessor, so slice 1 does not land without it.
2. A transported row carries the selection commit and selected basis
   coordinate. The four gates consume `RelationalHistoricalSelectionRead`
   built from those exact observations. Field and aspect revision facts and
   witness aspects use it only for restored members whose kind the committed
   selection names. Source identity and the settlement fence admit the
   original basis only through that evidence. Every ordinary row keeps today's
   branch/source fence byte-for-byte.
3. Multi-family recording and navigation profiles: eligibility admits every
   installed output tree whose source scope is owned by the model. Edit
   recording carries one completion per tree. Navigation's AwaitingOutputs
   advances every tree at one final endpoint.
4. Courts: undo/redo on `owned_outputs` with zero producer contacts; a changed
   producing build contacts exactly that family. Mutations: regenerate instead
   of select (skip slice 1), skip evidence (ordinary fence), and drop one
   family from verification.

### D1196 #57 slice 1 landed: navigation output custody and lineage transport

- Native exposes `RelationalInstalledApplicationPublicationAuthority::owns_source_scopes`
  and `RelationalRequiredOutputPublication::same_origin`.
- `navigation/output_custody.rs` is the one owned-graph preparation for eligibility and
  progression. When the publisher owns source scopes and Native finds owned producer
  publications, it prepares `prepare_historical_output_selections`, admits them with the
  installed owner and passes `RequiredOutputPublication::HistoricalSelection`. Otherwise it
  passes none, so a missing ownership substrate still refuses typed at Native coverage.
- Before World, progression prepares Native's publication custody for the exact candidate
  envelope (`prepare_required_output_publications`) and resolves every restored route to the
  selected step's own lineage row (`output_lineage/historical_transport.rs`). The row must
  hold that route's Native publication or its recovered locator; a missing or ambiguous row
  refuses `OutputLineage(SelectedSourceMismatch)` before any effect. At the performed
  selection commit, each row is recorded again at the new product coordinate with its facts,
  witness, implementation, correspondence and idempotency, naming the committed
  HistoricalSelection publication. Its new settlement has no mark row, so selection still
  verifies it in full and today selects Preserve; slice 2 supplies the evidence gates.
- Courts (`owned_outputs::custody`, serial and worker-only): undo and redo restore custody
  with zero producer contacts; a later settlement is Preserve for both families and keeps
  the original output identities. The former typed refusal court is retired because it flipped.
- Mutations: passing no custody returns the `OwnedPublicationRequiresCascade` refusal and
  fails both courts. Skipping the transport survives slice 1: in this fixture the selected
  and head rows coincide, and Native's transition-successor readmission names the current
  publication for Preserve. Slice 2's zero-contact court is the transport's kill evidence.

### D1202 slice 2 acceptance boundary

The carried evidence gates alone do not grant contact-free delivery. Verified in
source: native_selection admits exact retained delivery only for recovered rows,
CanonicalBitwise input reuse, or an existing Ready at the same current source.
source_admission then readmits only a recovered checkpoint. A transported row
therefore reaches Preserve even when all its evidence gates hold.

Generalize the existing restored-accepted origin at the output-demand registry:
Checkpoint retains its current identity and recovery authority; HistoricalSelection
retains the committed Native publication and exact transported settlement. The
selector supplies its exact retained binding/key only after currentness succeeds.
Admission resolves that exact row at the current selected Product, verifies it
again against the current head, and admits it through the same restored-demand
owner. Settlement leases that genuine Product observation, preserving its
historical-selection provenance and reporting that posture explicitly. It
requires no recovered-root authority. Ordinary and checkpoint paths keep their
current fences. Checkpoint capture may carry descriptive historical output
identity, but cannot encode unrebased original revisions as current checkpoint
reuse evidence.

The count courts must show two healthy families reused after undo, repeatedly,
on both serial and worker-only requests. Skip-transport and skip-evidence must
fail them. Include stale-at-selected-step and post-selection-change legs so
restored acceptance cannot bless stale rows. Preserve original computation and
upstream evidence in the transported row; a leaf-only fact comparison cannot
replace the full dependency-closure verifier for dependent families.

Current-roster qualification refinement: a current roster's decoded canonical
origin carries no live issuer Arc. The retained Native publication seal carries
that issuer. Authenticate it through worth-proof's installed-publication owner,
then compare the authenticated current roster and independent presence. Do not
compare serialized origin issuer fields to live installation identity. A Native
court exposed and corrected that mistaken premise before qualification.

D1202 slice 2 evidence: the two independent-family undo demands reuse the exact
historical rows twice without producer contact, for serial and worker-only
requests. A stale selected edit still triggers both Preserve producers. Eight
compiled mutants are killed, including skipped transport, omitted evidence and
skipped verification. The restored suites pass 126 public history courts, 52
Native ownership courts and 2260 Native library tests. This does not complete
slice 3: automatic verification of every route at one endpoint, dependent-family
and nested-selection courts, multi-tree recording and build-isolation remain.

### D1202 selection continuity court (before slice 3)

Boundary review: transport currently retains only the latest selected/restored
Native pair. A selected step may itself contain an inherited historical row:
an ordinary owned-member edit after undo need not invalidate model-sourced
outputs. Revisiting that new step must still compare the original producer
observations, not observations rewritten to the intermediate restoration stamps.
The current fixture has only a model-value edit, so it cannot expose this leg.

First add a concrete Member-scoped edit to the existing generic fixture and
court undo, edit that unrelated owned member, navigate away and return, then
demand both original output families with zero contacts. Run it before changing
production. If it exposes the lost evidence chain, retain the authentic prior
selection coordinate on transport and compose its comparisons at Native's
historical read owner. Every link must still be an actual committed selection;
currentness remains on the existing verifier, facts/witnesses remain immutable,
and worker-only requests keep their original checkpoints. Add a stale/ABA leg
and a compiled drop-prior-evidence mutant. No new test framework or authority
constructor. Then rerun the public history and Native ownership suites plus the
boundary/context/format/line-cap gates. This is continuity of D1119 evidence,
not a new navigation policy or a substitute for slice 3's multi-family owner.

The continuity court failed on the merged tree with two extra contacts. Transport
now retains the authentic prior coordinate. Native composes a flat oldest-first
list of independently authenticated selection stages, refuses empty/reversed
chains, and compares each needed member and restoration baseline. A matching
selected revision ends the comparison at that prefix; an intermediate ABA still
fails. Original source observations and witnesses are never rebased. Upstream
ancestry starts only at those authenticated selected roots, traversing their
actual canonical parents. Both coordinate traversal and Native stages use their
original request, checked memory and worker-aware checkpoints; no wire change.

The public continuity courts include an ordinary Member-scoped edit that leaves
the model-sourced families unchanged, a visit away and back, independent payload
reads and zero producer contacts. Their already-closed edit outcome is dropped
before document teardown because it retains readonly World history. Native
courts also cover intermediate observations, later changes, intermediate ABA,
empty/reversed/mismatched chains. Nine compiled mutants are killed: newest-only
field/aspect comparison, omitted field/aspect baselines, reversed/empty chain
admission, dropped prior coordinate, and raw accounting-only Native/Query
checkpoints. Every mutation was restored byte-for-byte. Pristine suites pass
55 Native ownership and 128 public history courts; boundary/context, formatting,
whitespace and the 16-file dirty Rust cap pass. The primary structural review
read all 12 function advisories; no independent review is claimed. Slice 3 is
still required, so this is no issue-batch READY.

### D1202 required ruling: a restored source absent from current control C

Named owner: Query source-scope admission, especially
`application_query/admission/access_observation.rs:105-115`,
`entity_resolution/freshness.rs:25-47`, and
`application_query/document_history/output_read.rs:53-59` (all under
`worth-query-execution/src/domain_computation/primary_graph`).

Verified in code: history's source control plan must select C for both data and
security, and ordinary source admission validates the scope's exact full-generation
identity through the identity index at that security snapshot. This check precedes
query authorization and also applies to a public query. A source E retired at C
cannot obtain that control plan, even if the owned graph selection restores the
same E live at private D. Currentness can reuse healthy rows without this lookup;
a failed verification requiring the original tree reaches this gate. Selecting D
before qualification would violate the atomic navigation boundary. Using D's
principal/grants as current control would violate current authority.

Concrete leg: an owned Feature-like Member E has a publication at the selected
step; it is retired at C. Checkout restores E and its output at D. If verification
fails because the producing build or another input changed, the existing fallback
source entry cannot resolve E at C. The current two-family fixture uses Model
sources, so this restored-only-source fallback leg has not yet been executed as
a public court; the gate and its original request/security basis are verified
from the production path. No new authorization posture was implemented.

Ruling requested before the child-source/forest cutover:

1. Add an explicit historical-source control posture: validate source E's exact
   presence and owned-domain correspondence at D, using the genuine Native graph
   selection; freshly evaluate principal, permission, capability and request facts
   at C. A permission C cannot establish still refuses typed. D provides data and
   presence, never grants. Preserve the original root/read/disposal owners and
   the unchanged ScopeEntityRecord path.
2. Keep the current-C source-liveness prerequisite and refuse navigation before
   selection whenever fallback needs a source absent at C, naming E and its family.

Option 1 is recommended for invertible deletion/build-change navigation. This
is a source-admission authority choice, beyond merely collecting more trees.
The lane stops pending that choice, under the task's explicit design-stop rule.

### D1207 historical-source control implementation

Root selects option 1. Master b5d39e00eb is merged (plan-only). The boundary is
current authorization C versus source presence on the exact private navigation
candidate D. Ordinary source presence at C keeps its existing path. No D grant
may authorize a read or producer. Source permission refusal precedes any new D
identity/presence refusal; the subject/family come from retained original
correspondence, not a probe of D.

Verified handoffs: SDK history output ingress currently resolves C scope before
calling Native; root query admission checks freshness at the C security snapshot;
retained producer permission, in-handler source queries and operation admission
also require current issued scope. Updating only the first read would fail later.
The existing prepared Query permission and history invocation owners already
separate current security from candidate truth; extend them, never create another
producer engine. Ordinary and ScopeEntityRecord entrances remain unchanged.

Implementation order:
1. Native's authentic historical-selection read reports exact restored-source
   liveness/kind/domain membership. Its result is readonly fact, not permission.
   Keep all reads at the capsule's own D; no caller-supplied substitute basis.
2. Extend the canonical retained scope selector to retain the original sealed
   entity description needed for C authorization before resolving D. Do not
   synthesize a typed identity from caller coordinates or rediscover a scope by
   name on another generation.
3. Add the explicit historical-source posture to prepared Query permission and
   its root/retained/handler-source consumers, using fresh C principal, policy,
   capabilities and request checks before D presence/domain checks. Bind it to
   the actual navigation invocation/candidate and retain existing denial causes
   with E/family attribution. Source presence at C uses the old path.
4. Join child-scoped output profiles and SDK tree ingress to that posture, keeping
   original root, source, qualification and disposal custody. Extend the generic
   multi-entity producer fixture to exercise a source absent at C, actual fallback
   at D, denied C permission despite D permission, retired D source, and workers.
5. Kill the C-identity/D-grant/skipped-D-liveness mutants with byte-exact SHA256
   restoration. Run Native/source owner/public history courts, boundaries,
   context, format and dirty caps; then proceed to the complete #57 forest and
   remaining issue work. Final root/Query versus clean master remains required.

The placement of source presence is Native (it retains actual runtime roots).
The Query posture composes the existing concrete history invocation and native
fact evidence; it grants no independently constructible permission. No format
change or fallback to history storage is introduced.

D1207 execution evidence: the serial and worker-only public courts restore a
declared Member source missing at C, use its changed selected value at D, and
contact the producer exactly once. A separate restricted family refuses through
the original C authorization owner despite a granting selected model; its source
and family attribution and unchanged selection are asserted. These admission
courts use independent singleton outputs without a publication relation. They do
not substitute for the two original opted-in publication-custody families.

The public deletion/restoration leg exposed an actual Native selector omission:
incident relations retained for audit after entity deletion must route through
the authenticated restored after-image, just like DeletedRetained relations. The
selector now permits that exact lifecycle while continuing to require matching
live after metadata; missing-live corruption and suspended-slot checks remain.
A worker-only failure was the SDK output-registry accounting-only checkpoint,
not the current identity lookup. It now follows that owner's existing charge-or-
poll dispatch. New Native work uses request_stage::checkpoint.

The remaining cutover includes nested handler queries, later dependent sources,
multiple trees, all-route verification and recording at one endpoint. These are
not implied by the root-source admission courts. No issue-completion claim.
