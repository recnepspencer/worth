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

### D1207 checkpoint and #57 forest boundary

Source-admission checkpoint `d488beeb30` is local, clean and qualified by 131
public history courts, 56 Native ownership courts, 2262 Native library tests
(31 ignored), four Query source-control courts and eight restored compiled
mutations. Boundary, generated context, formatting and dirty caps pass. No
complete-workspace or issue handoff is implied.

The next vertical slice is a collection of original output trees for an owned
graph document. The existing single-tree driver contains both typed SDK custody
and per-call Query READ/NAV admission. Calling it repeatedly would perform several
independent navigation transitions and could adopt before another family is
qualified. That path is insufficient. The existing original-endpoint owner derives
E from actual World publication movements; the completed-set currentness owner
can qualify an entire original tree at an explicitly selected E. Those remain the
only endpoint and currentness authorities. Native still exclusively owns closure,
roster and route presence. A profile is descriptive installed correspondence,
never ownership, admission or a settlement.

Plan:
1. Extend the canonical history profile owner with an immutable, funded collection
   of original single-tree profiles. Preserve the single-profile path and its
   custody comparison. A collection is graph-only; members retain their own
   program/root/source/recipe correspondence and cannot nest or duplicate a root
   demand. Eligibility checks every relevant installed delivery against the whole
   collection before effects.
2. Add an SDK collection mode that retains each original typed tree behind its
   compiled driver, with runtime-length addition. It preserves each slot, original
   source, failed cleanup and close owner. Collection allocation and traversal use
   the existing charged/polling owners; no new framework or format is introduced.
   Keep existing `with_outputs` and ScopeEntityRecord behavior unchanged.
3. A single fresh history READ/metadata/NAV admits one navigation advancement.
   It lends each registered tree its own borrowed output authority for the same
   private D and same current C. A callback cannot substitute another profile or
   basis. Healthy restored rows use the existing currentness reuse gates; only a
   failed family enters its original producer continuation.
4. Retain authentic complete settlement shares and dependency custody from every
   original tree. Derive one endpoint from their union of actual World movements,
   then ask the existing completed-set owner to qualify each whole tree there.
   Never substitute the current head, infer ancestry from ordinals or manufacture
   a settlement. Only the complete collection can enter Selecting. Selecting
   freshly readmits every original tree; disposal keeps every actual remainder.
5. Join edit completion to the same collection at its one final endpoint. Add
   public two-family serial/worker undo-redo zero-contact courts, changed-build
   family isolation, partial/refused cleanup and a family-drop mutant. Then cover
   dependent source admission and new candidate members through Native ownership,
   codec-retired restoration, remaining issue courts and #56.

Checks for the slice: focused public owned_outputs/history courts, Native ownership
and library courts, Query currentness/contribution/output-demand courts; compiled
mutations with exact SHA256 restoration; boundary/context, formatting, function
review and scoped caps. Full root and Query tests versus clean current master are
required before an issue READY. The profile collection is process-local and leaves
CHK18 unchanged.


### D1207 forest fallback evidence and direct-verification correction

The original-tree forest now passes 21 public owned-output courts. The dependent
fixture strengthens membership: its generated Member reaches the model only
through Native publication custody; UpstreamOf is an ordinary reference. Ordered
fallbacks preserve both original identities and publish 10/1000. Reversing the
order leaves the earlier child stale, so final whole-set qualification refuses
without adoption. Serial and worker-only legs cover both outcomes.

Three compiled mutations fail their intended courts: skip the first tree's final
currentness, skip expected-route coverage, and ignore route family identity.
Every mutated file is restored with the same SHA256, recorded in
build-cache/history53-d1207/forest-mutation-results.json. These qualify the forest's
fallback collection mechanics, not completion of #57 or D1119.

Review against D1119 found a remaining boundary mismatch: even healthy roots
currently execute source disclosure and create original SDK trees before reuse.
Zero producer contacts alone does not prove that healthy selection avoids the
original source/root/disposal path. Direct restored-route verification must enter
the existing output-currentness owner before any SDK root. It must use Native's
exact selection requirements and actual current roster/presence, the canonical
lineage lookup and installed producer edition; no synthetic settlement or second
producer engine. Unknown/stale evidence enters the original fallback driver.
The private candidate cannot be adopted before all actual routes pass.

Next plan: add a canonical lineage route read, compose it with the currentness
owner at the original untouched restoration candidate, and let the registered
navigation progress to Selecting only when every Native route passes. Repeat
verification before any later adoption retry. The ordinary ScopeEntityRecord
path stays unchanged. If a registered root has no selected owned publication,
it keeps the original source/root fallback path, including D1207 current-C
source permission. Then narrow mixed healthy/stale progression to only the
failed families, retain all final-endpoint checks, and join recording. Public
courts must distinguish the healthy path from source disclosure, rather than
merely counting producers. No issue READY until those paths and final workspace
qualification are complete.


### Direct restored-route verification checkpoint

Master `0173a4da61` (root closure retirement carriage) is merged as `370cbc4b19`.
The fallback collection is committed at `39b71f4cfd`; the direct-verification
continuation is qualified separately. Its canonical route lookup uses Native's
exact selected family/scope/partition/version and the existing metered lineage
reader. The output-currentness owner checks the installed producer edition,
actual current ownership pairing, source facts, witness and upstreams. The
navigation phase owner can progress the untouched committed restoration directly
to Selecting; it issues no synthetic root or settlement. A later adoption retry
repeats that verification. ScopeEntityRecord keeps its previous path.

All 26 public owned-output courts pass in direct-owned-3.log. Healthy selection
with or without configured fallback roots projects zero producer sources and
contacts zero producers, in serial and worker-only requests. A source-scope-owned
restricted family remains healthy even when C denies its producer-source READ;
verification does not attempt source admission or borrow D grants. The separate
D1207 fallback refusal continues to require that permission at C. Expected-route
coverage courts now select a genuinely stale step, because an omitted healthy
family can be verified directly without an SDK root.

Three compiled mutants fail their intended courts and restore SHA256 exactly:
reopen healthy original roots; skip direct currentness (observed stale 10/1001
instead of 20/1002); skip the non-restored-root guard (missing regenerated output
for the restored-only independent source). The pristine whole history and static
checks run after the mutations. This is not issue completion.

Remaining D1119 boundary: when only one independent leaf family needs fallback,
the current forest still opens every configured tree. Next, retain original trees
only for actual fallback, and let Native/currentness verify skipped healthy leaf
routes again at the single receipt-derived final endpoint. The complete fallback
trees retain their discovery dependencies and original disposal owners; no
settlement is invented for a skipped root. A registered non-leaf or non-restored
root conservatively keeps its original tree. The public mixed fixture must show
one stale family regenerated with no healthy source projection, and the later-
upstream invalidation court must still prevent stale adoption. Then add the real
changed-build/cold-reopen journey, owner-recorded fallback reasons, multi-tree
recording, codec-retired selection, remaining issue courts and #56, followed by
full root/Query comparison to clean current master.


### Mixed leaf verification boundary and implementation

The sealed installed graph shape supplies a leaf fact on each original profile.
Only a leaf with one exact selected Native family/source route is eligible to
skip its original tree; non-restored roots and non-leaf discovery keep their
original owners. Query's readonly leaf question consumes the borrowed history
frame, checks current C, reads the actual same private candidate head, and calls
the same currentness owner. Its boolean is no settlement or completion grant.
The SDK skips only an unstarted tree and leaves existing original continuations
and failed cleanup in their own registries.

