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

## Phase 3, slice 2: typed invariant reachability boundary review

Slice 1 is committed at `8c3abc1bb6e8356400d4290f631b1b06846f0655`.
The next complete surface is the installed invariant reader's walk, reaches and
multi-root closure. The adversarial condition is a deep edit whose rule still
appears healthy if any visited entity or physical relation is absent from
prepared authority. A complete native traversal must not become an unfunded
typed materialization, a partial answer, or a way to reuse another candidate's
identity.

Native structural views remain authoritative for candidate/before-image truth,
declared kind access, the sealed original request, work and memory. The typed
reader owns binding, candidate affinity, Proposed/Committed posture, expected
kind checks, read observation and preparation/evaluation admission. Query will
consume the complete native result, not rediscover its graph with one-hop calls.
It will validate an endorelation and every root before disclosure, record native
acquisitions even if later wrapping/admission fails, then admit every entity and
every matching physical edge, including non-discovery edges. Evaluation checks
the frozen admission before returning typed facts. A native-backed result owns
the native result and materializes fixed-size typed entities/edges through
borrowed iterators; it needs no second result Vec or detached payer.

The existing Query planning and frozen BTree sets are distinct allocations.
`freeze` currently clones them infallibly, and planning constructs its own Arc.
Their backing must be admitted before growth on the original native payer,
including coexistence during freezing. Native will expose a borrowed, typed
read-budget view of that same meter/request, without a constructor, allowance,
counter or graph mutation door. Its claims preserve interruption and memory
causes rather than bool-only cause loss. Query admission construction, insertions
and freezing become fallible; registration propagates failure without an empty
scope or unchecked fallback. Existing one-hop admission sites use this same
funded route. The native adapter already drops owned scope before its meter;
public retained walks keep their own native custody.

Review also found an input gap in the complete native driver: duplicate raw
seeds bypass unique-entity retention, so their membership work and safe points
are missing. Before exposing closure, add a required private input step before
deduplication. Complete policy checks/charges every raw root. Bounded policy
preserves its existing up-front raw-seed charge and uses a no-op input step.
Input work must not masquerade as entity metadata reads.

Direction is existing shared meaning, not new authority. There is no generic
foundational relation direction. `ApplicationQueryRootPathDirection` already
serves operation creation/path sources and signing in addition to declared
queries (29 public repository files); its current name is narrower than its
meaning. Move that single Forward/Reverse value to foundational
`relations/direction.rs` as `RelationDirection`, migrate existing uses and
exports, and leave no alias or parallel enum. Preserve variant order, serde
spelling and canonical/signing bytes. Foundation owns portable meaning; Native
and Query continue to own execution and authority. No private crate/doc use of
the old name was found.

Native custom applicability is declared-kind closure; Query's prepared sets
enforce read authority. There is no retained per-rule verdict cache. The
mandatory footprint mutant must therefore fail an exact fresh custom-violation
expectation, usually by producing an OutsidePreparedScope execution refusal;
it must not be described as accepting a stale cached verdict. Reverse discovery
from deep touched endpoints followed by forward discovery can accidentally
cover a three-edge chain even under a first-hop-only mutant. The court instead
uses meaningful stored root membership of Node entities, on a distinct relation
kind, to acquire a genuinely typed Node root without enumerating the tested
path or rebinding identities. Authentication and membership edges are not the
walked edges. One ordinary action changes only the third-hop path edge to a
blocked endpoint. A separate activation-field action touches only the root, so
deep entity admission cannot be supplied incidentally by touched endpoints.
Separate entity and relation mutants test the actual admission handoff. Both
hostile and lawful twins must reach the real deep evaluation; exact typed
refusal, hook activity and independently observed committed truth distinguish
correct behavior from generic denial or broken setup.

The current account lease/refusal fixtures have heterogeneous relations and
different responsibilities. A dedicated `application_attempt/invariant_walks`
sibling is the proper installed court, reusing ordinary installation,
authentication and action patterns. No new certification construction door,
all-entity scan, fake identity, cached verdict or committed/proposed cast is
needed. The persistent reviewer has accepted these boundaries and the input,
budget and mutant refinements. The material facts required for planning are
resolved; operation/declared reads retain their separate pinned-native owner.

### Slice 2 implementation plan

The installed rule's caller path is:

```rust,ignore
let history = view.walk(&self.previous, RelationDirection::Forward, &tip)?;
for step in &history {
    let feature = step.entity();
    let distance = step.distance();
}
let depends = view.reaches(&self.previous, &later, &earlier)?;
let ancestry = view.closure(&self.previous, RelationDirection::Forward, &roots)?;
```

`previous` is the resolver-issued endorelation binding. Forward follows stored
source to target; reverse follows target to source. Walk and closure include
unique roots at distance zero and order visits by minimum distance then native
identity. Reachability is reflexive on a valid local entity. The owned result
offers borrowed iteration, entities and physical relations; it retains native
custody rather than exposing an unfunded owned collection.

1. Repair native raw-root work/safe points before Query depends on closure.
   Add the private driver input step, preserve bounded up-front charges, and
   distinguish seed versus acquired-entity test events on the existing owned
   observer. Prove duplicate-input work/cancellation refusal and funded twins
   on real native graphs without inventing metadata reads.
2. Add `custom_rule/read_budget.rs`: a borrowed original-meter/request handle
   with typed checkpoint and retention admission. Only native view/planner
   getters construct it. Share original cause classification with complete
   reads; no new resource policy, counter, constructor or graph authority.
3. Move the existing direction meaning into foundational
   `relations/{mod,direction}.rs`, export `RelationDirection`, and mechanically
   migrate the old declaration enum and its real users. Preserve exact variants,
   order, serialized spelling and canonical encodings; delete the displaced
   enum/export and add no compatibility alias. Compile current canonical,
   signing, archive and external graph-input callers. The historical
   foundational Phase 1 responsibility report is not a registry for every later
   value family and does not acquire unrelated changes.
4. Make Query prepared admission funded and fallible. Charge planning Arc and
   BTree growth before allocation; admit frozen Arc plus cloned set backing
   before copying. Pass the same native read budget at all existing admission
   sites. Update scope-planner construction, `freeze_admission` and ordinary/
   checked registration propagation together, with no empty or ordinary-hook
   fallback. Name collection bounds in the owning admission module and preserve
   nested scope/payer drop order.
5. Add `application_invariant/typed_view/walk.rs` for orchestration and a
   subordinate native-backed result/iterator owner. Check installed binding,
   endorelation meaning, root binding/kind/candidate/posture, then lower once to
   native direction-specific complete traversal. Record native observations on
   both success and error before later admission can fail. Validate every kind
   and register every visited entity and physical relation before exposing the
   typed result. Add `walk`, multi-root `closure` and forward `reaches`; do not
   loop one-hop methods or copy a graph. Mint result identities with the exact
   binding, candidate affinity and view posture. Factor cheap identity checking
   from existing metadata checking only where that preserves old read behavior.
6. Carry typed interruption and original memory causes in the access-denial
   surface; bulk-fold native boundary observations into the existing shared
   `WorthQueryInvariantProjectionWork`. Export result and direction through the
   existing audience facades. No observation becomes an allowance or grant.
7. Build the cohesive `tests/application_attempt/invariant_walks/` family:
   schema, installed world, ordinary action preparation, real checked rule,
   footprint journeys and reader API/authority journeys as named children.
   Root membership targets a root role of the same Node kind, with an explicit
   domain-owned grouping relation, not a kind cast or a derived reachability
   cache. The edge action unlinks its actual observed third-hop relation and
   links the new target (the ordinary effect builder has no retarget shortcut).
   The activation action edits only the root field. Require exact decoded
   violation/subjects, preparation and evaluation activity, unchanged checkpoint
   and fresh graph after denial, and lawful twins whose deep checks also ran.
   Run separate entity-only and relation-only first-hop admission mutations at
   the real handoff. Restore sources and invalidate/rebuild the affected target
   before final proof; mutation artifacts are never final evidence.
8. Protect the typed API with actual installed direction/closure/reflexive and
   disconnected reachability, cyclic/diamond non-discovery scope, foreign
   binding/view, original cancellation/resource refusal, 1,000-step and
   fixed-walk 10/10,000-unrelated read observations. Keep read-only diagnostic
   probes in their own rule/admission frame so they cannot mask the footprint
   rule. Native independent random BFS proof remains its algorithm authority;
   Query evidence tests the real typed handoff and prepared authority.
9. Clarify Phase 3 specification root/direction/footprint semantics to match
   these real owners; add feature guidance and a compiled real-rule snippet,
   link it from the capability index and existing invariant guide. Document the
   source API direction name change without implying a wire change. Operation,
   declared closure and consumer migrations remain later slices of this phase.
10. Verify with the real application-build wrapper and toolchain 1.94: focused
    Native custom-rule tests; foundational direction and existing value tests;
    declaration/installation/archive libraries; focused installed Query courts,
    then the affected installed invariant, refusal, checked-hook, read-observation,
    signing and root-path families from the emitted Query artifact; targeted
    native graph-input guards and any actually affected public invariant example.
    Use selected Cargo package/target builds, not the known overbroad Query
    workspace portfolio. Run formatting, dirty line caps, function review,
    whitespace, normal boundary-check and agent-context. Gate implementation,
    both mutant kills and final results with the persistent reviewer under all
    three QA skills; commit only the cleared slice.

There is no undiscoverable product choice or new permission requirement. No
operation-reader or declared-query walk, orbit, aggregate identity or phase
merge is claimed by this slice. Any new material authority fact revises this
brief and plan before dependent implementation continues.


### Slice 2 verification refinement

Compile time constrains proof selection. The current foundational suite (73),
Native suite (1,820 passed; 31 existing ignored), and declaration (464),
installation (343) and archive (122) suites are green on the current production
source. Fixture-only corrections do not invalidate those owner results. Reuse
that evidence and the actual emitted Query artifact; do not repeat the owner
builds or expand to unrelated authority, Topology, Bank or private-CAD portfolios.
The required normal boundary-check, agent-context and dirty-scope checks remain.

The installed eight courts cover the new typed handoff. Select affected existing
invariant preparation, typed refusal, checked-hook, read-observation, signing
and root-path regressions from the actual binary inventory after the final
restored build. Public native graph-input guards qualify the moved direction
value. The real installed rule compiles the guide's callback; any additional
consumer example must protect an actual changed consumer boundary.

