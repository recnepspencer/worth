# 9.18.1 implementation

## Phase 1, slice 1: installed invariant read observations

Boundary review: retained snapshot projections already expose
`WorthQueryInvariantProjectionWork` through the host facade and its snapshot.
Installed rules instead use `WorthQueryApplicationInvariantReadView`, which
currently records no Query read observations. Relational continues to own
proposed-state access, declared access, prepared scope, interruption and commit
validation. Query observes its actual reads without granting additional access
or changing existing work budgets. Relation filtering happens after inspection;
discarded relation kinds must still count.

Adversarial constraint: repeated proposed/committed view construction, a filtered
edge or a failing field decode must not erase reads, manufacture a successful
evaluation or report a result-length estimate as work performed.

Plan:

1. Retain the existing public counter representation. Add observation storage
   under `application_invariant/read_observation.rs`, owned by each preparation
   or evaluation and borrowed by its read views. `planner.reads()` reports
   preparation; `context.reads()` includes that preparation and this evaluation.
2. Record adjacency acquisition, relation inspections, entity reads and field
   reads where they occur. Keep authority checks before reads and preserve all
   existing denial behavior. Extract field access from the existing oversized
   typed-view file into its own child responsibility.
3. Exercise observations through the existing real commit-boundary rule fixture,
   including preparation, repeated views, domain refusal and cancellation.
   Expose the common type in the invariant facade for compiled callers.
4. Run focused owner tests, formatting, dirty line caps/function inspection,
   boundary-check and agent-context. The persistent reviewer reviews the living
   diff and results before the slice commit.

No new budgets, admission accounting, evaluation engine or raw application
handles. Declared-query receipt unification, planar equality/growth evidence,
the full public guide and Proprietary adoption follow in subsequent Phase 1
slices. The phase cannot merge to master before those obligations close.

The next slice also separates snapshot projection endpoint reservations from
actual endpoint reads. Existing `traversal.rs` records reserved result capacity
before acquiring endpoints; the new installed-rule observations count acquired
metadata. Preserve the existing projection budget accounting while repairing
that observation boundary. This is a known Phase 1 obligation, not a claim that
the two existing paths already measure the same thing.

Interpretation: replace the specification's exact count golden with equivalent
read equality and meaningful locality/growth checks. MENTALITY section 7 and
the performance laws explicitly reject exact counts as performance contracts.
Counts describe reads, not a promise that lawful implementation changes preserve
an incidental number.

## Phase 1, slice 2: snapshot endpoint observations

Boundary review: snapshot traversal and body collection record native endpoint
reservation counts before acquiring endpoint metadata. Aggregate scans also
pass reservation counts to the same observation method despite performing no
separate endpoint lookup. Decision adjacency facts derive their recomparison
allowance from these public observations. Native adjacency reads, the existing
private projection budget and retained decision facts keep their authority.

Adversarial constraint: a native adjacency scan may reserve records and stop
before any endpoint lookup. A later endpoint read may acquire metadata and
reject its kind. Observations must distinguish those cases without changing
which ordinary reads or decision rechecks fit the existing budget.

Plan:

1. Remove the endpoint-reservation argument from `record_adjacency`, including
   aggregate and body-collection callers. Keep native budget consumption as is.
2. Record endpoint metadata only inside the native `endpoint` acquisition
   callback. Count acquired but invalid metadata; count nothing when no callback
   ran. Remove the body-collection anchor's duplicate pre-recording.
3. Move adjacency recomparison demand to the private budget's consumed-work
   delta, preserving its extra final check and minimum allowance. Public read
   observations no longer select that allowance.
4. Add real snapshot tests for partial scans, endpoint rejection and pre-read
   cancellation; retain the lawful counterpart. Exercise existing decision
   adjacency/currentness, body requirements and aggregate tests, plus the
   affected grouped public certification families. Check formatting, dirty
   composition/size/function advisories, boundary-check and agent-context.
5. The persistent reviewer checks the living patch and final results before
   this slice's commit. No master merge until all Phase 1 work is complete.

Destination topology stays under `invariant_projection`: `traversal` owns
endpoint observations, `work` owns the existing private budget, and aggregate
accounting consumes native charges without inventing endpoint observations.
Tests stay at their owning operation. No additional budget, traversal engine,
public metric type or exported test door is introduced.

Snapshot field preflight observations, declared receipt unification, public
examples/differentials and Proprietary adoption remain later Phase 1 work.
Managed computation range carriage also copies historical read observations
into a current run (`computation_calls/ranges.rs`). The shared observation slice
must keep actual read observations separate from the existing private carried
budget charges; historic reads must not masquerade as current acquisitions.

## Phase 1, slice 3: shared actual-read observations

Boundary review: field projections and aggregates record field admission before
native acquisition. Managed call charges retain diagnostic read counts, replay
them on reuse and rewind them on fallback. Their private demand/consumption
already carry the genuine budget contract independently. Declared query
receipts expose native adjacency observations but call preflight projection
capacity a field count. Native frontier guards scan records, not indexes;
they need acquisition-site field observations, not an inferred index count.

Adversarial constraint: reuse must not claim historical reads happened again;
fallback must not erase reads already performed. Optional absence and decoding
failure still read a field, whereas a refused acquisition does not. A new
receipt observable must preserve existing disclosure noninterference in every
receipt lane.

Plan:

1. Move the existing public observation type to the primary graph's shared
   `read_observation` owner, preserving its facade export. Keep the private
   projection budget in `invariant_projection/work`. Remove diagnostic data
   from managed call charge/start/summary state; preserve private charge,
   scope and fact carriage. Restart only the private budget, not observations.
2. Count snapshot/predecode/aggregate fields at native acquisition, including
   absent optional values and failed decoding. Retain original budget admission
   and native request controls. Add actual field observations to both native
   frontier guard implementations without changing their work charges.
3. Carry actual root/tree reads through RawOneShotRows and all receipt
   constructors: one-shot, continuation and live refresh. Expose `reads()` using
   the common type; migrate obsolete adjacency aliases to its canonical names.
   Preserve authorization and non-read work evidence independently.
4. Extend the existing disclosure receipt comparison, real managed reuse and
   fallback tests, and field/receipt regressions. Run focused owner evidence and
   affected grouped public targets, formatting, dirty structural checks,
   boundary-check and agent-context. The persistent reviewer gates the living
   patch and final results before commit.

No additional budget, public counter type, observation-based admission or view
engine. Public compiled planar examples, the feature guide and Proprietary
adoption still gate Phase 1 closure and its merge to master.

## Phase 1, slice 4: public planar evidence and read guide

Boundary review: the installed PositiveTurnRule prepares predecessor discovery
and then two outgoing successor hops per distinct affected root. Evaluation
reads coordinates from those prepared triples. PlanarQuery reads two outgoing
successor hops from its named root. Their whole counts describe different work;
only the same-root forward interval is an equivalent adjacency read. Installed
context observations include preparation, and survive ordinary domain refusal.
Retained snapshot projection readers are a separate public host read surface.

The existing TopologyConfiguration -> setup.invariant -> resolve_rule path can
carry an optional application-owned observer. Event envelopes hold the shared
Work snapshots and genuine EntityId, not a second counter or access authority.
Bootstrap also invokes rules, so the example clears its seed observations and
correlates one explicit edit's directly touched body before comparing reads.

Plan:

1. Add the optional ordinary observer to the topology configuration; existing
   callers supply None. Split the touched planar invariant into binding/trait,
   preparation and evaluation responsibilities. Record raw preparation,
   forward-hop and evaluation snapshots at their actual boundaries, including
   preparation/evaluation errors and ordinary violations. Never select a verdict
   from observations or derive counts from PLANAR_CHECKED_WORK.
2. Add a runnable `invariant_reads` example using the existing required-feature
   program and installation. Parameterize that seed's unrelated ring population
   while preserving its default. Compare a single touched root's actual forward
   interval with the ordinary PlanarRead receipt, then compare full observations
   across increasing unrelated rings. Exercise domain refusal and unchanged
   committed reads. Retain the original request and invariant budgets.
3. Write `foundations/invariants-and-read-observations.md` using the real public
   facades: installed proposed/committed rules, retained projections, decision
   reads, and ordinary declared receipts. Include methods, access/denial and
   lifecycle limits, small and realistic examples, and the shared counter units.
   Bind the realistic snippet to compiled source through the existing consumer
   documentation inventory/drift court; update capability/guide navigation.
4. Build/run the new example and existing required-feature tests, the compiled
   snippet drift court and affected public invariant acceptance consumers. Run
   formatting, dirty size/function/composition review, boundary-check and
   agent-context. The persistent reviewer gates the living patch and evidence
   before this slice's commit.

Proprietary adoption remains the next Phase 1 obligation. No master merge until
all Phase 1 requirements are proven; no new budget, platform test door, traversal
engine or observation-based production decision in this slice.

## Phase 1 closeout

Phase 1 is accepted on 2026-10-25. Installed invariant planning/evaluation,
retained snapshot and operation projections, declared receipts and published
read work expose the same actual-read value. Acquisition observations remain
separate from the existing private projection budget; reuse does not replay
historical reads and fallback does not erase reads already performed.

Public evidence includes focused owner and grouped certification tests,
full execution-owner test compilation and all 15 ownership/optional-aspect
tests using the explicitly migrated no-observer fixture callers,
the runnable planar invariant/declared-query differential, unrelated-ring
locality, refusal atomicity, compiled documentation drift, facade snapshots,
boundary-check, agent-context, formatting and scoped structural checks. The
feature guide is
[`invariants-and-read-observations.md`](../../workspaces/worth-query/crates/worth-query/docs/foundations/invariants-and-read-observations.md).

Proprietary adoption is committed against the published SDK revision
`97044ae9e9edcfe80c8f19b5389b74a2ada1af0f`. `LiveQueryCost` and its modeled
assertions are deleted. Genuine installed projections prove history-read
locality and growth. The full CAD court passes 129 tests, and full workspace
CI passes all 3,134 tests. Workspace doctests, the oracle gate, determinism
and scoped static gates pass. Six unchanged Windows script guard-hook failures
are recorded as pre-existing consumer repository debt. The history-only court
does not claim cell geometry, storage bytes or cell-bearing publication.

The persistent reviewer cleared every slice through qa-loop, qa-tests and
code-quality-qa. Phase 2 is next; no relation law, native walk, orbit,
component or rule-readable derived-relation obligation is claimed complete.

## Phase 2, slice 1: declared acyclicity and original typed refusals

Boundary review: `ApplicationRelationIntegrity` is a public Copy value used by
const relation references and struct-update expressions. Its canonical owner
encodes relation meaning in `canonical_identity/member/schema.rs`.
`schema_layout/registry_lowering.rs::lower_relation_integrity` already owns
lowering; Relational already has an acyclicity declaration, complete proposed
edge reachability, native closing-edge identities and commit-boundary
registration. Query must connect these owners, without another evaluator.

Native law fields currently disappear in three places: invariant candidate
failure translation, ordinary provider preparation at the application denial,
and bootstrap commit translation. Provider preparation already retains its
original native error. Invariant and bootstrap denials must retain the original
law fields too. A borrowed typed law view can expose those native identity
values from their owning denial without replay, string parsing or another
retained verdict. Entity/reference/kind values are diagnostics and grant no
read or mutation authority; runtime handles remain private.

Relational's evaluator returns no violation when a checkpoint stops it, but
the execution owner checks the original request around planning, packet
evaluation and reduction. Prove refusal through the complete lowered path;
do not infer a silent pass from the local Option return alone. Snapshot
publication checks also run before candidate publication, so connectivity's
existing boundary is not evidence of a later-than-commit gap.