The forest completion owner accepts an ordered subset of authentic complete
fallback trees. The original endpoint owner derives E from those actual World
movements. At E, every missing Native route must pass the canonical selected-route
currentness check, every skipped registered root must still be a qualified leaf,
and every complete original tree retains its discovery dependencies and passes
whole-set currentness. An empty fallback collection cannot invent an original
root or endpoint; the phase owner's all-healthy path handles that case. A failed
or unknown skipped route leaves the verified model set incomplete, with no
adoption. The old ScopeEntityRecord entry is unchanged.

The next public mixed court starts with two real owned publications, records an
ordinary Member source edit that makes only one stale, and requires one producer
contact with zero healthy Model-source projections and equal original output
IDs. Serial and worker requests use the same fixture. Existing late-invalidation
courts must continue to reject a stale final endpoint; their exact observed
failure path is being requalified under selective fallback. No result is yet
claimed for this dirty continuation.


Mixed leaf qualification passes: 148 public document-history courts, including
28 owned-output courts, plus the newly merged master's root-retirement court,
boundary/context and dirty caps. Three compiled mutants fail and restore the
same SHA256 (selective-mutation-results.json): reopen a healthy sibling; skip the
earlier actual fallback's final currentness; skip an unsettled Native family.
The second now exposes a stale completed original child, using the genuine
pre-attachment succession fixture. Pristine checks are in
selective-checkpoint-results.json. The source-projection mixed court remains
zero for the healthy sibling and counts exactly the one real stale-family run.

This checkpoint covers leaf forests and conservative complete original non-leaf
trees. Direct verification of closed non-leaf discovery dependencies, original
multi-tree recording, real changed-build isolation/cold evidence, owner-recorded
fallback causes, nested later sources, codec-retired selection, remaining issue
courts and #56 are still unfinished. No issue READY is implied.


### Changed-build isolation fixture boundary

Use a genuine Query checkpoint before history attachment: the ordinary checkpoint
owner captures the default occurrence, so this journey does not claim persistence
of the process-local document or its private branch selection. Reopen the same
schema/program under a new ApplicationBuildIdentity and a genuinely changed
immutable second-family handler (value +2000 rather than +1000). Refresh only
the first family through its actual producer, then attach history over the mixed
real current publications. Selecting that snapshot must retain the current-build
first family unopened and regenerate only the old-build second family, at the
same original output IDs. No lineage row, fact, build stamp or checkpoint bytes
are edited by the fixture. Serial and worker variants use the existing world.

Implementation shape: the fixture phase handler owns its immutable derivation
function, with the existing family installer selecting its ordinary default;
world configuration supplies the new second implementation only on the reopened
build. The checkpoint capture/reopen path uses the public host facade. The public
court asserts the resulting 2001 output and one contact during navigation.
The shared Model-source projection cannot attribute reads per family; the
separate mixed-scope court proves that the healthy first root stays unopened. A mutation that bypasses the build fences must
be tested against this real journey, and any surviving independent refusal is
reported honestly. Cold current-fact equality after HistoricalSelection remains
an obligation of the Native custody owner; no private-document persistence API
is inferred from a default-occurrence checkpoint.


### Verified cold-witness gap in historical transport

The real changed-build journey passes, but bypassing both direct build gates
survived. A temporary diagnostic (restored byte-exactly) found second's original
checkpoint facts present/comparable and its typed output witness absent. Ordinary
checkpoint demand readmission reconstructs that witness through
SealedNativeOutputWitness::from_checkpoint_facts; historical transport currently
copies the empty witness slot. This is a transport gap, not build-only evidence.

Repair at the canonical witness owner: prepare original output expectations from
the authenticated checkpoint facts and installed role/aspect coverage. Never
project expectations from H or D. Native historical-selection evidence and the
existing currentness owner must still compare them before reuse. Carry the new
witness only on the transported row; leave the original cold row unchanged, so
ordinary and ScopeEntityRecord entrances retain their existing behavior.

The new reconstructive call is bounded and charged, including role indexing and
fact/aspect visits, and retains memory with its witness. A sorted flat role index
makes transient storage explicit while preserving the existing ordered role
lookup and conflict/missing-fact refusals. The original constructor remains the
same semantic entrance, with an internal funded historical variant. No format,
producer, source observation, output revision, grant or settlement is invented.
Re-run the actual build-fence mutant, existing checkpoint-witness owner courts,
serial/worker public journeys and static checks before the checkpoint claim.


### Cold correspondence binding before witness construction

Boundary review found that ordinary same-schema checkpoint admission validates
installed role declarations but does not authenticate Query's role IDs against
Native's publication. The historical locator alone names producer/source, not
the complete Native role set. Reconstructing an output witness must not promote
that descriptive metadata to ownership. Native remains the sole publication
truth; closure evidence remains separate from published roles.

Before transporting any selected row, the correspondence owner compares its
complete role set to the selected Native publication: role, declaration name,
live/retired posture and full-generation entity identity. A Native
HistoricalSelection is a fresh Preserve of a live role, so Query's retained
original Create is accepted against that Preserve only for this origin; retired
roles cannot take that normalization. The sorted Query role index is
looked up for each actual Native role with request-stage charge-or-poll; equal
cardinality and Native's unique roles make omission or substitution impossible.
A mismatch refuses with OutputPublicationCorrespondence carrying the actual
publication, before the candidate is published or adopted. This is not a second
ownership reader and grants no custody. The constructor still creates only
original expected versions; the existing currentness owner verifies them.

Evidence plan: cold same-build serial/worker navigation must retain both families
with zero contacts and zero producer-source projections. A rechecksummed Query
role with an impossible generation, leaving the actual Native publication intact,
must refuse typed with that publication and leave document/head/outputs unchanged.
Kill omission of the role binding, omission of reconstructed witness transport,
and both build fences; restore every mutation with equal SHA256. Re-run all public
history and checkpoint-witness owner courts, boundary/context and scoped caps.


Cold witness and role-binding positive evidence: 33 owned-output courts pass
(cold-binding-positive-3.log). All three compiled mutants are killed and their
files restored SHA256-identically (cold-mutation-results.json). Dropping the
witness contacts 2 instead of 0; bypassing both build fences yields 1001 rather
than 2001. Role-binding omission loses the precise publication-correspondence
refusal. The original constructor's two checkpoint owner courts also pass.
Pristine whole-history/static qualification follows before committing.

Pristine cold checkpoint qualification passes: 153 public history courts, the
new master root-retirement court, both canonical checkpoint-witness owner
courts, boundary/context and scoped line caps (cold-checkpoint-results.json).
The final accounting includes the OnceLock/Arc headers. Primary structural
review keeps correspondence matching with the correspondence owner, original
expectation construction with the witness owner, and transport with lineage.
The long constructor is one role/aspect-coverage operation; phase/transport
orchestration retains its existing scope. No independent reviewer was assigned.
Remaining #57 non-leaf discovery, owner-visible fallback causes, multi-tree edit
recording, nested later sources and codec-retired selection remain unqualified.


### Refusal subjects must not retain Native custody

Primary review verified that RelationalRequiredOutputPublication.readmitted_at
retains an admitted basis, whose inner owns its Native root and observation
obligation. The new mismatch diagnostic needs only names and a full-generation
source ID. Retaining that publication merely to borrow its strings incorrectly
couples diagnostic lifetime to graph custody.

Cut over the mismatch subject to an immutable, funded data snapshot at its own
owner: producer, family and source only. Clone shares that snapshot and its
reservation. Copy work uses the original request's charge-or-poll stage. Native
custody does not cross this diagnostic boundary. Keep the same public accessors
and precise refusal. A read-only, owned capture observer watches a weak Native
basis through its publication owner; it never retains the basis or changes
admission. Use a genuine cold corrupted-role navigation and tear down its host
while retaining diagnostic clones. Its basis must be gone, with valid diagnostic
names, on serial and worker-only requests. A retain-publication mutant must keep
the weak basis alive and fail that court, then restore SHA256 exactly.