Use one temporary instrumented binary for the two independent admission faults,
then one restored clean build. A cfg(test) mode belongs to each WalkSafety
planning receiver. The entity fault omits only vertices at distance greater
than one; the relation fault omits only edges beyond the roots' first hop.
Native traversal, read observations, domain verdicts and frozen evaluation
checks remain complete and unchanged. The original eight mode-zero courts must
pass in that instrumented binary. Each separately selected faulty court must
then fail its exact domain-verdict assertion through OutsidePreparedScope and
NoDomainRefusal. No diagnostic admission frame may rescue either fault.

All temporary fields, methods, branches, wrappers and debug output are restored
byte-for-byte, without restoring file timestamps. Only the subsequent clean
artifact qualifies final runtime evidence. The persistent reviewer accepted
this receiver-owned fault design and the focused regression boundary under the
three QA skills. No global, environment or thread-local mutation arm is added.


### Slice 2 installed-fixture boundary correction

The first complete executable exposed two setup refusals before the new API
courts could run. Installation correctly refuses a dynamically extended
declaration for the same WalkSchema marker: publication compares the installed
schema against that marker's canonical declaration. Use one canonical schema
that declares Safety and Probe, bind both factories, and give ordinary footprint
worlds a permanently disabled Probe. Disabled preparation returns an empty scope
before any view/read; disabled evaluation passes. Enabled diagnostics still own
an independent frozen frame. Enablement is configuration; arming only controls
observations/callbacks. No StaleInstalledSchema check or admission is bypassed.

The edge action must seal the exact facts its opaque unlink read observes.
Adjacency and pair-relation keys are distinct. Resolve the old endpoint inside
the public projection and require_decision_relation for the exact source/old
pair. Resolve and require the replacement entity as before. Remove manual
entity observations that were absent from the sealed decision keys; retain the
actual observe_relation call for the opaque unlink token. Completion replays the
same sealed entity, identity and relation facts. This follows the existing
public grant-revocation pattern and preserves exact decision-key equality.

The persistent reviewer confirmed these boundaries from publication and
read_set/completion owners. Apply both fixture fixes before the next build.
Batch them with the accepted temporary admission instrumentation: that binary
must pass all eight original mode-zero courts before either fault is considered
meaningful. Restore and build the final source once after the independent fault
failures. Reuse unaffected owner evidence.


### Slice 2 proof recorded before final qualification

The combined temporary build passed all eight original mode-zero installed
courts in 40.19 seconds. They exercised exact deep edge/activation violations
and lawful twins, typed cycle/diamond/direction/closure/reachability semantics,
posture refusal, a real same-installation sibling candidate with local control,
a 1,000-step chain, walk-only read deltas with 10 versus 10,000 other group
members, and original serial/leased work and cancellation with funded twins.
The preparation of the diagnostic rule enumerates all group members; its
locality assertion concerns only the measured complete evaluation-walk call.

The independently armed entity fault failed at OutsidePreparedScope(entity);
the relation fault failed at OutsidePreparedScope(relation). Both failed the
unchanged exact-domain-refusal assertion as NoDomainRefusal with test exit 101.
They did not simulate stale cached acceptance or change native traversal.
The seven temporary source files were restored byte-for-byte with SHA-256
records and no conflicts. The subsequent clean build, restored positive courts,
focused existing regressions and structural gates are still required.

Current unchanged-owner evidence is 73 foundational tests, 1,820 Native tests
with 31 existing ignored, 464 declaration tests, 343 installation tests, 122
archive tests, and 41 public native graph-input guard tests, all green. The
shared direction's spelling/order and current canonical/archive consumers are
qualified. The guide's callback matches the actual rule's Rust tokens after
excluding only its diagnostic observer, and all five local links resolve.

Formatting, dirty line caps and whitespace pass across 80 Rust files. The
persistent reviewer accepted all 39 function advisories in causal scope and
conditionally cleared source under qa-loop, qa-tests and code-quality-qa.
That is not final commit clearance; final runtime, selected regression,
structural and documentation evidence remains pending.


### Slice 2 restored runtime qualification

The restored-source build succeeded in 414 seconds with Rust 1.94 incremental
compilation disabled. All eight installed courts passed again on that clean
artifact in 39.67 seconds, with no temporary fault fields, methods, wrappers or
debug output. Fifty affected existing Query regressions then passed in 9.98
seconds from the same artifact: invariant selection/installation, failure and
validation transport, checked hooks, typed refusals, snapshot read observations,
root selection and pinned root guards. No additional compilation was needed for
that selection. Public signing/direction compatibility is also covered by the
already-green declaration, installation, archive and graph-input owners.

The combined instrumented build took 413 seconds and proved the two independent
faults after its eight original mode-zero courts passed. Both source restoration
and the final artifact are qualified; neither temporary artifact is substituted
for clean-source proof. Rust 1.94's earlier evaluate_obligation incremental ICE
was retained as a failed build log and resolved by disabling incremental
compilation for these test builds, without altering production source or
weakening tests. All builds used the real application-build wrapper; the two
mutation-driver builds used --locked.

The persistent reviewer cleared the implementation, both fault failures, all 39
scoped function advisories and documentation under the three QA skills. The
required normal boundary-check and agent-context are still running. Commit
clearance remains pending those structural results and the final evidence pass.


### Slice 2 structural qualification

The normal boundary-check passed (602.47 seconds, Road 1 Cargo topology valid)
and agent-context check passed (0.56 seconds) on the restored source. Final
formatting, dirty line caps and whitespace were green across all 80 Rust files;
the 39 function advisories are explicitly reviewed and accepted. The final
qualification set is the clean eight installed courts, 50 affected Query
regressions, two independently killed admission faults with exact restoration,
73 foundational, 1,820 Native (31 existing ignored), 464 declaration, 343
installation, 122 archive, 41 public graph-input tests, and callback/link proof.
No unrelated full Query, Topology, Graph, Bank or private-CAD portfolio was rerun.

Only final persistent-reviewer commit clearance remains. This closes the
installed-invariant walk slice, not Phase 3: operation/snapshot/declared closure
surfaces and downstream history adoption remain subsequent work. No Phase 3
default-branch merge or private SDK-pin update is authorized by slice completion.


### Slice 2 clearance

The persistent reviewer granted final commit clearance under qa-loop, qa-tests
and code-quality-qa after the restored runtime, independent faults, focused
regressions, documentation and required structural evidence were complete.
No material in-scope finding remains. Commit the installed-invariant walk slice
on codex/worth-query-9.18.1. Phase 3 remains open; its default integration and
private adoption follow the remaining operation/declared work and phase proof.


## Phase 3, Slice 3: Pinned Native Walks And Snapshot Projections

### Boundary brief

The next unfinished boundary is a complete pinned read consumed by the snapshot
projection reader. Operation decision attribution and portable declared-query
closure change different authority/identity contracts and remain subsequent
slices. This slice must end in a real snapshot adapter, not a substrate stub.

The entering truth is the owner-created RelationalPinnedRead: its immutable
selected root, schema, generation/version and exact context, together with the
original ExecutionRequest. Existing query execution consults that request and
enters its scope. Existing leased traversal already owns FIFO/cycle dedup and
physical edge visitation. Complete forward connectivity exists, but incoming
neighborhood is one hop. Legacy traversal order is depth/root/via/entity, its
depth is u32, and checked reduction discards the visit keys. Emitted-record and
packet counts are not acquisition observations. Neither legacy result vectors
nor QueryLeasedReadOutcome retain their output reservation. Those weaker
representations cannot certify the new product walk's distance, counts or payer.

The new pinned path may own a single global multi-source frontier, complete
Forward/Reverse traversal, checked usize distances, ordering by minimum distance
then EntityId, actual acquisition observations, and retained result memory. One
global packet avoids repeated overlap work across the legacy four-seed packet
partition; it implies neither parallel traversal nor a four-root product cap.
Share private FIFO/visited mechanics with the existing pinned traversal owner.
Keep custom_rule's separate driver/payer private, and preserve legacy query
scope, provenance order, canonical digest and result contracts.

Metadata authority already exists in projection/borrowed_records and
borrowed_adjacency. Extract/share their exact-root core with the pinned owner;
do not duplicate live/generation/schema decisions or clone full aspect records
for structural reads. Ordinary frontier reads are explicitly live-only; audit
retained edges belong to their separately named API. Require live,
generation-correct endpoints before exposing visits, filter audit-retained
relations, and refuse inconsistent matching live endpoints rather than certify
a partial successful path. Native storage materialization confers no principal
authorization. Query must preserve the same installed layout, local identity,
branch and read/disclosure contracts as its one-hop snapshot reads.

ExecutionMemoryReservation is an owned, non-Clone ticket on the original
serial/leased lineage. ExecutionRequest::run_measured_result already lends
MapKernelContext plus ExecutionResultMemory, admits cooperating allocation
before growth, measures completion and leaves that exact reservation holding
the output. The final native checkpoint choice below preserves optional accounting; do not
rewrite generic legacy query resource transport or use the measured path to
manufacture an absent contract. Scratch,
result and temporary overlap must remain funded through their real lifetimes.
Ordered scratch maps can establish canonical order, then allocate final vectors
once at known sizes. This avoids the legacy exact-per-record growth helper's
quadratic copying without changing that unrelated growth policy.

The snapshot adapter validates its installed endorelation and root authority,
lowers once, folds actual Native observations on success or refusal, and mints
typed identities directly from already-read metadata. Calling endpoint or
one-hop readers for every result would be a second acquisition path. Borrowed
iteration preserves the original projection authority cookie and root name;
no raw full Native record or metadata-derived authority is exposed.

Realized entity scope is separate retained state: it moves from the reader to
the completed projection/snapshot and can outlive a returned walk. Its own
reservation must move with WorthQueryRealizedProjectionScope. New entries mean
physical insertions into entity_ids, not broad contains(), which also recognizes
carried computation ranges. record() also writes an optional per-call set, so
both physical trees must be admitted when present. A from-empty nodes<=keys
bound does not bound incremental allocations in an existing tree: ancestor
splits need an explicit height/insertion bound. Native result custody and scope
backing remain separate owners on the same original account.

Reader completion already carries native_read_stop, retention_exhausted and
local-work exhaustion. A swallowed acquisition/admission error must still veto
projection completion with its original cause and previously performed reads.
No empty scope, false reaches answer or partial walk substitutes for refusal.

Later boundaries are concrete: operation reads distinguish capability from
sealed decision facts; their existing exact-key completion must consume genuine
walk dependency evidence. Declared root paths need an authored closure step,
canonical/archive/installation meaning, receipts and actual adjacency revision
sources. Current RootPathSourceBuilder clones owned sets per row, so closure
must avoid multiplying a complete dependency footprint by every returned row.
Private basis/thread.rs currently iterates previous() across three reader
families and performs domain body/branch/name checks. Adoption must replace the
structural loop while preserving those domain judgments. None is claimed here.