Documentation boundary: `modeling/schema-validation.md` describes the internal
canonical-query engine and explicitly directs applications elsewhere. Keep
that audience clear: the ordinary application law guide belongs in
`modeling/relation-laws.md`, linked from schema validation and the capability
index, rather than mixing application commits into the editor engine guide.

Plan:

1. Add a public acyclicity flag and const `.acyclic()` builder to the existing
   integrity value; the existing constructor defaults to no acyclicity law.
   Encode the enabled law in canonical meaning. An absent law preserves the
   existing meaning; enabling/removing it changes schema/program identity.
2. Keep one lowering seam. Extract relation-integrity lowering and its tests
   into a named `registry_lowering/relation_integrity` child, then lower the
   flag through `with_acyclicity_contracts` with its native plan revision.
3. Place `WorthQueryRelationLawDenial` under the existing invariant-execution
   denial owner. It is a borrowed typed view of original native fields, beginning
   with acyclicity's relation/contract and closing-edge endpoints. Retain fields
   at invariant and bootstrap translation, and expose the same view from
   application commit denials, including provider preparation. Keep existing
   refusal categories, original control causes and untouched diagnostics.
4. Add a neutral installed graph court under the existing `application_graph`
   certification target, with separate schema, operation, installation and
   evidence responsibilities. Use ordinary Decl/Host installation, authenticated
   program-owned link mutations and retained projections; no raw native graph
   access or new exported test door. Its one mutation can carry multiple edges,
   so a proposed two-edge cycle is not tested as two separate intermediate states.
5. Prove hostile/lawful twins, closing a cycle across commits and within one
   commit, self-edges, a 1,000-edge cycle, unchanged committed truth on refusal,
   and original-request interruption with a completing twin. Compare seeded
   small-graph edits with an independent reachability model. Cover schema/program
   identity drift and incompatible old meaning through existing admission courts.
   The self-edge declaration permits self endpoints so acyclicity itself judges
   it. Interruption evidence must reach native law evaluation; cancellation
   before admission alone cannot expose an evaluator that silently passes.
   Strengthen the existing native packet court to observe the checked
   evaluator callback's own typed stop, retaining its completing twin. The
   outer execution owner independently preserves stops, so an outer refusal
   alone cannot expose a worker that swallows the evaluator's failure.
6. Document the actual acyclic declaration and typed refusal in
   `modeling/relation-laws.md`, linked from `modeling/schema-validation.md`, using
   a compiled excerpt from the public court, and update the capability index.
   The other four laws, public fixture
   cutovers and Proprietary's cycle-check deletion remain Phase 2 obligations.
7. Run declaration/lowering/denial owner tests and the affected grouped public
   certification target, compiled-doc drift, formatting, dirty line caps/function
   review, boundary-check and agent-context. The persistent reviewer gates the
   living diff and evidence through qa-loop, qa-tests and code-quality-qa before
   commit. No Phase 2 master merge until all five laws and consumer adoption pass.

No additional work budget, accounting system, traversal engine or application
law checker is introduced. Resource exhaustion tests do not replace the semantic
and cancellation courts above.

### Identity boundary correction discovered by the real compatibility court

The new public court exposed two missing handoffs. `with_schema` currently
captures only operation bodies, so validated program revisions ignore relation
laws. Native exact recovery preserves historical schema carriers correctly,
but its current-cell target comparison checks codecs only. A lawful old graph
therefore silently restores under changed laws. These are required repairs
within this slice, not acceptable compatibility evidence.

The declaration owner will supply one normalized enabled-law representation
per authored relation and endpoints. Schema identity and program meaning use
that same representation. `with_schema` retains it; manifest and semantic facts
include it under an explicit relation-law family; support admission compares it
with the installed schema. Empty law collections preserve existing revisions,
while unbound `begin().validated_program()` cannot bypass enabled laws.

Extend the native current-cell readmission fence with a named relation-law
comparison against target declarations in both directions, using its existing
execution context and typed SchemaMismatch. Historical-only carriers retain
their own authority; explicit codec-transition custody remains separate. Do
not broaden this into whole-registry equality or re-author the schema during
program validation. Add/remove laws, unbound program admission, lawful restore
twins and archive semantic facts must expose the repaired handoffs. The
persistent reviewer independently confirmed these owner boundaries before edits.

The compiled-document guard recognized registered binary/example targets only;
it refused the genuine `application_graph` integration-test owner. Admit
explicit registered Cargo test targets through the same confinement, active
module reachability and exact region checks. Extend its existing hostile/lawful
registration court to cover examples and integration tests, without accepting
unregistered or disabled targets or moving the graph fixture into a second build.

### Slice 1 verification

Declared acyclicity now lowers into the existing native contract. Commit and
bootstrap refusals retain the original typed law fields. The completed schema's
enabled law meaning binds schema identity, program revision, semantic archive
facts and installed support. Exact recovery refuses changed laws on current
branch heads while preserving historical schema carriers and codec-transition
custody.

Evidence on the living slice:

- All 9 public relation-law courts pass, including proposed batch cycles,
  self-edges, a 1,000-edge history, seeded independent reachability, refusal
  atomicity, original-request cancellation, archive identity, genuine checkpoint
  add/remove refusal and unbound-program installation refusal.
- The full `application_graph` target passes 402 tests with 5 existing ignored
  tests. Declaration passes 460, installation 343 and archive 116 unit tests.
- The full execution-owner test library compiles; its 26 focused lowering,
  denial, bootstrap and ownership courts pass.
- Native acyclicity resources pass 8 tests, including the checked evaluator's
  own interruption result and completing twin. Native checkpoint restoration
  passes 18 tests with 1 existing ignored test; codec transitions pass all 29.
- Compiled documentation drift passes. The documentation guard passes its 39
  focused tests and production-scan fence regression. Normal boundary-check,
  agent-context, all 46 scoped Rust formatting checks, dirty line caps,
  whitespace and function scrutiny pass. The 14 function advisories were
  reviewed for coherent responsibility; no threshold-only restructuring was
  required.

The transient disk-full build failures were retried after Cargo removed this
task's disposable package artifacts. The interrupted formatter's tooling test
file was restored from Git plus its intended edit and compiled successfully.
No failed build is counted as passing evidence.

The remaining four laws, public fixture cutovers and Proprietary's
`PreviousFeature` adoption are still Phase 2 obligations. This slice does not
close Phase 2 or authorize its master merge.

## Phase 2, slice 2: paired symmetry and scoped edge uniqueness

Boundary review: the declaration/identity/lowering/original-denial handoffs now
exist. Extend them rather than creating another rule lane. Native symmetry and
uniqueness already consume prepared relation-integrity scopes. Deletion and
`UpdateEndpoints` correctly schedule scopes and remove old records from their
proposed pair counts. However, symmetry currently returns on empty
`planned_edges` and judges only creates. A deletion-only candidate can leave an
unmatched inverse without being judged. The persistent reviewer independently
confirmed the final ownership boundary: `directed_pair_counts` supply proposed
multiplicities, while explicit changed/revalidated pairs supply selection.
No new scope scan or traversal is needed. Preserve the other native symmetry
modes' existing posture.

Scoped uniqueness already judges physical proposed multiplicity: directed scope
counts one direction, normalized scope counts both directions together. An
inverse pair therefore satisfies directed uniqueness but violates normalized
uniqueness. Paired symmetry stays on the same relation, whose endpoint contract
must also permit its reverse; homogeneous endpoints are the ordinary example.
A permitted self-edge is its own inverse. No inverse-relation registry or
automatic deduplication belongs here.

The current public court only adds edges, and its readout is a reachability set.
Extend its one operation to carry removals and additions together. An unlink
first reads genuine complete adjacency through `DecisionReader`, then consumes
those completed facts through `CandidateWriter::unlink`; declare the required
relation read/unlink capabilities. The readout must retain physical edge
multiplicity for uniqueness evidence, alongside its existing reachability set.

Plan:

1. Add const `.symmetric()` and `.unique(scope)` declarations, with a Query-owned
   portable uniqueness-scope enum. Defaults preserve existing policy and
   identity. Extend the declaration-owned enabled-law encoding so schema,
   manifest, semantic facts and support admission continue to agree.
2. Lower to native `PairedTwinRequired` and the exact declared uniqueness scope.
   Extend the borrowed denial view for original symmetry and uniqueness fields;
   route the original native fields through existing invariant/bootstrap paths.
3. Repair only native paired symmetry evaluation to inspect affected surviving
   directed pair keys. A direct native `UpdateEndpoints` court must expose a
   repair that overlooks the abandoned pair, with a balanced replacement twin.
4. Extend the existing public mutation and real projection court as above.
   Prove paired insertion, single/both deletion, unbalanced/balanced replacement,
   self-edge semantics, typed original refusals and unchanged committed truth.
   Prove both uniqueness scopes using existing-plus-new and same-candidate
   duplicates, inverse and distinct-pair twins, and removal/replacement.
5. Add seeded edit sequences checked against an independent physical multigraph
   model. Check identity/archive/support/restore handoffs for both new laws with
   the same genuine programs and checkpoints, without duplicating the host.
6. Extend the relation-law guide and compiled examples, including endpoint
   compatibility and the actual paired/normalized-uniqueness composition.
   Update the capability index and facade inventory only as required.
7. Run focused declaration/lowering/denial/native pair courts, the public grouped
   graph target and compiled drift, formatting, scoped caps/function scrutiny,
   normal boundary-check and agent-context. The persistent reviewer gates the
   living slice and results before commit. No master merge until Phase 2 closes.

Minimum reachability, partition isolation and consumer cutovers remain later
Phase 2 work. No application law evaluator, new traversal, resource allowance or
accounting subsystem is introduced.

### Pair-court boundary corrections from actual refusals

Native merged-intent identity independently forbids two create/update commands
with the same partition/kind/ordered endpoints before invariant evaluation.
Do not change that identity rule to force a later uniqueness verdict. The
directed uniqueness batch court instead proposes an existing duplicate plus a
distinct new edge; its lawful twin changes the duplicate. Seeded candidates use
distinct ordered create pairs per proposal while retaining physical duplicate
counts across commits. Normalized uniqueness still judges an inverse pair
inside one proposal. Document earlier-refusal preemption honestly.

For endpoint replacement, moving onto an already-existing ordered pair also
preempts symmetry at native identity admission. Instead, update one old edge
and create its new reverse in the same candidate: the new pair is balanced,
but the abandoned old reverse must refuse. Rewiring both old edges is its
balanced twin, without any duplicate proposed identity.

An outgoing adjacency read carries existing matched relation IDs for unlink,
not a manufactured pair token for a nonexistent edge. The fixture handler
retains the genuine read, skips absent removal effects, and coalesces repeated
removal commands. It must never deduplicate physical records in the readout or
oracle. The persistent reviewer independently confirmed these boundaries;
the failures do not justify broader production identity or authority changes.

### Paired evaluation requires its explicit affected-pair domain

The seeded multigraph court exposed a second native boundary error in the
initial repair: pair counts contain incidental edges from partial incidence
scans. For an existing A→B create, `scan_full_on_create` schedules outgoing A,
incoming A and outgoing B, but not incoming B. Judging every discovered pair
can falsely reject incidental B→C because C→B was not requested. Global
incidence sets also serve other relation kinds with different read needs, so
adding one scan direction would not make every incidental key authoritative.

The persistent reviewer independently confirmed that selection responsibility
belongs in preparation. Retain an explicit affected-pair set only for native
paired symmetry requirements. Record both orientations of creates, deleted old
pairs, endpoint updates and explicit relation revalidation. Existing entity
removal and revalidation paths identify their incident affected pairs from
already retained visible edges; keep explicit revalidated entity identities
separate from connectivity's candidate sets. Paired evaluation judges only
surviving orientations of those pairs using the existing proposed counts.
Other modes and cardinality/uniqueness consumers keep their existing scopes.