No parent/discovery option is assumed. This is an independent correction to the
already implemented refusal subject. Re-run focused/public history and static
checks, then qualify the changed lane against the unchanged master comparison.


Refusal-lifetime checkpoint evidence: 154 public history courts, both checkpoint
witness-owner courts, master root-retirement and boundary/context/caps pass.
The final retain-publication mutant fails both actual host-teardown liveness
courts and restores denial.rs SHA256 c86a2a4ca953eb99acf700b7d8c17d7bc434d070833a667a3afffae2070ba229.
The final four cold courts pass after restoration. Native's read-only weak probe
now lives in its custody test-door module; its native library check and final
boundary/context/caps pass after this extraction. The published commit/ordinal
is copied from the existing Proof descriptor and carries no issuer custody.
The capture observer is owned, thread-affine and clears on unwind. No framework,
format or producer path is added. A new complete lane run follows; the prior
complete workspace results belong to38adee58d5. No issue READY is implied.


### D1214 verified historical parents: boundary review and implementation plan

Root selected option 1. The existing history dependent/discovery owner will mint
an opaque verified parent after the exact D route passes canonical lineage,
custody, implementation, source-fact and witness currentness. It is a distinct
runtime observation posture, never an ordinary or history settlement. The
program/profile, full-generation source and branch incarnation remain bound to
the navigation's original candidate. Every read still enters through current-C
authorization before D identity is inspected.

The ordinary settled path stays authoritative for actual producer fallbacks.
The verified path runs the installed DiscoveryBinding over the verified output;
its complete dependent set comes from those actual rows and retained query
dependencies, not Native's route census. Native membership cross-checks coverage.
Changed build/connection meaning or stale output enters actual fallback; malformed
custody or parent evidence preserves a typed refusal before adoption.

Implementation order:
1. Add the sealed parent posture at navigation/output_source/dependent and mint
   it through the existing program history owner, using canonical currentness.
   Add a separate discovery entrance and retention path with current-C checks.
2. Thread that posture through the sealed SDK graph traversal and qualification.
   Healthy roots stay unadmitted; completed verified discovery is distinct from
   real root settlements. Recheck the complete graph at the final exact endpoint.
3. Public generic non-leaf courts: exact discovered children with zero root
   contacts; actual stale-parent fallback; corrupt lineage/witness no-effect
   refusal; changed discovery/connection fallback; current-C permission denial.
   Include worker-only execution. Kill roster-substitution, skipped parent
   evidence and historical-grant mutants, restoring SHA-256 exactly.
4. Run focused owner/public history checks, boundary/context, format and scoped
   caps, then full root/Query comparison against the same clean master.

The earlier complete Query process ended without a final result during its
charge sweep; its partial logs remain interrupted evidence. The supplementary
clean-master graph run finished and reproduced lookup-growth's population
assertion. No partial full-run totals establish issue qualification.


D1214 checkpoint evidence (not issue completion): the opaque verified-parent
posture, real installed discovery/retention, and actual child demand admission
are implemented. No verified parent converts to a settlement. A retained tree
holds either an original root or a verified root, never both. Mixed completion
derives its endpoint only from authentic producer publication movements, checks
all real child settlements/discovery dependencies there, then checks the verified
root and the complete Native route set through the existing currentness owner.
The settled path's original refusal ordering and text are preserved.

Six public courts pass: healthy/stale non-leaf, an empty computed set while a
matching Native publication exists, and a C-denied/D-granted discovery read;
each has a worker-only variant. The empty-set court proves no child source opens
merely because Native has a matching route. The stale-parent court independently
reads the parent's regenerated value. All 160 public history courts pass after
exact mutation restoration. Boundary/context, dirty 400-line caps, rustfmt/diff
and the verified-to-settled negative doctest pass.

Compiled mutation (a), skip discovery and rely on Native route verification,
fails all six courts. Mutation (b), bypass canonical parent lineage/witness
currentness, fails four, including the independent stale-parent value oracle.
Every changed file restores identical SHA-256; active-d1214-mutant.json is null.
Evidence is d1214-mutations-ab-results.json and d1214-pristine-* under
build-cache/history53-d1207. No independent reviewer was assigned; the primary
structural pass retained phase signatures as explicit inputs and separated
parent meaning from posture-specific diagnostics.

Still required for D1214: corrupt-parent no-effect and changed-discovery-meaning
courts; the D-grant mutation; and the historical-source discovery entrance when
its scope is absent at C. That entrance must use the D1207 current-C permission
owner with exact private-candidate identity, without another ownership reader or
per-step discovery material. Nested later sources, multi-tree edit recording,
codec-retired selection, remaining issue courts and #56 remain in sequence.
No full-workspace equivalence or READY is claimed by this checkpoint.


### D1214 follow-up: source partition is part of parent identity

Boundary review verified that Native's RequiredOutputPublicationMeaning carries
source_partition and WorthQueryObservedSource.partition_identity is exactly the
admitted parameter-binding identity. The current profile matcher checks only
family and full-generation scope. Equal scopes with different parameter sets
must not stand in for the selected root requested by its captured intent.

Add a getter of the existing admitted parameter identity on the intent; compare
it in the existing profile matcher, with bounded charged work. Do not recompute
or invent a digest. A real parameterized source query/producer fixture will
record an output under one parameter set and configure a different set whose
actual source is absent. Navigation must refuse without adopting, instead of
certifying the other partition as its parent. Mutating away that comparison must
make the court fail. Run the public history suite and scoped static checks, then
continue the historical-only discovery and remaining D1214 proofs. This repairs
an observed identity boundary within the current slice; it creates no new storage
format, permission posture or alternate producer path.

### D1214 historical-only discovery admission

The current SDK admits a discovery intent at C before the parent owner sees it;
that cannot read a restored output which is absent at C. Cut verified discovery
over to the existing SourceQueryReaderAccess owner: it already binds C-issued
principal/control, D query execution and C reauthorization. Extend its private
scope posture to permit an exact D-issued scope only during the same sealed
history invocation. Current scopes retain their existing admission. A historical
scope never supplies a principal or grants. Existing selected-access admission
must authorize at C before its canonical D liveness/domain check. If D lookup
cannot establish a scope, preserve the original nondisclosing C scope refusal;
current permission denial must likewise reveal no extra historical existence.

Keep this posture distinct from a settled output. The program owner validates
the verified parent and installed connection, executes the real discovery intent
through that reader, retains its actual observation/dependencies, and checks the
computed set at completion. SDK passes intent and current principal/request;
it no longer requires an ordinary C-scope plan for verified discovery. Ordinary
and settled-history traversal remain unchanged. A public generic downstream
parent deleted at C/restored at D will expose the old entrance refusal and prove
zero parent contacts and the actual dependent set, including worker placement.
Existing C-denied/D-granted and fresh-source fallback courts continue to guard
authority. Mutate grants to D only after the pristine courts pass, restore exact
SHA-256, then run the whole history family and static enforcement.

The deleted-parent fixture exposed the existing authored-retirement contract:
a plain deletion is correctly refused OwnedPublicationRequiresCascade. The
member_delete fixture deliberately has no authored anchor and served only
independent publications. Add a separate, genuinely declared authored member
retirement/restoration pair in the generic fixture, with exact Retire/Create
roles and recorded pre-image correction meaning. Register it through the
contribution and normal program/action/history declarations. Keep the plain
delete unchanged. This is fixture admission through the landed cascade owner,
not permission to weaken custody or infer an authored anchor from reachability.