### Boundary qualification: optional accounting and scope growth

Further source review narrows the execution choice. run_measured_result is the
right retained-result mechanism for an accounted opaque callback, but its
for_run path must not manufacture accounting for a genuinely unaccounted Native
request. The dedicated pinned path instead uses the existing
ExecutionRequest::run_native_with_checkpoint. Its for_native posture retains
real serial/leased backing and accounted ancestors, charges the parent and
performs a final poll. Cooperative original reservations fund scratch and
output before allocation; the returned Native result owns its output ticket.
Do not create a SerialMemoryBudget or treat a physical report as reads.
Only the exact absent-active-memory case may remain unreserved for a genuinely
unaccounted request; other lineage/admission/memory refusals remain failures.
Effective inherited accounting must be respected as well as the request's own
posture. Small no-account and accounted-ancestor controls protect this boundary.

Root lowering, Native traversal and realized-scope admission remain inside one
original active extent. Reusing a request value in separate completed serial
stages could refresh a ceiling; that is insufficient. Actual read totals live
in owner-local state outside a callback's returned payload, because final poll
can discard that payload while correctly refusing handoff.

For scope-set growth, after-total N keys imply at most floor(log2(N+1))
nonempty levels under the weak two-child invariant. A new insertion needs at
most one split sibling per level and a new root/initial leaf; levels+2 nodes is
a conservative bound. Quote NEW physical entries independently in entity_ids
and the optional call set, using the Rust1.94 node bound
11*sizeof(EntityId)+14*sizeof(usize). Check old+new, N+1, products and cumulative
reservation arithmetic before growth. Keep the reusable cumulative ticket in
the realized scope, with both sets dropping before it. The persistent reviewer
confirmed the bound and these original-accounting constraints.

### Slice 3 implementation plan

The new user path is a snapshot projection using the existing relation ref:

```rust,ignore
let start = reader.resolve_entity(FeatureKey::reference(), key)?;
let history = reader.walk(PreviousFeature::reference(), RelationDirection::Forward, &start)?;
for step in &history {
    inspect(step.entity(), step.distance());
}
let depends = reader.reaches(PreviousFeature::reference(), &later, &earlier)?;
let shared = reader.closure(PreviousFeature::reference(), RelationDirection::Forward, &roots)?;
```

This is the existing snapshot projection authority and identity family, distinct
from installed proposed/committed invariant handles. Operation reader methods,
sealed decision attribution and declared path closure are not exposed here.

1. Reuse VisibilityProjectionView::from_pinned, an existing private Native
   conversion that already retains the exact immutable basis/schema/symbols for
   both snapshot and observation pins. The new pinned path delegates to the
   existing borrowed entity/relation metadata and selected-kind adjacency core.
   No extraction or duplicate metadata logic is needed. Keep public projection
   methods unchanged and retain generation/schema/live and no-aspect semantics.
2. Introduce a private FIFO/visited frontier under query_execution and use it
   in both legacy leased_traversal and the new complete policy. Preserve legacy
   root/via ordering, depth representation, packetization, digests and result
   behavior. New complete reads use one global root set and checked usize depth,
   canonicalize actual metadata identities, and include every live matching
   physical relation, including nondiscovery edges. No custom_rule driver moves.
3. Add query_execution/pinned_walk with separate execution, traversal, retained
   outcome and allocation-extent responsibilities. The pinned receiver seals the
   source; the original native checkpoint extent owns work/controls. Record
   actual list/candidate/metadata acquisitions at their real boundaries, retaining
   totals on every refusal. Keep local optional projection-read limits separate
   from diagnostic counts and original execution accounting. No partial result
   escapes cancellation, deadline, allocation, kind or endpoint failure.
4. Admit ordered scratch sets/maps/queue before growth on original optional
   reservations. Build final canonical step/relation vectors once at known
   sizes, retaining overlap until scratch backing is dropped. Return a lean,
   non-Clone result whose vectors precede its owned output reservation in drop
   order. Use metadata only, no full aspect payload clones. Preserve absent
   accounting and any real accounted ancestor; do not reset serial work.
5. Add invariant_projection/walk with root lowering, Native orchestration and a
   subordinate typed result/iterator owner. Validate relation installation,
   endorelation meaning and every root's existing authority cookie/kind before
   graph reads. Fund root-ID backing and checkpoint lowering on the same original
   extent; Native is called once. Fold performed reads even when acquisition or
   later admission fails, and latch the original completion stop. Mint wrappers
   directly from trusted already-acquired metadata, sharing the existing root
   name/cookie. Empty closures allocate no identity names.
6. Add realized_scope/retention_extent for the checked insertion bound and retain
   a reusable original reservation in WorthQueryRealizedProjectionScope. Count
   actual new entries in each physical set; reserve before calling record().
   Checkpoint every new-entry counting and insertion loop on the same original
   context. The snapshot retains its own scope payment after a walk drops. Keep
   Native result and realized-set custody separate; never wrap a walk as a fake
   ComputationCallSequence or make a graph read to rebuild scope.
7. Expose the typed snapshot result through the existing projection audience
   facade. Extend the feature guidance directly (without feature-doc-writer),
   capability index and a real compiled snapshot callback. Clearly retain the
   distinction from installed invariant views and from future operation/declared
   surfaces. No authority/disclosure promise exceeds existing one-hop reads.
8. Build the full Native and snapshot source/tests before the large gates.
   Native proof uses a small independent graph with cycle/self/diamond/overlap,
   Forward/Reverse, duplicate/permuted five-plus roots, canonical distances and
   physical edges. Add old-pin/new-pin commit/retirement and explicit audit
   separation; original serial/leased in-progress stop and funded twins;
   genuinely unaccounted plus accounted-ancestor behavior; held-result
   competing-original-reservation/drop custody. Reuse the existing pinned
   commit/frontier and fragment reduction seams rather than repeat every
   accepted structural BFS proof.
9. Snapshot courts use the real installed graph/authority and test canonical
   reads, local/foreign identity, swallowed-stop refusal, actual observations,
   result/snapshot scope custody and a cumulative serial budget whose whole
   work exceeds either individual stage. Prefer a few cohesive cases and
   completing twins over a new portfolio. No fake native records or identities.
10. Compile one scoped Native owner selection and one Query execution selection
    with the real build wrapper/toolchain. Reuse their emitted artifacts for the
    focused pinned/metadata/legacy traversal/reduction and snapshot/read/refusal
    regressions. Preserve the proven Rust1.94 nonincremental workaround where
    needed. No foundational, archive, installation, Graph/Topology/private-CAD
    portfolios are repeated for unchanged boundaries. Run normal boundary-check,
    agent-context, dirty caps/fmt/whitespace and scoped advisory review once after
    stable source. Gate the complete slice with the persistent reviewer under
    qa-loop, qa-tests and code-quality-qa, then commit it before the next slice.

There is no product preference or approval question. Native lifetime/accounting
and snapshot scope are the proof-bearing boundary. Any contradictory source or
runtime evidence revises this brief before dependent code continues.


The persistent reviewer approved this Slice 3 plan under all three QA skills.
The final implementation uses native checkpoints, with original checkpoints in
scope-membership counting and insertion as well as lowering and traversal.


Implementation source discovery found the existing private
VisibilityProjectionView::from_pinned<Snapshot> conversion. It clones immutable
custody, not rows, and already delegates all borrowed metadata/adjacency reads
to their one owner. Reuse it instead of extracting those cores unnecessarily.
The accepted authority boundary is unchanged and the implementation/gate scope
is smaller. No snapshot-only public door is added to observation-backed pins.


### Slice 3 acquisition-boundary correction

Incremental persistent review identified that the existing borrowed adjacency
`List` event is pre-admission, not acquisition. Preserve that event's legacy
meaning, and add `Acquired` after the selected list lease. The new Native walk
counts there, including empty lists and lists whose later edge scan stops.
Legacy callers ignore the new notification before charging work. Entity and
relation observations count acquired metadata only; failed probes still spend
original/private work. Iterator temporary payment remains held through edge
callbacks and scratch growth until the borrowed iterator drops. These source
findings refine steps 3 and 4 without changing the accepted authority model.

The implementation and its focused tests are drafted; no completion is claimed
before build/runtime evidence, the normal boundary/context gates and final
persistent reviewer clearance.


### Slice 3 optional-accounting execution finding

The first focused Native run passed 30 of 32 courts, including all selected
legacy regressions, the new 1,000-hop forward/reverse court, canonical graph
facts and leased cancellation/custody. Its genuinely unaccounted serial case
exposed that even `MapKernelContext::checkpoint(0)` requires accounting. The
new Native/snapshot/scope checkpoint helpers must call checkpoint(units) only
when the original context has accounting, and otherwise poll its original
controls. Their independent checked local read-work meter still advances;
real accounted ancestors retain their metered branch. No generic execution or
legacy traversal policy changes are needed. The persistent reviewer confirmed
this source-backed correction.

The other failure was an overstrong test assertion: an outer execution scope
may return a correctly typed inner Native WorkExhausted refusal. Assert that
exact original cause and absence of a walk, with a funded twin, rather than
requiring a duplicate outer failure. The current Query build remains frozen
until it finishes; corrections will be batched with its actual runtime results.


### Slice 3 current Native qualification

The corrected Native build 3 passed in 182.88 seconds using the real build-ID
wrapper, locked Rust1.94 toolchain and nonincremental workaround. Its emitted
artifact passed all 32 selected owner/regression courts (zero failed/ignored,
2.18 seconds). They include the six new complete-walk courts, all five existing
pinned custody/commit/audit courts, canonical legacy neighborhood and parallel
traversal/reduction, packet stops/storage, and budgeted authorization controls.
The previously failing genuinely unaccounted request and exact inner original
work-exhaustion cases now pass, including the identical 4,096-root funded twin.
No full Native/Query/Topology/private portfolio was repeated.

Query build 2 identified only test placement errors: the three new courts had
called an owner-private projection-limit entrance from a neighboring fixture.
Move them into invariant_projection/walk/tests.rs. Reuse the same real installed
schema/factories through cfg(test) visibility bounded to primary_graph, rather
than widening the production entrance or adding a test constructor. The public
compiled callback remains the same authored code and its guide link follows
the move. The persistent reviewer cleared this correction, exact refusal-owner
assertions, original-accounting branches, all scoped function advisories and
Native facade exports. Query runtime and final normal structural evidence are
still required before slice commit.