Keep the exact seed and add its small counterexample as a public regression.
This introduces bounded preparation metadata in its existing owner, not another
graph walk, read engine, allowance or accounting model. No incidental adjacency
key becomes a substitute for a changed/revalidated pair's actual provenance.

### Slice 2 verification

Paired symmetry and directed/normalized edge uniqueness are declared through
the same integrity value, canonical law representation and native lowering
owner as acyclicity. Original native mode, scope, multiplicity and endpoint
identities remain typed through refusals. Native paired selection now follows
explicit affected pairs across creation, deletion, endpoint updates and
revalidation, without treating incidental adjacency keys as complete evidence.

The frozen source passes:

- All 19 public relation-law courts, including four independent 72-edit
  physical multigraph sequences, paired deletion and replacement, self-edges,
  both uniqueness scopes, typed diagnostics, identity/support/archive/restore,
  and the focused incidental-neighbor false-refusal regression.
- The full `application_graph` target: 412 passed, 5 existing ignored.
- Native relation integrity: 65 passed; invariant access: 11 passed;
  acyclicity resources: 8 passed. Native endpoint updates cover both paired
  modes; explicit relation/entity revalidation proves actual native check reach.
- Full execution-owner test compilation and all 27 focused lowering, original
  denial, bootstrap refusal and ownership tests.
- Declaration: 461 passed; installation: 343 passed; archive: 116 passed.
- Compiled documentation drift, including all three relation-law excerpts;
  normal boundary-check; agent-context; formatting and whitespace for all 33
  scoped Rust files; dirty line caps; and function scrutiny with 13 reviewed
  advisories and no scan errors.

The persistent reviewer found no material code or test issue through qa-loop,
qa-tests and code-quality-qa. The corrected plan distinguishes proposed counts
from explicit selection. Native duplicate-proposal identity admission remains
unchanged, and every failed intermediate court was resolved without suppressing
the seeded multigraph evidence or replacing physical counts with sets.

Minimum reachability, partition isolation, the public fixture cutover and
Proprietary adoption remain unfinished. Phase 2 is not yet ready for master.

## Phase 2, slice 3: Same-partition endpoints

Boundary review: declaration-owned law meaning already feeds schema identity,
program manifest/facts and support admission. Native partition isolation evaluates
planned and affected visible edges at commit, but drops their endpoint identities
when constructing its original violation. Preserve those original references at
that seam, through native diagnostics/witness keys and Query's borrowed denial.
The evaluator remains the only law owner. No graph reread or partition-derived
entity identity is permitted.

Ordinary CandidateWriter creation already obtains an issued mutation partition;
create_entity_in_context uses a genuine projected existing entity's partition.
Use these existing authoring paths in the public graph court. Both endpoint
policies allow explicit cross-context edges, so the new native law must make the
verdict. Relation record placement is independent of endpoint isolation.

Plan:

1. Add const `.same_partition_endpoints()` and its default-false field to
   ApplicationRelationIntegrity; append its enabled-law canonical field without
   altering earlier preimages. Lower only through lower_relation_integrity to
   native SamePartitionEndpoints.
2. Preserve source/target EntityReferences in the original native partition
   violation, its diagnostic projection and witness identity. Query borrows them
   through its existing shared law-denial routing. Add focused transport evidence.
3. Extend the existing public graph program with a genuine create-node action,
   using ordinary issued/context creation, initialization and optional linking.
   Add no runtime or fixture authority door. Retained projection reads establish
   committed node and physical edge truth.
4. Prove Existing/Created and Existing/Existing hostile candidates, context twins,
   explicit cross-partition permission without isolation, multiple affected edges
   and seeded edits over real partitions against an independent endpoint model.
   Verify original endpoint identities and refusal atomicity. Exercise identity,
   archive, unbound installation and checkpoint law add/remove handoffs.
5. Document native partition meaning and the existing contextual creation path
   with a compiled excerpt; update the capability row. Run affected owner and
   grouped public/native tests, compiled docs, formatting, scoped caps/function
   scrutiny, boundary-check and agent-context. The persistent reviewer gates the
   living diff and evidence before commit.

Minimum reachability and its publication/created-identity boundaries remain the
next slice. Public fixture cutover and Proprietary adoption still gate the Phase 2
merge. No new traversal, budget or accounting model belongs in this slice.

### Slice 3 verification

The const same-partition endpoint declaration lowers through the existing
relation-integrity owner to native SamePartitionEndpoints. The original native
violation now preserves both endpoint references through serialization, typed
diagnostics, distinct witness keys and Query's borrowed denial. Endpoint policy
and relation-record placement remain independent.

All frozen-source gates pass:

- The 23 discovered public relation-law tests, including four new real-partition
  courts: original Existing/Created and Existing/Existing refusals, direct absent
  node lookup after refusal, contextual creation and no-law cross-partition
  twins, whole-candidate atomicity and a 72-edit independent partition-group
  oracle. Compatibility tests hold endpoint permission fixed while changing only
  isolation and cover program/archive/support/checkpoint handoffs.
- The full application_graph target: 421 discovered, 416 passed, 5 existing
  ignored. The new creation action declares its actual field initialization
  write capability and uses distinct per-attempt idempotency commands.
- Native partition-related tests: 70 passed; relation integrity: 65 passed;
  invariant access: 11 passed. The first group includes original diagnostic,
  serialized endpoint and witness-distinction evidence; these filters overlap.
- Declaration: 462 passed; installation: 343 passed; archive: 116 passed.
  The full execution-owner test library compiled (1,971 tests), and all 34
  focused lowering, refusal-translation, bootstrap and ownership tests passed.
- Compiled documentation drift: 1 passed, including the partition excerpt and
  its exact inventory. Normal boundary-check and agent-context passed.
- Scoped formatting and whitespace, dirty Rust line caps, and function scrutiny
  passed across all 25 scoped Rust files (19 advisories, zero scan errors).

The persistent reviewer completed one living-diff pass with qa-loop, qa-tests
and code-quality-qa. Its two named corrections replaced an outcome-selected
absence observation with a direct optional projection lookup and completed the
compiled-doc excerpt inventory; both were accepted. No new native evaluator,
traversal, budget, accounting model or test authority door was introduced.

Minimum reachability, public fixture cutover and Proprietary adoption remain
unfinished. Phase 2 remains on its task branch until those gates close.

## Phase 2, slice 4: Minimum reachable targets

Boundary review: native connectivity counts distinct reachable targets of the
selected kinds, follows this relation in source-to-target direction, excludes
its starting source even through cycles, and obligates every visible entity of
the relation's source kind. SnapshotPublication is inside actual application
commit validation, before publication. The evaluator stays the only graph-law
owner.

Publication currently discards the original InvariantViolation in native
InvariantFailure::into_publication_error before Query can inspect it. Preserve
that original payload at this conversion, then carry/borrow it through Query's
invariant, provider and bootstrap refusal paths; do not reconstruct it from the
diagnostic text or rerun the candidate.

Authored minimum kinds are portable entity names obtainable from typed entity
references. Validate them against the completed schema, normalize their set for
canonical meaning, and resolve them through the complete entity_kinds binding
already produced by schema_layout::lower. Carry that binding through the existing
registration/lowering seam as an explicit Option: ordinary authored schemas pass
Some, generated workflow relations with no named minimum pass None. Enabled
minimum laws with missing bindings or unknown names must refuse, never disappear.
No registry rescans, empty proxy maps or second lowering wrapper are needed.

The existing numeric enabled-law encoding must gain a declaration-owned typed
numeric/kind-set value. Keep old numeric preimages and no-law absence unchanged;
use the same normalized representation for canonical schema, program manifest,
semantic facts and support admission. Do not hash entity names into numeric flags.

There is an unproven identity hazard: publication scope can observe already
allocated records as Existing while also retaining symbolic Created plan edges.
Connectivity unions these references, potentially counting one target twice.
The mutation engine already issues CommitCreatedEntityBindings; detached proposal
construction currently discards them, and actual publication validation also
fails to consume them. Prove the public failure before choosing a repair. If
exposed, carry the mutation owner's applied-state/correspondence truth at those
handoffs. No inferred slots, key scans, alternate traversal or guessed IDs.

Plan:

1. Add const `.reaches_at_least(minimum, kinds)` with a named portable minimum
   value. Reject zero, empty sets and undeclared kinds at completed declaration.
   Extend the single law-meaning encoding as above and preserve existing defaults.
2. Make native relation-integrity lowering fallible and carry the explicit kind
   binding through existing registration calls. Resolve/sort/deduplicate target
   IDs, use the declared from_kind as source domain, and lower the native
   SnapshotPublication minimum contract. Prove missing-binding refusal and exact
   no-minimum generated meaning.
3. Preserve native original publication invariant payloads and extend Query's
   original borrowed law denial with source identity, actual distinct count and
   declared minimum. Cover native conversion/serialization and all Query handoffs.
4. Reuse the existing public graph installation and authentication. Extend its
   cohesive vocabulary with Leaf and Feeds (Node to Leaf), and a real operation
   that reads/removes observed Feeds edges, creates contextual Leaves and links
   them. Keep Precedes Node-to-Node for independent cycle/self/duplicate-path
   courts. No fixture law evaluator or second host/install system.
5. First run the alias reproducer: an existing Node starts lawfully reaching two
   Leaves; replacing both links with one newly created Leaf must refuse minimum2.
   Its two-distinct-Leaf twin commits. Probe attempted keys directly on refusal.
   If it exposes the suspected alias, revise this brief with the actual evidence
   and smallest owner repair before editing that substrate seam.
6. Prove existing-only removals, created targets/sources, isolated sources,
   source/target deletion, cycles and source exclusion, duplicate physical/path
   reachability, target-kind normalization, typed counts and refusal atomicity.
   Add seeded edits against an independent distinct-reachability graph model.
   Extend schema/program/archive/support/checkpoint law handoffs with lawful
   bootstrap states rather than empty graphs that already violate the minimum.
7. Document exact source obligation and distinct-target semantics with a compiled
   example. Run focused native/public/owner tests and the full grouped public
   graph target; compiled docs, normal boundary/context, formatting, caps and
   function scrutiny. The persistent reviewer gates the living diff and evidence
   before commit. Public fixture and Proprietary adoption still gate Phase 2.

Only minimum reachability and the necessary original-publication/allocated-truth
handoffs belong here. Other milestone phases and consumer cutovers remain later
slices; no new budgets, work counters or traversal engine are introduced.

### Minimum court confirms the applied-identity boundary defect

The compiled ordinary public court was run before any native identity repair.
Replacing an existing Node's two Feeds links with one newly created contextual
Leaf returned Committed (commit/version 3) under a declared minimum of two.
The source already existed and Leaf is not a source kind, so an isolated newly
created source cannot mask or explain the result. The retained reproducer log is
`targets/query-9.18.1-minimum-created-target-reproducer.log` outside the checkout.
This confirms the mixed Existing/Created successor representations can falsely
satisfy a real application law.

Repair the causal handoff: detached authoritative apply returns one private
AppliedInvariantState containing its WorkingState and its own owner-issued
CommitCreatedEntityBindings. Carry that value through proposal validation and
prepared execution. Actual mutation already has a separate correspondence;
actual snapshot validation must borrow that actual mutation's pair, never the
detached one. The native observation retains an explicit borrowed applied basis
only from these post-apply constructors, and request/packet/context preserve it.

