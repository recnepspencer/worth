# Relational certification worlds

Milestone 9.17.1 uses **Supply Chain** as its canonical semantic world. This
document freezes the semantic model, pure oracle, causal production compiler,
branch-local MVCC execution, seeded stateful model, reproduction records, and
cost lanes completed through Phase 12. The runtime feature itself is documented
in [`BRANCH_LOCAL_MVCC.md`](./BRANCH_LOCAL_MVCC.md); this file owns the test
world and the extension contract retained for later merge certification.

## Authority split

The world definition, semantic keys, baselines, scenario deltas, oracle, and
expected observations contain descriptive meaning only. They do not contain a
Relational runtime, commit/version/snapshot IDs, branch heads, roots, indexes,
leases, pins, authority witnesses, or private constructors. Semantic names
such as `world.ports.southpoint` are not runtime handles and are never converted
to guessed integers.

Phase 3 compiles the same immutable definition through public Relational
schema and transaction facades and binds those names to owner-issued handles.
Its observed projection is a separate type and code path from the pure
oracle. No Phase 2 test may import a production DTO, facade, canonicalizer,
query projection, digest implementation, or lowering helper.

## Phase 3 causal baseline contract

The Phase 3 production slice adds only the adapter cone needed to certify an
installed baseline. `program.rs` owns the immutable semantic-to-public schema
and seed intent; `compiler.rs` creates a fresh runtime, installs the schema
through the initial-installation authority, stages bulk entity/relation intents,
commits through the public transaction facade, and obtains an owner-issued
snapshot; `handles.rs` binds semantic names through sealed commit-result
correspondences; normal, relation-aspect, and bulk relation creation all feed
that same owner correspondence; `observation.rs` reads the public snapshot view and maps its
records back through those bindings; and `baseline_audit.rs` compares that
projection with the independent expected observation. The pure definition,
oracle, and comparator remain outside this adapter cone.

Entity and relation handles are distinct typed bindings. An entity binding is
accepted only from the owner's created-entity correspondence. A relation
binding requires the corresponding owner-issued relation result; changed-record
order, endpoint matching, query lookup, allocator arithmetic, and guessed raw
IDs are not valid substitutes. The production result API must reject missing,
duplicate, ambiguous, wrong-kind, and foreign-runtime bindings before a
`CertifiedSupplyChainBaseline` can be constructed.

The deterministic `1000`/`2000` kind ranges in the schema vocabulary are
declaration keys for the Supply Chain schema only. They are never used to
derive an entity or relation identity; all record identities still come from
the owner's sealed commit correspondences.

Phase 3 certifies empty and operating Court/Standard baselines. The returned
baseline may carry a descriptive branch envelope for traceability and an
owner-issued snapshot, but that envelope is not an admitted branch basis and
cannot authorize a transaction. The Phase 4 fork-only source token and exact
fork provenance are now exercised by the production Supply Chain target; the
Phase 3 descriptive envelope remains non-operational. General branch-basis
admission/readmission and visible-root currentness are owner-issued Phase-6
operations: `observe_branch` returns a non-operational serializable descriptor
and an admitted basis, while reads consume only its
`RelationalBranchObservation`. Transported descriptors must pass owner
`readmit_branch_basis`, and external retention is represented by an explicit
lease. Phase 4 certifies runtime-clone
affinity, singular immutable catalog identity, exact source/target fork
provenance, typed empty/duplicate/stale/foreign denials, and constant
*logical* metadata-only fork deltas at 1/64/512 fan-outs. The probe does not
yet claim physical COW bytes or all population-scan absence; those need the
later currentness and cost gate. Scale remains a semantic/cost lane until its
production installation cost is separately bounded. The production compiler
uses the public
`PublicationConfig` override of 16,384 patch records
for these causal baselines; the Standard seed emits 8,211 records, so this is
an explicit fixture budget rather than a hidden split commit or validation
bypass. The empty declaration also executes a required public no-op commit
(zero patch records) so its owner-issued commit snapshot is observed through
the same public read-view path; there is no optional-commit or hand-built empty
observation fallback.