### Slice 3 runtime and review qualification

The final Query build 3 passed in 635.03 seconds with the same real build-ID
wrapper, locked Rust1.94 and nonincremental setting. Its emitted artifact passed
19 focused courts, zero failed/ignored, in 7.94 seconds: the three new snapshot
walk courts, two field-acquisition courts, three one-hop acquisition/stop courts,
and eleven existing snapshot projection/read/identity/lifecycle courts. Combined
with the current Native artifact's 32 passes, this proves the final source's
canonical full traversal, old pinned truth, original cancellation, genuinely
absent accounting, ancestor/leased payment, original cumulative work exhaustion,
result/snapshot scope custody and the affected legacy paths. Both snapshot walk
and reaches swallowed-cancellation cases preserve exact Cancelled with no later
acquisitions; the cumulative 4,096-root case preserves exact WorkExhausted and
its identical uncapped twin completes. Neither court requires a redundant outer
failure after the responsible inner owner already returned the original cause.

Final formatting, dirty line caps, function scrutiny and whitespace checks pass
on all 30 scoped Rust files. The persistent reviewer accepted all 20 function
advisories: new functions retain one read/accounting responsibility, and the
newly included fixture/transaction bodies changed only visibility or reflow.
No file exceeds 400 lines. The direct feature guide is source-linked to the
compiled snapshot callback; its six relative links and six API tokens resolve
against the passing artifact. No feature-doc-writer skill was used.

The official boundary snapshot regeneration passed in 435.15 seconds. Its only
logical inventory change is the three intended public snapshot walk result,
step and iterator types; no facade documentation debt was added. The initial
normal guard's expected inventory drift and three missing result rustdocs were
corrected explicitly. The post-update normal boundary/context run must still
finish before final commit clearance. Logs, commands and machine-readable
results use the query-9.18.1-phase3-slice3 prefix under the task's targets folder.

All three QA reviews have cleared production source, tests, documentation,
exports, generated inventory, scoped advisories and current runtime evidence.
Operation capability/decision walks, declared closure/wire/receipt integration,
private adoption and Phase 3 master integration remain subsequent work. This
slice does not claim the whole phase is complete.


### Slice 3 final structural qualification

The post-update NORMAL boundary-check passed in 507.36 seconds and reported
Road 1 Cargo topology valid. Agent-context check passed in 1.21 seconds. This
is the ordinary enforcement mode on the final source, not snapshot-update or
sealed-source-only evidence. The generated diff remains exactly three facade
exports; regeneration added no documentation debt or dependency-graph change.
All runtime, direct-guide/source-link, dirty formatting/size/advisory/whitespace
and structural requirements for this slice are now green. The persistent reviewer
granted final commit clearance under qa-loop, qa-tests and code-quality-qa with
no remaining material in-scope finding. This slice is complete; Phase 3 remains
open and does not yet integrate into master.


## Phase 3, Slice 4: Operation Walks And Recorded Dependencies

### Initial boundary review (implementation plan not yet written)

The next unfinished handoff is operation use of the completed pinned snapshot
walk. It must include both the operation capability reader and the tracked
handler reader used by real applications. Existing operation read_access methods
require Relation: OperationReads<Operation> and delegate to the underlying
snapshot reader. They intentionally do not record decision facts. The handler's
relations_from/to instead call decision_relations_from/to, which admit the
installed decision target, record the adjacency key and its success/failure,
and retain handler versus computation-call attribution through DecisionReads.
A forwarding-only change would leave the tracked application's walk missing.

Authoritative graph truth remains the exact Native pin and its original request;
relation/kind meaning and allowed decision targets remain installed contracts.
The existing projection cookie validates typed roots. Pure direction, raw IDs,
returned DTO metadata and public result names grant no additional authority.
The operation layer owns attribution and sealing; it must not grow its own BFS
or replace original execution/payer authority.

The current sealing path matters. ExpectedFactKeys shares exact handler and
computation keys. Projected completion re-observes ordinary Entity/Field/Relation/
Adjacency keys through their existing fact owners. Each adjacency observation
pins the same lease snapshot and performs a one-hop Native frontier read;
publication currentness repeats that bounded observation and compares every
physical relation triple, including absence. A walk cannot casually register a
per-anchor set and then claim its sealing cost was one traversal. Repeating the
whole walk's retained body per member would also create quadratic storage.
Whether an existing batched/issued observation owner can carry the complete
walk without either problem remains to be established before planning.

The source-publication fact path demonstrates an existing carried-value
boundary: the original reader retains owner-issued values separately from keys;
completion requires the exact issued Arc and admission, rather than promoting
a descriptive key to proof. This is evidence to examine, not permission to reuse
a source-query authority for structural reads. Ordinary physical facts retain
through the original StorageControl/RetainedFactStore and canonical locator
owners. New structural evidence must enter those real retention, identity,
currentness and canonical encoding paths honestly if a new fact shape is needed.

A strict graph_reads_exactly_cover_fact_keys helper exists, but completion uses
it only when no expected projected keys are present. The projected path still
requires exact expected-key equality and installed-scope coverage for every
fact. Do not infer that one semantic relation role necessarily means one
physical anchor, or weaken exact coverage to accommodate a walk.

Downstream evidence is concrete. Proprietary basis/thread.rs runs PreviousFeature
one hop at a time through direct snapshot, installed invariant and tracked
operation readers. The tracked implementation uses the handler's decision
reader, so predecessor reads are dependencies today. Replacing that loop must
preserve dependency coverage and the domain's own body, branching, retirement,
introduction and duplicate-name judgments. This slice must establish the public
tracked walk handoff; private cutover remains a later qualified slice.

Open source questions before the implementation plan: the smallest complete
structural dependency representation; whether existing issued fact/value storage
can retain a move-only Native outcome with its original payer; exact handler and
partitioned-computation handoff; complete/empty/multi-root and failed-read
coverage; currentness and canonical encoding impact; and a focused real-handler
court that makes a deep/leaf/nondiscovery change stale while a lawful twin commits.
No Slice 4 production source has been edited.


### Slice 4 resolved boundary decisions

The persistent reviewer confirmed that no existing generic issued structural
walk carrier closes this handoff. Use one logical structural-walk key and one
privately issued shared Native value. Do not reuse SourceQuery scope, authority
or semantic types. Reuse its ownership pattern and the real retained-fact,
canonical-locator, currentness and original-memory owners. The fresh value binds
its actual operation admission, selected original snapshot/cookie and installed
relation contract. Completion consumes that exact issued value; a descriptor
alone is insufficient. Publication compares against one genuine Native complete
walk at its selected current snapshot, including entity liveness/kind/distance
and every physical relation ID/kind/endpoints. No Query BFS or per-anchor reseal
reads are introduced. Leaf absence is covered by the complete frontier, and
invalidation posts every visited entity and selected adjacency boundary.

Native output currently owns its vectors and reservation move-only. A private
Query Arc backing can share that same Native owner between the public result
and issued observation. Fund its own header/control/source fields before
allocation on the original request; never quote the already-paid Native vectors
again in generic fact storage. Canonical-root descriptor and registry/index
allocations have their own original payments. Build the authoring registry in
place and move it into ExpectedFactKeys at completion; that view is move-only,
so no growing-Vec clone or second immutable-map allocation is required.

Checkpoint facts deliberately exclude ordinary Adjacency and SourcePublication
already. Keep the new live structural observation unsupported for checkpoint
reuse as well; do not add a Native-from-metadata constructor or codec/version
migration for this slice. It still needs complete canonical dependency identity,
locator/body equality/merge, currentness and effect-routing treatment.

Live computation carriage is a separate authority question. Native refresh can
consume old fact payload/custody honestly only after its exact selected snapshot,
completed carried calls/ranges, checked layout, contracts, root and handler-key
conditions hold. It can decline late, leaving the generic expected-key path
without a newly issued structural body. The observation-work guard is not a
solution: it runs only with accounting, and unaccounted fact indices use None
for optional work even for supported facts. The agreed first-slice policy is an
explicit early Full eligibility decision for a prior containing the new typed
structural key, in both postures, before old owner output or dirty-ordinal fast
paths are accepted. Compute the presence bit in the existing index build and
keep it independent of optional work. Do not alter older fact policies or relabel
an old admission. Fresh execution issues the new value normally. Qualified live
structural carriage is later work with its own complete fallback proof.

### Slice 4 implementation plan

The ordinary developer calls stay small:

```rust,ignore
let history = reader.walk(PreviousFeature::reference(), RelationDirection::Forward, &root)?;
for step in &history {
    inspect(step.entity(), step.distance());
}
let depends = reader.reaches(PreviousFeature::reference(), &root, &earlier)?;
let all = reader.closure(PreviousFeature::reference(), RelationDirection::Reverse, &roots)?;
```

On the raw operation projection these are declared capability reads. On the
handler DecisionReader and lent WorthQueryComputationReader they also retain
one complete structural dependency and the existing caller attribution.

1. Add a private retained backing below invariant_projection/walk for the
   existing PinnedStructuralWalk, its actual original snapshot/cookie/query
   signature and a separately quoted Arc/header payment. Change only the
   private public-result representation; preserve its method signatures,
   canonical iteration and identity minting. Construct it inside the original
   checkpoint extent, with all before-growth claims and stops. Source-owned
   Native vectors/ticket remain unchanged and are shared, never cloned.
2. Add operation_reader/structural_walk with subordinate query, issued-value,
   registry, canonical/currentness responsibilities as needed, each under400.
   The query key owns one canonical root set derived from successful Native
   distance-zero visits, plus installed endorelation/direction meaning. Quote
   its root/control storage once. The issuer accepts only the current real
   admitted reader and matching original backing, captures actual contracts and
   original source binding, and is private to that owner. No public data-to-proof
   factory, new authority marker, fresh request or budget is introduced.
3. Add walk/closure/reaches capability methods to the operation reader, with
   Relation: OperationReads<Operation>. Add decision variants which admit the
   actual installed relation target and retain the issued complete value. The
   original outer extent covers traversal, key/value allocation and attribution;
   reaches also checkpoints its membership scan. Add handler and computation
   reader methods delegating to the decision variants under the same trait
   bounds. A decision read without a real admission is refused precisely.
4. Extend DecisionReads and ExpectedFactKeys with a funded authoring structural
   registry moved, not copied, at completion. Store one body per logical walk;
   repeated calls verify the same original source/body and reuse that entry.
   Record through the existing handler/computation attribution owner. Preserve
   the first failed decision read and veto sealing even when swallowed. If a
   failed walk has no complete canonical key/body, represent the failure
   explicitly rather than inventing a successful key/value or partial walk.