Connectivity consumes completed visible sources/successors alone in that basis,
without appending symbolic planned sources or successors. Existing committed+
plan fixture modes keep their current semantics; other native law scopes are
unchanged. On refusal, recover an unpublished source's Created reference from
the same mutation owner's stable correspondence, with checkpoints during the
failure-path correspondence lookup. No graph or name/slot identity lookup is
introduced, and no success-path reverse-index allocation is needed. The
persistent reviewer independently endorsed this repair after the public failure.


### Slice 4 verification

All five relation laws are now declarable through the ordinary Query schema.
Minimum reachability resolves the completed schema's portable target-kind names
through the sole native lowerer, normalizes their set in canonical meaning, and
uses the existing native SnapshotPublication contract before actual publication.
Generated workflow registrations explicitly have no named-kind binding; an
enabled minimum cannot silently lower without one. Existing numeric law meaning
and the no-law default keep their previous canonical encoding.

The original publication invariant now survives the native conversion and all
Query denial handoffs. Callers borrow its native source identity and distinct
reachable-target count, including an original Created source. Diagnostic strings
are never parsed into evidence. Native serialization still accepts old errors
without a typed payload.

The public failing-before court exposed the applied-identity defect described
above. The repair carries each applied graph together with that apply's actual
created-entity correspondence. Connectivity traverses its completed visible
records without appending symbolic plan edges or sources. The failure path uses
the same owner's retained stable correspondence to recover a Created source,
with cancellation checkpoints. Other law scopes and committed-plus-plan fixture
semantics retain their previous behavior.

The deletion court also exposed a fixture evidence omission: resolving a key and
naming a mutation target do not supply the Entity fact required for retirement.
The ordinary decision reader now reads the declared Node/Leaf entity before
requesting deletion; its production binding and access scope are unchanged.

Verification of production and public behavior:

- The ordinary public relation-law court passes all 31 tests. The full grouped
  application_graph target discovers 429 tests: 424 passed and five existing
  tests remain ignored. This includes the one-created-target false-pass
  reproducer, its two-target twin, Created-source refusal and return-edge twin,
  bootstrap, prior target deletion, source deletion, duplicate physical paths,
  cycles/self exclusion, normalized kind sets and 72 independent graph edits.
  Refusals probe attempted keys directly and preserve the committed physical bag.
- Native validation-engine tests: 67 passed; created-entity correspondence:
  three passed; transaction core: 240 passed; materialization: 62 passed and two
  existing ignored; proposal: 26 passed; invariant access: 11 passed; request
  checkpoints: 11 passed. These filters overlap and are not a unique-test sum.
- Declaration 464, Installation 343 and PackageArchive 116 tests passed (923
  total). Execution's complete 1,973-test library compiled; its focused seven
  lowering, five denial, seven bootstrap and 17 ownership tests passed.
- The compiled-example drift test passes. The guide explains every source-kind
  entity's obligation, distinct targets, excluded starting source, cycles,
  creation/deletion and the native publication boundary. It distinguishes
  unsatisfiable homogeneous acyclic minima from the valid Node-to-Leaf model.

The normal compiled-source ownership guard then found a tooling defect: a direct
#[path] module and its unselected conventional sibling were both inventoried.
The registered example/test court fails before repair when that sibling is a
decoy. Rustc's direct path now overrides conventional candidates, while
cfg_attr-only selectors retain every selectable branch and conventional
fallback. Missing or malformed direct paths refuse, including a malformed direct
path beside a valid conditional selector. This corrects source selection rather
than weakening the compiled-doc guard or renaming the public fixture.

The persistent reviewer has reviewed the full living slice with qa-loop,
qa-tests and code-quality-qa, and separately accepted the named final tooling
correction. All 320 boundary-tool unit tests passed; after the final malformed
attribute correction its three source-selection tests and the registered
example/test decoy court passed again. The 14-test production source-resolution
integration owner also passes. The final normal boundary check passes with
"Road 1 Cargo topology is valid" (exit zero); the reviewer confirmed the broader
181-case authority suite is not an additional commit gate. Its remaining optional cases were intentionally stopped after the
required source-resolution owner passed, because their many concurrent Cargo
metadata processes contended with the mandatory normal check. No assertion had
failed, and the broader suite is not claimed as completed.
Final agent-context, formatting of all 60 dirty Rust files, whitespace and dirty
line caps pass. Function scrutiny reports 44 advisory candidates and no scan
errors; the scoped candidates were reviewed and accepted. The persistent
reviewer granted final commit clearance under all three requested QA skills.
Public consumer cutovers and Proprietary adoption still gate Phase 2; this slice
does not close the phase or merge it to master.


## Phase 2, slice 5: Planar consumer declaration cutover

Boundary review: PlanarSuccessor is a same-context, no-self Body-to-Body
relation with exactly one incoming and outgoing edge. Its finite components
are directed rings. The handler currently refuses CreateCycle inputs shorter
than three before authoring a candidate. Declare
`.reaches_at_least(2, &[Body::reference::<Schema>().name()])` on that existing
relation: two distinct other reachable Bodies is the ring's structural minimum.
Acyclicity would forbid the intended graph; symmetry would force two-cycles
under the existing degree contract. Extra uniqueness/partition flags would not
replace any remaining hand algorithm here.

The empty command creates no Body or edge and only preserves an anchor output.
It must retain a narrow domain denial, renamed to EmptyCycle to state its actual
meaning. Nonempty one-/two-member candidate rejection moves to the platform's
existing endpoint, geometric and graph-law boundaries. Geometry may reject a
two-cycle before publication minimum executes; do not weaken that rule to force
a particular diagnostic. Slice 4's independent public court already proves the
native minimum. Consumer evidence must prove the actual declaration/program
binding, removal of the handler size judgment, non-publication of malformed
rings, and lawful three-, four- and twelve-member rings.

PositiveTurnRule's exactly_one extracts a typed neighbor from the current Vec
read API for its geometric triple. Keep it and the determinant/coordinate rules:
Phase 3 supplies the walk API and Phase 4 the orbit replacement. The final
producer's explicit three-member closure is that output group's triangle shape,
not a general ring-minimum validator. It stays with its output owner.

This is the first law in the shared topology schema. Every real selected or
supported program must capture that schema through existing with_schema before
its revision is minted. ConsumerRoot's direct authoring, deployment-grant source
revisions, checkpoint fixtures, standalone examples, and Execution's source-
included ownership fixtures all participate. Their unbound paths cannot remain
ordinary installable programs. Bind the actual completed declaration; reuse a
locally held declaration where available. Preserve intentionally omitted rules,
bindings and malformed contributions so their tests still reach their named
refusals. No new authoring wrapper or admission bypass is introduced.

Plan:

1. Add minimum2 to PlanarSuccessor's existing integrity. Change only the empty
   CreateCycle input to a domain denial, rename its value, and update its public
   no-publication proof. Keep geometric and typed-neighbor semantics.
2. Cut over the shared-schema selected/support program authoring sites, including
   deployment source revision calculation, to with_schema. Do not bind unrelated
   schemas merely because their program builder has the same shape.
3. Extend the existing public consumer publication court with nonempty one-/two-
   member candidate refusals, unchanged committed version, and direct absence
   queries for every attempted key. Retain lawful 3/4/12 rings and reversed-
   winding twins. Add a focused schema/program meaning assertion using the
   actual declaration rather than a parallel law oracle.
4. Compile and run the standalone required-feature/read-count/shared-execution
   journeys and the ordinary ConsumerRoot journey. Compile its grouped tests and
   run affected selected/support/deployment/checkpoint consumers; use failure
   evidence to complete any missed schema-binding cutover, preserving each
   refusal's intended cause.
5. Compile Execution's full test library to catch shared source callers and run
   the 15 ownership courts. Run the existing grouped public graph law target,
   compiled documentation, normal boundary and context checks, scoped formatting,
   caps and function scrutiny. Avoid the unrelated full boundary-tool suite:
   this slice changes no source resolver or authority-sealing implementation.
6. The persistent reviewer applies qa-loop, qa-tests and code-quality-qa to the
   living scoped diff and final evidence. Commit/push after clearance. Phase 2
   remains open for private acyclic adoption and its verified native-before-
   custom preparation prerequisite; no phase master merge in this slice.

Destination topology stays with the current owners: relation meaning in
TopologyEntry's planar_topology, command semantics in handler and ConsumerValues,
public evidence in ConsumerRoot's publication family, authoring at each existing
program root/installer, and shared-source ownership evidence in Execution. No
new engine, traversal, budget, runtime test door or general helper is added.


### Slice 5 boundary expansion: Portable schema law ownership

The required source-included ownership suite compiled and ran: 14 passed, but
optional_aspects_archive_birth_preserve_and_cold_complete_roster failed at
fresh reconstructed package identity validation. The real package contained the
new PlanarSuccessor minimum. PackageArchive's relation codec writes only the old
endpoint/cardinality/deletion fields; decoding calls the old no-law constructor.
All five law declarations are therefore lost at this actual portable-schema
handoff. Program-description archive tests from Slice 4 exercise a different
record and do not prove this boundary. The failing-before log is
`targets/query-9.18.1-slice5-ownership-tests.log` outside the checkout.

An additional ownership defect prevents merely appending bytes: authored
minimum kinds are &'static slices, correctly supporting Copy const relation
references, while decoded untrusted names require owned storage. Leaking strings,
interning them forever, guessing authored values, or storing a parallel minimum
sidecar would not restore portable truth. Preserve authored Copy/const values and
materialize one owned ApplicationRelationIntegrityRecord at the existing typed
relation-to-erased-member boundary. Its owned minimum holds Vec<String>. Keep
that erased record as the sole downstream law truth: canonical schema meaning,
retained normalized law values, completed-kind validation, archive decoding and
native lowering borrow it. Numeric preimages, normalized kind-set preimages and
no-law absence remain exactly the same. Authored and decoded kind-name order and
duplicates retain their existing set semantics; the canonical-law owner still
normalizes them. No record is converted back into a fabricated static authoring
value.

Revised implementation steps before consumer closeout:

1. Add relation_integrity/record.rs and its subordinate minimum record, exporting
   the pure owned record vocabulary through the existing declaration facade.
   ApplicationSchemaMember::Relation owns this record. The declaration builder's
   relation method performs the authored conversion once. Carry references
   through canonical/law validation and native registry lowering; generated
   workflow no-law values convert at their existing registration calls into the
   same sole record lowerer. Update direct record constructors where the compiler
   exposes this ownership boundary. No alternate lowering or validator is added.
2. Move retained law-kind values to owned strings. Keep the one normalized set
   representation for schema, manifest, facts and support identity. Preserve the
   original numeric/default encodings and genuine completed-schema kind binding.
3. Encode/decode all five law fields at the existing relation codec, preserving
   uniqueness scope and minimum count/names. Separate its subordinate integrity
   codec if needed for cohesion/400-line caps. The default relation tag applies
   only to the complete default/no-law record, never a law-bearing default shape.
   Thread the existing RecordDecodeAttempt into schema decode; minimum's variable
   kind list uses its existing nested-entry/depth admission before allocation.
   This extends existing archive protection, not runtime resource bookkeeping.
4. Advance record grammar 13 to 14. The archive owner's declared exact-current
   record compatibility window already establishes current-only support; reject
   13 at the typed version boundary before payload work, without interpreting it
   as lawless 14. Update the current grammar description and relevant published
   compatibility prose. Other independent protocol layers retain their versions.
5. Add one coherent codec proof family for all five fields/uniqueness scopes,
   heap-owned target names, canonical/schema identity after round trip, malformed
   discriminants and existing structural-limit admission. Prove previous record
   version rejection before work with the existing decoder boundary. Retain and
   rerun the actual failing optional-aspects package court; do not replace it
   with member-only serialization evidence.