The adapter reports declaration, schema-installation, transaction, entity
binding, relation binding, observation, oracle, and comparison failures as
distinct typed stages. A failed construction or audit is fixture evidence, not
an MVCC product outcome. The required causal cases are
`supply_chain_world_compiles_causally_through_public_facades`,
`supply_chain_named_handles_are_owner_issued_and_complete`, and
`supply_chain_baseline_matches_independent_oracle`, plus one independent
mutation for every failure stage (including budget denial, wrong-kind,
wrong-endpoint, incomplete/duplicate correspondence, foreign or missing
snapshots, and unknown production identities) and the existing Fintech/generic
preservation suite. Private runtime access, production-derived expected state,
and imports from the pure oracle remain forbidden.

## Semantic world

The world has eight typed entity kinds: `Port`, `Terminal`, `Berth`, `Vessel`,
`Voyage`, `PortCall`, `CargoLot`, and `Inspection`. Their fields have explicit
units and meanings: capacities are units, berth depth is metres, cargo mass is
metric tons, and departure/arrival/inspection times are minutes from the
deterministic seed epoch. Status, posture, booking, inspection, vessel class,
region, and hazard values are closed enums. Hazard schema V1 and V2 are
different schema/meaning identities, not a renamed field.

The ten relation contracts are `TerminalAtPort`, `BerthAtTerminal`,
`VesselAssignedToBerth`, `VoyageUsesVessel`, `VoyageHasCall`, `CallAtPort`,
`CallPrecedes`, `CargoBookedOnVoyage`, `InspectionCoversVessel`, and
`SharesPilotageZone`. `CargoLot` also carries a deterministic,
customer-neutral synthetic code; that code is semantic data, not a customer or
runtime handle. Contracts pin endpoint kinds, minimum and maximum
cardinality, uniqueness, symmetry, ordered route edges, cross-partition
legality, and route acyclicity. Complete-world validation enforces minimum
cardinality; partial observation validation intentionally does not. Symmetric
pilotage edges are stored in both directions, `VoyageHasCall` follows
port-call sequence, and `CallPrecedes` links consecutive calls in one voyage.
Complete validation also rejects duplicate call ownership, orphan or missing
predecessor links, sequence gaps, and V1 cargo carrying the V2-only hazardous
meaning. Cross-region routes remain legal when their endpoint contracts are
valid. The semantic field vocabulary is exhaustive: port codes, vessel call
signs, inspection minutes, cargo customer codes, and schema meaning each have
distinct keys rather than aliases. Relation identity is a typed key independent
of its endpoints, so port-call rewiring preserves call identity.

The semantic owners are physically separated: `definition_entities.rs` and
`definition_relations.rs` declare the two topology families;
`scenario_delta_vocabulary.rs` owns the shared delta/precondition vocabulary;
and `comparison_state.rs` plus named ancestry/entity/schema/relation comparison
modules feed a small comparison orchestrator. These boundaries prevent read,
delta, and comparison modules from becoming mutual dependency authorities.

## Profiles and baselines

`SupplyChainScale` has three semantic profiles with the same meaning:

| Profile | Regions | Ports | Terminals | Berths | Vessels | Voyages | Calls | Cargo |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Court | 2 | 4 | 8 | 16 | 12 | 16 | 48 | 128 |
| Standard | 4 | 16 | 32 | 64 | 64 | 128 | 384 | 4,096 |
| Scale | 8 | 64 | 128 | 256 | 256 | 512 | 1,536 | 65,536 |

The complete-world totals are 244 entities/247 relations for Court, 4,848/
3,363 for Standard, and 68,544/38,019 for Scale. Every entity and relation
kind is checked against its profile formula, every port is checked against its
seeded region, and repeated construction with the same profile/seed is equal.

Named Court anchors remain present in every profile. Generated members use a
seed-derived name/value mix and represent every declared region (including
generated regions beyond North/South); they do not alter the Court facts.

The immutable baseline dependency graph is:

```text
EmptyInstallation -> Operating -> ContestedPlanning -> RetentionPressure
                  \-> VersionBoundary
```

- `EmptyInstallation` contains only profile/schema declarations and no
  records.
- `Operating` contains the complete accepted topology and baseline oracle
  state.
- `ContestedPlanning` adds validated Storm, Maintenance, Customs, and Rewire
  branch-creation intents without applying a branch delta or reference movement.