5. Add one private structural variant to FactKey and ObservedFact, reusing the
   existing canonical locator/AdmittedFactKey/RetainedFactStore path. Supply
   complete body equality/merge and canonical encoding of descriptor, visits
   and physical relations. Fund canonical scratch by actual root/visit/edge
   counts even though generic ChargedBytes/deep-clone reports no new Native
   backing for an Arc clone. Keep checkpoint encode unsupported deliberately.
   Audit all exhaustive fact/key consumers in one batch before the build.
6. In both ordinary and qualified-native projected completion paths, consume
   only the exact issued value for a structural key after actual admission,
   original snapshot/source and installed-scope checks. Generic observe_fact
   refuses descriptor-only structural reconstruction. Publication and source
   currentness pin the real selected current snapshot, run one Native complete
   walk and compare the complete canonical truth with original checkpoints and
   a final poll; stops never turn into false or a partial success. Do not infer
   permission or currentness from a diagnostic count, head version or digest.
7. Add a posture-independent structural-presence field to the already-built
   computation fact index, and an explicit full-run reason. Check that new
   eligibility before computation_changes/dirty-ordinal shortcuts or any
   carried output is accepted, while preserving prior basis-drift priority.
   Both accounted and unaccounted runs then execute their owner freshly and
   issue the current admission's value. Older comparison/budget policies stay
   unchanged. Prove this using a genuinely sealed retained run and another real
   admitted computation attempt, not fabricated key/flag metadata.
8. Integrate the new physical body into invalidation, resident/index quotes,
   FactRoutes, entity-touch/postcondition and source comparison consumers.
   Build funded visit/edge routes once; retain existing first-match and explicit
   link/unlink requirements. Never scan the whole walk separately per effect,
   copy it per member or fabricate a post-effect Native outcome. If an effect
   changes the read, use actual postcommit comparison/own-write-stale behavior.
9. Reuse the authored WalkSchema/World fixture and ordinary edits. Add one
   authoritative operation/binding/real tracked handler as necessary, using
   the central schema definition and actual installation/authentication/read/
   sealing/compare-and-commit path. A retained candidate must become stale for
   a third-hop edge edit, a new child at an empty reached leaf and a nondiscovery
   edge edit preserving reached IDs; an unrelated edit must let that same
   retained candidate commit. Share empty/multi-root/reaches, handler versus
   computation attribution, swallowed-stop, foreign/original-issuer and
   same-issued-Arc/no-reseal-read controls in the small fixture. Add the genuine
   reuse-attempt fresh-run control in both postures. Do not reopen Native BFS
   or the already accepted installed-invariant mutant portfolios.
10. Before building, review every new call's actual visibility and trait/type
    boundary, all fact dispatchers, original allocation/drop order and the
    concrete test fixture against the plan. Gate the source with the persistent
    three-skill reviewer. Then compile one Query execution owner selection with
    the real build-ID wrapper, locked Rust1.94 and proven nonincremental setting.
    Reuse its emitted artifact for the new courts, Slice3's19 projection courts
    and only affected canonical/currentness/computation regressions. Native
    source is intended to stay unchanged, so reuse Slice3's32 Native passes.
    Run required normal boundary/context and dirty fmt/caps/whitespace/function
    review once on stable source. Update the direct guide/capability index with
    compiled handler calls and the explicit fresh-computation/checkpoint reuse
    limitation, without feature-doc-writer. Obtain final three-skill clearance
    and commit the slice before continuing.

The persistent review approved the concrete plan under qa-loop, qa-tests and
code-quality-qa. Live issuance provenance must remain separate from canonical
graph truth. Duplicate successful logical walks retain one issued body after
same-source/truth validation and preserve caller attribution; a subsequent
failed call still vetoes sealing. Declared closure/wire/receipts, qualified live
structural carriage, private cutover and master integration remain outside it.

### Slice4a boundary and implementation plan: capability-only operation reads

Implement the raw operation reader capability as a small prerequisite commit.
Its existing `read_access.rs` delegates declared one-hop reads to the snapshot
reader and does not record decision dependencies. The handler decision reader
is a separate surface. Adding raw complete reads here cannot satisfy the
recorded-handler contract; that remains the next Slice4b implementation above.

1. Add `walk`, `closure` and `reaches` to the operation reader in a dedicated
   read-access child. Require the same `Relation: OperationReads<Operation>`
   capability as existing `relations_from/to`, and delegate directly to the
   already qualified snapshot executor. Reuse its result, original work and
   memory payer, scope admission, foreign-identity checks and stop latch.
2. Exercise the actual admitted-operation projection using the existing
   centrally declared WalkSchema/World. Add an inspection-only operation with
   explicit entity/key/relation reads and no arbitrary projection cap, leaving
   the existing bounded edit operations intact. Compare actual result and
   acquisition counts with the snapshot path; cover reverse/multiroot/empty and
   reachability, foreign roots and swallowed original cancellation. Compare a
   bounded refusal with the same graph under the uncapped inspection operation.
   No fabricated admission, Native DTO or new test authority door is needed.
3. Document this raw inspection surface and its distinction from recorded
   decision reads. Add the capability-index entry. Do not expose these calls on
   the handler/computation decision reader before issued dependencies exist.
4. Review source before one batched Query owner build, then reuse its emitted
   artifact for the new courts and the already focused projection/operation
   regressions. Native source stays unchanged; reuse Slice3 Native evidence.
   Run dirty formatting/line/function/whitespace checks and the required normal
   boundary/context checks, obtain persistent three-skill clearance and commit.
   Continue Slice4b with the accepted issued-value plan after this prerequisite.

### Slice4a qualification and handoff

The raw operation reader now exposes the three complete-read methods, each with
the existing `Relation: OperationReads<Operation>` bound and one delegation to
the snapshot reader. The result owner, Native executor, decision facts, handler
and computation surfaces are unchanged. The guide and capability index identify
these as inspection reads without publication-dependency protection. The
central fixture declares InspectWalk consistently for all installations; its
uncapped projection still consumes the original request's resources. Existing
bounded edit-operation contracts remain intact.

Evidence prefix outside the checkout:
`C:\programming\worth worktees\targets\query-9.18.1-phase3-slice4a-`.

- Corrected Query owner build3 passed in332.26seconds through the real fresh-ID
  build wrapper, locked Rust1.94 and the established nonincremental setting.
  Its emitted artifact ran22 focused tests: all3 actual admitted-operation
  courts plus Slice3's19 projection/acquisition regressions. All22 passed,
  none ignored, in7.01seconds. This includes exact cancellation after callback
  entry, swallowed installed-limit denial and completion of the same32-node
  graph with the uncapped inspection operation.
- The final normal boundary check passed in346.16seconds and agent-context
  check passed in0.83seconds, both begun on the corrected frozen source.
  No snapshot updates or checker changes were required.
- Final dirty static checks passed for7 Rust files: formatting,400-line caps,
  whitespace and function scanning with no scan errors. The persistent reviewer
  accepted all5 function advisories. All7 guide links resolve. Native source
  did not change; Slice3's32 Native passes remain the causal evidence.
- Earlier attempts are not green qualification: build1 found the path-attributed
  parent needed explicit `#[path = "read_access/walk.rs"]` (one root diagnostic
  with17 cascades,110.17seconds). Build2 compiled successfully in357.86seconds;
  its test run passed21/22 because the cancellation fixture attempted an
  unrelated sibling request inside the host extent before entering its callback.
  Moving the genuinely retained admission outside that host extent matches the
  existing counted-stop court; no production behavior was changed for this
  fixture correction. The initial normal run had captured the old module path
  and failed; it was replaced by the fresh final normal pass above.

Persistent boundary/prebuild/correction and final result review cleared all
three QA lenses, with no remaining material finding or additional build needed.
Slice4a is complete and cleared for commit on the task branch. Slice4b still
must implement the previously approved issued-body decision integration; this
raw capability prerequisite does not complete Phase3 or authorize its merge.

### Slice4b implementation boundary reconciliation

Continue the already reviewed ten-step issued-value plan above from the clean
Slice4a commit0d87e40653923dde338b1162c31ba7fe35100c1e. The persistent reviewer
approved these concrete source decisions before implementation:

- Retain the actual contracts Arc from admission in the operation reader and
  issued registry; unadmitted capability readers cannot issue structural
  decision facts. Two private Arc owners separate the funded logical query
  signature from the already-funded Native result and original snapshot/cookie.
  The public result remains move-only. Logical ordering/equality and canonical
  graph truth exclude admission, snapshot identity, diagnostics and payment.
- Registry insertion grows in place under the original extent, including its
  actual new handler/call-attribution cells. Successful duplicates validate
  origin and complete truth, then attribute the canonical first key. Failed
  calls independently veto sealing even before a key exists. Original authoring
  payment moves to the attribution until its authoring storage is consumed.
- Both projected seal paths and compiled-body extension consume only the
  original issued body under actual admission/contracts/snapshot/scope checks.
  Descriptor-only observation refuses reconstruction. Physical visit/edge
  routes are funded once, preserve first-match behavior, and do not create an
  implicit explicit relation-pair fact.
- Structural prior presence is derived while building the real computation
  index, independently of optional accounting/work bounds. Reuse begins with
  Full(StructuralWalkRead), after existing basis-drift priority and before
  dirty/currentness shortcuts, in both accounting postures.
- Storage/routing use their existing original allocation-control polls;
  canonical work evidence is separate from request-charged logical work.
  Review required fallible per-item polls in the new structural canonical leaf
  and duplicate-body merge. Both preserve the original StoreDenial. The locator
  byte bound is checked in constant time before any allocation/root formatting.
- Postcommit producer rebase classifies this as an original source read. It
  compares the actual committed snapshot and keeps the original value as
  current or superseded. It never constructs a post-effect Native observation.
  Live carried structural values and checkpoint reconstruction remain
  deliberately unsupported.

The new real installed handler retains a candidate across ordinary deep-edge,
empty-leaf and nondiscovery-edge edits. At the actual compare-and-commit
boundary the precise expected terminal is Stale with one changed fact; this is
the native sealed-dependency equivalent of the plan's source-changed intent.
The unrelated-field/global-head twin must let the same retained candidate
commit. Separate owner courts cover original Arc consumption, another real
admission's refusal, raw/descriptor-only refusal, failed-call vetoes, explicit
pair requirements and actual retained-computation fresh issuance under both
accounting postures. No fake Native result or injected structural-prior flag
is used.