6. Recompile the complete affected owner libraries and external consumer tests.
   Run Declaration, Installation and PackageArchive owners, focused native law
   lowering/denial/bootstrap/ownership courts, the rebuilt public graph court,
   actual consumer/examples and affected checkpoint evidence. Reuse already
   accepted test intent but rerun changed authority/serialization seams. Final
   boundary/context, formatting, scoped caps/function scrutiny and the persistent
   review gate apply to the expanded final living diff before commit.

The reviewer independently confirmed this causal boundary and owned-record
shape using all three requested QA skills. Earlier Slice 5 source review remains
closed except this named new ownership/codec seam and its mechanical compiler
propagation. Private acyclic adoption and safe native-before-custom sequencing
remain later Phase 2 work. The observed archive defect is repaired in this slice;
it is not deferred as repository debt.


### Slice 5 original self-edge refusal

The ordinary ConsumerRoot executable reaches platform validation for the first
one-member candidate and returns ProviderRejected, not the custom geometric
violation provisionally expected by the new court. Native already owns typed
RelationEndpointKindSelfEdge fields. Query's original-field mapping recognizes
the five new laws but currently discards this older self-edge payload. Extend
only the existing borrowed non-exhaustive relation-law view with SelfEdge,
preserving the exact contract/relation IDs, both original endpoint references
and self_edge flag through the existing storage path. Keep the outer rejection
category unchanged; add no inference, second verdict or generic legacy project.
The one-member court requires this original cause, the actual endpoint contract
and equal original Created references. Its direct authored-key absence and
unchanged-version probes remain. The two-member case still requires the actual
PositivePlanarTurn violation if confirmed by execution. Extend the existing
invariant/provider/bootstrap transport family with this payload. The persistent
reviewer accepted this named correction before implementation.

The rebuilt ordinary consumer confirms that native naming admission interns the
original creation key: ClientKey::canonical_text renders its opaque symbol
coordinate, not the authored text. The public request has no interner authority.
Remove the unsupported raw-text comparison from this proof and its helper
parameter; preserve the original payload rather than reinterpret it or expose a
new interner door. This candidate creates exactly one vertex. Its equal typed
Created endpoints and exact endpoint contract establish the refused creation
origin, while the caller independently queries every authored key for absence
and checks the unchanged source version. The transport court separately compares
all original fields through each Query translation. The persistent reviewer
approved this bounded correction before editing.

### Slice 5 verification and review evidence

The actual optional-aspects archive court first failed fresh package identity
validation because the relation codec discarded the declared minimum. After the
owned-record repair, all 15 ownership-oracle cases pass, including that exact
archive/reconstruction/cold-completion journey. The complete Execution unit
library compiles with 1,974 tests; its eight relation-integrity lowering cases
and five original-denial translation cases pass. The translation case exercises
invariant, provider and bootstrap paths with original fields. Two exploratory
filters matched no tests and are not counted as evidence.

Declaration's 464 and Installation's 343 owner tests pass. PackageArchive's full
run passes 122 unit tests, 78 archive-protocol integration tests and one
compile-fail doc test. Initial golden failures were version13 expectations, not
payload repair: current record14 standalone vectors preserve all unrelated payload
bytes, and the envelope differs only at its nested record header and existing
checksum. Historical record13 vectors remain intact as unsupported inputs. Independent
header replacement and SHA-256 reproduce the current archive/envelope vectors.

The rebuilt public application_graph court passes 424 tests with five existing
ignored cases; its 31 relation-law cases are included in that total. Rebuilt
ConsumerRoot passes all 80 tests, and shared_execution's integration test passes.
All three ordinary production examples pass: required_feature, invariant_reads
and shared_execution. The final ordinary ConsumerRoot executable also passes its
whole public journey after the typed creation-identity correction, including
one/two-member refusals, independent absence/version checks, lawful 3-, 4- and
12-member rings and geometric refusals. The two-member refusal remains the
actual custom PositivePlanarTurn violation; the one-member refusal preserves
native SelfEdge.

Before owned record materialization, the complete 301-case Topology suite passed
in 1,389.61 seconds with the new planar minimum already installed. The reviewer
approved retaining the two expensive history-locality/regeneration passes from
that run: materializing per-relation kind names and borrowing them into the same
lowerer adds no per-history traversal or evaluator. The final rebuilt run covers
the other 299, including compiled-example drift, checkpoint, selected/support
construction and worker-matrix evidence. All 299 pass in 1,359.58 seconds with
zero failures and the two named cases filtered out. This is not a final-source
rerun of all 301 cases.

All 105 scoped Rust files were formatted and passed the 400-line cap. Function
scrutiny scanned 104 files with 59 advisory candidates and no scan errors; the
known long Windows path was scanned explicitly, adding one file/two candidates.
The persistent reviewer accepted all 61 advisories in scope. Whitespace passes.
Normal boundary-check reports Road 1 Cargo topology valid on the expanded
ownership/codec source, and agent-context check passes. The later tiny consumer
assertion correction changes no authority/dependency boundary and is covered by
its rebuilt ordinary executable, scoped format/scrutiny, caps and whitespace.
The all-targets test build emits five dead-code warnings in the shared_execution
example's cfg-disabled-main test harness; its ordinary example builds and runs
successfully. No warning-free claim is made for that extra harness.

The persistent reviewer has cleared the initial consumer changes and the focused
owned-record/codec/SelfEdge follow-up using all three requested QA skills. Final
Slice 5 commit-and-push clearance was granted after the completed 299-case
Topology result and closeout evidence. Phase 2 private acyclic adoption and
native-before-custom preparation remain open; this slice does not authorize a
Phase 2 master merge.

## Phase 2, slice 6: Native law prerequisites before custom preparation

### Boundary review

The public tree is clean at dd6cea4393 and the private adoption tree is clean at
cf9f3841d6. The latter still pins the accepted Phase 1 SDK97044 and retains the
manual cycle guard. Current source confirms that CAD Rule::prepare_scope enters
Access::candidate and basis::thread::live while preparing an installed custom
invariant. Removing that guard before a platform guarantee would permit an
unbounded walk on a proposed cycle. Its body/branch/retirement/name/definition
and geometry checks have separate meanings and must remain.

Relational owns the missing prerequisite. InvariantEngine creates one candidate
input plan, but packet_selection eagerly prepares all applicable custom scopes
while collecting native.chain(custom), before any native packet runs. Merely
sorting packets, adding another CAD/Query search, or relying on a later native
denial cannot protect custom preparation. The current proposal authority already
provides the correct proposed WorkingState, request, observation, version,
merged plan and typed failure conversion; preserve that path.

The engine must complete native planning, execution and reduction before custom
scope preparation at the same execution point. A relevant native blocking
failure returns its original result through the existing authority. Native
AuditOnly/advisory evidence does not block a custom hook. Request cancellation,
native interruption and reduction failure refuse before custom preparation.
The existing Automatic/unchecked-custom metadata admission stays up front:
administrative rejection invokes no hook and retains its current precedence.
This does not promise that a later SnapshotPublication check establishes an
earlier execution point's precondition.

Two handoffs constrain the implementation. First, successful native evidence
must survive in the final result, including checks, canonical decisions, packet
identities, preparation failures/conflicts and proof scope. The current reducer
has no stage-composition owner. Second, native evidence remains live while
custom preparation and execution allocate. Existing lease/serial reservations
and the existing shared RequestWorkBudget must cover that lifetime; no fresh
work account, policy, ceiling, cost contract or generic execution framework is
introduced.

Current planning supplies an important simplification: all packets for one
request share execution point, observation and canonical partition_scope.
ValidationReductionKey orders packet_index before group mask, and registration
order is native then custom. Stable custom ordinals offset by completed native
packet count therefore form a canonical suffix. Preserve actual packet keys,
worker identities and the last identity across that seam, and verify the prefix
condition instead of assuming arbitrary stages may be concatenated.

### Intended result and topology

There is no new authoring syntax. An application's existing declared native law
can establish its candidate-graph prerequisite before the same execution
point's custom scope reads that graph. CAD acyclic adoption follows only after
this prerequisite is implemented and independently certified; this public slice
does not delete the private guard or claim Phase 2 closure.

- validation/engine/engine.rs owns native-then-custom orchestration and liveness.
  A sibling stage_execution.rs extracts the existing plan/packet/evaluate flow,
  used once per selected stage, without another evaluator.
- execution/planning owns explicit native/custom selection and packet offsets.
  Native selection never invokes a custom executable. Existing checked planning
  and packet preparation remain the shared kernels.
- reduction/prefix owns an incremental private native prefix and checked
  adapter. Retain each completed stage's actual sorted worker envelopes and
  memory ticket, moving their backing rather than cloning check payloads.
  Absorb processes only newly completed results and updates the same summary
  policy/conflict semantics. It does not sort or reduce the native prefix again.
- result/execution_summary keeps one failure-effect classification; its existing
  whole-result construction and the incremental reducer use that owner. Final
  result construction consumes the accumulated summary, projects decisions once
  and moves checks into the existing public result; no public staging DTO.
- planning/proof_boundary retains actual partition IDs, widening causes and
  touched-only evidence until composition. Counts add, partitions union. The
  private strategy fold reports actual staged parallel execution if any
  completed nonempty stage selected it, otherwise the actual common serial
  reason. Serial-stage diagnostics remain visible; stages do not overlap.

### Implementation and proof

1. Add a same-owner native prerequisite court using issued DAG identities,
   ordinary validate/publish and a custom rule with separate preparation and
   evaluation observations. Seed one lawful edge; hostile closes its cycle,
   while the lawful twin additionally removes the old edge. Record the current
   failing-before evidence that custom preparation ran. Require original native
   contract/closing endpoints, both hooks absent and unchanged head on refusal;
   the twin must invoke both hooks and retain native plus custom evidence.
2. Split registration selection and thread its stage/ordinal through existing
   checked/unleased planning. Extract the current stage runner and preserve one
   runtime candidate-input instance, request, lease and cumulative account.
   Empty stages add no fictional worker/reducer result or strategy event.
3. Refactor the existing reducer into absorb/finish at its own owner. Native
   absorb really completes ordering, checkpoints, admission, summary and conflict
   reduction before the gate. Retain original envelope groups/keys, avoiding
   per-check key copies and native re-reduction. Compose result metadata only
   from stages actually run; output projection and its clones are admitted and
   charged as work actually performed.
4. Keep the native evidence reservation live throughout custom planning, workers
   and reduction. Existing run_measured_result and original-request scoped
   reservation APIs admit growth before allocation and retain backing on the
   original lineage. Existing envelope/check/identity size helpers measure the
   backing actually retained. Summary failure clones, conflict identities,
   retained keys and raw proof/strategy backing created by absorb are admitted
   there before growth; later finish admission cannot cover them retroactively.
   A stage's already-held input is not billed again
   as a new native copy. Final vector/decision/metadata growth has its own live
   peak admission while moved payloads remain covered until engine return.
   No release-then-reserve transfer or fabricated serial budget is allowed.
5. Extend the court for real native interruption with a completing twin,
   pre-cancel/no hooks, native-pass/custom preparation failure/cancel, advisory
   continuation, custom-only behavior and unchecked metadata precedence. Prove
   retained ticket lifetime, constrained/funded memory twins and release on
   refusal/drop. Exercise the real proposal's shared cumulative account; never
   give custom work a fresh copy of the original allowance. Preserve existing
   deterministic custom installation order and exact proof/decision provenance.
6. Update the existing relation-laws and installed-invariants guides' prerequisite/failure prose,
   stating the execution-point and administrative-admission limits. Rebuild the
   complete native owner library and affected Query/consumer targets; run native
   engine, request-work, checked planning/reduction and custom-order courts,
   application_graph, actual ConsumerRoot and all three production examples.
   Run the complete affected topology court after this scheduling change;
   previous DTO-only evidence does not certify this new ordering. Interpret any
   changed planar refusal by its actual original typed cause, not a catch-all.
