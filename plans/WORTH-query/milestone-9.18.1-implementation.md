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