D1214 follow-up runtime checkpoint: all11 hierarchy courts pass, including two
real historical-only discovery cases (serial/worker) with zero producer contacts,
two parameter-partition refusals, the actual authored-retirement premise, and the
six earlier discovery/current-permission courts. The authored fixture uses exact
Retire roles for the source and its generated member, observes its demanded
fields and actual relation facts, and explicitly retires the corresponding
edges. Native proves the complete publication roster; reachability alone supplies
no ownership. No seeded creation provenance or synthetic retirement is claimed.
Native merge qualification passes56 ownership-history and3 restoration-header
courts. Source-partition and D-grant mutants are running, with exact restoration;
whole history and static checks follow. Corrupt/changed-meaning and D1119
healthy-child/source discipline remain unfinished; no issue completion is claimed.

### D1214 child-source admission through the same parent-query owner

Boundary review: discovery now enters through the canonical source reader, but
verified child-source admission and settled-history parent queries still enter
SDK with_admitted_query at C. A live child only in D cannot reach its actual
fallback. Both parent postures already carry genuine, distinct custody. Keep
that distinction in a private borrowed parent sum at the existing history owner;
never convert verified custody into a settlement. Share current-C principal /
permission and exact-D query execution in history/parent_query, reusing
SourceQueryReaderAccess. Validate the actual parent, connection, program,
artifact and basis after C READ and before data projection. Preserve the old
ScopeEntityRecord control-plan path; the owned-graph path alone takes actual
intents. Derive family diagnostics from the installed parent/child demand type,
without changing the invocation's current authority or admitting D grants.

Add a public ModelMember discovery rooted at the still-live model. A root First
publication remains healthy while its discovered Downstream member is edited
5->6 (without settling), then truly retired with its publication. Selecting the
pre-delete step must read member6 at D although absent C, use the actual child
fallback and publish600, with one child contact and no root contact. Include
worker placement, current-permission refusal via the restricted-owned family,
and a stale-root leg to exercise the genuine settled-parent path. The computed
member set and actual output IDs/values are independent observations. Then
qualify all history courts and scope guards/mutations. Corrupt/changed-meaning
and healthy-child verification remain separately unfinished.

The child-source cutover now uses a private borrowed parent sum in parent_query:
settled custody and verified custody remain distinct, with no conversion. Both
owned-graph entrances execute the actual intent through SourceQueryReaderAccess;
parent/basis validation follows fresh C admission and precedes D projection.
The SDK retains the original control-plan path for ScopeEntityRecord. Invocation
family diagnostics derive from the installed typed child demand, while all
current request, principal and selection guards remain unchanged.

The new generic child court initially passed six serial/worker cases, then the
complete history family passed171 courts. Its discovery fixture is being
strengthened to the existing ExactlyOne DTO with a nested Many member collection:
this matches the installed connection contract and computes empty as well as
populated sets. Redo to the deleted snapshot must discover the empty set without
opening a healthy model root. Permission-denied twins also observe zero protected
source projections, baseline C scope refusal, no contacts and no adoption.
Mutation qualification and final pristine rerun remain outstanding for this slice.

Child-source checkpoint qualification is complete: all171 public history courts
pass after exact mutation restoration; the nested member fixture's six serial /
worker cases include populated fallback, empty redo, genuine parent fallback and
C-denied disclosure. Forcing C-only source scope resolution fails four courts;
actually taking D grants fails both permission courts. The repeated D-grants
case first establishes unchanged selection, zero source projections and zero
contacts, then rejects the later StaleAuthorization fence in place of the
required early C scope refusal. Each edited source restores to the same SHA-256.
Boundary/context, formatting, dirty caps and the primary composition review pass.
No independent review or complete-workspace equivalence is claimed.

### D1214 remaining parent courts: cold lineage and changed meaning

Boundary review: a selected program revision resolves through the existing
branch program owner, while implementation reuse is checked by the currentness
owner against ApplicationBuildIdentity. Dependent connection declarations carry
portable feature/port/connection meaning; their compiled discovery and demand
functions belong to the implementation build. The old and new implementations
must never share a build identity. Native roster rows cannot substitute for
running the current installed discovery.

Use the existing generic fixture/checkpoint recorder and parameterize its cold
restore constructor by the actual program type. Keep the same schema and pure
program declaration for an old versus new discovery implementation, with distinct
build identities; assert the portable revisions match. The old tree discovers
Second(a) from First(a). The new connection runs the actual nested ModelMember
query and discovers Downstream(m). Refresh only Second to the current build
before attaching history. Selecting the earlier model value must run the stale
root's actual Preserve producer and the newly discovered child Initial producer,
retain the old Second output and produce Downstream500. An old roster pretending
to be discovery cannot supply that child. Include worker placement and original
identity/value assertions. This is an existing changed-build fallback court,
not a second saved discovery artifact or a new recording format.

For corruption, capture the same non-leaf world through the real recorder, alter
only the first parent's Query role generation with the existing closed checkpoint
body door, and preserve Native's actual publication. Cold restore may retain the
descriptive row, but selection must refuse its wrong parent lineage with typed
publication correspondence, the true parent source/family, no contacts and no
selection/data effect. The existing binding between Query lineage and Native
publication remains the sole verifier. Register new door callers through the
script, run both parent scenarios and the complete history family, then continue
healthy-child source avoidance. If the actual cold program/build premise fails,
investigate its owning contract before implementing another authority path.

The four cold-parent serial/worker courts pass. The new implementation preserves
the portable program revision, refreshes only Second, then navigation invokes
First Preserve plus the current discovery's new Downstream Initial. The new
child500 is absent from the old roster; First's original identity/value and
Second's refreshed2003 survive. Corrupt first-parent Query lineage refuses typed
with source/family and no selection change or producer contact.

The single installed-producer build predicate mutant survived honestly: the
canonical lineage reader independently screens noncurrent implementations first.
Removing both build checks compiled and failed both changed-meaning courts
(one producer contact instead of two). Removing Native/Query publication
correspondence compiled and failed both corruption courts by contacting a
producer before refusal. All edits restored to equal SHA-256; the final pristine
history family passes175 courts. Boundary/context, formatting, whitespace and
seven dirty Rust caps pass. Primary composition review accepts the two long
court journeys as explicit setup/action/independent observation; no independent
review was assigned. Healthy-child source avoidance remains the next boundary.

### D1214 healthy dependent verification

Boundary review: the SDK edge currently projects each discovered child's source
and admits a real demand before it can reuse an output. Producer-contact counts
therefore miss a D1119 violation: a healthy child's original source is opened.
The existing SourceQueryReaderAccess can freshly admit the actual installed
source query at C without projecting its result. Its admitted full-generation
scope and admitted parameter partition identify the candidate route; neither an
SDK demand nor a Native roster is read or settlement authority. Native's
selected-route currentness owner must still check lineage, build, facts and
witness at the candidate and again at the final endpoint. Discovery remains the
installed query's computed set. A verified child is distinct from a settlement.

Adversarial constraint: an arbitrarily long discovered set must not re-scan all
selected publications per child, borrow a grant from D, or let a later sibling
producer invalidate a previously verified child unnoticed. Keep one retained
Native selection read per navigation, with a derived route index owned by that
read capsule. The cache contains no grants and changes no checkpoint format.