7. Run normal boundary-check, agent-context, scoped formatting, line caps,
   function scrutiny and whitespace. The persistent reviewer applies qa-loop,
   qa-tests and code-quality-qa to the coherent final patch and evidence. Commit
   and push only after clearance. Then begin the separate private SDK adoption
   and guard deletion, with its own installed hostile/twin court and all private
   repository gates. No Phase 2 master merge is authorized by this slice alone.

The persistent reviewer independently confirmed the native staging owner,
canonical-prefix property, existing memory/request APIs and required barrier
proofs. The private general-modeling, reset, CAD laws, roadmap, README and
adoption instructions were read before implementation; they require this
platform repair at its public owner rather than a private workaround.

### Native prerequisite failing-before evidence

The new same-owner court compiles in the complete 1,826-test native library.
Its original run fails on the first unleased closing-edge proposal after the
original typed no_cycles/Kind2/exact endpoints have passed: both hook counters
advance from the lawful seed's (1,1) to (2,2), although native acyclicity refuses.
This directly disproves native denial as a preparation prerequisite today. The
log is targets/query-9.18.1-slice6-native-before-test.log outside the checkout.
The reviewer's sole fixture correction makes release assertions inspect the
actual SerialRequest memory budget on the unleased path, rather than the unused
lease; leased paths still inspect their original lease. No verdict assertion or
production code changes before this reproducer.

### Retained native backing clarification

The current envelope helper treats Native rule data as zero bytes, although
InvariantRule owns lowered contracts with ContractId(String) and kind vectors.
Planning's related helper counts some vectors but omits those strings. The new
cross-stage carry cannot truthfully reuse that omission. Add one crate-private
owned-capacity helper at InvariantRule's data owner and a capacity projection at
ContractId's owner, then reuse it from registration, envelope, identity and
decision retention. This measures existing backing; it introduces no resource
policy or runtime cost contract.

The measured result adapter separately accounts known input and returned heap.
Moved envelope backing is therefore held on its own original-lineage ticket,
while the measured absorb ticket covers only new summary/conflict allocations.
Older native tickets remain live during the custom stage. Final projection
admits its new vectors/decision/metadata backing while the stage tickets still
cover moved payloads. An originally unaccounted serial request remains native;
an already active accounted ancestor is honored. No fresh serial budget is
created to make an otherwise unavailable reservation succeed.

This carry begins at the existing reducer input seam. The packet runner still
returns materialized envelopes after releasing its worker-result tickets; the
new reducer admits that input's actual backing before retaining it across the
next stage. This slice does not claim continuous worker-to-caller ticket
transfer. Once a prefix ticket is held, no stage releases and re-reserves it.

### Initial implementation evidence

Normal Native check passes. The first expanded native library builds without
warnings; all seven barrier courts pass. They cover the three-posture hostile
cycle/lawful twin, real checked native worker exhaustion and funded twin,
original pre-cancellation, custom preparation refusal/cancellation after a
native pass, native advisory/HarnessAudit continuation, and actual retained
native backing across a refused/funded custom-stage admission. The existing
one/two/eight-worker overlap court now also retains a lawful native prefix and
three original checks, decisions and proof packets.

The complete native run reports 1,801 passes, 31 existing ignores and one
failure in the fixed five-entry work invoice. The source-derived change is
113 minus two empty scope walks of three units plus two new result-move
checkpoints, yielding 109. The test now expresses those independent terms and
retains its one-unit-below original-cause/head refusal. Its corrected rebuild,
additional publication/admin/accounting courts and consumer gates are pending;
these initial results do not constitute slice clearance.

### Consumer request handoff review

The final native run is green: 1,805 passes and 31 existing ignores. All ten
expanded barrier courts pass. Running the complete 1,974-case Query Execution
library, rather than only compiling it, exposes mixed-request native fixture
writes in aggregate, output-lineage/checkpoint and one provider-readmission
court. They enter an installed host phase but pass a separately constructed
accounted serial request to native mutation. The engine's new original-request
scope correctly refuses that unrelated request under the active leased parent.
Graph runtime access borrows a custody marker; it does not replace execution
authority or make the independently captured serial description a descendant.

The persistent reviewer confirmed the causal handoff. Correct only the failing
installed callbacks to borrow their phase's existing exact request and carry it
through native preparation, commit and publication. Where a test helper already
receives phase custody, remove its misleading second request argument instead
of silently ignoring it. Keep standalone serial native paths and independent
bounded/cancellation probes. Preserve all existing invalidation, equality,
checkpoint and retirement assertions. No production lineage relaxation, ambient
request lookup, new scope API or invented authority is part of this correction.
Rebuild and run those complete affected fixture families, then repeat the Query
owner suite as warranted by the corrected handoff and full consumer gates.

The production-identical full Query Execution run terminates after 1,005.51
seconds with 1,960 passes, 13 failures and one existing subprocess-probe ignore.
Every failure carries the diagnosed UnrelatedNestedLease cause; the long
authority-resource oracle passes. The reviewer accepts reuse of those untouched
passing tests, conditional on rebuilding and passing the complete aggregate,
native-journey, native-output-witness checkpoint and managed-readmission
families from the corrected artifact. Reused and rebuilt evidence will be
reported separately. A production change or different failure reopens it.

The installed application_graph court passes all 424 tests with five existing
ignores. The consumer-root test entry passes all 80 cases, and shared_execution
passes its one test. The positive consumer all-targets build succeeds for the
three named supported packages. An earlier overbroad workspace command also
selected hostile_foreign_schema and was refused with the intended E0277 foreign
schema membership fence; that command is not reported as a positive build.
The optional shared_execution example test harness retains its five existing
dead-code warnings. Ordinary executable builds and the complete 301-case
Topology scheduling court remain separate pending evidence.

The final Query fixture artifact compiles without warnings in 2m39s. Its exact
inventory remains 1,974 tests. Complete affected-family filters pass: aggregate
9, native_journey 31 (including concurrent required hints), checkpoint 2 and
managed readmission 28. These 70 current-artifact passes replace 57 earlier
passes and all 13 diagnosed failures. Together with 1,903 production-identical
unaffected passes and the existing ignored probe, this supplies partitioned
evidence for 1,973 passes and one ignore. It is not a single final-source full
run. The inventory/failure replacement check is recorded outside the checkout
in targets/query-9.18.1-slice6-query-partition.json. The reviewer accepts this
scope and the unchanged authority oracle's actual terminal pass.

The normal dev build of ConsumerRoot and all three ordinary examples succeeds
without warnings. The actual Root public candidate journey and required_feature,
shared_execution and invariant_reads executables all complete. Native, Graph,
consumer-root and shared test results above remain current for production.

Final normal boundary-check reports Road 1 Cargo topology valid, and the fresh
agent-context check passes. All 54 scoped Rust files pass formatting and the
400-line cap; function scrutiny reports 42 advisories and zero scan errors,
all accepted by the persistent reviewer. Whitespace passes. The complete
unfiltered Topology court is still running; no slice commit or Phase 2 merge
clearance is claimed before its terminal result.

### Topology observer boundary correction

The fresh unfiltered 301-case Topology run terminates in 1,049.02 seconds with
300 passes and one failure. The dropped-dependency journey passes its local
decision and demand-contact assertions, then reads process-global
producer::PROVIDER_CONTACTS as four instead of zero. Its later unresolved-root
assertion was not reached in that run. A complete isolated rerun passes,
including that later assertion; this alone does not clear the failed court.

Both real provider entry sites increment a shared process counter, so it cannot
attribute work to the application whose absence of work the court claims.
Checkpoint observation guards exclude participating courts but do not scope
the receiver's count. The reviewer verified the insertion boundary:
TopologyContribution::configure_features receives default and all three custom
provider constructions before setup.producer. That convergence must attach an
optional caller-owned Arc counter to the real provider; constructor-only
injection would miss custom key-mask/handler configurations.

Keep this correction in the fixture owner. Add a cfg(test) optional observer to
TopologyConfiguration, retain it on InitialPlanarProvider, and increment at the
same operation_input/idempotency_key entry sites. Extend private installation
support to accept the caller's observer; existing unobserved wrappers keep
their current meaning. Migrate initial-provider global reset/read courts to
their actual installed application's retained counter, then remove the global
initial-provider counter. Do not add a TLS registry, identity lookup, SDK API,
new budget, verdict selector or serialization workaround. Keep the observation
guard for its other existing receiver-bound recording duties.

Prove custody with two genuinely installed applications and distinct counters:
foreign real demand work must leave the subject counter unchanged, then subject
work must advance only its own counter. Preserve the dropped-dependency local
and unresolved-root assertions. Run its complete sibling court and the fresh
concurrent unfiltered Topology suite, then the affected ordinary consumer build
and scoped static gates. Existing Native/Query production evidence is reusable
because this follow-up changes only fixture observation ownership; any different
typed/local outcome reopens the corresponding production boundary.

The observer correction now retains the caller's optional counter on the real
provider through the central contribution path, including custom key-mask and
handler constructors. All initial-provider counter courts use explicit retained
observers; the independent final-output recorder is unchanged. The two-installed-
application custody court passes, and the complete diamond court passes all 20
cases, including the earlier failure's later unresolved-root assertion.

The rebuilt consumer-root library passes all 80 tests and shared_execution its
one test. The three named supported consumer packages pass their all-targets
build; the optional shared_execution example test harness retains its five
existing warnings. The ordinary dev build is warning-free, and the actual Root
candidate journey plus required_feature, shared_execution and invariant_reads
executables all pass again. Agent-context, formatting, whitespace and the dirty
line-cap guard pass. Function scrutiny covers 86 scoped Rust files with 69
advisories and zero scan errors; the persistent reviewer accepts all advisories
and the observer source under the three QA skills. The final normal boundary
check passes with Road 1 Cargo topology valid. Only the final concurrent
Topology terminal result remains pending.

The fresh concurrent 302-case run terminates after 1,449.71 seconds with 301
passes and one failure. The repaired dropped-dependency journey and all other
ordinary cases pass. The sole named matrix finding is family 7, five workers,
declaration 0, certified dispatch 0, compute failure 6, no reuse: the child
observes three distinct computing-member worker threads where its wave expects
four. All 921 runnable rows were executed; the matrix records one red row and
the declared 111 inapplicable empty-family rows. No typed verdict, value or
price comparison failure is reported for the remaining rows.

The existing query gate gives this exact full matrix a CPU-exclusive lane after
all shared jobs, and runs ordinary Topology cases in separate processes. The
additional concurrent run did not preserve that matrix resource posture. Run
the unchanged complete matrix in its prescribed lane, retaining the concurrent
failure as evidence. Do not lower its worker assertion, change a timeout,
exclude a row or claim a clean concurrent 302-case result. Its terminal result
and the reviewer's assessment remain open.

The unchanged full matrix passes in the pre-existing exclusive lane in 518.83
seconds: all 1,032 declared rows are accounted for, with 921 executed price-twin
rows, 111 named empty-family inapplicabilities and zero red rows. Combined with
the 301 actual concurrent passes, this closes the Topology evidence for this
slice under the repository's supported test posture. The reviewer accepts that
partition and requires no source change or blanket concurrent repetition. The
earlier 301/1 concurrent outcome remains recorded above; this is not a single
green unfiltered 302-case invocation.

All slice gates are now closed, including the final normal boundary/context
checks, ordinary consumer executables and scoped static checks. The next slice
is private CAD SDK adoption and replacement of its manual cycle guard after
this public revision is published. Phase 2 remains open until that installed
consumer proof and the private repository gates pass; no master merge is
claimed by this slice.