- `RetentionPressure` adds semantic descriptions of snapshot, observation,
  transaction, candidate, and external-basis obligations at named ancestors;
  each target must be a declared branch, every path is rooted at Operating,
  and every adjacent ancestor pair must be a declared branch-creation intent.
  These are not runtime pins or leases.
- `VersionBoundary` derives directly from Operating and names the pre-upgrade
  and post-upgrade hazardous-cargo schema descriptors. It does not depend on
  ContestedPlanning.

## Scenario deltas

The reusable typed `SupplyChainScenarioDelta` vocabulary contains exactly:

1. Storm Reroute Aurora;
2. Maintain Atlas Berth;
3. Hold Medical Cargo;
4. Expand Southpoint Capacity;
5. Competing Aurora Arrival;
6. Retire Atlas While Inspecting Aurora;
7. Rewire Aurora Port Call; and
8. Adopt Hazard Classification V2.

Each delta declares typed preconditions, a dedicated entity/relation/field/
schema/branch/history read footprint, exact entity/relation/field write
footprint, identity basis, invariant posture, and schema meaning. Target and
current relation endpoints are explicit read entities; unrelated source facts
are not smuggled into the footprint. Every V1 delta requires the V1 schema and rejects
reapplication on the same branch. The vocabulary is not a generic operation
bag and does not classify merges. A failed application returns a typed error
and leaves its input branch unchanged.

## Independent oracle and observations

`OracleState` is an ordered semantic map/set model of facts, relations,
absence, and schema identity. `OracleAncestry` separately owns recursive semantic
branch lineage, common-ancestor calculation, and accepted-delta order;
`OracleBranch` composes the two. Accepted history events retain their branch
owner and flattened order across forks. An explicit ordered-history fixture records
multiple accepted delta identities without pretending those branch-specific
domain mutations were successfully replayed on one branch. The oracle
does not use Relational MVCC, roots, indexes, queries, history classifiers,
production encoders, production digests, or latest-head selection.

`ExpectedSupplyChainObservation` is distinct from oracle state and the
`ObservedSupplyChainState` comparator carrier. It carries entity/relation
absence and full ancestry, while the observed carrier also preserves a raw
relation vector so duplicate-edge mutations cannot disappear into a map. The
comparator proves vector/map key and endpoint parity, rejects vector-only
omissions, replacements, and extras, and still accepts insertion permutations.
Its canonical bytes and digest are versioned and independently implemented.
Hand-authored vectors pin the Court baseline and every delta class, including
unchanged facts, explicit absence, ordered calls, relation sources and
targets, schema identity, and accepted-delta ancestry. Digest equality is only
a report; the comparator checks semantic fields and paths. The pure observed
carrier is structural evidence; the Phase 3 adapter separately certifies
causal production observation through the public snapshot/read-view facades.

Comparison failures distinguish missing writes, entity/relation absence,
sibling fact leaks, floating/latest-head selection, wrong parent and accepted
history, relation source/target changes, duplicate relations, illegal
endpoints, schema meaning drift, and unexpected facts. A wrong-ancestry probe
keeps domain facts equal and changes only lineage.

## Reproduction, mutation, and cost lanes

Every semantic trace records format version, profile, seed, baseline, branch,
ordered delta IDs, a typed mutation ID/operation, canonical pre-mutation input,
the raw duplicate-preserving relation-vector bytes, and the first typed
divergence. `SemanticTrace::replay` verifies the profile
format version and profile seed, reconstructs the baseline, applies the ordered deltas, executes the
typed mutation, reruns comparison, and returns the actual canonical mutated
input and divergence. The returned trace is itself replayable without replacing
its recorded pre-mutation input. A caller-supplied false first divergence is
rejected, typed delta-application errors retain their step and source, and
repeated replay is deterministic across Court, Standard, Scale, and the
non-empty named baselines. The replay fingerprint is supplemental evidence and
does not replace execution.