Implementation: first add serial/worker courts distinguishing a healthy
historical-only member child from its stale fallback, observing source
projection before public assertion queries. Establish the red baseline. Then
share the exact Native selection read through navigation custody and add a
bounded, polled route lookup at Native. Generalize the existing verified-parent
custody to retain its own family, full-generation source, parameter partition
and exact verified candidate separately from the navigation root profile.
Only the existing parent/connection owner may issue a verified child, after
fresh C permission and parent validation, without executing the source query.
A stale route returns to the original read/demand fallback. Retain each typed Query verification custody in its SDK edge, separately from
genuine ProgramOutputRecords; it can start verified discovery but never become
a settlement. Reauthorize closed children at C. The existing complete Native
route-coverage owner already verifies every restored route not covered by a real
settlement at the one final endpoint. Reuse that owner instead of inventing a
second erased completion inventory or redundant child verifier. Preserve ScopeEntityRecord's old path.

Qualification includes the complete public history family, serial/worker
healthy and stale children, C-denied children, nested verified discovery, and
mutation of child verification and final currentness. Run boundary/context,
formatting, dirty line caps and primary structural review. Keep the ongoing
immutable root/Query comparison running. Progressed producer-created source
scopes and codec-retired restoration remain subsequent #57 evidence; this
slice does not claim those cases or overall issue completion.

This slice is qualified: final pristine179 public history and56 Native ownership
courts pass after exact mutation restoration. Ten member-child serial/worker
courts cover healthy children below verified and genuinely regenerated parents,
stale child fallback, C-denied historical-only sources, empty redo and original
identities/values. The source recorder has positive stale-child/root controls.
The Native index courts distinguish family, full-generation source and parameter
partition and refuse cancellation even after the derived index is populated.

Six compiled mutations fail their courts: ignore route source, partition or
family; omit index request polling; reopen a healthy child's source; skip child
currentness. Restoring each edited source yields the identical SHA-256. The final
pristine history and Native runs rebuild the restored sources. Boundary/context,
formatting,36 dirty Rust line caps and whitespace pass. The primary structural
pass accepts explicit authority inputs and real court journeys; no independent
review is claimed. This completes healthy-child source avoidance, not #57 as a
whole. Progressed producer-created source scopes and recording remain next.

### Progressed producer-created source: boundary and implementation plan

The healthy-child checkpoint is b81bf6f683. The next adversarial case is a real
recorded edit whose outputs did not exist at the preceding step. Selecting the
preceding step retires those outputs; the root's actual Initial fallback creates
a new full-generation output with the same business key. That output becomes a
child producer's source. C's same key may still resolve to the retired future's
different generation. Resolving only at C is insufficient; checking presence
only in the original restored D0 is also insufficient for the actual progressed
private D1. Neither condition permits taking a grant from D.

Native already owns exact-basis declared-relation and current-publication custody
traversal in branch/model_ownership. Its read and candidate paths share the same
owner algorithm. Historical source presence currently checks only the sealed
restoration capsule's D0. Query's admitted historical access retains the actual
truth product/root/snapshot and performs an exact truth freshness check after C
authorization. Extend that existing handoff rather than inventing an ownership
reader or treating matching business keys / branch incarnation as identity.
Transferred selected members must remain anchors to their own selected subset;
a transferred member must not authorize all other records of its B document.

First create a generic public fixture using the actual fixed-action history
recorder, with a root Model producer and a child whose source is FirstOutput.
Attach before any outputs exist, record a Model3-to5 edit with its real tree,
then select the pre-edit step. Assert the root's fresh identity differs from the
future's source, the child derives from value30 rather than50, and redo restores
the future's original identities without contacts. Run serial and worker. A
protected child source must keep C permission and no unauthorized projection or
selection effect. Establish the real refusal before changing production.

Then repair the exact owners exposed by that court. Source scope admission must
use the candidate's actual full generation when C's selector finds a different
one, while preserving the existing C path for the same generation and retaining
nondisclosing C refusals. Native must validate liveness and declared/custody
membership at the exact progressed basis through its ownership traversal and
sealed selected boundary. Query may pass only the candidate selected inside
this atomic qualification; it never adopts D first. Add retired-at-progressed-D,
foreign-boundary and worker evidence and mutation of C-only identity, D grants
and omitted progressed liveness. Requalify public history, Native ownership,
facades, formatting and scoped structural laws. No new format, alternate
producer engine, CAD type or saved discovery artifact is involved.

The real fixed edit now records both outputs successfully. The first valid red
run (d1214-generated-source-red-3.log) stops earlier than source admission:
First Initial is given the future's warm Query publication as predecessor,
although Native's selected route is HistoricalAbsence. Native correctly refuses
`predecessor is not the publication selected at the native before image`.
The existing Native readmit_required_output_route_absence already issues exact
absence custody for HistoricalAbsence and the existing Birth posture consumes
it. Query required_origin::bind currently skips that owner whenever a warm head
exists. For owned-history producer cutover, consult the actual Native absence
owner even with a warm Query head and let its proof select Birth. A positive
Native route keeps ordinary succession. ScopeEntityRecord and ordinary producer
entrances retain their existing ordering. This is a prerequisite owner repair,
not a new origin or relaxation of predecessor validation. The same public
serial/worker court will qualify it before exposing the next admission refusal.

The actual public journey exposed both generation-collision owners: Query source
intent admission and the producer's mutation-scope resolution. Each now resolves
the private truth generation only under the actual owned-navigation posture.
The same full generation at C uses its old path. C supplies all permissions; D
only supplies the exact source and liveness. Source presence now consumes that
actual progressed candidate through Native's existing ownership traversal and
the sealed selection boundary. The selected transferred member is an anchor;
other members of its outside owner are not. Retiring a selected member at D2
does not inherit D0 liveness, and retaining D1 never reads today's D2 instead.
Native's source-read denial is separate from general selection/currentness, so
this posture does not broaden unrelated reuse. Nested custody execution stops
retain their actual control cause when Query translates source admission.

The public fixture records a genuine fixed edit with root and child output
publications, selects its pre-output step, derives3000 from a newly created
First30 rather than C's First50 generation, and redoes to the original recorded
First50 and child5000 identities with zero contacts. Its protected version
refuses before protected projection or new-generation diagnostics and leaves
the document selected at C. Both run in serial and worker-only modes. Native
serial/worker courts prove selected transferred-subset membership, exact D1/D2
liveness, a wrong kind, an older basis and bounded request refusal. The initial
pristine runs pass183 public history and58 Native ownership courts.

Seven compiled mutations fail their courts and restore every edited byte to
the same SHA-256: prefer a warm Query head over Native absence, resolve Query
identity only at C, resolve producer mutation identity only at C, actually read
grants at D, reuse selected D0 liveness at retired D2, admit every other B member,
and admit a basis older than the atomic restoration. The JSON ledger is
`build-cache/history53-d1207/d1214-progressed-source-mutations-results.json`.
An additional purpose court distinguishes parent discovery from child source
when both use the identical installed query binding. Final pristine reruns and
scoped checks remain pending for this checkpoint.

The shared-binding purpose courts pass in serial and worker-only modes. Removing
the explicit purpose guard compiles and fails both by misnaming discovery as the
child family (`restricted-from-first` instead of `first`). The source is restored
to SHA-256 `82d7983404242dc1a4720f0a551a73dede5b9bc75987755c69d241807dfdb4bf`.

The checkpoint's final pristine run passes192 history-related public courts
(185 certification plus7 cost courts), with2 intentionally ignored cost cases;
Native ownership passes58. Boundary/context, rustfmt,45 dirty Rust line caps and
whitespace pass. Primary structural review keeps the Native traversal owner,
separate source denial, explicit query purpose and authentic continuation
custody; no independent review is claimed. The D-grants mutant is caught at
the required early C admission boundary; a later stale-authorization fence still
refuses the mutated read, so it does not demonstrate an unauthorized success.
The precise JSON and logs retain that distinction. The private-source stopped
permission diagnostic scrub remains a targeted follow-up court before overall
handoff; no completed issue or final workspace qualification is claimed here.