## Phase 2 integration prerequisite: concurrent upstream changes

Public slice 6 is committed and published at
`a2d26cee1665f850fce199211e01c812ffddda63`. The private adoption prepared a
coherent seven-pin/17-package SDK update, schema-bound program authors and
PreviousFeature acyclicity with only the manual cycle owner removed. Its real
installed hostile/twin passes before and after deletion; CAD library 130/one
existing ignore, static gates, oracle corpus/eight OCCT courts and ten
determinism sections pass. Its three broad workspace lanes are still open.

Fresh origin/master is now
`d6f5fed45c7d8ca2a3380c00c8d408270c1f560f`, thirteen upstream commits since
the public common base. They add typed producer-domain refusals and vacuous
producer-required invariant applicability/provenance, including packet
selection and receipt ownership touched by this phase. Fresh private main is
`98e3f261791bfde278dd15d94af8f6297c8d66b9`, 45 commits beyond the adoption
base, adding certified cell publication and joint moves across its thread,
schema, invariant and installed-test owners. Neither default branch is an
ancestor of its current task head; no fast-forward integration is claimed.

The reviewer accepts superseding only the three positively identified task-owned
old-base broad jobs and preserving the full private patch in a verified stash.
Those jobs must be recorded as superseded, not passed. Before source changes,
wait for their termination and keep both original dirty checkouts untouched.

Merge fresh public origin/master into the task with no automatic commit. At
packet selection, preserve the new upstream applicability/provenance contract
while keeping stage exclusion before custom preparation and the same original
request/account/native prefix. Preserve typed producer values, refusal identity
and current publication/restoration contracts through the Query facade. Review
actual conflict resolutions and new causal interactions under qa-loop,
qa-tests and code-quality-qa. Run Native/Query affected owner and installed
consumer courts, ordinary examples and the normal boundary/context/static gates
before committing/publishing the public integration candidate.

That published qualified candidate becomes the sole private SDK revision.
Fast-forward the private task to origin/main, restore its exact preserved patch
and resolve overlapping 2d/joint-move semantics deliberately. Re-run its actual
installed hostile/twin and all mandatory private gates on that integrated tree;
old-base evidence is historical rather than integration clearance. Reassess
baseline script debt against current upstream. Only after both integrated
reviews and private gates pass may final Phase 2 closeout advance the default
branches, with a fresh ancestry check and no force push.

The public merge has no Rust text conflicts. Its sole capability-index conflict
retains the relation-law row and both upstream producer-denial/required-invariant
rows. Cross-owner review confirms stage exclusion still precedes metering and
custom preparation; all three non-applicability bases travel unchanged through
the worker, reduced result and receipt collector. Producer admission remains
fail-closed for missing/unexplained/scoped-out/preparation-refused receipts.

One combined real-candidate court extends the upstream basis fixture with a
benign actual native MaxMergedIntents check. It requires that native pass in the
final commit-boundary report and exactly one matching custom receipt carrying
NotApplicable/CandidateOutsideClosure. This proves the nonempty native prefix
and custom provenance handoff together without a synthetic receipt, new work
model or duplicated law suite. Compile and execute it alongside the existing
native prerequisite/accounting and producer-requirement courts.

The three old-base private broad jobs were intentionally stopped and contribute
no passing evidence. Their task-owned process trees and remaining private-target
compiler jobs terminated before source changes; the shared compiler cache server
remained running. The complete 17-file private patch is preserved in stash
`a0a39a34f5767a985e7975230f7c2035606a6d9c` and an external binary patch.
The private task is clean at the new main revision, awaiting the published
qualified public candidate before patch restoration.

The merged Native build has zero diagnostics. Its full 1,843-case inventory
terminates with 1,812 passes and 31 existing ignores in 141.74 seconds. Discovery
confirms five basis courts including the new combined case, three narrowing
courts, two provenance decoders, ten native prerequisites, custom ordering and
the fixed five-entry cumulative-account invoice. The merged Graph build also
has zero diagnostics; all 424 active courts pass with five existing ignores in
62.98 seconds, including all 31 relation-law courts.

Normal boundary and context checks pass. All 79 scoped Rust files pass formatting,
line caps and whitespace; the reviewer accepts all 21 function advisories with
zero scan errors. One upstream fixture import was normalized before its build.
The application-correction example compiles successfully. The current Query
inventory has 1,990 tests and is executing through the existing chunk/probe
isolation, including actual typed producer-denial, invariant-refusal,
producer-requirement and restoration-admission courts. BankDomain compilation,
consumer targets/ordinary journeys and the canonical fixture gate remain open.
The complete fixture matrix retains its existing CPU-exclusive lane after
other heavy verification work; no unsupported single-process full run is used.

The complete current Query artifact passes in its existing chunk/probe lanes:
1,989 passes, one existing subprocess-only ignore, zero failed tests or bad
exits, in 800 seconds. All 1,990 names are accounted for; discovery includes
seven producer-value tests, twelve typed invariant-refusal tests (two producer
requirements), three restoration-admission tests and two domain-denial outcome
tests. The authority/resource oracle actually passes in its 790-second chunk.
No pre-merge Query partition replaces this current integration execution.

BankDomain passes its all-targets check and the changed application-correction
example compiles, both without warnings. The three supported consumer packages
pass their complete all-targets build with the five retained optional
shared_execution harness warnings. Their actual inventory has 390 tests:
Parameter 3, ConsumerRoot 80, Topology 302, SharedExecution 1, RequiredFeature 3
and InvariantReads 1. A separate zero-test optional profile contributes no test
evidence. The ordinary dev build is warning-free; actual non-harness Root and
required_feature, shared_execution and invariant_reads executables all pass.

The existing fixture scheduler consumes those exact all-targets artifacts,
including example harnesses, without another compile. It retains per-case
Topology processes and runs the complete matrix in its CPU-exclusive lane
after all shared jobs. Other task-heavy lanes are finished. Its full terminal
result remains the last execution gate before public integration commit/push
review; the private patch remains preserved and Phase 2/master closure is open.

The canonical fixture gate now terminates successfully: all 390 tests pass,
zero failures and zero bad exits, in 719 seconds. Its CPU-exclusive complete
matrix passes in 247.48 seconds after the shared jobs; its existing assertions
account for all 1,032 declared rows (921 executed, 111 explicitly inapplicable)
with zero red rows. No test limits, matrix width or scheduling policy changed.
All public integration execution and static gates are closed. This clears the
public merge candidate for final reviewer commit/push review on the task branch;
private restoration, qualification and Phase 2/default-branch closure remain open.

## Phase 2 final ancestry refresh: fork-history integration

A further fetch before default-branch integration finds public master at
`399bf81e89e6664c2d9362dabc746bd378b4ce98` (eleven commits and 51 changed paths
since d6f5fed45c) and private main at
`1ba3774617a7274efaea7193e322105993ebea07` (three commits and six paths since
98e3f26179). The qualified public candidate remains 5ef1004018; the current
private patch is staged and recoverable, and its current workspace CI passes
3,202 tests across 96 binaries with 16 existing skips. All 214 doctests pass,
including nine CAD facade examples. Its remaining pure gates are still running.
These results remain evidence for those exact sources, not the new merge.

The new public boundary is World branch custody/history release and Query owner
cleanup, with real correction fork-adoption courts. Topology's descendant-close
courts now require the ancestor's close to complete while descendants keep their
history. Private main changes SDK pins, two progress documents and its installed
close court to that same behavior. Preserve this new ownership/close contract;
do not restore the previous pending-cleanup expectation or replace its pin with
an older SDK. The new gate script adds an existing UI command's compiler-wrapper
workaround; fixture scheduling and matrix assertions are unchanged.

Before further implementation, the reviewer assesses this bounded sequence:

1. Finish the private pure oracle/determinism/sketch gates without changing the
   sources they compile. Their dependency closures contain no public SDK and
   the six new private paths do not change those owners. Keep this honest
   reusable evidence, the accepted unchanged freshness partition and confirmed
   HookMode baseline; preserve both original dirty checkouts.
2. Merge the new public origin/master into the task without automatic commit.
   Review actual World/Query retirement, correction and observation overlaps.
   Relational and all its dependencies are unchanged by these 51 paths, so the
   qualified Native law/prerequisite evidence remains applicable. Qualify the
   changed World owner/certification, Query execution, Graph/correction and
   consumer/ordinary fixture surfaces, plus constitutional/scoped static gates.
   Use the supported canonical fixture scheduling and hold default branches.
3. After current private jobs terminate, preserve its complete current 18-file
   patch in a new exact recoverable stash/backup, fast-forward the clean private
   task to the new main and restore deliberately. Keep the upstream installed
   close court and updated capability facts. Adopt one published qualified
   public merge revision in all seven direct pins and 17 locked packages.
4. Run current SDK-dependent all-targets check, workspace CI and doctests on the
   final private candidate; its CI includes the real history hostile/twin and
   all current CAD courts. Recheck affected static source/doc constraints.
   Reuse only unchanged pure owner evidence, without repeating already green
   unrelated suites. Final reviewer clearance and fresh ancestry checks still
   precede any default-branch advance. Phases 3-6 remain future work.

### Archive fixture compatibility exposed by the broader build

The canonical Query group build exposes two real E0308 errors in the native
path-input certification target: child_admission and retiring_admission assign
ApplicationRelationIntegrity into the owned ApplicationRelationIntegrityRecord.
This is a missed test-fixture migration from this goal's Phase 2 portable archive
DTO change. It is causal compatibility work, not unrelated incoming debt.

The failed broad build's remaining native_operation_bodies portfolio compiler
was positively traced through its sccache/Cargo parents to this task's exact
output directory, then intentionally stopped (PID 77156 only). That unrelated
portfolio is not passing evidence; the failed build and cancellation record
remain preserved. Its partial compiler stream includes both real errors and
successfully emitted current QueryExecution/Graph artifacts.

Before the repair: convert the child's intentionally replaced whole declaration
to its owned record explicitly. In the retiring fixture, mutate only the owned
cardinality field so endpoint/deletion policy and owned law metadata survive.
Keep all existing malformed-archive denials and lawful admission expectations.
Run the complete native_graph_input_guards target after the fix, plus scoped
format/size/function/whitespace checks. It is outside World's dependency closure;
World's existing owner/certification job may finish unchanged. QueryExecution
and Graph do not include these two fixture files, so their exact already-emitted
current artifacts can use the original canonical execution lanes without another
broad portfolio build. Do not call the interrupted whole-group build green.

The two archive fixtures now use the owned boundary correctly, with no changed
denial or lawful-control assertion. Their complete native_graph_input_guards
target passes all 41 tests in 2.26 seconds. The reviewer accepts both exact hunks
and the three additional existing court advisories. Final scoped static checks
cover 47 Rust files: formatting, caps and whitespace pass; all 30 advisories are
accepted, with zero scan errors. Normal boundary/context checks pass and remain
applicable because their only subsequent changes are these fixture conversions.

World passes all 279 unit courts and 42 active certification courts, with three
existing ignores; its public compile-fail contract cases and actual publication
example also pass. The consumer all-targets build passes with its five retained
optional shared_execution harness warnings. Its actual inventory remains 390:
Parameter 3, ConsumerRoot 80, Topology 302, SharedExecution 1, RequiredFeature 3,
InvariantReads 1; the extra zero-test profile contributes no evidence. Current
Query/Graph execution, actual changed correction/ordinary consumer journeys and
the canonical fixture gate remain open. The complete matrix is deferred until
all other task-heavy jobs finish.