The production seeded model compiles every generated delta through the same
public schema, transaction, observation, and publication facades as the named
cases. It maintains a separate semantic model state, compares every accepted
step with an independently derived production observation, and records seed,
step, branch, operation, owner outcome, and first semantic divergence. Failure
records shrink by removing individual scenarios, normalizing the retained
scenario lifecycle labels, and replaying each candidate; a shrunk record must
preserve the same failure identity against a fresh runtime rather than a
captured production DTO. Controlled pause schedules cover the named
cancellation boundaries and keep pre-linearization interruption distinct from
performed work with a late interruption.

The Court ordinary lane permits one immutable definition, eight accepted delta
steps, 128 trace steps, 512 observations, 128 cargo lots, and 512 setup
entities/relations. Standard constructs its 4,848-entity/3,363-relation
definition with 8,192 observations and setup entities, 4,096 setup relations,
4,096 cargo lots, and 1,024-step oracle/trace ceilings. Scale is scheduled and
constructs its exact density under a 65,536-cargo, 70,000-entity, and 40,000-
relation ceiling. Every one of the seven dimensions is present in a typed,
machine-readable report bound to the profile and deterministic seed, and has an
independent over-budget denial; these are deterministic semantic budgets, not
timeout-only assertions.

The mutation matrix includes missing write, explicit entity/relation absence,
entity field value, relation source/target, sibling fact, floating branch,
wrong parent/history, duplicate relation, illegal endpoint, schema drift, and
deletion markers. Each has a valid positive state and a one-axis negative twin;
the six named trace controls are executable and the remaining controls are
covered by direct comparator/application courts. Production mutations such as
eager fork cloning, latest-root reads, global publication locks, and partial
root swaps are exercised by the MVCC root, selection, publication, and cost
courts; they are not inferred from the pure oracle.

## Ordinary and scheduled evidence commands

The ordinary certification command is:

```text
cargo test -p worth-relational --test relational_certification --no-fail-fast
```

Two public examples are executable compatibility contracts rather than
compile-only samples. The `Run tests` step of the `build-and-test` job runs
`cargo test --workspace`, and that command executes each example's own `main`.
No separate lane exists or is wanted: the manifest keys alone place these
examples in the lane that already runs.

| Executable example | Manifest declaration | Local reproduction |
| --- | --- | --- |
| `crates/worth-signal/examples/branch_bases.rs` | `[[example]] name = "branch_bases", test = true, harness = false` | `cargo test -p worth-signal --examples` |
| `crates/worth-relational/examples/branch_local_mvcc.rs` | `[[example]] name = "branch_local_mvcc", test = true, harness = false` | `cargo test -p worth-relational --examples` |

Cargo leaves an example at `test = false` by default, so an unregistered example
builds and never runs while the command above still reports success. `test =
true` is what makes it evidence, and `harness = false` makes `main` the whole
test, so there is no `test result:` line to count and the proof is the example's
own named output.

The complete Scale admission court, maximum 4,096-fork slope, and maximum
retained-history ceiling are `#[ignore]`d so the ordinary loop stays
responsive. They are not optional evidence: the `WORTH Relational scheduled
certification` job executes all three on the nightly schedule and on
`workflow_dispatch`, each under its own step and its exact compiled name.

| Scheduled proof (exact compiled name) | CI step in the `WORTH Relational scheduled certification` job |
| --- | --- |
| `scale_invariant_admission::large_runtime_keeps_global_enforcement_and_filters_graph_planning` | Run mandatory ignored Scale admission proof |
| `root_fork_sharing::phase5_standard_fork_copy_slope_is_flat_through_4096_forks` | Run mandatory ignored maximum fork-slope proof |
| `root_cost_scale_axes::selected_publication_cost_is_flat_through_documented_retention_ceiling` | Run mandatory ignored retained-history ceiling proof |

Each step runs the command below verbatim, so a local reproduction and its lane
cannot drift apart:

```text
bash scripts/ci/run_relational_named_test_selection.sh \
  --test relational_certification --exact --ignored \
  --selection scale_invariant_admission::large_runtime_keeps_global_enforcement_and_filters_graph_planning

bash scripts/ci/run_relational_named_test_selection.sh \
  --test relational_certification --exact --ignored \
  --selection root_fork_sharing::phase5_standard_fork_copy_slope_is_flat_through_4096_forks

bash scripts/ci/run_relational_named_test_selection.sh \
  --test relational_certification --exact --ignored \
  --selection root_cost_scale_axes::selected_publication_cost_is_flat_through_documented_retention_ceiling
```