### #57 multi-tree edit recording: boundary review

Progressed-source checkpoint01dc5afe96 is merged with master8a04cd1349 at
0021982c4d. The next incomplete obligation is recording all configured trees,
not only verifying their restored versions on navigation. SDK edit sealing
currently accepts one source receipt or one actual fixed-action complete-set
proof. The configured runtime-length forest participates only in navigation.
A source-only edit can therefore be recorded while its other configured trees
are stale. Settling them afterwards moves the head outside history, which is
correctly refused. Repeating the edit solely to run another root is not a
solution.

The existing owners suffice: SDK request/program_outputs starts real original
read trees at an explicitly retained source observation; ProgramOutputHandle
owns demand progression, dependent discovery and original cleanup. Query's
OriginalOutputTree and qualify_output_collection derive one endpoint from
actual publication movements and qualify every whole tree there. Neither a
roster, a latest-head read nor an ordinal can substitute for that endpoint.
History reservation/sealing owns the source receipt, original candidate and
single atomic adoption. These owners must compose without manufacturing a
navigation authority, a settlement, a root fulfillment or a second producer
engine.

The configured collection is explicit opt-in for OwnedModelGraph. NoOutputs and
single-tree ScopeEntityRecord edits retain their existing behavior. Each new
recording tree keeps its original source, genuine settlements, computed
discovery and disposal custody. Failure after one family runs keeps the whole
candidate private. A later tree can invalidate an earlier one; final
qualification must expose that, rather than accepting per-tree intermediate
completions. Grouping retains one atomic adoption and the current redo policy.

### #57 multi-tree edit recording: implementation plan

1. Add a sealed read-only recording origin at the existing Query history
   reservation/sealing owner. It validates the actual receipt (and original
   required-output completion for a fixed action), candidate and installed
   collection, retaining the exact origin observation before extra tree work.
   It grants no read, producer, mutation or navigation permission.
2. Extend the SDK forest's compiled typed dispatch to start ordinary original
   program trees from that same origin, retaining their real handles inside the
   move-only edit. Reuse ordinary program output progression and cleanup; keep
   recording custody distinct from navigation slots and verified parents.
3. Retain each complete original tree through Query's existing collection
   owner. Derive one endpoint from actual movements and freshly qualify every
   tree there. Fixed actions additionally requalify their original declared
   complete set at that endpoint. The source receipt remains the source action,
   never an extra root producer receipt.
4. Add a recording completion accepted by the existing atomic history seal only
   for that reservation, candidate, installed profile and original receipt. It
   must prove the candidate's exact final endpoint. Adoption records one step.
   Every refusal keeps original tree/operation/candidate disposal custody.
5. Prove source-only and fixed edits with two distinct configured roots, serial
   and worker-only; undo/redo must select the recorded outputs with zero
   contacts. Prove partial refusal/disposal, a later tree invalidating an earlier
   tree, missing-tree completion and a wrong source/candidate. Mutate omission
   of one tree, intermediate-only qualification and endpoint/source checks;
   restore every mutation by SHA-256. Extend the same composition to grouped
   recording before declaring #57 complete.

Use the existing court layout and caller script, <=400 lines per code file,
request-aware checkpoints, public generic Model/Member fixtures, focused and
full history/Native tests, boundary/context, formatting and primary structural
review. No persistence format, new test framework or application-supplied
completion callback is introduced. Codec-retired disposition selection remains
next; #55/#53 acceptance and #56 correction history follow.

The source-only two-root red court now fails in serial and worker-only modes:
the recorded step still contains10/1001 after Model changes to5, rather than
50/1005. This is a valid behavioral red, not a fixture installation failure.

Recording completeness also needs Native's independent current obligations.
The configured collection alone cannot prove that a model has no other stale
published family. Native's branch/model_ownership owner already derives the
exact closure and reads current publication custody from touched records; add a
readonly publication-obligation projection there, using that same traversal and
publication roster reader, at the final candidate. Preserve the distinction
between published origins and typed codec-retired closures. Query checks every
live route through the existing selected_route_is_current/require_current_read
owner, including unconfigured families. A still codec-retired obligation cannot
pretend to be a live role; it must be rebuilt by its actual configured producer
or refuse precisely. Add an omitted-family recording court and mutation of this
complete Native obligation pass. No alternate ownership or currentness reader
is permitted.

The implementation now keeps a distinct recording qualification denial, rather
than adding recording-only cases to ordinary read-output qualification. Roots
start sequentially. Their common initial origin is provenance for deriving the
final endpoint; it is not a frozen data snapshot for every later root. Each
root reads the then-current private candidate. The recording owner checks that
exact data observation against the same candidate before admitting producer
work, and compares the original full source description and canonical parameter
artifact. Its actual query, native source facts and cleanup remain ordinary SDK
custody. All publication movements must still connect to the shared origin,
and all complete sets are checked at their one derived endpoint. No private
candidate is adopted before that qualification.

A compile qualification caught a causal regression: storing an erased forest
inside every Selecting edit made existing NoOutputs edits lose Send, failing
three existing threaded courts. It is not master debt. The actual mutable
recording forest now lives on its opted-in WithOutputTrees receiver, alongside
that receiver's existing original navigation owners. The edit borrows it only
inside the sealed SDK callback. NoOutputs and the single-tree mode create no
recording state or extra cleanup advancement. Successful adoption drops the
already drained readonly recording frame; refusal and disposal retain the same
receiver's original owners. This preserves ordinary edit auto-traits rather
than imposing Send on program continuations or weakening the threaded courts.

The first integrated recording run reached the independent obligation pass and
refused ordinary fresh publications: the selected-route reader deliberately
accepts only HistoricalSelection. Preserve that restriction. Add a distinct
current-route entry at the same lineage/currentness owner, accepting only a
sealed borrowed Native roster coordinate from the exact final candidate. Both
entries share the existing canonical lineage lookup and require_current_read;
no settlement fabrication, duplicated verifier or roster-as-discovery shortcut.
The recording court must pass through this ordinary currentness verification,
including a missing-family negative and a skip-verification mutation.

### #57 grouped recording: boundary review (next slice)

Native's group owner already authenticates each ordinary member's full intent,
operation scope, candidate and exact movement from its predecessor. It retains
the actual captured endpoint and completion posture per member. Final adoption
currently selects the last captured endpoint; it has no output-forest endpoint.
SDK group storage retains each concrete ordinary source/fixed completion behind
its closed driver. The last member's driver freshly admits the adoption at C.
Additional original trees and cleanup remain on the opted-in document receiver.
No group member may be relabelled as an output receipt or synthetic settlement.

The collection's original receipt for this extension is the last real member's
completion, after Native has authenticated the entire captured member chain.
Earlier members can intentionally supersede each other's output publications;
the final Native obligation inventory and original configured trees must prove
the final current graph. A group reservation must not masquerade as an ordinary
single-operation reservation: their intent and scope correspondence differ.
Native's group owner must mint the recording origin from its own completed
chain and consume the final completion at that same owner. Single-record and
NoOutputs group transitions retain their existing behavior.

### #57 grouped recording: implementation plan (next slice)

1. Add origin capture at Native's existing group owner after exact complete-chain,
   last receipt, captured endpoint and candidate validation. Share the immutable
   recording origin constructor; do not call the single-operation intent gate
   with a fabricated reservation. Retain the same Node/expected/candidate/source
   provenance used by the isolated edit's completed forest proof.