Prebuild source review under all three QA lenses cleared these boundaries and
the scoped corrections. Qualification is pending: one Query owner build, new
courts plus focused existing snapshot/operation, canonical, merge, reuse and
source-currentness regressions; required normal boundary/context and dirty
static gates. Native is unchanged, so Slice3 Native evidence is reused.
This is not Phase3 closeout, declared-query/wire work, or private adoption.

### Slice4b qualification and handoff

Current source implements the issued-value decision handoff for operation,
handler and lent computation readers. A successful logical query retains one
original complete body, with its actual original payer and attributions. Both
seal paths reject copied descriptors and foreign admissions; publication reads
current Native truth once per logical walk. The current handler courts prove
actual deep-edge, empty-leaf and nondiscovery-edge changes produce one stale
fact, while the same retained candidate survives an unrelated field edit that
actually moved the graph head. The genuine retained-run court proves fresh
gathers and current issuance in both actual accounting postures.

Evidence prefix outside the checkout:
`C:\programming\worth worktees\targets\query-9.18.1-phase3-slice4b-`.

- Query owner build3 passed in458.26seconds with no compiler diagnostics, using
  the real fresh-ID wrapper, locked Rust1.94 and the qualified nonincremental
  setting. Its emitted binary ran35 focused tests in10.55seconds: all35 passed,
  none ignored. This includes all6 new courts, the22 prior focused projection/
  operation regressions,2 canonical content oracles,2 source-merge courts,
  2 existing computation-reuse controls and the source-revision postcommit court.
- One additional existing regression,
  `unobservable::more_retained_keys_than_the_request_work_runs_in_full_uncompared`,
  passed on build2. It constructs4,097 synthetic retained copies and dominated
  that run. It uses IdentityExecutionSchema; its implementation, schema and
  production owners did not change after build2. Persistent review explicitly
  accepted carrying this evidence rather than repeating it for the later
  CheckWalk fixture-only contract correction. The final evidence is35 current
  passes plus that1 carried pass, not36 freshly rerun tests.
- Normal boundary passed in354.90seconds (Road1 topology valid); agent-context
  passed in0.51seconds. This run began before three small compiler corrections:
  a unit-test enum import, the pure canonical helper's sibling-owner visibility,
  and a test local's borrowed lifetime. The later fixture-only program flag
  correction also changed no source-module, dependency, facade or production
  authority topology. Persistent review explicitly accepted this limited gate
  carry alongside the current corrected build/runtime and final static checks.
  No checker edits or inventory snapshot updates were needed.
- Final static gates passed:66 Rust files formatted and within400 lines and
  whitespace clean. The initial dirty function scan saw65files/40advisories;
  precommit inspection exposed Windows'270-character path limit on the provider
  postcommit resolution file. The unchanged scanner's explicit-file mode with
  Windows extended paths then scanned all66files:42advisories,0scan errors.
  The extra2advisories are the existing prepare/resolve functions in that file;
  the source change there is only the structural source-read match arm. All9
  direct-guide links resolve. Persistent review accepted all42advisories and
  reaffirmed commit clearance after this complete-coverage pass. Native
  source did not change; the32 Slice3 Native passes remain the causal evidence.
- Earlier failures are not green qualification. Build1 stopped in113.21seconds
  on the three compiler wiring defects above. Build2 compiled in347.77seconds
  and ran36 tests in280.62seconds:34 passed, but both new handler publication
  courts were stopped before currentness by ApplicationProgramRequired. The
  new fixture had copied `program required` from a different fixture while
  actually using ordinary authenticated commit. Removing only that extraneous
  flag makes its declaration match the ordinary handler contract used by the
  existing OptionalOutput fixture. It changes no production program gate;
  current tests retain actual admission/sealing/effects, exact stale counts and
  the same-candidate unrelated-edit twin. Build3 replaces that failed evidence.

The direct guide and capability index describe the recorded methods, actual
compiled handler example, explicit pair/field obligations, fresh-computation
policy with basis-drift priority, and unsupported live-carried/checkpoint reuse.
Persistent final review cleared all three QA lenses, including these current
results and the explicitly partitioned evidence carries. No remaining material
finding, additional build or suite was required. Slice4b is complete and cleared
for commit on the isolated task branch.

Phase3 still requires declared-query `forward_closure` / `reverse_closure` with
step/read receipts, the topology_entry hand-hop cutover, private history-reader
adoption and phase closeout. Continue with the declared-query closure boundary;
do not merge this incomplete phase or mark the six-phase goal complete.
### Phase 3 Slice 5a — shared complete-walk truth owner

Boundary brief (2026-10-25, before implementation):

- Native's pinned `complete_relation_walk` is the sole traversal authority. Its
  move-only result owns complete visits, physical edges, actual reads/work, and
  the original retention payment. Query must preserve that result directly.
- The current reusable fact body and currentness implementation live beneath
  `invariant_projection/walk/observation`, although operation facts, canonical
  identity, output reuse and computation already consume them. Declared closure
  will need that same semantic truth under its own source proof.
- `read_observation` is explicitly diagnostic and cannot own this proof.
  Introduce `primary_graph/structural_walk` as the actual-read owner instead.
- The shared owner must execute the Native read itself and privately retain the
  successful payload. It must have no constructor accepting supplied Native
  payloads, visits, revisions, digests, or reconstructed metadata. Its source
  comes from the exact pinned receiver, not a separately supplied snapshot.
- Projection retains typed-root locality, installed relation validation,
  realized-scope admission and its typed result. Projection identity cookies
  belong to that result and the issued decision registry, not shared truth.
  The registry continues to require the real operation admission/contracts and
  exact original result provenance before sealing its original Arc.
- Native stops, header funding/allocation failure and final cancellation must
  preserve acquired reads/work even when no observation escapes. Original
  backing and header payment remain with the retained body until it drops.
- Cut over all existing fact, canonical, registry and computation consumers to
  the new owner directly; leave no forwarding implementation under projection.
- The following slice must integrate declared closure through portable meaning,
  archive/signing, installation/planning, governed execution, receipts and row/
  set source proofs. Root paths are also consumed by operation bodies, so those
  consumers cannot silently interpret closure as a single hop. Guard step
  ordinals cannot become a traversal-depth cap.

Persistent reviewer boundary ruling: extraction is justified; keep cookies out
of the shared signature, retain failed-read metrics, and expose no closure API
until complete execution/evidence is integrated. No unresolved material facts
remain for this prerequisite.

Implementation plan (after boundary ruling):

1. Move complete observation/key/currentness to `primary_graph/structural_walk`.
   A private retention function receives only its own successful actual Native
   invocation. An internal query value supplies the exact relation, kinds,
   roots, direction and optional work ceiling used by that invocation.
2. Move the existing optional execution-memory helper into that owner; retain
   original request accounting/control and drop order. No new runtime lane.
3. Have snapshot projection validate/lower roots, obtain its original pin and
   invoke the shared producer. Record actual reads/work from success or denial
   before realized-scope admission and outer final handoff.
4. Move cookie checks to the privately constructed projection result. The
   decision registry records that actual typed result, retains its cookie next
   to admission/contracts, and preserves duplicate/conflict and failure-latch
   behavior. Fact sealing still retains the exact original body Arc.
5. Migrate all current consumers directly and remove old exports/files. Public
   API behavior, canonical fact contents and Native implementation stay the
   same. Declared closure, wire changes and consumer cutovers are next-slice
   work; no partial closure API is introduced here.
6. Before the single owner build, review the coherent diff and run scoped
   formatting, dirty line caps, function advisories and whitespace checks.
   Build Query execution once through `application_build.py`, incremental=false,
   then reuse the emitted test executable for the existing focused snapshot,
   raw/recorded decision, currentness, canonical and computation courts. Reuse
   existing Native evidence; do not rerun the unrelated 4,097-key court.
7. Run normal boundary-check and agent-context check, obtain persistent
   qa-loop/qa-tests/code-quality-qa clearance, record exact evidence and commit.
   Phase 3 remains open and is not merged by this prerequisite.

Slice 5a qualification:

- Complete observations, logical keys, exact currentness and original payer
  custody now belong to `primary_graph/structural_walk`. Its read producer
  invokes Native and its only retention constructor is private. All existing
  consumers import that owner directly. Native itself is unchanged.
- A final source review found that moving the cookie out of shared truth also
  requires permanent pairing inside the typed projection result. The result's
  body is now private and only an immutable `&Arc` accessor is exposed internally.
  The actual walk producer remains its only constructor. Three read sites were
  migrated; no setter or reconstruction constructor exists. Empty and nonempty
  result cookie checks were added to the existing snapshot court.
- Historical build 1 passed in 550.14 seconds with zero compiler errors before
  this encapsulation correction. It is not the final-source test evidence.
- Corrected Query execution build 2 passed in 549.39 seconds with zero compiler
  errors. The build used the original guarded application-build entry point,
  Rust 1.94, the existing Query target and incremental=false.
- Its emitted executable passed all 35 selected courts, zero failed or ignored,
  in 13.03 seconds. These cover snapshot traversal/locality/retention/stops,
  raw and recorded operation reads, exact original Arc sealing, real handler
  deep/empty/nondiscovery currentness, managed computation fresh issuance,
  canonical/source merge and postcommit currentness. No new fixture or test
  function was added; the unrelated 4,097-key court was not rerun.
- Normal boundary-check passed in 481.78 seconds and agent-context check in
  0.63 seconds. Both completed before the private body/accessor correction.
  The persistent reviewer explicitly accepted carrying these checks: the
  correction narrows an internal field and adds immutable borrowing, with no
  module, dependency, facade or authority-topology change after the green scan.
- Final scoped formatting, dirty Rust line caps and whitespace checks passed.
  The full explicit-file function scan covered all 18 scoped Rust files:
  10 advisories, zero scan errors; all advisories were reviewed and accepted.
- Evidence prefix outside the checkout:
  `C:/programming/worth worktees/targets/query-9.18.1-phase3-slice5a-`.
  Current results are `query-build2-result.json`, `query-courts2.json`,
  `structural-gates.json` and `static.json`. Existing Native qualification is
  carried because no Native source or dependency changed.

Phase 3 remains open. The next boundary is the complete declared-query closure
vertical slice: typed steps and portable validation, canonical/archive/signing
and planning semantics, governed pinned execution, receipt reads and retained
row/set source currentness. No closure declaration API was introduced by this
prerequisite. Topology/private consumer cutovers and phase closeout follow it.

### Phase 3 Slice 5b — complete declared-query closure

Boundary brief (before implementation):

- The typed root-path declaration currently represents a single hop per step;
  the installed ordinal named `depth` also places guards. Neither representation
  can stand in for complete reachability. Explicit traversal meaning must reach
  portable validation, canonical identity, archive decoding and planning.