Those are compiled test names, not source paths. The certification target
declares every module with `#[path]`, so `root/sharing/fork.rs` compiles as
`root_fork_sharing` and `root/cost/scale_axes.rs` as `root_cost_scale_axes`; a
filter written from the directory layout selects nothing and a bare
`cargo test` reports that as `0 passed`. The selection authority is what turns
that into a red lane: under `--ignored` it counts what will really execute and
fails at zero, so a renamed module, a deleted proof, or a proof that quietly
lost its `#[ignore]` convicts here instead of passing silently. `--exact` is
kept because `selected_publication_cost_is_flat_through_ordinary_retained_histories`
shares the ceiling proof's prefix.

The same job runs the two hostile CDC resume certifications as `--lib`
selections through that one authority.

Scale is scheduled because its production installation is the complete
106,563-record causal world, not a reduced substitute. The scheduled test
retains the independent definition/live-snapshot count, Global commit and
baseline-publication ceilings, direct GraphComposition `Touched` result and
one-call counters, ordinary publication lowering, ordinary graph exclusion,
and duplicate-rejection behavior.

## Feature-gated evidence commands

Two features hold evidence out of the ordinary command above.

Three certification courts are compiled only under the `test-operation-control`
feature, because they need the test-only boundary pause hook. The feature
supplies observation and pausing only; it cannot change authority, outcome
meaning, or production transition logic.

The allocation-slope court is compiled only under `allocation-probes`, which
installs a counting global allocator. An ordinary run must not pay for that
instrumentation, so without the feature the court does not exist rather than
being skipped.

The ordinary command compiles neither family, so each court has its own push/PR
CI step that names it exactly:

| Court | Gating feature | CI step in the `build-and-test` job |
| --- | --- | --- |
| `mvcc_cancellation_publication_boundaries::` (2 tests) | `test-operation-control` | worth-relational operation-control cancellation lane |
| `mvcc_owner_phase_locality::…preparation…` | `test-operation-control` | worth-relational operation-control preparation locality lane |
| `mvcc_owner_phase_locality::…publication…` | `test-operation-control` | worth-relational operation-control publication locality lane |
| `mvcc_owner_phase_locality::…settlement…` | `test-operation-control` | worth-relational operation-control settlement locality lane |
| `schema_transition_cancellation::…` | `test-operation-control` | worth-relational operation-control schema transition cancellation lane |
| `substrate_edition_budgets::allocation_slope::` (4 tests) | `allocation-probes` | worth-relational allocation-slope lane |

Each step runs the command below verbatim, so a local reproduction and its lane
cannot drift apart:

```text
bash scripts/ci/run_relational_named_test_selection.sh \
  --test relational_certification --features test-operation-control \
  --selection mvcc_cancellation_publication_boundaries::

bash scripts/ci/run_relational_named_test_selection.sh \
  --test relational_certification --features test-operation-control --exact \
  --selection mvcc_owner_phase_locality::paused_supply_chain_preparation_leaves_an_unrelated_branch_commit_unblocked

bash scripts/ci/run_relational_named_test_selection.sh \
  --test relational_certification --features test-operation-control --exact \
  --selection mvcc_owner_phase_locality::paused_supply_chain_publication_leaves_an_unrelated_branch_commit_unblocked

bash scripts/ci/run_relational_named_test_selection.sh \
  --test relational_certification --features test-operation-control --exact \
  --selection mvcc_owner_phase_locality::paused_supply_chain_settlement_leaves_an_unrelated_branch_commit_unblocked

bash scripts/ci/run_relational_named_test_selection.sh \
  --test relational_certification --features test-operation-control --exact \
  --selection schema_transition_cancellation::cancelled_schema_transition_leaves_no_target_or_branch_residue

bash scripts/ci/check_relational_allocation_probes.sh
```

The allocation lane keeps a wrapper because its declaration is long, not because
it is a second engine: `check_relational_allocation_probes.sh` names the four
tests it requires and then `exec`s the same selection authority.