2. Let the SDK closed member driver supply its actual final source completion
   and freshly requalify fixed delivery at the forest's derived final endpoint.
   Extend Native group adoption to consume that proof and select its exact E;
   keep group journal, redo and idempotency ownership unchanged.
3. Before ReadyToComplete, opted-in grouped documents progress the same original
   recording trees. Pending work has an explicit qualifying posture. NoOutputs
   and single-tree groups keep their existing transition path. Complete adopts
   once only after the original owners are drained and qualification is sealed.
4. Abort closes the actual additional forest owners before retiring the private
   candidate, preserving failed cleanup custody on retry. A refusal or stop
   cannot publish a partial group or release its reserved journal edge early.
5. Public serial/worker courts: homogeneous and heterogeneous gestures record
   both families at final E in one step; contact-free selection of those versions;
   a later output invalidating an earlier result refuses atomically; partial
   extra-tree cleanup preserves the selected document. Mutate omitted final
   qualification and group/receipt/candidate correspondence; exact SHA restores.

The isolated-edit slice is first qualified and committed before this cutover.
Codec-retired disposition restoration follows it; remaining issue qualification
and #56 are not declared complete by either recording slice.

### Recording qualification follow-up: retained and removed roots

The first pristine204-court run found8 causal failures (196pass): two genuine
forked historical-demand index collisions, two attempts to open a root whose
source the same recorded cascade retired, and four changed redo contact premises
because recording now refreshes its configured non-leaf root. These are not
master debt. Hold the isolated recording checkpoint until they are resolved.

A configured tree is immutable correspondence, not an instruction to recreate
an entity retired by the action. The recording owner must use Native's existing
exact-basis ownership reader before opening a root. A source no longer owned by
this document has a distinct retained omission posture; a current healthy leaf
has a distinct verified-current posture. Neither is an OriginalOutputTree or a
settlement. Complete forests may consist entirely of these postures. Derive E
from the original action endpoint and only authentic original-tree movements,
then verify every actual original tree and every Native current obligation atE.
Recheck omitted-domain evidence there. Non-leaf live roots still run real
installed discovery through their original owners; the roster cannot discover
children. A stale or missing publication on an owned live source runs its real
producer. No source scope is recreated by configuration.

Implement these distinct recording postures at recording_outputs, retaining the
existing original endpoint derivation and currentness owners. The SDK asks for a
verified/omitted entry before opening an original root. Add worker/public
retirement and all-verified recording evidence. Preserve the previous historical
scope navigation courts by retaining their original pre-delete source/profile;
for model-root navigation-only courts, configure after deletion so their stale
pre-delete and redo snapshots remain the actual subject. Do not alter expected
contacts without preserving the producer-fallback premise. Investigate the
historical fork posting owner as needed for live non-leaf roots; never weaken
exact address or occurrence checks to work around its collision.

### Non-leaf recording after checkout: boundary audit and next proof

Checkpointc4a5f1d685 qualifies isolated forests, current leaves and removed
sources (206history/61Native;9mutants), not every live historical-parent fork.
A navigation's verified-parent owner is sealed to its actual navigation
qualification and currentC grants. Recording has its own original action origin
and ordinary SDK root/cleanup; it must not fabricate a navigation frame or an
ordinary settlement. The demand registry's exact posting index remains single
owner per canonical settlement address, including original branch incarnation.

First add a public serial/worker court using the real installed non-leaf tree:
settle it, record a stale source-only future, undo, rejoin the restored tree atC,
then record an unrelated owned-member edit on its new private candidateD. The
healthy parent must preserve original identities, run actual discovery of the
expected children and make zero producer contacts; final source/member/output
reads independently check the recorded step. Observe the real failure before
choosing the owner cutover. Preserve exact posting, occurrence and authority
checks. If the inherited historical row cannot be accepted atD, the real
current-read/parent owner must carry that acceptance; no fake settlement,
parallel ownership reader or roster-derived dependent set is acceptable.

### D1248 master cutover prerequisite

Masterdb554ad361 replaces parallel entity/aspect/field declarations with native
Fragment/Entity/Aspect/Record membership and FieldSelection. Merge54b9a10124
keeps installed history ownership and the borrowed selector resolution, with
scalar validation from master. The remaining borrowed issued-scope method must
consume RecordSelection, not removed ApplicationFieldRef. Branch-only public
owned-model/output fixtures must declare their actual row shapes and fragment
members and use the canonical selectors in scope/query/access bindings. Preserve
semantic names, grants, declared ownership edges, operation output roles and all
existing test oracles; do not add old macro wrappers or a second schema lane.
Compile the focused parent court and existing history suite on that substrate
before addressing its known historical posting collision. This merge prerequisite
is causal; no unrelated native schema redesign belongs to this lane.

### D1248 recording-parent integration plan

The public non-leaf court exposes a reused historicalC posting being reserved
again for a newD demand. Fixing the posting alone would still open the healthy
root source. The existing verified-parent continuation is the right cutover.

At the existing output-control owner, add an explicit recording posture issued
only after fresh current documentREAD and the actual recording operation's
admission/reservation are checked. It retainsC control, the private recordingD
and revocable callback/selection lifetimes. It is not a navigation admission;
NAV is not invented or required for recording. Source identity/domain atD uses
the same installed Native model ownership reader; all authority remainsC.
The existing verified-parent issuer supports that actual recording posture by
checking the currentD publication route/lineage/witness/installed meaning through
the canonical currentness owner. Navigation keeps its selected-version evidence.

Share actual discovery and child traversal through the existing verified parent
and ProgramOutputContinuation owners. Retain the distinct verified-tree evidence;
never convert it into an original root settlement. Derive finalE from the action
origin and all genuine child/publication movements, then requalify every current
parent, discovery dependency and complete child set atE. Native's independent
current obligations are still checked. Stale parents take their real original
producer path under that same current control, preserving source and disposal.

Implement in order: explicit recording control/context at output_authority;
canonical current-route verification entry shared by parent and child issuer;
verified continuation in recording tree owner and final recording collection
qualification. Extend public court with stale parent, discovery changes and
current-permission refusal as needed; mutations skip discovery, parent facts,
Cpermission and final basis must fail. Keep existing navigation courts unchanged
and all single-record paths byte-for-byte behavior. No changes to settlement
address admission or posting ownership are authorized by this plan.

### Recording boundary refinement from the full-suite counterexample

C in the historical-parent posture is the document's selected head. It is not
an invented replacement for the ordinary program's current basis while that
program completes an actual authored action. The generated-source courts expose
the distinction: an opted-in fixed action creates FirstOutput and its protected
child together on its real ordinary candidate. Re-running that already completed
action's discovery as a historical C read incorrectly refuses its new scope as
UnknownEntity. Preserve the actual ordinary completion posture, including its
source-artifact check, original discovery and disposal; never substitute a
verified historical parent for that genuine completed action. The origin carries
whether Native qualified such an original required-output completion. Its owner
chooses that posture before attempting historical-parent admission. This is not
D-grant fallback after a failed historical permission.

Verified recording parents with no completed ordinary required-output action
still use C for every discovery/dependent permission and the exact private
candidate for data, via the same Query parent and Native ownership/currentness
owners. Stale recording roots continue the existing genuine ordinary producer
path. Navigation's historical-source and parent paths remain exclusively C
controlled, including every actual producer fallback. Do not broaden navigation's
historical admission into ordinary program execution or silently change the
already qualified fixed-action behavior.