The selected current Query artifact passes its full supported chunk/probe lanes:
1,990 passes, one existing subprocess-only ignore, zero failed tests or bad
exits in 878 seconds. The actual authority/resource oracle passes in its
867.66-second chunk. Graph passes all 425 active courts with five existing
ignores in 58.77 seconds (430 names discovered), including the changed correction
surface. Their successfully emitted artifacts remain independent of the two
repaired archive fixtures; the interrupted expanded build is still not green.

The actual changed application_correction main passes, including its intended
injected-unwind recovery. The ordinary consumer build has no diagnostics, and
all four actual non-harness programs pass: consumer-root, required_feature,
shared_execution and invariant_reads. The canonical fixture gate now uses the
current all-targets artifacts without recompiling. All other task-heavy jobs
have finished; its complete matrix remains in the existing CPU-exclusive lane
following shared cases. Its terminal 390-test result is the last public
execution gate before Y2 commit/push review. Private remains clean at 1ba3774,
with exact 18-file stash 57de2e1c01d3f0b482301d252b76252aaaf911fd and verified
external binary backup awaiting that published qualified revision.

The canonical current fixture gate terminates green: all 390 tests pass with
zero failures and zero bad exits in 712 seconds. Its exclusive complete matrix
passes in 303.80 seconds after every shared job, preserving all 1,032 declared
rows (921 executed price-twin rows, 111 named inapplicable rows) and zero red
rows through its existing assertions. No worker assertion, matrix width,
timeout or scheduling policy changed. All Y2 public execution/static gates
are now closed for final reviewer task-branch commit/push clearance. The failed
expanded build and its deliberate unrelated-portfolio cancellation remain
explicit above. Private final SDK restoration/qualification and Phase 2/default
branch closure are still open.

## Phase 2 closeout

All five relation laws are declarable and lower to the existing native contracts:
acyclicity, paired symmetry, scoped uniqueness, same-partition endpoints and
minimum distinct reachability. Enabled laws participate in canonical schema and
schema-bound program meaning, including owned portable records and cold
readmission. Ordinary application commits preserve the original typed law cause
and actual subjects. Checked native prerequisites finish on the original request,
candidate and account before custom preparation, without a second validator,
replacement work model or discarded native prefix.

Topology's applicable minimum-two declaration replaces its duplicate small-ring
check; geometry, endpoint extraction, empty-domain and final closure semantics
retain their owners. The [relation laws guide](../../workspaces/worth-query/crates/worth-query/docs/modeling/relation-laws.md)
contains the compiled law examples and compositions. [Schema validation](../../workspaces/worth-query/crates/worth-query/docs/modeling/schema-validation.md)
links to it while retaining its internal query-bundle legality responsibility.

The qualified published implementation is
`78fde5c68b05a893518caecb180d2709796972e6`. Current affected proof includes
World 279 unit and 42 active certification courts (three existing ignores),
Query 1,990 passes/one existing ignore, Graph 425 passes/five existing ignores,
the repaired native-input target's 41 passes, all 390 canonical consumer tests,
the complete 1,032-row zero-red matrix in its CPU-exclusive lane, actual changed
correction and ordinary consumer programs, and constitutional/scoped static
checks. Native's 1,812 passes/31 existing ignores remain applicable through its
unchanged dependency closure. The broader Query build's two real archive-fixture
errors are fixed; its unrelated portfolio compilation was deliberately stopped
and is not claimed passing. Successfully emitted current Query/Graph artifacts
have verified dependency separation from those fixture-only corrections.

Proprietary adoption is published at
`0397e1d31789a3ae8e330c078a85ac588d384ebe`, coherently pinned to that exact
public implementation. PreviousFeature declares acyclicity and the shared
traversal no longer owns a visited-feature cycle detector. Its real installed
hostile/twin preserves the native refusal and refused checkpoint/current graph,
then commits the exact observed-edge reversal. Five-kind version membership,
body/branch/anchor checks, numerical budgets and the new complete-close contract
remain intact. The final SDK-dependent all-targets check is warning-free;
complete CI passes all 3,202 tests across 96 binaries with 16 existing skips,
and all 214 doctests pass, including nine CAD facade examples. Unchanged oracle,
determinism and sketch-policy proof is retained. Script evidence remains an
honest partition: 126 passes plus six independently confirmed baseline HookMode
failures, preserving the original freshness timeout and one unchanged isolated
passing court. It is not a wholly green script run.

The persistent reviewer gates the actual source, tests, composition and terminal
evidence under qa-loop, qa-tests and code-quality-qa. This closeout records the
qualified Phase 2 result for final integration review. Default branches advance
only after fresh ancestry checks and without force, preserving both original
dirty checkouts and the private local main. Phase 3 walks/reachability is next
after this reviewed Phase 2 is integrated; Phases 3-6 are not claimed complete.

## Phase 3, slice 1: complete native traversal boundary review

Phase 2 is integrated. The next missing foundation is a complete, kind-filtered
native walk. An adversarial graph can have more than 32 hops, cycles, duplicate
routes and unrelated relation kinds at a visited vertex. A product read must
either exhaust its frontier or refuse; it must never certify a truncated answer.

Relational's `StructuralRelationView` owns proposed/committed visibility and
declared kind access. Its original custom meter owns work and retained memory.
The caller's original `ExecutionRequest` owns cancellation/deadline even in the
ordinary serial lane. These authorities must be consumed, not reconstructed from
IDs or replaced with a fresh allowance.

The existing `BoundedStructuralTraversal` is scope widening: it traverses all
declared relation kinds, applies touched-derived quotas and a depth maximum of
32, and returns identity-sorted sets without distances. It is not a product
walk. Its existing policy and accounting remain separate from complete reads;
both policies will use one private native breadth-first driver. Query must not
grow a host loop of one-hop reads. Snapshot projection and leased native query
traversal have separate visibility/resource lifecycles; their consumer adapters
belong to the subsequent operation/declared-query slice.

The native complete result must carry ordered entity visits with kind/distance,
every examined matching physical relation (including edges to already visited
vertices), and actual adjacency/edge/endpoint observations. A refusal carries
observations but no partial successful result. This is the handoff the next
typed invariant slice will consume to admit every entity/relation in preparation
and enforce that frozen admission during evaluation. Those allow-sets are not
native touched inputs or read counters. Declared native access still controls
kind applicability; no copied graph or synthetic footprint becomes authority.

The capability index and installed facades provide one-hop reads and scope
widening but no complete typed reader walk. Complete traversal needs no new
legality witness, substrate authority or public construction bypass. This slice
ends at the native facade and real native graph; it does not claim the Query
reader API, installed footprint court or Phase 3 complete. Material source facts
for this boundary are resolved before implementation planning.

### Slice 1 implementation plan

The native caller uses
`relations.walk_outgoing_from(&[root], relation_kind)?`; incoming
uses the corresponding method. These are complete read operations, not the
bounded scope planner's methods. Roots are included at distance zero; duplicate
roots and cycle/diamond arrivals appear once. Entity visits sort by distance,
then native identity, independently of storage/seed order. This fixes the
otherwise unspecified canonical BFS tie break for the consumer slices.

Boundary correction before the next source change: passing an execution request
to the walk itself would permit fresh cancellation state beside an old meter.
The native view will instead privately carry the exact original request from
its existing planner/context construction; the walk consumes that request with
no substitutable argument. Legacy reads merely carry this field and retain
their existing behavior. Metadata acquired before an access refusal must count
as an acquisition, while pre-acquisition work refusal must not. The complete
result also retains the original meter's memory custody while its owned facts
remain live; returning a result must not release its payer early.

The first focused run exposed an invalid fixture assumption, before its random
traversal could execute: Native rejects duplicate relation creation identities
against existing storage as well as within one proposal. The actual authority
is `relation_creation_admission::reject_existing_relation_identity` and
`relation_identity_scan`, not client-key uniqueness. The generator must author
distinct `(kind, source, target)` pairs and pass that same legal graph to the
independent model. Remove the attempted multi-commit duplicate seeding path;
do not weaken Native identity admission or fabricate parallel records. Self,
back and diamond edges still prove non-discovery footprint retention. Other
focused courts passed, but that run is not a passing slice portfolio.

1. Add the private BFS driver beneath `validation/custom_rule/traversal/`.
   Retention and expansion are policy-owned steps. Cut the current bounded
   walker over to this driver, preserving its seed/frontier/step charges,
   depth limit and public result contract. Existing bounded session tests
   protect that cutover.
2. Add complete traversal data and reader policy as children of that same
   traversal owner, exported through the existing runtime facade. The policy
   filters one installed relation kind, validates visible/readable entities,
   checks the original request at entry and every expansion, and uses the
   original custom meter for reads and pre-growth retention. Complete traversal
   has no touched-derived or constant depth limit. Typed refusal retains reads
   and discards output; successful output records all matching edges even when
   they discover no new entity. Complexity is O((v+a) log(v+a)) time and
   O(v+e) retained space, where a includes candidates at visited vertices and e
   is matching physical edges; unrelated disconnected entities do no work.
3. Add focused real-native-graph proofs under `traversal/tests/`: seeded
   independent BFS comparison (both directions, multiple roots, self/cyclic/
   diamond/disconnected shapes), a 1,000-edge chain, access filtering, unchanged
   observations with 10 versus 10,000 disconnected entities, original serial
   and leased cancellation/deadline versus funded twins, and work/memory
   exhaustion without partial success. Test the shared driver only through
   these production readers and the existing bounded courts.
4. Run scoped formatting, dirty line caps/function advisories, focused native
   traversal/custom-rule tests, broader Relational tests appropriate to the
   shared cutover, and normal boundary-check/agent-context. Gate the patch and
   final evidence with the persistent reviewer using qa-loop, qa-tests and
   code-quality-qa; commit only after clearance.

The native result carries facts, never Query identity authority. The next slice
adds endorelation-typed invariant walk/reaches/closure, preserving binding,
candidate affinity and Proposed/Committed posture, then the installed deep-edge
footprint court and mandatory first-hop-only mutant. Operation/declared paths,
consumer migrations, compiled feature documentation and Phase 3 merge remain
later work. No unresolved blocker or new permission is required for this slice.

### Slice 1 implementation and qualification

Complete structural reads now consume the request sealed on their native view,
preserve original work/memory causes, retain result custody, and expose actual
boundary acquisitions on success and refusal. The shared private driver stays
under `custom_rule/traversal`: its two consumers share that view/access/meter
lifecycle. The typed invariant adapter will consume these facts. Operation and
declared-query reads instead belong to the existing native pinned-query
executor, whose checkpoint, packet memory, visibility and fragment-reduction
lifecycle must remain authoritative. Their later boundary review must reconcile
its root-seed/via-edge ordering with the promised distance/identity order; it
must not import custom-rule internals or assume the existing snapshot traversal
already satisfies the product contract.

The final native artifact passes all 28 focused custom-rule courts and the full
Relational library: 1,819 passes, zero failures and 31 existing ignores. The seven
new courts cover seeded independent BFS in both directions and multiple roots,
non-discovery edges, 1,000 hops and disconnected growth, actual candidate versus
committed walks during an ordinary retarget, acquired-but-denied metadata,
original serial/leased interruption with completing twins, work/memory denial,
and owned-result payer retention/release. The earlier 27-pass/one-invalid-fixture
run remains a failed setup run, corrected by legal tuple-distinct generation.

Downstream Query compilation and agent-context check pass. All 17 changed Rust
files pass the dirty line cap and formatting; whitespace checks pass, and the
persistent reviewer accepts the final twelve function advisories by cohesion
with zero scan errors. Source, tests and composition are reviewed under qa-loop,
qa-tests and code-quality-qa. Normal boundary-check also passes with valid Cargo
topology. All slice checks are terminal and passing, and the persistent reviewer
clears this native foundation slice to commit. Phase 3 is not complete.