- Native's pinned complete walk remains the sole traversal authority. The
  shared owner from 5a retains its actual immutable body and original payer.
  Query must not implement BFS, issue a walk per frontier anchor, or fetch an
  adjacency revision per reached entity after that complete read.
- A multi-root closure must also identify a real predecessor witness for each
  terminal row. Native currently discards its discovery origin. Carrying that
  actual origin through its FIFO permits each row to inherit one real prior
  source and share the complete body. Unioning every predecessor into each row
  would instead create quadratic evidence. Canonically funded/deduplicated
  seeds make the selected minimum-distance origin deterministic.
- Query's admitted `ReadInterruption` can be distinct from the host execution
  request. Native must accept a borrowed refusal-only caller checkpoint as an
  additional narrowing control, while retaining its original execution lease,
  checkpoints, work ceiling, memory and final handoff. No fresh request/budget.
- Row/set source conversion must emit the existing structural-walk fact with
  the exact original Arc. Actual bodies, not their commitment or query key,
  establish truth. Original-source shortcuts require real pin admission first.
- Required-set capture/currentness and source-meaning interning iterate rows.
  They need funded verification/equivalence extents that share completed results
  for the same actual immutable Arc (or actual Arc pair) in that one call.
  No cross-request/snapshot cache is introduced. Query coverage continues its
  existing unsupported-kind fallback; adding a whitelist alone would still
  register a whole closure's addresses for every row.
- Source commitments may summarize retained complete bodies once for canonical
  identity and cheap negative comparisons. Equal commitments never replace
  actual truth comparison, and no commitment can construct a walk body.
- The public receipt currently has aggregate reads but no path-step record.
  Small descriptive executed-step records must survive non-live, continuation
  and live handoffs, retaining their original funding. They reveal authored
  step meaning/ordinal/direction and actual reads, never Native IDs or origin
  witnesses. Existing disclosure observations must include the new evidence.
- Root paths also serve operation-body capture/input/value/creation paths.
  Their existing profile requires one distinct identity per hop. This phase
  explicitly rejects closure in that central portable shape predicate and at
  the defensive runtime boundary; it does not invent multi-target mutation
  semantics. Signing/canonical paths still preserve explicit traversal meaning.
- Archive records have an exact-current grammar. Deliberately advance record
  version 14 to 15 with closed combined step tags: 1/2 retain existing one-hop
  directions and 3/4 mean closure directions. Preserve nested byte width and
  one-hop canonical preimages where possible; closure gains explicit meaning.
  Refuse old/unknown grammar rather than defaulting closure to one hop.
- Persistent boundary review agreed on the Native origin/control seams,
  query-only operation-body restriction, complete receipt handoff and scoped
  verification extents. Physical owners and actual source facts were inspected;
  remaining choices are implementation mechanics, not unresolved authority.

Implementation plan (after boundary review):

1. Add explicit one-hop/closure traversal vocabulary to root steps and typed
   `forward_closure`/`reverse_closure`, accepting only Current-to-Current and
   returning Self. Portable validation independently rejects non-endorelations.
   Carry it through installation, planning and all canonical/signing encodings;
   retain authored guard ordinals, with no sentinel or constant depth limit.
2. Update the exact-current archive grammar/version and step codec. Refuse
   unknown traversal tags and unsupported closure-bearing operation bodies;
   preserve their lawful one-hop twins and fail closed before runtime looping.
3. In Native, fund canonical seed ordering before growth, preserve original
   per-input checkpoints, and carry actual discovery root through the frontier
   and result visit. Update original scratch/result bounds and canonical fact
   material quote/encoding for the additional complete fact.
4. Add the borrowed refusal-only Native control seam at existing safe points
   and final handoff. Thread it through the shared actual-read owner. Query
   latches the exact first caller denial, records acquired reads/work even on
   refusal, then polls again after header/source work before publication.
5. Add one closure advance in root-selection that calls Native once over the
   whole current frontier, chooses each row's real origin source, and shares
   the complete observation Arc. Keep ordinary one-hop execution as one-hop;
   guards apply at authored boundaries and can precede/follow either step.
6. Extend root-selection source evidence, accounting and identity with shared
   walk dependencies. Compute body commitments once under original controls;
   retain full bodies. Materialization/layout validation and duplicate handling
   consume exact existing structural-walk facts without resealing from metadata.
7. Share actual body verification once per selected-pin extent across retained
   row/set capture and required-set currentness. Share actual pair equivalence
   once across row/set source interning. Fund temporary registries before
   growth, retain their original bodies, poll original controls and discard the
   extent after the call. Preserve existing non-walk fact behavior and coverage.
   Audit cold source serialization/reconstruction: missing original walk custody
   remains an explicit unsupported reuse refusal, never an empty dependency list.
8. Carry funded executed-step read evidence from root selection through every
   kernel/result/receipt lane and capacity boundary. Expose descriptive records
   through the existing host receipt facade; aggregate actual reads remain
   counted once per Native invocation. Update disclosure observation/proofs.
9. Add proportionate evidence using existing fixture owners: portable identity/
   codec/rejection twins; actual installed declared forward/reverse closure,
   root-zero/cycle/diamond/multi-root witness and guard composition; long walk/
   unrelated growth/shared-custody; original caller stop/lawful twin; exact row/
   set source currentness and receipt/disclosure propagation. Extend Native's
   existing model/permutation/control/custody cases rather than a broad suite.
10. Before compilation, obtain persistent three-skill source QA and run scoped
    formatting, all dirty Rust line caps, full explicit-file function scrutiny,
    whitespace and guide-link checks. Split touched oversized files; no Cargo
    feature expansion or unrelated debt cleanup.
11. Batch Native and Query owner builds through guarded application-build with
    fresh build identities, existing targets/Rust 1.94 and Query incremental=false.
    Compile the affected declaration/installation/archive unit owners in the same Query
    workspace invocation, then run only selected tests from emitted artifacts.
    Run normal boundary-check and agent-context once on the coherent topology.
12. Update the direct feature guide/capability index with a compiled declared
    closure example and its query-only body restriction. Never invoke the
    prohibited feature-doc-writer skill. Record exact evidence, obtain final
    persistent qa-loop/qa-tests/code-quality-qa clearance and commit this slice.

Phase 3 remains open until topology/private consumer cutovers and closeout.
No Phase 4 work, default-branch merge or private SDK pin change belongs to 5b.

Implementation boundary revision: inspecting actual admission derivation found
that both ordinary and admitted application-query shapes always select the
`DirectEdge` operator, even when the installed view now describes closure.
Changing the installed digest/view alone would therefore omit traversal-workset
and visited-set requirements. Step 1 also covers those two admission owners and
their downstream requirement/support/cost meaning. Add explicit complete-closure
vocabulary rather than using a bounded operator or ordinal-as-depth surrogate;
qualify actual admitted requirement rows and their dynamic resource posture.
The persistent reviewer independently found the same gap. This repair must be
resolved before source/runtime clearance or compilation.

That admission trace is resolved: these estimates are exported unknown-
conservative heuristics, not Native funding bounds. Complete closure therefore
gets its own operator, complete-workset complexity and discovered-workset memory
basis, matching runtime support rows in both constructors. Its heuristic size
must ignore the authored ordinal. Actual backing still grows under the original
Native request. A focused admission court must cover the real requirement rows,
support match and ordinal-independent unknown estimate.

Source-allocation revision: logical copy charges and post-return capacity quotes
do not pre-admit new closure row/index allocations. Add a conservative original
per-read metadata grant activated by closure. Claim before root-map insertion,
source-map/name copying or mutation, candidates, final Vecs and witness Arcs.
Work/selection retain it until transient buffers drop; immutable source witnesses
retain it through result handoff. Already-paid metadata is not charged again by
the result capacity seam; Native bodies/descriptors retain their separate
payments. Pure one-hop reads keep their existing allocation path. This grant is
conservative grouped custody, not a measurement of resident bytes.

Runtime-discovered integration revision: the real installed closure courts were
all refused by `validate_one_shot_shape` because its declared-path branch
required the root entity kind to differ from the scope kind. The actual root
selector already dispatches nonempty installed paths before exact-scope
selection. Remove only this kind-inequality restriction; retain the installed
path roster, empty graph-predicate requirement and declared result. The
persistent reviewer confirmed the boundary. This is an implementation defect
found by runtime evidence, not a fixture bypass. The original build3 passed all
six affected unit owners; its runtime batch passed 65 tests and exposed this
same admission refusal in all five new closure courts.

Boundary rulings for the added courts are exact-file registrations: installed
metadata lookup in the declared-query read fixture and admission-owner closure
fixture, plus program-publication access in the actual query currentness court.
Each preserves real admission, actual output-demand source custody and ordinary
commits; no broader factory or production exception was added. Temporary
comparison keys are symmetric membership lookups and establish no address-based
semantic order. Receipt descriptors reserve their finite authored roster once.

### Phase 3 Slice 5b closeout

Built complete declared-query forward/reverse endorelation paths with ordinary
hop/guard composition, actual Native discovery origins, shared original walk
bodies, funded source/receipt backing and full row/set dependency currentness.
Admission now requests complete workset/visited support with truthful unknown
cost posture; runtime kind-wide support configuration covers both bounded and
complete profiles. RecordFrame 15 carries the closed step discriminator while
one-hop preimages remain unchanged. Operation-body closure remains refused.
The direct consumer guide and capability index describe these contracts.

Verification (artifact prefix `query-9.18.1-phase3-slice5b-` under the task's
external targets directory):

- Native build1 passed in 274.20s; 11 focused complete-walk/pin courts passed.
- Query build4 compiled all six affected unit owners in 540.17s with zero
  compiler errors. Its runtime batch passed 69 tests and exposed one fixture
  lifecycle error: it tried to re-capture old output after an unrelated commit.
- The fixture now preserves its original dependency capture across edits and
  retains only fresh output at its current occurrence. Production retention
  checks were unchanged. Execution-only build5 passed in 387.00s with zero
  compiler errors; the exact corrected currentness court passed in 0.44s, including
  both hostile edits and the unrelated-current control. Thus 70 distinct Query
  courts pass using retained build4 evidence plus the corrected court, not one
  final 70-test invocation. Native's separate 11 passes also carry.
- Actual tests cover long 1,001-node traversal and repeated semantic interning
  under a generous measured original-work envelope, 10,006-node unrelated growth,
  cycle/diamond/guard origins, root-zero and empty-frontier receipts, caller
  cancellation/live twin, original source/body memory release and a real policy
  memory refusal at `closure source backing` with a funded twin. The finite
  refusal used limit 58,095 and reported requested 56,456/admitted 55,871 at that
  allocation boundary, then released all original charges.