That script is the single selection authority for named Relational lanes. Its
preflight and its execution share one filter vector, so a lane cannot assert one
selection and run another, and it fails the lane when the filter reaches zero
compiled tests or only `#[ignore]`d ones. The three locality selections are
`--exact` on purpose: a namespace filter proves only that some court ran, so a
deleted or renamed court would keep the lane green.

Reaching one executable test is the floor, and it is not enough for a lane whose
claim depends on a known set of tests running together. Such a lane declares
that set with repeated `--expect-name`, and the authority then requires the
declared names to be exactly the executable ones. The allocation-slope lane
needs that: its two driver tests re-execute the test binary with the probe gate
set, and its two isolated probes return immediately and pass when that gate is
absent, so deleting a driver would leave a selection that still lists tests,
still executes them, and still measures no slope at all.

The locality courts park one Supply Chain branch inside a real owner phase, at
the first `CandidatePreparation` observation, at `BeforeCriticalSection`, and at
the `Settlement` observation inside the one settlement executor, and require an
unrelated branch to complete a full ordinary commit through the public facade
while that park is held. The evidence is exact zero coordination contact and
wait deltas on the parked branch, an unchanged parked branch reference, a
maintenance head that advances exactly one canonical commit, and an oracle match
once both phases finish. A regression to a whole-runtime exclusive borrow
convicts at compile time rather than here; what these courts convict is a
runtime gate that serializes independent branches.

The settlement court is the one that reaches the pending-settlement registry
every owner phase contacts. Its park holds an installed record in `Executing`,
that record's per-commit executor gate, a published-snapshot slot, and a moved
but still unsettled canonical route, so migrating the single-executor gate to
registry scope, or holding the registry index lock across the settlement effect,
deadlocks any unrelated branch's ordinary commit and is convicted here. The
court proves its park position before relying on it, since a branch that has
already moved and still retains its pending record cannot be parked at an
earlier boundary, and it proves resumption leaves no residue: the exact commit
identity settles, the observed head is that canonical commit, and no pending
record survives.

Their reach is bounded accordingly. Preparation parks near the top of its phase
and cannot speak for a gate taken later in it. Settlement parks after the
durable append and derived completion have already returned, so it cannot speak
for a lock taken and released inside them, and the certified world is
memory-resident, so no durable-I/O locality is claimed.
`paused_settlement_locality::phase3_paused_settlement_does_not_block_an_unrelated_branch_commit`
makes the same settlement claim at the focused lib boundary, against a synthetic
schema rather than the production Supply Chain world.

Every wait in these courts is bounded and every exit opens the park before it
returns. That is load-bearing rather than tidy: a parked branch holds an
admitted runtime operation, and the owner close that dropping the world runs
waits on exactly that, so a court that panicked with its park still closed would
hang in drop and print no diagnostic at all. Each court therefore holds its
release in a guard it declares after the runtime, so drop order opens the park
before the close that waits on it, and the two exits that a scope join reaches
first open it themselves.

## Preservation evidence

Supply Chain is additive evidence. It does not replace Fintech, CAD, Chip, or
generic runtime behavior. The preservation lanes are:

```text
cargo test -p worth-relational --lib tests::domains --no-fail-fast
cargo test -p worth-relational --lib
cargo test -p worth-relational --test relational_certification --no-fail-fast
```

Focused owner-correspondence, branch-reference, and phase-boundary courts remain
required when their seams change. Strict Relational `--no-deps` Clippy,
formatting, boundary-check, generated agent-context validation, and dirty Rust
line-cap checks are closure gates. Existing unrelated workspace warning debt is
reported separately and cannot be cited as world or MVCC evidence.

## Phase 4 currentness and compatibility-court requirements

The canonical Phase-4 court remains Supply Chain; no phase-specific substitute
is permitted. The court must exercise a real owner-admitted branch basis and
branch-bound transaction, then compare production observations with the pure
Supply Chain oracle. Its fork/currentness matrix
must include:

- one shared immutable catalog artifact with distinct runtime-affine target
  branch cells;
- source advancement that cannot alter a forked target, and unrelated branch
  progress that cannot stale another binding;
- stale generation, stale truth version, foreign-runtime identity, duplicate
  target, empty source, malformed target, exact-parent drift, and missing-local
  basis denials with zero branch/catalog residue;