The recording C-denial court must change an independent grant fact while keeping
the parent's actual source facts current. Changing ModelValue changes both the
grant and the producer source, so it tests real ordinary stale fallback instead.
Use a separate installed discovery permission depending on the owned member's
value: C denies at7, the candidate grants at5, and Model3/First30 remain healthy.
Assert zero producer/source contacts, zero discovery disclosure, a typed current
permission refusal and no adoption. Keep the existing stale-parent public read
and actual-contact court as the separate ordinary fallback proof.

### D1248 stop: opaque non-leaf discovery input is not frozen by source intent

At checkpoint82de9fa66c, the new verified recording path passes214history courts
and61Native ownership courts; six compiled mutants fail and restore by SHA256.
The next public counterexample exposes a separate input boundary at
`application_installation/program/demand/history/{dependent,discovery,parent_query}`
and the SDK `ProgramOutputContinuationFactory::start_verified`.

The SDK retains an arbitrary Clone-only RootDemand. A source intent captures the
root query's full-generation selector and complete canonical parameters, not all
other fields of that demand. The legal generic court demand has an immutable
source `a` and a separate shared discovery choice. Retain the A profile while the
choice isA, then switch only discovery toB. Both fields of the complete root
source query remain `a`. A's parent publication verifies correctly. The installed
discovery then reads FirstB, discovers SecondB, runs its genuinely authorized
producer once and A's recording adopts the private candidate: SecondB changes
1099→1007 at its original ID. Both serial and worker courts are red with that
actual adoption; no fake parent/settlement or permission is used. B's own source
was already7 atC, so C authorization is real. This is the frozen graph-input and
owned-history boundary, not evidence of forged permission.

Checking the root source intent again is insufficient: that entire input never
changed. A Clone bound cannot establish immutable discovery inputs. Reopening a
healthy original source to reconstruct an original root violatesD1119/D1214.
Do not silently add a domain rule for every discovery read: an installed read
can legitimately follow an outgoing reference to authorized foreign material.
Do not invent step-authenticated discovery evidence or claim the Native roster
is the computed set. Stop for a representation ruling under the standing rule.

Options sent to root:
1. Freeze canonical discovery request recipes alongside the existing source
   intent at configuration/child admission. This is immutable request data, not
   authority or a dependent-set assertion. Execute real discovery atD and freshly
   admit every C permission/request fact. Do not require a not-yet-live discovery
   scope to exist at configurationC; its selector is data until actual admission.
2. Give output demands a declared canonical immutable input binding. Capture the
   complete demand value at retention and reconstruct it for the existing
   installed discovery mapping. This changes the public demand contract; a
   Clone-only opaque carrier cannot claim the posture.

No option is implemented. The two red courts remain enabled and committed as
an explicit incomplete checkpoint. Full root/Query comparison of immutable
82de9fa66c against clean masterdb554ad361 continues independently; it excludes
these later counterexample courts and cannot certify the unresolved boundary.

Ruling-question clarification: the case does not demonstrate unauthorized work,
false ownership or a change to B's authored fields. The source-scope producer
forB has real current permission and retains B's custody. It may be legitimate
incidental derived maintenance under the existing live-input contract. The
unresolved question is whether retention promises to freeze all graph input
fields beyond the expressly captured source intent. Do not impose that extra
contract or a blanket ban on authorized foreign reads/producer work by inference.
The amended root question therefore includes keeping live discovery inputs and
authorized derived work, as well as the two freezing representations above.
The enabled negative courts characterize the proposed freeze/refusal contract;
their expected refusal is provisional pending that ruling. If live inputs are
intended, correct those oracles to the observed authorized maintenance and keep
production behavior. No unsupported security or data-loss conclusion follows
from their present reds.

### D1255 boundary review and implementation plan: recorded discovery recipes

D1255 supersedes the D1248 representation stop. The adversary is a Clone-only
root demand whose source intent stays identical while a shared discovery input
changes after retention. Navigation must compute the selected step's dependent
set, even after that change, without opening a healthy original root.

Query's source-intent owner already owns canonical parameters and scope selector
data without grants. Extend that boundary with canonical discovery request data:
the installed query identity, normalized parameter basis, and symbolic principal
or encoded field selector. No opaque demand, projection result, dependent set,
permission, or request principal is retained as recipe authority. The existing
source-reader admission remains the only C principal/permission/request owner;
it consumes either a live request or a recorded normalized request. The existing
parent owner verifies exact D custody before running real discovery. SDK factories
transport recipes but cannot mint them or substitute roster membership.

The journal's completed source binding owns each step's immutable recipes. A
recording gathers actual canonical discovery requests under its reserved node;
completion seals them with that source endpoint. Initial enrollment captures only
the original current endpoint, never an older selected step. Navigation borrows
that selected binding; missing, corrupt or foreign recipes refuse typed naming
the selected step. Historical source/parent admission and current installed
meaning retain their existing controls and changed-build fallback. ScopeEntityRecord
continues its existing path. Native CHK18 does not change for these Query annotations.

Implement in dependency order:
1. Normalize ordinary query scopes without evaluating a principal, and capture
   funded immutable canonical discovery requests under the Query source-intent
   owner. Add the frozen-input posture to the existing source reader, sharing
   its C/D scope resolution and permission owner with live reads.
2. Carry a recording/selected recipe posture through history output authority;
   seal recording recipes into completed journal bindings and previews. Capture
   initial root-edge requests through existing typed continuation factories.
3. Cut owned-graph historical discovery over to recorded requests. Actual new
   recording reads live requests once; navigation never calls the mutable demand
   mapping to fill a missing recipe. Preserve actual ordinary fixed-action custody.
4. Replace the provisional negative counterexample with serial/worker navigation
   round trips asserting the D dependent set, no B adoption and zero root contacts;
   add missing/corrupt/foreign recipes and current-permission no-effect courts.
   Kill the live-input, skipped-integrity and D-grant mutants, restore by SHA256.
5. Run focused history/Native courts, formatting, dirty line caps, boundary and
   agent-context checks. Continue grouped recording and codec-retired selection,
   then remaining issue courts and full root/Query comparisons against clean
   current master before any issue READY.

No new demand binding or serialization obligation is imposed on applications.
The normalized recipe is input, never a replacement discovery authority.

D1255 qualification refinement: selected recipe data is never reconstructed for a
bare step that predates output enrollment. The full history run correctly exposes
22 older fixture legs that recorded their selected steps before configuring the
non-leaf output mode; their missing recipes are not master debt. Migrate those
journeys to record source intent and canonical requests before the selected steps,
while preserving the independently disputed stale-parent, historical-only child
and current-grant conditions. Do not backfill their old nodes or weaken the new
missing-recipe fence to recover an old green count. The new focused recipe and
current-permission courts pass independently before those fixture migrations.

D1255 fixture migration preserves genuine stale-publication navigation by making
ordinary source-only prerequisite edits before attaching the document. The
initial retained snapshot can therefore contain legitimately stale producer
observations, without forging facts or retroactively supplying missing requests.
Enroll its source intent/recipes before recording the future. The configured
future completes its outputs; redo of that future is now contact-free. Cold
changed-build recording must also configure the independently stale second-family
seed tree, so final Native obligations are satisfied. The old grant denial coupled
to a producer's source value is replaced by the new independent-grant navigation
court: the parent's source stays healthy while C denies and D would grant.

Recipe map slots now use HistoryInsertionAllocation, the existing journal-funded
B-tree growth owner. Immutable request data can be shared, but each copied index
owns its own insertion grants. Recipes live with RetainedHistorySource, not the
permanent annotation. Exclusion releases them together; a pending navigation's
preview carries its original set back through fill_navigation_source. RecordNode
keeps its original annotation-only shape. These lifecycle/resource corrections
require pristine and mutation requalification before the checkpoint is certified.