- Final formatting, dirty line caps, whitespace and explicit long-path function
  scrutiny passed for all 117 Rust files. The persistent reviewer accepted all
  84 composition advisories in this causal scope; there were zero scan errors.
  The new guide's source link resolves. Both edited consumer documents preserve
  original UTF-8 prose outside their intended additions.
- The final audience audit added the promised step-read type to the outer
  Execution/Host facade, with a direct public-path annotation in the actual
  receipt court. It also narrowed the internal source constructor to its Query
  owner and removed newly unused/redundant syntax. Guarded Execution library/test
  checking passed in 282.44s and Host library checking in 21.26s, with zero errors.
  Remaining compiler warnings are the pre-existing private structural-walk fact
  type and unused observation methods; this is not a warning-free claim.
- The public export required its stable rustdoc and governed facade inventory.
  The supported snapshot updater passed in 369.00s and changed exactly one
  export line in `tools/boundary-check/snapshots/facades.toml`; no doc-debt waiver
  was added. Subsequent normal boundary-check passed in 381.44s and agent-context
  in 0.69s on the final code/snapshot state. These fresh normal gates supersede
  the earlier build3 carry approved for the private shape/fixture-only change.
  The reviewer accepted the prior runtime results across this static export,
  visibility, import, annotation and documentation-only delta; no additional
  runtime suite was run.

Earlier failed attempts are retained as diagnostic evidence. Query build1 was
stopped after a child test-module path error; build2 exposed fixture imports,
visibility, projection bounds and borrowed-plan lifetime errors. Build3 compiled
successfully but its runtime batch exposed the same-kind admission defect
recorded above. None of those failures is counted as passing verification.

The persistent reviewer granted final qa-loop, qa-tests and code-quality-qa
commit clearance after inspecting the corrected currentness result, accumulated
runtime evidence, final static checks and exact evidence carry. Slice 5b is complete.
Phase 3 remains open for topology-entry and private `basis/thread::live` reader
cutovers, adoption and phase closeout. No default-branch merge or private SDK
pin update is claimed by this slice.

## Phase 3 Slice 6 — topology invariant walk consumer

### Boundary brief

The ordinary `topology_entry` consumer still acquires two predecessor hops per
touched body, then two successor hops per affected root in
`planar_invariant/preparation.rs`. Its domain rule evaluates the signed turn of
three consecutive points. The two-edge geometry window is domain meaning; it
must not become a traversal depth cap or a different whole-ring geometry rule.
The existing undeclared-access and evaluation-expansion one-hop calls are
intentional negative controls and remain intact. Fixed query projections and
ring/orbit identity belong to other owners (or Phase 4).

The installed `PlanarSuccessor` law requires one incoming and one outgoing edge,
no self edge and two reachable targets. The persistent reviewer traced native
ordering: Query lowers minimum enforcement to CommitBoundary; native endpoint,
maxima and minima registrations block there; `execute_inner` reduces native
violations and checks `prefix.prevents_custom(CommitBoundary)` before planning
custom rules. Required blocking registrations are not omitted for cost. Thus an
ordinary candidate reaching this rule consists of finite degree-one components,
and one complete reverse closure contains the outgoing physical records needed
for its affected triples. This premise does not generalize to direct/audit rule
invocation; missing or duplicate lookup entries still refuse explicitly.

The resolver-issued relation binding, candidate view, original checked budget
and Native complete walk remain authority. The consumer may choose affected
roots (minimum reverse distance <=2), build a domain successor lookup over actual
returned physical records and evaluate the original triples. It cannot issue
new read authority, fabricate counts, run another graph search or recover by
looping graph one-hop calls. Complete Native observation widens structural
preparation to each reached component; geometric evaluation remains restricted
to the affected triple roots. Typed values keep their real view/proposal affinity.

`invariant_reads` currently compares a recorded forward-hop interval with its
fixed two-hop query. That equivalence becomes false after this cutover. Its
caller-owned observer must record the actual complete walk instead. It can prove
locality, complete acquisition, geometry refusal and unchanged committed truth;
existing negative modes can cover the retained scope fences in the same real
installed example. A native degree-breaking edit and completing lawful twin
will qualify the ordinary native-before-custom premise without a broad consumer
portfolio. Documentation's compiled example excerpt must change with its source.

### Implementation plan

1. Replace the ordinary hand-hop preparation path with one reverse `view.closure`
   over the actual touched entities. Handle both existing negative modes before
   any complete read, preserve caller checkpoints and skip an empty touched set.
   Keep the Native owner alive while consuming its records. No second traversal
   or one-hop fallback is added.
2. Add a private `planar_invariant/successors.rs` owner for the geometry lookup.
   It indexes actual physical edges by native source identity, refuses duplicate
   and missing successors and returns only actual typed target identities. Fund
   conservative BTree node backing from actual record count before allocation
   through the original checked preparation budget; checkpoint bounded domain
   work before insertions/lookups. This is temporary domain lookup, not traversal
   or proof authority. It does not persist beyond preparation.
3. Select roots directly from canonical Native visits at distance <=2. Pre-count
   them with original checkpoints; claim all result bytes and reserve the triple
   Vec once before filling it. Derive each fixed `[root,middle,next]` from the
   funded lookup. Keep X/Y reads and determinant evaluation unchanged. Remove
   the displaced root-dedup and ordinary one-hop acquisition helpers.
4. Replace the observer's `ForwardHops` event with a descriptive complete-walk
   event carrying actual before/after platform observations and actual result
   sizes. Update `invariant_reads` to assert one complete read, a retained
   two-hop query with its own distinct counts, unchanged unrelated-ring growth,
   lawful geometry, exact custom violation and unchanged truth after refusal.
   Exercise both existing negative access probes and the native degree-law
   refusal/no-custom-contact with a real lawful repair in this same example.
5. Update the direct guide and pinned `invariant_reads` excerpt, preserving the
   original evaluation excerpt. Record that preparation is component-wide and
   query projection remains a fixed shape. Do not use feature-doc-writer.
6. Obtain persistent qa-loop/qa-tests/code-quality-qa source clearance. Run cheap
   scoped fmt/line-cap/function/whitespace/document checks. First guard a narrow
   `cargo check -p worth-query-topology-entry --example invariant_reads`; only
   after it succeeds compile that example test artifact once with guarded
   `cargo test ... --example invariant_reads --no-run`, incremental=false and
   the existing Query target. Run only its changed real proof. No Native or
   Execution unit rebuild/portfolio is requested by this consumer-only delta.
7. Run normal boundary-check and agent-context on the coherent consumer/doc
   topology, inspect any generated obligations, obtain final persistent review,
   record honest evidence and commit this slice. Phase 3 still needs private
   history-reader adoption and its phase closeout before default-branch merge.

Out of scope: orbit laws/identity, component/derived relation work, fixed query
shape redesign, generic path language, new SDK factories, private SDK pins and
unrelated consumer test debt. No blocker remains after the native ordering trace;
any contrary runtime evidence revises this brief before changing the plan.

### Runtime boundary correction

The first focused runtime attempt reached the new retarget control but refused
at public request admission with `ApplicationProgramRequired`. The reference
required-feature program declares `MutatePlanar` only as a conditional operation;
an ordinary `PlanarEdit` request cannot inherit that conditional authority. This
is a fixture integration failure, not evidence of Native degree-law refusal.

The correction stays local to the runnable invariant example: retain the shared
reference schema, contribution, required outputs and rules, and define its own
program with an additional topology-edit feature. Declare `PlanarEditBinding`
there as an ordinary action with `ApplicationSourceCompletion`, through the
existing typed program authoring path. Reuse installation without changing its
shared reference program or exposing a certification factory. Both hostile and
lawful retarget controls must use that same installed action. Recheck the
example's types before rebuilding its single test artifact, rerun the same proof,
and obtain review of the final scope and evidence carry.

### Qualification evidence

The ordinary preparation path now acquires one complete reverse Native walk,
indexes its actual physical records under the original checked budget, and
derives the original affected geometry triples from canonical visits within
distance two. Evaluation's coordinate reads and determinant rule are unchanged.
The two intentional one-hop scope probes precede complete acquisition. The
temporary successor lookup refuses missing or duplicate records; it never
repairs the graph or traverses through one-hop reads.

The persistent reviewer cleared the boundary, plan, production cutover and
corrected local program source. Scoped static checks passed for all eight Rust
files: formatting, dirty line caps, explicit function scan (8/8 files, four
advisories, zero scan errors), and whitespace. The guide's changed compiled
excerpt matches the formatted source exactly. Shared required-feature program
and installation sources have no diff.

Guarded compilation used the separate `consumer_entry/Cargo.toml` workspace,
only `worth-query-topology-entry --example invariant_reads`, locked dependencies
and incremental compilation disabled. Check2 passed in 290.06s; build1 passed in
363.73s. After correcting the example's ordinary action declaration, check3
passed in 3.56s and build2 in 24.04s. Each successful compilation had zero
compiler errors and the existing structural-walk observation visibility warning;
this is not a warning-free claim.

Only the exact emitted example test was run. Runtime2 passed 1/1 in 1.40s,
covering zero, one and six unrelated rings. It asserts actual complete Native
walk/read sizes, the fixed query's distinct read counts, lawful geometry,
the exact `PositivePlanarTurn` violation without publication, both genuine scope
failures, a degree-breaking retarget refused as ProviderRejected at
InvariantExecution before any custom preparation, and a lawful physical ABA
retarget that commits and reaches complete custom preparation. Each refusal
retains the committed basis and query truth. This degree control does not claim
a public typed cardinality diagnostic that the public denial vocabulary lacks.

Failed attempts remain diagnostic evidence: check1 selected the wrong workspace
and stopped before compilation; runtime1 reached `ApplicationProgramRequired`
before Native because its ordinary action was undeclared. Neither counts as a
passing proof. Evidence files use the external prefix
`targets/query-9.18.1-phase3-slice6-`.

Fresh normal boundary-check passed in 431.63s and agent-context in 1.00s on the
corrected local-program module topology. These supersede the initial normal
439.73s/context 0.43s passes before that fixture correction. The reviewer
verified build2/runtime2 evidence and accepted all four function advisories.
No broad consumer portfolio, Native test binary or Execution test binary was
rebuilt for this consumer-only slice. Phase 3 remains open for private history
reader adoption and phase closeout; this slice does not merge a default branch
or update private SDK pins.