- metadata-only movement that increments generation without changing local
  truth, and truth movement that increments the local truth version; recovery
  must restore the exact cell or fail closed;
- merge-parent order and replay continuity proofs that use owner bindings and
  canonical artifacts, not raw branch ids or diagnostic heads; and
- setup-separated 1/64/512 fan-out counters for branch-cell lookup, catalog
  lookup, artifact construction, reference allocation, and cell contact. The
  per-fork deltas must be constant and no population scan may occur in the
  operation path. These are logical metadata-cost claims; physical COW bytes
  remain a later release-court obligation.

The independent oracle is mandatory for every positive and negative semantic
case. Production-derived expected state, catalog-latest reads, branch-head
maps, and guessed ids are not valid oracles. Every failure must be typed and
must compare branch-cell checkpoints, catalog length, and artifact identity
before and after the attempted operation.

The Phase-4 target does not call the then-existing historical-read,
downstream-publication, application-commit, or replay compatibility surfaces.
Direct behavior and compiler-boundary tests keep those surfaces outside the
transaction path. The application-commit compatibility case additionally
confirms that exact lease admission does not move branch currentness. Phase 6
has now removed the consumer-facing current/latest adapters: snapshot,
history, merge-basis, and change-source reads require an admitted exact
observation, while replay remains a separate cert-only lane.

The current boundary evidence commands are:

```text
cargo test -p worth-relational --lib
cargo test -p worth-relational --test relational_certification --no-fail-fast
cargo test -p worth-relational --lib merge_replay_continuity
cargo test -p worth-relational --test branch_reference_contract --quiet
cargo test -p worth-relational --test branch_reference_compile_time --quiet
cargo test -p worth-relational --test phase_boundaries_compile_fail --quiet
cargo clippy -p worth-relational --lib --no-deps -- -D warnings
```

The repository-wide Clippy command still reports known unrelated
`worth-signal` production warnings; those are recorded as debt and do not
replace the Relational `--no-deps` gate above.

## Phase 12 MVCC closure contract

The production court executes all eight named deltas through branch-bound
transactions and compares every exact-basis observation with the independent
oracle. It includes sibling isolation, shared ancestry, independent branch
progress, same-reference races, atomic old-or-new root visibility, stale and
foreign red controls, archive/delete behavior, cancellation before and after
linearization, and deterministic seeded mixed-operation traces.

Structural evidence is causal rather than inferential. Fork counters prove zero
copied truth and commit envelopes; allocation identities prove unchanged roots,
regions, and admitted schema registries remain shared at both small and large
registry sizes; touched-region counters prove write amplification follows the
declared footprint; and branch-local contact/wait counters prove unrelated
branches contribute no coordination work. Logical bytes and unique physical
authoritative bytes remain separate observations.

The Version Boundary lane installs Hazard Classification V2 through the public
schema transition, executes V1/V2 acceptance and rejection cases, observes
exact pre- and post-transition roots, and proves canonical replay and recovery
preserve schema meaning. The controlled transition-cancellation test is gated
by `test-operation-control`, so the ordinary certification command above does
not compile its test-only pause hook. The push/PR operation-control lanes build
the feature-enabled target and execute this court under its own exact named
selection; see [Feature-gated evidence commands](#feature-gated-evidence-commands).

## Contract retained for merge certification

Later merge work may add new branch programs, merge-specific observations, and
merge comparisons. It must not change these existing authorities:

- `SupplyChainDefinition` and its named baselines remain immutable semantic
  input, not production fixtures.
- `SupplyChainScenarioDelta` remains the sole vocabulary for the eight shipped
  domain changes; merge planning cannot reinterpret or mutate a delta.
- `OracleState`, `OracleAncestry`, and `OracleBranch` remain independent of
  production roots, queries, indexes, digests, and merge algorithms.
- Production observations continue to originate from exact owner bases and
  public snapshot/read facades.
- The comparator continues to check fields, absence, endpoints, duplicates,
  schema identity, ancestry, and accepted-delta order independently of digest
  equality.
- Reproduction records continue to rebuild a fresh runtime from semantic input;
  captured production state is not an oracle.

Merge certification therefore extends the world at named modules and typed
comparison seams. It does not replace the baseline, delta, observation, or
oracle rules with a merge-aware fixture.
