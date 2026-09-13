# Milestone 9.17.1: Exact Owner Bases And Relational Branch-Local MVCC

Status: Closed on 2026-08-28. Phases 1-13 are implemented, independently
reviewed, and verified. Milestone 9.17.1.1 is the required corrective
prerequisite discovered after closure; Milestone 9.17.1.2 is the final
owner-service correction. Milestone 9.17.2 must consume both
corrected owner-component surfaces and may not recreate component currentness,
retention, mutation, publication, or settlement authority in Runtime World.

## Goal

Establish the owner-local state model that every later product branch depends
on:

- one causally installed, independently modeled Supply Chain certification
  world that can prove semantic isolation, exact ancestry, and structural
  sharing now and support honest merge certification later;
- one shared Foundational vocabulary for immutable bases, mutable branch
  references, exact reference observations, forks, comparisons, and movements;
- private-minted, runtime-affine Relational and Signal component bases that raw
  ids, equal ordinals, copied descriptors, digests, and derived state cannot
  substitute; and
- genuine Relational branch-local MVCC in which observation, transaction,
  conflict, publication, history, and retention authority are qualified by one
  exact branch reference and unrelated branches make concurrent progress;
- explicit in-memory residency and bounded retention for every live owner basis,
  transaction, snapshot, and branch root.

MVCC is the primary deliverable. The certification world ships first because
the implementation must not author its own oracle after its mechanics are
chosen. The vocabulary and owner-basis work exists to make MVCC authority
honest and to give 9.17.2 a stable language for composing owner results. This
milestone does not create a composite world commit, a product branch, Runtime World
composition authority, or a public Query branch workflow.

Closure means a caller can observe an exact Relational branch basis, execute a
repeatable branch-local transaction without broadly borrowing the runtime,
prepare detached immutable work, compare-and-publish through that branch's
owner authority, and receive a new exact committed basis. At the same time, a
writer deliberately blocked on another branch cannot prevent that lifecycle.

## Roadmap Placement

[Milestone 9.17](./milestone-9.17.md) is the governing umbrella. Milestone
9.16.1 carries typed branch affinity through Query's provider-session
progression, but intentionally stops before multiple live heads, owner-local
MVCC, and exact Relational-plus-Signal component composition. Milestone 9.16.2
is closed before this work begins; its package identity and fresh-readmission
contracts are inherited. Package records and reconstructed definitions carry no
component authority.

9.17.1 supplies the owner facts:

```text
Foundational descriptive reference grammar
        |                                  |
        v                                  v
Relational owner                       Signal owner
        |                                  |
        v                                  v
exact admitted basis                  exact admitted basis
        |
        v
branch snapshot -> detached transaction -> prepared candidate
        -> branch compare-and-publish -> performed owner commit
        -> next exact Relational basis

No arrow above mints a product branch or composite commit.
```

[Milestone 9.17.1.2](./milestone-9.17.1.2.md) first completes the Relational
bundle and Signal independent-progress boundary. Then
[Milestone 9.17.2](./milestone-9.17.2.md) may orchestrate the frozen owner ports
through the dedicated Runtime World composition owner. It may not reconstruct
owner currentness from descriptors or wrap either owner runtime in a global
mutex. [Milestone 9.17.3](./milestone-9.17.3.md) then carries the resulting
composite authority through Query.

## Authority And Responsibility Matrix

| Concern | Canonical owner in 9.17.1 | Explicitly not the owner |
| --- | --- | --- |
| Cross-runtime branch/reference meaning | `worth-foundational` descriptive values | runtime head maps, freshness, legality |
| Authority carrier, binding, admission/readmission, and performed progression law | `worth-proof` | branch identity, currentness, or live owner state |
| Owner-specific authority seal and checked issuance | Relational or Signal authority module over concrete `worth-proof` carriers | callers, Foundational, Bridge, Query |
| Relational branch head, MVCC, conflict, commit, and retention | `worth-relational` | Query, Bridge, Foundational, Proof |
| Supply Chain semantic definition, production compiler, independent oracle, and comparison | Relational certification subsystem, with production effects only through public owner facades | private runtime access, production-derived expected results |
| Signal definition/snapshot branch basis and lifecycle | `worth-signal` | Relational, Query, Bridge |
| Installed component correspondence | Runtime Bridge | either component owner in 9.17.1 |
| Future composite history and product currentness | 9.17.2 Runtime World owner | either component owner in 9.17.1 |
| Query session carriage and public workflow | 9.17.3 | this milestone |

Foundational standardizes what a branch reference observation means when it
crosses a boundary. Proof supplies the concrete witness, binding, freshness,
readmission, checked-outcome, linear-resource, and performed-effect carriers.
The runtime owner defines the sealed marker and is the only module able to
issue a concrete Proof carrier after its live checks. Only that owner can say
that an observation is current, issue a retention lease, admit a transaction,
or move its branch reference.

## Current Boundary And Required Cutover

### Foundational

`crates/worth-foundational/src/transitions/branches/vocabulary.rs` currently
provides `FoundationalBranchId`, epoch-shaped fork/observation bases, and a
comparison basis. These types distinguish branch-local candidates from
committed authority, but they do not describe the mutable-reference fact that
both Relational and Signal now need: one named branch reference was observed at
one generation and targeted one exact immutable owner basis.

The new grammar must extend the existing Milestone 5 vocabulary rather than
creating Relational and Signal dialects. It remains descriptive and
constructible; construction proves structural validity only, never owner
currentness.

### Relational

The existing implementation has most nouns but not the required authority
topology:

- `history/data/mod.rs` places `commit_id`, global `version_id`, `branch_id`,
  and parents together in `CommitReference`; a fork copies that value into the
  new branch head even though its embedded branch remains the source branch;
- `runtime/state/subsystems/history.rs` holds every branch head plus global
  commit/version allocation in one history subsystem;
- `transactions/runtime_entry.rs` creates `RelationalTransaction<'a>` from
  `&'a mut RelationalRuntime`, making public independent branch mutation
  impossible even before any lock is considered;
- `transactions/data/primitives.rs` allows optional `target_branch`, optional
  `ExpectedBranchHead`, ambient defaults, and commit-id-only expected-head
  comparison;
- `history/authority/commit_publication.rs` advances global sequences and
  mutates branch heads, the commit graph, envelopes, and patch-stream indexes
  through one broad runtime authority; and
- bridge presentation currently finds a branch by string, reads its current
  head, and then loads a commit envelope. That is a projection path, not an
  exact reusable owner basis.

The destination must split immutable commit identity from mutable branch
reference state, replace the broad transaction borrow with independently
borrowable owner services, and make branch-reference movement the only
Relational currentness transition.

This milestone adds no persistence backend. Relational's branch axes must remain
semantic owner facts so Worth Store can later persist them without discovering
or redefining currentness inside a physical adapter.

### Signal

Signal already has `SignalBranchId`, `SignalBranchHandle`, snapshot ids,
branch-head generations, `SignalBranchBasisArtifact`, owner validation, fork,
restore, and targeted transactions. The current public meaning is spread
between `state/lifecycle.rs` and
`logic/transaction/runtime/state/branching/`. A targeted transaction also
temporarily transfers stored branch state into the active runtime, so its
mechanics are not the model for Relational concurrency.

9.17.1 does not replace Signal's engine. It cuts Signal's public branch
identity, reference observation, fork comparison, basis, and readmission
surfaces over the same Foundational grammar used by Relational. Its private
numeric keys and graph storage may remain private implementation details.

Signal state remains memory-resident. Its exact basis and owner readmission
surface must be sufficient for Runtime World composition without introducing
a serialized component artifact in this milestone.

### Shared substrates

### Relational certification-world deficit

Relational has broad test coverage and a substantial Fintech fixture, but it
does not yet have a world capable of certifying this milestone honestly:

- `tests/domains/fintech/fixture/FintechWorld` exposes and mutates a
  `RelationalRuntime` directly rather than forcing scenarios through one
  public certification driver;
- branch actions construct raw `BranchId("main")`, `"analysis"`, and
  `"audit"` values and call history authority directly;
- most named Fintech scenarios currently select the same baseline setup and
  depend on later imperative workflow calls for their real preconditions;
- Fintech probes obtain observed values through production query code and then
  compare those production-derived projections, so they are useful parity
  checks but not an independent semantic oracle for MVCC; and
- the generic seeded scenario harness uses entity, relation, snapshot, and
  branch slot numbers. It explores sequences, but the slots are not a
  causally named world and do not prove physical sharing or future merge
  meaning.

These are not reasons to delete the existing suites. They explain why this
milestone must first add a canonical, causally complete, merge-ready
certification world with a public-facade runtime adapter and an independently
authored model oracle. Fintech remains a preservation suite and may later adopt
the new world infrastructure where its own proof claims benefit.

### Dependency Direction

The manifests already permit Relational and Signal to consume Foundational and
Proof. Relational also depends on Runtime Bridge for existing presentation
contracts, while Runtime Bridge depends on Signal. Therefore:

- shared reference grammar belongs in Foundational;
- concrete authority carriage and progression belong to the existing Proof
  types rather than a local witness/proof framework;
- the owner-specific marker and its sealed minting function live in the
  Relational or Signal authority module, because moving the seal into Proof
  would either prevent the owner from minting or require a forgeable public
  minting door;
- concrete component artifacts and mutation ports belong in their owners;
- no new Bridge type may be required for owner-local MVCC to work; and
- 9.17.2 must compose exported owner facades without introducing a dependency
  cycle or moving Relational authority into Bridge.

## Canonical Certification World: Supply Chain

### Why this world

The Supply Chain world is a deterministic synthetic logistics-operations
world. It is the canonical Relational certification world for 9.17.1 and the
retained baseline for later Relational merge certification.

The domain is chosen because alternative operational plans naturally fork from
one accepted world, share almost all infrastructure and history, and diverge on
small graph regions. It supplies meaningful instances of:

- independent changes to disjoint ports;
- conflicting changes to the same voyage, vessel, berth, or cargo lot;
- relation endpoint rewiring when a vessel or port call is reassigned;
- maximum-cardinality and uniqueness pressure over berth assignment;
- deletion versus update/inspection pressure;
- ordered route topology and acyclicity;
- cross-partition voyages with partition-local infrastructure;
- schema/definition coexistence for hazardous-cargo classification; and
- dense unchanged cargo and infrastructure suitable for structural-sharing
  and memory-amplification measurements.

The world is synthetic, contains no captured identities or secrets, and scales
by deterministic topology parameters without changing its semantics.

### Baseline topology and contracts

The operating baseline contains two named geographic partitions, North Reach
and South Reach, and these semantic entity kinds:

| Entity kind | Required baseline meaning |
| --- | --- |
| `Port` | stable code, name, region, and operating posture |
| `Terminal` | port-local terminal identity and capacity |
| `Berth` | terminal-local berth, depth, capacity, and open/maintenance posture |
| `Vessel` | call sign, class, capacity, and operating posture |
| `Voyage` | vessel plan, status, departure/arrival minute, and revision |
| `PortCall` | one ordered stop in a voyage plan |
| `CargoLot` | mass, class, hazard posture, booking status, and customer-neutral synthetic code |
| `Inspection` | owner-issued inspection occurrence, result, and inspected vessel |

The baseline relation families are:

| Relation | Contract exercised |
| --- | --- |
| `TerminalAtPort` | terminal has exactly one port; endpoint-kind validity |
| `BerthAtTerminal` | berth has exactly one terminal; endpoint-kind validity |
| `VesselAssignedToBerth` | a vessel has at most one current berth assignment; uniqueness/cardinality |
| `VoyageUsesVessel` | a voyage uses exactly one vessel |
| `VoyageHasCall` | a voyage has at least two calls in the accepted baseline |
| `CallAtPort` | a call targets exactly one port, including lawful cross-partition voyages |
| `CallPrecedes` | directed acyclic route ordering |
| `CargoBookedOnVoyage` | a cargo lot is booked on at most one current voyage |
| `InspectionCoversVessel` | inspection occurrence targets one vessel; retained audit topology |
| `SharesPilotageZone` | declared symmetric port relationship |

Exact minimum-cardinality enforcement may use the existing deferred/commit
posture where Relational already requires it, but the accepted baseline and
every successful scenario must satisfy the complete declared graph. The world
compiler records which invariant was checked synchronously and which was
verified by baseline certification; it may not silently weaken either.

The court profile exposes named semantic handles including:

- `world.ports.meridian` and `world.ports.southpoint`;
- `world.terminals.meridian_container`;
- `world.berths.atlas` and `world.berths.beacon`;
- `world.vessels.aurora`;
- `world.voyages.aurora_eastbound`;
- `world.calls.aurora_meridian` and `world.calls.aurora_southpoint`;
- `world.cargo.medical_supplies` and `world.cargo.machine_parts`; and
- `world.inspections.aurora_arrival`.

Tests consume these handles. They never reconstruct valid runtime identities
from slots, integers, names, or digests.

### Scale profiles

`SupplyChainScale` is a deterministic semantic profile, not an arbitrary row
count:

| Profile | Regions | Ports | Terminals | Berths | Vessels | Voyages | Port calls | Cargo lots | Ordinary lane |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `Court` | 2 | 4 | 8 | 16 | 12 | 16 | 48 | 128 | every MVCC correctness run |
| `Standard` | 4 | 16 | 32 | 64 | 64 | 128 | 384 | 4,096 | incremental structural-sharing and property lane |
| `Scale` | 8 | 64 | 128 | 256 | 256 | 512 | 1,536 | 65,536 | scheduled slope/memory lane |

Generated members remain semantically named from the declared seed and
coordinates. The named court anchors exist in every profile. Increasing a
profile adds unrelated ports, voyages, and cargo around those anchors; it does
not alter the court scenario's intended facts.

### Canonical baseline portfolio

The world subsystem owns these reusable immutable declarations:

1. `EmptySupplyChainInstallation`: runtime profile plus admitted schema, no domain
   records.
2. `OperatingSupplyChainBaseline`: one production-authored seed commit containing
   the complete accepted topology and baseline oracle state.
3. `ContestedPlanningBaseline`: the operating baseline plus named, validated
   branch-creation intents for storm, maintenance, customs, and rewire
   alternatives, with no reference or branch delta yet applied. Each court
   materializes the required references through the public fork facade.
4. `RetentionPressureBaseline`: contested planning plus descriptive snapshot,
   observation, transaction, candidate, and external-basis retention paths at
   named ancestors. The Phase 2 semantic world does not admit runtime
   snapshots, leases, or external-basis pins; production-backed phases later
   lower these obligations into owner-issued retention artifacts.
5. `SupplyChainVersionBoundary`: operating baseline with the declared pre-upgrade
   hazardous-cargo schema descriptor used by readmission and later merge
   coexistence tests.

These are semantic world states, not five independently rebuilt fixtures.
Each later state is produced from the prior state by a named production-valid
delta. Tests request the narrowest state that establishes their preconditions.

### Scenario-delta vocabulary

`SupplyChainScenarioDelta` is an independent semantic action program. The initial
program contains:

- `StormRerouteAurora`: change Aurora's voyage status and arrival minute,
  rewire its South Reach call, and leave the North Reach infrastructure
  unchanged;
- `MaintainAtlasBerth`: close Atlas for maintenance, assign Aurora to Beacon,
  record the distinct maintenance-delay voyage status/arrival plan, and leave
  cargo bookings unchanged;
- `HoldMedicalCargo`: change only the medical-supplies cargo status;
- `ExpandSouthpointCapacity`: change a disjoint Southpoint terminal and berth;
- `CompetingAuroraArrival`: assign a different arrival minute to the same
  voyage field as the storm plan;
- `RetireAtlasWhileInspectingAurora`: retire an infrastructure endpoint while
  another branch records a dependent inspection/update path;
- `RewireAuroraPortCall`: change relation endpoints while retaining call
  identity and route ordering; and
- `AdoptHazardClassificationV2`: apply the declared schema/meaning boundary
  used by later merge compatibility tests.

All deltas are executable as ordinary branch-local transactions in 9.17.1.
9.17.1 does not merge them. Later merge milestones consume the same baseline,
handles, deltas, and oracle to classify disjoint adoption, same-field conflict,
delete/update conflict, endpoint rewiring, topology conflict, and schema
incompatibility without inventing a new test world.

### World construction and independent oracle

The world has four separately diagnosable stages:

```text
SupplyChainWorldDefinition
    -> CompiledSupplyChainProgram
    -> ProductionSeededSupplyChainWorld
    -> CertifiedSupplyChainBaseline

SupplyChainWorldDefinition
    -> SupplyChainOracleBaseline
    -> SupplyChainOracleBranch + SupplyChainScenarioDelta
    -> ExpectedSupplyChainObservation
```

`CompiledSupplyChainProgram` contains immutable, canonically ordered schema and
semantic seed intent. It is reusable across tests. Each test receives a fresh
runtime namespace and executes that program through the real public Relational
schema and bulk-transaction facades. In Phase 3 the compiler returns
owner-issued entity, relation, and snapshot handles plus a descriptive
baseline-branch envelope and a typed construction report. That envelope is
carry-only metadata; it is not an admitted operational basis, does not mint
currentness, and cannot start a transaction. Exact owner-issued branch
reference/basis observation is introduced by the Phase 4/6 Relational
reference and admission work. The compiler has no private head, version,
storage, index, retention, or authority constructor.

`SupplyChainOracle` is a pure domain model keyed by semantic supply-chain
identities. It interprets `SupplyChainScenarioDelta` directly into maps, sets,
ordered call lists,
branch ancestry, and expected domain facts. It does not import Relational
transaction, MVCC, conflict, merge, query, index, canonicalization, digest, or
history algorithms. The runtime adapter separately lowers the same semantic
delta into public Relational intent. The observation adapter reads public
branch observations and projects them into semantic supply-chain rows. Only then
does a comparator evaluate expected versus observed state.

Sharing one declarative delta between adapters is allowed; sharing the disputed
classifier, comparator, normalization, branch selection, ancestry resolution,
or digest implementation is not. Fixture construction failure, runtime action
failure, observation failure, oracle failure, and comparison failure are
different typed test outcomes.

## Adversarial Courtroom

The closure court compiles `ContestedPlanningBaseline` through the public
facade. Its `storm` and `maintenance` branches are forked from the exact
retained operating-baseline commit. Both branches update
`world.voyages.aurora_eastbound` to different status and arrival values; they
also make different relation changes around the same vessel. This prevents
partition separation or disjoint fixtures from masquerading as branch
isolation.

1. The world compiler and baseline auditor establish the production-issued
   handles, schema, topology, baseline commit, exact root, and oracle state.
2. Fork `storm` and `maintenance`. Their initial reference observations differ
   by branch identity/generation but target the same immutable commit and
   branch root. Fork reports zero record, relation, history-envelope, and
   authoritative-byte copying.
3. Observe exact bases `S0` and `M0`. Begin transactions from both and confirm
   repeatable reads plus read-your-writes overlays.
4. Apply `StormRerouteAurora` on `storm` and pause after all fallible
   preparation but before the publication critical section. The pause may hold
   storm state and candidate retention; it may not hold a runtime-global
   mutation guard, allocator lock, catalog lock, maintenance lock, or
   maintenance-branch state.
5. Apply and commit `MaintainAtlasBerth` on `maintenance` while `storm` remains
   paused. The Supply Chain oracle must match `M1`; the baseline and storm
   observations must remain byte-for-byte/semantically unchanged.
6. Release and commit `storm` to `S1`. The Supply Chain oracle must match `S1`; it
   must not contain maintenance's berth assignment. Both commits share the
   baseline ancestor and unchanged storage regions without sharing mutable
   branch fate.
7. Race two `CompetingAuroraArrival` writers from the same exact `S1`
   reference. Exactly one publishes; the other receives a typed stale-reference
   outcome naming expected and observed references. It performs no partial
   mutation.
8. Delete the maintenance reference while retaining an `M1` snapshot and
   baseline pin. Reclaim no `M1` truth while that snapshot remains; after its
   release, reclaim only maintenance-unique unretained regions. Storm,
   baseline, shared ancestry, and every still-pinned observation remain valid.

The same court includes:
- `A1` and `B1` with equal branch-local version numbers and unequal authority;
- a branch-A id paired with branch-B reference generation, commit, snapshot,
  schema basis, owner id, or retention lease;
- a valid old commit id paired with the current-looking local version;
- a copied or deserialized descriptor used without owner readmission;
- a fork whose new reference targets the retained source commit but begins its
  own local version line;
- a metadata-only reference movement whose reference generation changes even
  when its truth version and content root do not;
- `HoldMedicalCargo` committed on a third branch while storm and maintenance
  touch Aurora, proving both semantic isolation and sharing of unrelated cargo;
- a Standard/Scale fork fan-out in which 4,096 branch references initially
  target one immutable root and one baseline commit without 4,096 copies;
- single-record and single-relation deltas that copy only the declared
  immutable storage regions and correctness-index paths, never all cargo,
  ports, history, or sibling roots;
- archive while snapshots, transactions, prepared candidates, and external
  component pins remain live;
- cancellation before observation admission, after snapshot pin, during
  planning, after candidate creation, immediately before publication, and
  after the publication linearization point;
- unrelated identity allocation, history append, and reclamation pressure
  during branch publication;
- one exact Signal basis reused by at least 64 future-consumer holders without
  graph clone, evaluation, or current-head lookup;
- foreign-runtime, stale-reference-generation, restored-but-unreadmitted, and
  definition-incompatible Signal descriptors; and
- a Signal cache digest, snapshot id, branch handle, or diagnostic artifact
  offered where admitted basis authority is required.

The independent oracles inspect public owner observations, immutable commit
history, and lifecycle counters through paths different from the write path.
They must prove:

- branch B completes while A is paused;
- the only visibility change is the selected branch reference movement;
- readers observe either the complete before root or complete after root,
  never mixed storage/index/schema/history state;
- a losing race and every pre-linearization cancellation leave the reference,
  truth root, history, patch stream, and retention baseline unchanged;
- post-linearization cancellation returns the performed result rather than
  reporting a false cancellation;
- all exact live obligations prevent reclamation and the last release makes
  the retired basis reclaimable;
- the number of unique canonical commit artifacts equals performed commits,
  not branch-reference count;
- incremental fork work and unique bytes attributable to branch fan-out remain
  flat as unrelated world size and baseline history grow;
- branch-local publication allocates only touched immutable regions plus the
  declared root/index path, while untouched region identities remain shared;
- Signal reuse performs zero owner work after the initial admitted basis is
  shared;
- each committed basis traces to one performed owner publication;
- every Relational or Signal descriptor crossing an authority boundary requires
  fresh owner readmission before operational use.

Any of these implementations must fail the court:

- per-branch mutexes reached through one global `&mut RelationalRuntime`;
- a global coordinator hidden behind an actor, queue, facade, or async task;
- branch ids added to transactions while storage or publication remains one
  mutable world;
- commit id, version, generation, or digest comparison used as authority;
- eager deep clone on fork, whole-world clone on first write, duplicated
  ancestor envelopes, or refcount/pin accounting presented as copied truth;
- pointer equality used as the only sharing oracle without stable owner-issued
  structural observations and byte/copy counters;
- derived-state swap followed by a head update in a separate visible step;
- tests that mutate private head maps or use a test-only coordinator; or
- a Bridge wrapper around pre-9.17.1 publication.

## Product Decision Lock

### 1. Shared branch-reference grammar

The platform vocabulary distinguishes:

- **branch identity**: the stable name/key of one mutable reference;
- **immutable target basis**: the owner-defined immutable state selected by a
  reference, or the typed empty basis;
- **reference generation**: the monotonic generation of that one mutable
  reference;
- **exact reference observation**: branch identity + immutable target basis +
  reference generation;
- **fork basis**: the exact source reference observation from which a distinct
  target reference was created;
- **comparison basis**: the exact expected reference observation used by a
  conditional movement; and
- **movement description**: exact before and after observations plus movement
  kind, without a claim that an owner performed it.

`worth-foundational` owns these meanings. Relational and Signal must reuse
them in their boundary artifacts instead of publishing local tuples. Owner
runtime identity, liveness, currentness, locks, tables, clocks, counters,
retention, and authority do not move into Foundational.

The canonical vocabulary additions are generic over the owner's descriptive
immutable target, not over owner authority:

- `FoundationalBranchTarget<TargetBasis>` with explicit `Empty` and
  `Basis(TargetBasis)` variants;
- `FoundationalBranchReferenceGeneration`;
- `FoundationalBranchReferenceObservation<TargetBasis>` containing exactly
  `FoundationalBranchId`, `FoundationalBranchTarget<TargetBasis>`, and
  reference generation;
- exact-reference `FoundationalBranchForkBasis<TargetBasis>` and
  `FoundationalBranchComparisonBasis<TargetBasis>` wrappers;
- `FoundationalBranchReferenceMovement<TargetBasis>` containing exact before
  and after observations plus a structural movement kind; and
- `FoundationalBranchReferenceMismatch<TargetBasis>` preserving which
  structural axis differed.

`TargetBasis` must be an immutable, equality-comparable, canonically encodable
owner descriptor. It is not a trait object, callback, owner handle, proof
marker, retention lease, or currentness capability. Relational specializes it
with its immutable commit/root descriptor; Signal specializes it with its
snapshot-plus-definition descriptor. This generic structural reuse is the
shared vocabulary. It does not create a generic operational branch runtime.

Phase 1 freezes the adapter vocabulary and the authority-door shape; it does
not pretend that a hand-built descriptor is a production observation. The
Relational and Signal phase-1 adapter tests use deterministic owner-shaped
fixtures to pin field mapping, owner affinity, and canonical bytes. Causal
production lowering is a separate proof obligation: Relational is proved when
the Supply Chain compiler installs owner-issued handles in Phase 3, and Signal
is cut over from its live basis engine in Phase 11. A phase-1 `admit_*` helper
may prove only that a concrete owner witness is required; it must not be
described as currentness validation, readmission, or a live owner path until
the owning phase supplies the runtime-issued witness and denial cases.

The existing epoch-only fork/observation shapes are superseded for operational
branch references. They may remain only where Milestone 5's non-authoritative
candidate grammar genuinely describes an epoch rather than a reference. No
compatibility constructor may silently synthesize an exact reference from an
epoch or equivalence id.

### 2. Immutable commit and mutable branch reference are different facts

A Relational commit is immutable and may be targeted by more than one branch
reference. It carries commit identity, ordered parent commit identities,
canonical truth/schema roots, canonical patch/publication material, and
authoring provenance. Its authoring branch is provenance, not part of the
commit's reusable identity.

A Relational branch reference is mutable owner state. Its current observation
contains:

- Relational runtime-instance identity;
- Foundational branch identity;
- exact target commit or typed empty target;
- branch-local truth version;
- branch-reference generation;
- exact canonical truth and schema basis roots;
- lifecycle posture; and
- the retention identity needed to keep that observation available.

The current `CommitReference` must not remain the authority for both facts.
Its responsibilities are split and every public consumer cuts over. A legacy
alias or second head table is forbidden.

### 3. Version and generation law

- `RelationalBranchVersion` orders truth-bearing commits only within one
  Relational runtime and one branch.
- A new branch forked from a retained source commit begins at local version
  zero while targeting that exact source commit.
- The first truth-bearing commit authored on that branch advances its local
  version to one.
- `FoundationalBranchReferenceGeneration` advances on every successful
  reference movement, including metadata-only movements whose canonical truth
  root and branch-local truth version remain unchanged.
- Commit identity and patch-stream position remain globally unique within the
  runtime, but neither orders or identifies branch currentness by itself.
- Equal versions, generations, commit ids from another runtime, or basis
  digests never imply equivalent authority.

Overflow is a typed owner failure before effects. Wrapping, saturation, and
generation reuse are forbidden.

### 4. Fork law

Fork creation is one owner transition from an admitted exact source
observation to a new branch reference. The target reference:

- has a newly owner-minted branch identity and its own generation line;
- initially targets the exact retained source commit and roots;
- records the exact source observation as fork provenance;
- creates or acquires its own head-retention obligation; and
- does not copy a commit value whose embedded identity claims it belongs to the
  new branch.

Source movement after the fork does not move the target. Forking empty state is
explicit, not encoded as a missing lookup.

### 5. Exact owner component bases

Each owner exports two different surfaces:

- a serializable/descriptive component-basis descriptor suitable for carriage;
  and
- a private-minted admitted component basis suitable for operations.

The admitted Relational basis binds runtime instance, exact branch-reference
observation, truth/schema roots, retention lease, and owner proof. The admitted
Signal basis binds runtime/graph instance, exact branch-reference observation,
snapshot and definition basis, restore posture, retention/lifecycle, and owner
proof.

Deserialization, checkpoint restoration, process transfer, or owner restart
produces only a descriptor with weakened freshness. Operational reuse requires
the named owner readmission port. Owner identity mismatch, unknown branch,
unavailable retained target, stale generation, incompatible schema/definition,
archived posture, and unsupported descriptor version remain distinct outcomes.

An admitted immutable basis is safe to share inside the admitting process and
is `Clone + Send + Sync` through one shared owner-issued lease; cloning it does
not reacquire, revalidate, or extend owner retention independently. Governed
external retention is a separate explicit pin operation with its own typed
release. Descriptors are serializable but never operational. Owner service
handles used for independent branch work are `Clone + Send + Sync` and expose
only capability-scoped methods, not a cloneable mutable runtime.

### 6. Relational MVCC isolation contract

9.17.1 implements branch-scoped optimistic MVCC with conservative exact-head
validation:

- begin from one admitted exact branch basis;
- reads are repeatable against that immutable basis;
- the transaction observes its own overlay writes;
- validation uses the same immutable basis, authoritative read footprint,
  write footprint, schema basis, and branch invariants;
- publication succeeds only if the branch's complete current reference still
  equals the expected observation; and
- any intervening reference movement returns stale, even when changes appear
  disjoint. Automatic rebase and merge are later milestones.

Successful publications therefore form a serial order per branch. 9.17.1 does
not claim a cross-branch serial order or cross-runtime transaction isolation.
Cross-branch reads require separately admitted immutable observations and are
never smuggled into one branch transaction as ambient current state.

### 7. Independently borrowable branch state

The public ordinary transaction path must not retain `&mut RelationalRuntime`
for the transaction lifetime. A transaction holds:

- an admitted immutable observation and retention lease;
- detached transaction identity and overlay;
- branch-qualified read/write footprints;
- immutable schema/planning services or narrowly scoped handles; and
- an owner publication port used only after preparation.

The runtime owns independently addressable branch coordination cells. Private
storage may use atomics, sharded maps, immutable roots, or another honest
mechanism, but neither Rust borrowing nor hidden runtime locking may serialize
unrelated branch work. Global id and stream-position allocation may use bounded
nonblocking atomics; it cannot hold or wait behind user work.

### 8. Canonical branch root and visibility linearization

One immutable `RelationalBranchRoot` is the canonical visible state selected
by a branch reference. It binds the authoritative storage/version root,
schema root, correctness-critical index roots or explicit fallback posture,
visibility root, and canonical commit identity.

The branch reference compare-and-replace is the visibility linearization
point. Public readers see the complete prior root or complete next root. The
next root owns or references the complete canonical commit artifact and its
parent link, so current reads and branch-history reads do not depend on a later
global-catalog update. They must never assemble currentness from separately
mutable storage, index, schema, history, or visibility tables.

The publication critical section is branch-local, bounded, non-cancellable,
and contains no user callbacks, graph scans, history scans, I/O, derived-index
rebuilds, cache work, or post-commit consumers. All fallible preparation occurs
before it. Once entered, it resolves to a determinate performed, stale, denied,
or failed-before-movement owner outcome.

### 9. Canonical commit and derived state

One canonical immutable commit artifact is authored on successful publication.
It is prepared privately, becomes authoritative only through the successful
branch-root movement, and is reachable immediately from that root. A global
commit-id or patch-position index is a lookup accelerator, not publication
authority; it may expose an entry only when it can resolve the performed
artifact, and an accelerator miss must use the declared canonical fallback.
History, receipts, patch-stream views, Bridge component views, diagnostics,
and replay inputs derive from the performed artifact; no peer commit
representation may disagree. Losing candidates and pre-linearization failures
must not enter public history or patch streams, and any private reservation
residue is released before the outcome returns.

Correctness-critical indexes needed to answer ordinary queries must either be
part of the atomic branch root or carry an explicit authoritative fallback that
is valid immediately. Optional caches and diagnostic projections may lag and
must declare that posture. A derived index, cache, digest, or projection cannot
move a branch or mint a component basis.

### 10. Prepared candidates and performed publication

`PreparedRelationalCommitCandidate` is an opaque, runtime-affine,
branch-bound, attempt-bound, non-cloneable owner artifact. It contains detached
immutable roots and all completed fallible validation but no performed commit
claim. It retains its base and prepared roots until consumed or discarded.
It may be moved once across an in-process worker boundary (`Send`), is not
`Clone`, `Sync`, or serializable, and cannot survive owner restart as an
operational artifact.

Only the Relational owner publication port may consume it. Publication
revalidates the complete expected reference and returns exactly one of:

- `PerformedRelationalCommit`, carrying the canonical commit and next admitted
  basis;
- `StaleRelationalBranchObservation`, carrying exact expected and observed
  descriptors;
- a typed denial for lifecycle, retention, schema, invariant, or owner
  mismatch;
- cancellation before entry into the critical section; or
- a typed owner failure proving no reference movement occurred.

There is no indeterminate in-process branch movement in 9.17.1. If an
implementation cannot determine whether its in-memory reference moved, it
cannot close this milestone. Durable/process uncertainty belongs to later
Store and recovery work.

An owner-performed commit is Relational-current only. It becomes
product-current only through 9.17.2 composite authority.

### 11. Signal cutover law

Signal retains one engine and one branch catalog. Its `SignalBranchHandle`,
head-generation tuple, and `SignalBranchBasisArtifact` are cut over so the
public exact basis contains the shared Foundational reference observation and
the stronger Signal owner axes. No second Signal head registry or compatibility
basis is allowed.

An admitted Signal basis is immutable and cheaply shareable. Sharing it does
not revalidate, clone the graph, capture a new snapshot, evaluate nodes, or
consult latest head state. Signal mutation, restore, or fork uses an owner port
and returns a new basis. A restored descriptor is not admitted merely because
its snapshot id still exists.

### 12. Retention and lifecycle law

Retention is obligation-based. At minimum the owner tracks distinct typed
obligations for:

- current branch head;
- admitted observation/snapshot;
- active transaction;
- prepared candidate;
- external component-basis pin intended for 9.17.2; and
- future correction/recovery holds, which may be declared but are not executed
  here.

Each obligation has one owner-issued lease and one terminal release path.
Dropping an ergonomic guard may call that path, but `Drop` is not the evidence
type returned to governed callers. Double release, foreign release, and release
after owner loss are typed outcomes.

Archive prevents new mutation admission but does not invalidate existing
immutable reads or release their obligations. Branch-reference deletion waits
for active branch operations and removes only the mutable reference. Immutable
commits remain while any branch, snapshot, candidate, external pin, history
policy, or future correction/recovery obligation retains them. Reclamation is
a separate maintenance lane.

### 13. Cancellation law

- Before an obligation is acquired: return cancelled with no owner residue.
- After snapshot/transaction/candidate acquisition but before publication:
  release exactly the acquired obligations and prove no movement.
- Immediately before the critical section: cancellation wins and publication
  does not start.
- Inside the bounded critical section: cancellation is deferred.
- After linearization: return the performed commit and record that caller
  cancellation arrived too late; never report cancellation as if no effect
  occurred.

Timeout follows the same effect boundary. No background task may later publish
after returning cancellation or timeout.

### 14. Outcome and mismatch taxonomy

At minimum, public owner outcomes preserve these non-interchangeable classes:

- malformed or unsupported descriptor;
- foreign runtime/graph instance;
- unknown branch;
- archived or deleting branch;
- empty versus committed target mismatch;
- wrong immutable target;
- stale reference generation;
- wrong branch-local truth version;
- wrong schema or definition basis;
- unavailable retention;
- cross-branch snapshot/read/write/candidate pairing;
- transaction conflict or invariant denial;
- cancelled before effect;
- deferred for declared owner-local backpressure;
- performed success; and
- internal failure with proved no movement.

Generic strings, booleans, `Option`, and a single `Stale` bucket do not satisfy
the contract. Exact sensitive values may be redacted at untrusted boundaries,
but the typed class and safe locator remain.

### 15. Certification-world authority and causality law

The Supply Chain world is a certification subsystem, not a second Relational
runtime.
Its declarations, named semantic handles, scale profiles, and scenario deltas
describe intended meaning but carry no storage, branch, schema, transaction,
retention, or publication authority. Every production-backed baseline is
installed through the same public schema and transaction facades available to
an ordinary caller, and every branch is observed, forked, mutated, published,
retained, and deleted through the real owner ports specified here.

The compiled world program is immutable and reusable. Each test receives a
fresh runtime, installs a fresh baseline through production entry points, and
receives only owner-issued runtime identities, entity/relation identities,
branch bases, and semantic-handle bindings. A test may never turn a declared
semantic key into a guessed raw id or inject a preassembled root, commit,
index, branch head, generation, or retention count.

World compilation must emit a `CertifiedSupplyChainBaseline` only after a public-
surface audit proves that the installed schema, records, relations, declared
invariants, baseline branch observation, and oracle observation agree. A
failed audit is fixture failure, not a product failure. Existing Fintech and
generic Relational worlds remain valuable preservation suites, but they are
not the authority for 9.17.1 closure because they do not provide this causal
construction and independent-oracle boundary.

### 16. Independent semantic oracle law

The Supply Chain oracle is a small pure model over semantic keys, ordered semantic
maps, declared schema contracts, branch ancestry, and the named scenario-
delta language. It must not import or reproduce Relational MVCC roots, indexes,
commit encoding, patch encoding, branch-head lookup, visibility filtering,
digest functions, conflict classifiers, retention tables, or production query
algorithms. Expected branch state is derived from the declarative baseline
plus accepted semantic deltas; it is never copied from a production result.

A separate observation adapter projects public Relational output into a
canonical semantic observation. A separate comparator evaluates that observed
projection against the oracle. Construction failure, owner-operation failure,
observation failure, oracle rejection, and comparison mismatch remain distinct
results so fixture bugs cannot be reported as MVCC defects.

Mutation evidence is mandatory. At least one deliberate mutation per oracle
claim must make the comparator fail: omit a branch-local write, leak a sibling
write, select the latest head instead of the admitted basis, duplicate a
relation, accept an illegal endpoint, or misreport ancestry. Production and
oracle paths may share declarative input values; they may not share the logic
whose correctness the test claims.

### 17. Structural-sharing and physical-isolation law

Forking a Relational branch reuses the exact immutable source commit and
canonical root. Fork creation copies zero entity truths, relation truths,
aspect payloads, index contents, schema contents, or commit envelopes. It may
allocate only the new mutable reference, its owner coordination state, and its
retention obligation. The source commit artifact exists once even when many
branches target it; reference or lease counts are not duplicate truth bytes.

Publishing a branch-local write uses copy-on-write only for immutable storage
regions and index/root paths causally touched by that write. Untouched regions
remain physically shared with ancestors and siblings. “Region” is an
implementation-chosen stable inspection granule; this specification does not
prescribe pages, chunks, nodes, arenas, or a particular persistent structure.
Sharing physical memory never shares mutable fate: branch-local overlays,
coordination, reference generations, lifecycle, and publication outcomes are
independent.

`RelationalBranchSharingObservation` is a public read-only inspection artifact
over safe owner-issued region locators and aggregate byte/copy counters. It
must support assertions about reuse and uniqueness without exposing writable
storage or relying only on process pointers. At minimum it reports:

- fork-materialized entity, relation, and authoritative byte counts;
- copied commit-envelope count and shared-root acquisition count;
- publication touched-region, reused-region, and newly materialized byte
  counts;
- unique canonical commit-artifact count;
- logical branch bytes versus unique physical authoritative bytes; and
- reclaimable unique bytes after named obligations are released.

For this contract, an authoritative byte is a storage-owner-accounted byte in
an immutable truth, schema, correctness-index, root, or canonical-commit
allocation, including the storage structure that makes that payload reachable.
Branch-reference/cell/lease metadata, allocator bookkeeping, optional caches,
and process RSS are reported separately rather than silently included or
excluded. The accounting method and region granule are versioned inspection
semantics, and logical bytes count each branch-visible region while unique
physical bytes count each owner allocation once.

Pointer equality may be an implementation-local diagnostic, never the only
oracle. Deleting a branch reclaims only uniquely owned, unretained storage;
shared ancestors and sibling-visible regions remain available and unchanged.

### 18. World lifecycle and cost-lane law

The immutable Supply Chain declaration and compiled operation program may be
cached
for a test process. Mutable runtimes, branch references, owner-issued handles,
oracle branch state, counters, pause seams, and retention obligations are
fresh per test. Tests cannot depend on execution order or on residue from a
previous case.

The named profiles have distinct purposes:

- **Court** is the deterministic semantic, conflict, cancellation, and
  mutation-sensitivity lane;
- **Standard** exercises representative density, fan-out, and multi-branch
  model sequences in ordinary CI; and
- **Scale** is the instrumented cost and memory-sharing lane, run in its
  declared CI/nightly posture without changing semantic assertions.

Every seeded or generated sequence records the world profile, seed, ordered
delta trace, relevant pause schedule, and canonical failure observation.
Shrinking may reduce the trace but must replay through production facades and
the independent oracle. Fixture installation cost is measured separately and
cannot be charged to branch fork, transaction, publication, or reclamation.

### 19. Memory residency and future physical boundaries

Every live branch root, snapshot, transaction, prepared candidate, Signal basis,
and retention obligation has a bounded owner-managed in-memory lifecycle. This
milestone defines no serialized owner artifact, backend port, checkpoint
contract, or restart guarantee. Worth Store later consumes exact owner bases and
performed publications but cannot choose or infer branch currentness.

## Compiler-Enforced Progression

The legal shapes are fixed even if implementation-local names below a private
module differ:

```text
Foundational reference descriptor
    -> RelationalForkSourceDescriptor                 [Phase 4]
    -> AdmittedRelationalForkSourceBasis              [Phase 4, fork only]
    -> ResolvedRelationalBasisDescriptor              [Phase 6]
    -> AdmittedRelationalBranchBasis                  [Phase 6, reads]
    -> RelationalBranchObservation                    [Phase 6, repeatable read]

RelationalTransactionIntent
    -> BranchBoundRelationalTransaction                [Phase 7]
    -> ValidatedRelationalProposal
    -> PreparedRelationalCommitCandidate
    -> PerformedRelationalCommit
    -> AdmittedRelationalBranchBasis

Foundational reference descriptor
    -> ResolvedSignalBasisDescriptor
    -> AdmittedSignalBranchBasis
```

Each arrow consumes or immutably borrows only the predecessor appropriate to
that phase and requires a concrete
`worth_proof::AuthorityWitness<OwnerSealedMarker>` or a stronger concrete Proof
carrier containing it. The marker is a named, private-constructor type in the
owner's authority module; the governed facade never accepts an `Auth:
AuthorityMarker` parameter. Constructors for admitted, validated, prepared,
and performed artifacts are private to their owners. Public fields that allow
structural forgery are forbidden.

Phase 4's fork-only basis has no read, transaction, publication, or general
retention capability. Compile-fail evidence must prove a caller cannot:

- build either fork-only or general admitted basis from raw Foundational values;
- use a fork-only basis to open a read, begin a transaction, publish, or acquire
  general retention;
- use Relational authority as Signal authority or the reverse;
- substitute equal version/generation values from another branch or runtime;
- begin a governed transaction without an exact admitted basis;
- combine a branch-A transaction, snapshot, candidate, or lease with branch B;
- publish a validated proposal or raw prepared payload;
- clone or reuse a consumed prepared candidate/publication witness;
- treat a descriptor restored from bytes as admitted;
- treat `Performed<T>` with a caller-selected marker as owner authority; or
- reach the retired optional-branch/global transaction entry after cutover.

### Canonical public artifact inventory

These names are the stable facade vocabulary. Internal storage types may be
more specialized, but must not create peer public artifacts with the same
meaning.

| Artifact | Owner | Construction and capability law |
| --- | --- | --- |
| `RelationalBranchBasisDescriptor` | Relational | serializable description; never operational |
| `RelationalForkSourceDescriptor` | Relational | Phase-4 descriptive source/fork request; never a read or transaction authority |
| `AdmittedRelationalForkSourceBasis` | Relational | Phase-4 private Proof-backed source token; consumable only by `fork_branch` |
| `AdmittedRelationalBranchBasis` | Relational | Phase-6 private-minted immutable read authority; not present in Phase 4 |
| `RelationalBranchObservation` | Relational | Phase-6 repeatable read view and exact root; no mutation capability |
| `RelationalTransactionIntent` | caller/Relational facade | unbound mutation meaning; no read or publish capability |
| `BranchBoundRelationalTransaction` | Relational | detached overlay bound to one admitted basis |
| `ValidatedRelationalProposal` | Relational | owner-checked footprint/schema/invariant result; not publishable |
| `PreparedRelationalCommitCandidate` | Relational | opaque single-use pre-effect owner candidate |
| `PerformedRelationalCommit` | Relational | post-linearization canonical commit and next admitted basis |
| `RelationalBranchRetentionLease` | Relational | Phase-10 explicit owner obligation and terminal release |
| `RelationalBranchSharingObservation` | Relational inspection | read-only safe region locators and aggregate physical-sharing/copy/reclamation evidence; no mutation capability |
| `SignalBranchBasisDescriptor` | Signal | serializable snapshot/definition/reference description; never operational |
| `AdmittedSignalBranchBasis` | Signal | private-minted, immutable, shared in-process component authority |
| `SignalBranchRetentionLease` | Signal | one explicit external owner obligation and terminal release |
| `SupplyChainWorldDefinition` | certification | immutable semantic schema, records, relations, profiles, and named handles; no runtime authority |
| `CompiledSupplyChainProgram` | certification | immutable sequence of public-facade installation operations derived from the definition |
| `CertifiedSupplyChainBaseline` | certification | audited pairing of a fresh production world, owner-issued entity/relation/snapshot handles, descriptive Phase-3 baseline branch metadata, and an independent oracle baseline; a fork-only source token is issued only by the Phase-4 branch facade and a general admitted read basis only by Phase 6 |
| `SupplyChainScenarioDelta` | certification | named semantic branch-local action language reused by MVCC and later merge courts |
| `SupplyChainSemanticHandles` | certification | owner-issued runtime identities keyed by declared semantic names; never guessed ids |
| `SupplyChainOracleState` | certification | pure expected semantic state and ancestry over semantic keys |
| `ObservedSupplyChainState` | certification | canonical public-surface projection, kept separate from the oracle and comparator |

The four public authority carrier aliases are concrete, not caller-generic:

- `RelationalBranchObservationAuthority` =
  `worth_proof::AuthorityWitness<RelationalBranchObservationAuthorityMarker>`;
- `RelationalBranchMutationAuthority` =
  `worth_proof::AuthorityWitness<RelationalBranchMutationAuthorityMarker>`;
- `RelationalBranchPublicationAuthority` =
  `worth_proof::AuthorityWitness<RelationalBranchPublicationAuthorityMarker>`;
  and
- `SignalBranchBasisAuthority` =
  `worth_proof::AuthorityWitness<SignalBranchBasisAuthorityMarker>`.

`worth-proof` owns `AuthorityWitness` and all generic progression law. The
Relational or Signal authority module declares the corresponding sealed marker
with the Proof marker-authoring mechanism and keeps its constructor and witness
minting function private. The owner artifact may contain the concrete carrier;
the public governed operation names its exact carrier or stronger owner
artifact and never exposes `Auth: AuthorityMarker`. Possessing observation
authority cannot be upgraded into mutation or publication authority.

This is the required meaning of “concrete platform authority from
`worth-proof`”: a Proof-owned carrier specialized to a named owner-sealed
marker, not a runtime-local witness struct and not an arbitrary generic bound.

The performed commit, stale-reference report, denial, cancellation, deferred,
and failure-before-movement variants form one named
`RelationalPublicationOutcome`. An implementation may use
`worth_proof::TransitionOutcome` as its checked outer topology, but may not
collapse the owner-specific variants or let callers choose the authority type.

## Public Owner Port Contract

### Relational facade

The public `worth_relational::facade` exposes the following stable semantic
operations, not deep storage traits:

Phase 4 exposes only the first two operations below. The remaining operations
are destination ports, not Phase-4 availability:

1. `observe_fork_source` receives a branch identity and returns a
   fork-only source descriptor/token;
2. `fork_branch` consumes target-creation intent and the fork-only source
   token;
3. `observe_branch` receives a branch identity and returns a Phase-6 admitted
   read basis plus descriptor;
4. `readmit_branch_basis` receives only a descriptor and Phase-6 owner
   readmission context;
5. `begin_branch_transaction` consumes transaction intent and borrows a
   Phase-6 admitted basis in Phase 7;
6. `prepare_branch_transaction` consumes the branch-bound transaction and
   returns an opaque candidate;
7. `compare_and_publish` consumes that candidate and returns
   `RelationalPublicationOutcome`;
8. `discard_prepared_candidate` consumes a candidate and returns cleanup
   evidence;
9. `retain_component_basis` and `release_component_basis` govern an external
   owner obligation;
10. `archive_branch` and `delete_branch` preserve distinct lifecycle outcomes;
   and
11. `observe_branch_sharing` and `observe_mvcc_cost` return read-only,
   authority-free inspection artifacts over admitted safe locators and exact
   counter scopes.

Observation and mutation are separate capabilities. A read capability cannot
prepare or publish. A publication capability is owner-bound and cannot mint an
expected basis. Inspection cannot reveal writable storage, acquire operational
authority, or substitute for semantic observation.

### Signal facade

The public Signal owner surface exposes these stable semantic operations:

1. `observe_signal_branch_basis` and `readmit_signal_branch_basis`;
2. `fork_signal_branch` and `restore_signal_branch` through Signal authority;
3. `validate_signal_basis_compatibility` for exact definition/snapshot axes;
4. `retain_signal_component_basis` and `release_signal_component_basis`; and
5. `advance_signal_branch`, which invokes the existing Signal-owned mutation
   engine and returns a new admitted exact basis.

The owner surface must not expose mutable graph internals or require Runtime
Bridge to temporarily become Signal authority.

## Destination Topology And Migration Ledger

The paths below are the architectural destination. A path marked **MOVE**
retains semantic ownership while splitting an overloaded file; **CREATE** adds
a missing responsibility; **REWORK** keeps the path but changes its authority;
**REMOVE** has no successor compatibility lane.

### Proof

| Destination | Action | Responsibility |
| --- | --- | --- |
| `crates/worth-proof/src/proof/witnesses.rs` | REUSE | canonical `AuthorityWitness<Marker>` carrier and non-duplicable witness law |
| `crates/worth-proof/src/proof/marker_authoring.rs` | REUSE | sealed owner-marker declaration contract |
| `crates/worth-proof/src/binding/` | REUSE | complete branch/runtime/basis/attempt binding axes |
| `crates/worth-proof/src/assumption/` | REUSE | freshness downgrade and runtime readmission progression |
| `crates/worth-proof/src/effect/performed.rs` | REUSE | performed-effect carrier after owner linearization |
| `crates/worth-proof/tests/ui/` | REWORK/EXTEND | generic-authority and forged-marker compile denials where Proof owns enforcement |

No Relational- or Signal-specific live state, identity table, currentness check,
counter, lease, or minting policy enters `worth-proof`. No new parallel
authority carrier is created in either owner.

### Foundational

| Destination | Action | Responsibility |
| --- | --- | --- |
| `crates/worth-foundational/src/transitions/branches/identity.rs` | MOVE from `vocabulary.rs` | `FoundationalBranchId` and structural validation |
| `crates/worth-foundational/src/transitions/branches/local_state.rs` | MOVE from `vocabulary.rs` | non-authoritative candidate/staged category definitions |
| `crates/worth-foundational/src/transitions/branches/reference.rs` | CREATE | target basis, reference generation, exact observation, mismatch |
| `crates/worth-foundational/src/transitions/branches/fork.rs` | REWORK/MOVE | exact-reference fork basis |
| `crates/worth-foundational/src/transitions/branches/comparison.rs` | REWORK/MOVE | exact expected-reference comparison and movement descriptions |
| `crates/worth-foundational/src/transitions/branches/artifacts.rs` | REWORK | consume the new grammar without claiming owner currentness |
| `crates/worth-foundational/src/transitions/branches/mod.rs` and `crates/worth-foundational/src/transitions/mod.rs` | REWORK | one canonical export surface |

The oversized `vocabulary.rs` is removed after migration. Epoch-only concepts
that still have independent meaning receive a precise file; they are not kept
as aliases for exact references.

### Relational

| Destination | Action | Responsibility |
| --- | --- | --- |
| `crates/worth-relational/src/branch/identity.rs` | MOVE | owner branch identity and runtime binding |
| `crates/worth-relational/src/branch/reference.rs` | CREATE | mutable reference observation and local version/generation law |
| `crates/worth-relational/src/branch/root.rs` | CREATE | immutable atomic truth/schema/index/visibility root bundle |
| `crates/worth-relational/src/branch/basis.rs` | CREATE | descriptor, admitted basis, readmission |
| `crates/worth-relational/src/branch/authority.rs` | CREATE | sealed observation/mutation/publication markers and private checked Proof-witness issuance |
| `crates/worth-relational/src/branch/fork.rs` | MOVE/REWORK | fork from exact retained observation |
| `crates/worth-relational/src/branch/lifecycle.rs` | MOVE/REWORK | create, archive, delete transitions |
| `crates/worth-relational/src/branch/coordination.rs` | CREATE | independently addressable branch publication cell only |
| `crates/worth-relational/src/history/commit/identity.rs` | MOVE | immutable commit identity |
| `crates/worth-relational/src/history/commit/artifact.rs` | MOVE/REWORK | one canonical immutable commit envelope |
| `crates/worth-relational/src/history/commit/parentage.rs` | MOVE | ordered immutable parentage and fork provenance |
| `crates/worth-relational/src/history/commit/catalog.rs` | MOVE/REWORK | append-only committed artifact lookup |
| `crates/worth-relational/src/history/retention/obligation.rs` | CREATE | typed reasons for retention |
| `crates/worth-relational/src/history/retention/lease.rs` | CREATE | owner-issued acquisition/release |
| `crates/worth-relational/src/history/retention/reclamation.rs` | MOVE/REWORK | cold eligibility and reclaim execution |
| `crates/worth-relational/src/mvcc/observation.rs` | CREATE | repeatable branch observation |
| `crates/worth-relational/src/mvcc/transaction/intent.rs` | MOVE | unbound mutation intent |
| `crates/worth-relational/src/mvcc/transaction/bound.rs` | CREATE | exact branch-bound transaction |
| `crates/worth-relational/src/mvcc/transaction/overlay.rs` | MOVE/REWORK | detached read-your-writes state |
| `crates/worth-relational/src/mvcc/transaction/footprint.rs` | MOVE/REWORK | authoritative read/write footprint |
| `crates/worth-relational/src/mvcc/validation.rs` | MOVE/REWORK | schema, invariant, footprint, exact-head validation |
| `crates/worth-relational/src/mvcc/conflict.rs` | MOVE/REWORK | typed owner conflict classification |
| `crates/worth-relational/src/mvcc/publication/candidate.rs` | CREATE | opaque prepared candidate |
| `crates/worth-relational/src/mvcc/publication/authority.rs` | MOVE/REWORK | branch-local compare-and-publish |
| `crates/worth-relational/src/mvcc/publication/outcome.rs` | CREATE | exact performed/stale/denied/cancelled/failure topology |
| `crates/worth-relational/src/inspection/mvcc/sharing.rs` | CREATE | stable read-only region reuse, copy, unique-byte, and reclamation observations |
| `crates/worth-relational/src/inspection/mvcc/cost.rs` | CREATE | ordinary versus maintenance MVCC counter snapshot |
| `crates/worth-relational/src/facade/branches.rs` | CREATE | observation, fork, lifecycle, retention facade |
| `crates/worth-relational/src/facade/mvcc.rs` | CREATE | transaction preparation/publication facade |
| `crates/worth-relational/src/facade.rs` | REWORK | export only the cohesive new surfaces |

Required removals after cutover:

- `CommitReference` as combined commit/head authority;
- `ExpectedBranchHead::{Empty, Commit(CommitId)}` as an operational
  precondition;
- optional `target_branch` and ambient-main behavior on governed transaction
  entry;
- `AuthorityMode::SerializedCommit` and `CommitAuthority` if they still claim
  one runtime-global writer;
- `RelationalTransaction<'a>` ownership of `&'a mut RelationalRuntime`; and
- the old broad `HistoryAuthority::publish_commit` entry as a parallel path.

Existing history, visibility, snapshots, storage, indexes, durability, merge,
publication, and bridge presentation modules migrate to the new commit and
reference truth source. They are consumers, not alternate owners. A temporary
adapter is allowed only inside one implementation phase, is not publicly
exported, and is removed before that phase closes.

The primary source-to-destination moves are fixed:

- `history/data/mod.rs` splits across `branch/identity.rs`,
  `branch/reference.rs`, and `history/commit/*`;
- `history/authority/branch_management.rs` moves to `branch/fork.rs` and
  `branch/lifecycle.rs`;
- `history/authority/commit_publication.rs` splits between
  `mvcc/publication/authority.rs` and `history/commit/catalog.rs`;
- `runtime/state/subsystems/history.rs` splits branch cells from immutable
  commit lookup and allocation;
- `transactions/transaction.rs`, `transactions/runtime_entry.rs`, and the
  governed pieces of `transactions/data/primitives.rs` move to `mvcc/transaction/*`;
- snapshot and visibility authority consume `branch/root.rs` and
  `mvcc/observation.rs` rather than keeping a second current-version source;
  and
- `presentation/bridge/runtime_source/branch_heads.rs` projects the admitted
  component descriptor returned by the branch facade rather than rebuilding a
  basis from a string branch and commit lookup.

### Signal

| Destination | Action | Responsibility |
| --- | --- | --- |
| `crates/worth-signal/src/branch/identity.rs` | MOVE/ADAPT | public owner identity over Foundational branch identity |
| `crates/worth-signal/src/branch/reference.rs` | CREATE | Signal exact reference observation over existing generation state |
| `crates/worth-signal/src/branch/basis.rs` | MOVE/REWORK | descriptor, admitted basis, definition/snapshot axes |
| `crates/worth-signal/src/branch/authority.rs` | CREATE | sealed basis marker and private checked Proof-witness issuance |
| `crates/worth-signal/src/branch/readmission.rs` | MOVE/REWORK | owner validation after boundary weakening |
| `crates/worth-signal/src/branch/fork.rs` | MOVE/REWORK | exact-reference fork contract |
| `crates/worth-signal/src/branch/lifecycle.rs` | MOVE/REWORK | owner lifecycle contract |
| `crates/worth-signal/src/branch/retention.rs` | CREATE or prove existing owner | external component pin/release |
| `crates/worth-signal/src/logic/transaction/runtime/state/branching/` | REWORK | private mechanics consuming canonical branch types |
| public Signal facade/module exports | REWORK | one exact basis surface |

The old `SignalBranchTransactionHead` tuple and strong-basis identity cannot
remain parallel public authority. Existing merge and targeted-transaction code
must consume the canonical reference/basis types even if their private engine
continues to store numeric keys.

The Signal source move begins at `state/lifecycle.rs`,
`logic/transaction/runtime/state/branching/basis.rs`, `basis_runtime.rs`,
`fork.rs`, `lifecycle.rs`, and `targeted_transaction.rs`. The existing
`branches/catalog.rs` remains the private state owner only if it stores the
canonical types and no public head tuple bypasses their facade.

### Certification

| Destination | Action | Responsibility |
| --- | --- | --- |
| `crates/worth-relational/tests/relational_certification.rs` | CREATE | one intentional public-facade integration target for the Supply Chain and MVCC modules |
| `crates/worth-relational/tests/relational_certification/world/supply_chain/definition.rs` | CREATE | immutable semantic world declaration only |
| `crates/worth-relational/tests/relational_certification/world/supply_chain/schema.rs` | CREATE | Supply Chain entity/relation/invariant contracts |
| `crates/worth-relational/tests/relational_certification/world/supply_chain/scale.rs` | CREATE | Court, Standard, and Scale profiles |
| `crates/worth-relational/tests/relational_certification/world/supply_chain/handles.rs` | CREATE | typed semantic-name to owner-issued-handle bindings |
| `crates/worth-relational/tests/relational_certification/world/supply_chain/program.rs` | CREATE | immutable production installation program |
| `crates/worth-relational/tests/relational_certification/world/supply_chain/compiler.rs` | CREATE | public-facade fresh-runtime installation |
| `crates/worth-relational/tests/relational_certification/world/supply_chain/baseline_audit.rs` | CREATE | causal installation and baseline agreement proof |
| `crates/worth-relational/tests/relational_certification/world/supply_chain/delta.rs` | CREATE | reusable named scenario-delta vocabulary |
| `crates/worth-relational/tests/relational_certification/world/supply_chain/runtime_driver.rs` | CREATE | production facade execution of semantic deltas |
| `crates/worth-relational/tests/relational_certification/world/supply_chain/observation.rs` | CREATE | public-result to semantic observation projection |
| `crates/worth-relational/tests/relational_certification/world/supply_chain/comparison.rs` | CREATE | observation/oracle comparison and typed mismatch report |
| `crates/worth-relational/tests/relational_certification/world/supply_chain/oracle/state.rs` | CREATE | pure ordered semantic entity/relation state |
| `crates/worth-relational/tests/relational_certification/world/supply_chain/oracle/application.rs` | CREATE | independent semantic-delta interpretation and rejection |
| `crates/worth-relational/tests/relational_certification/world/supply_chain/oracle/ancestry.rs` | CREATE | semantic branch parentage and accepted-delta history |
| `crates/worth-relational/tests/relational_certification/world/supply_chain/scenarios/empty.rs` | CREATE | admitted empty Supply Chain installation declaration |
| `crates/worth-relational/tests/relational_certification/world/supply_chain/scenarios/operating.rs` | CREATE | accepted operating topology declaration |
| `crates/worth-relational/tests/relational_certification/world/supply_chain/scenarios/contested.rs` | CREATE | named branch-creation intents over operating state |
| `crates/worth-relational/tests/relational_certification/world/supply_chain/scenarios/retention_pressure.rs` | CREATE | named retained-basis obligations over contested state |
| `crates/worth-relational/tests/relational_certification/world/supply_chain/scenarios/version_boundary.rs` | CREATE | pre-upgrade hazard-schema boundary declaration |
| `crates/worth-relational/tests/relational_certification/mvcc/baseline_validity.rs` | CREATE | causal world compilation, audit, and oracle mutation sensitivity |
| `crates/worth-relational/tests/relational_certification/mvcc/independent_progress.rs` | CREATE | deterministic A-blocked/B-progress court |
| `crates/worth-relational/tests/relational_certification/mvcc/same_reference_race.rs` | CREATE | one winner, exact loser, no residue |
| `crates/worth-relational/tests/relational_certification/mvcc/semantic_isolation.rs` | CREATE | no sibling crossover and exact shared ancestry |
| `crates/worth-relational/tests/relational_certification/mvcc/structural_sharing.rs` | CREATE | zero-copy fork, touched-region COW, and physical-memory evidence |
| `crates/worth-relational/tests/relational_certification/mvcc/retention.rs` | CREATE | obligations, lifecycle, and shared/unique reclamation |
| `crates/worth-relational/tests/relational_certification/mvcc/cancellation.rs` | CREATE | every declared effect boundary |
| `crates/worth-relational/tests/relational_certification/mvcc/model_sequences.rs` | CREATE | seeded branch-local delta sequences against the independent oracle |
| `crates/worth-relational/tests/relational_certification/mvcc/cost_slopes.rs` | CREATE | declared world/branch/history/footprint axes |
| `crates/worth-relational/tests/ui/` additions | CREATE | public compile-pass/fail authority cases |
| `crates/worth-signal/tests/branch_basis_contract.rs` | CREATE | exact reuse, readmission, substitution, retention |
| Foundational transition tests/UI cases | REWORK | grammar parity and non-authority law |

No new certification crate is invented. Tests live at the owning crate's real
public facade boundary and use production constructors. The one Relational
integration target avoids paying for a separate crate compilation per scenario;
files below it retain one named semantic responsibility.

The later Relational merge milestone extends this same integration target with
a sibling `relational_certification/merge/` domain. Its initial cases are
disjoint adoption, same-field conflict, delete-versus-update, relation-endpoint
rewire, and schema-boundary reconciliation over the same Supply Chain baselines
and deltas. Those merge files and merge behavior are not implemented in 9.17.1;
the reusable world, oracle, observation, and delta contracts are.

Every production and test Rust file follows the default 400-line cap. This
specification grants no new exemption; the repository's pre-existing,
explicitly allowlisted facade aggregation remains repository debt and is not
expanded by this milestone.

Existing repository topology may require responsibility-preserving names rather
than literal directories, but the final structure must make branch identity,
history, MVCC, and facade ownership spatially obvious. Forbidden placement
includes `helpers.rs`, a new `branch_manager.rs`, Bridge-owned component
currentness, Query-owned basis minting, SQL inside either component owner,
adapter-selected branch heads, and a test-only coordinator.

## Phase Plan

Each phase closes through production entry points before the next phase relies
on it. The world and oracle precede the MVCC rewrite so later implementation
work cannot weaken its own courtroom to match convenient mechanics.

### Phase 1: Freeze Proof And Foundational Reference Meaning

Add the Foundational exact-reference grammar, migrate the vocabulary where the
new meaning applies, and certify that it remains descriptive. Freeze canonical
encoding, mismatch axes, and the concrete Proof-carrier/owner-sealed-marker
pattern. Pin each owner's structural lowering adapter and concrete admission
door without claiming that a synthetic fixture is a production-issued
current basis.

Exit proof: both owners expose one shared grammar adapter; epoch/equivalence
ids cannot masquerade as exact observations; no runtime state enters
Foundational; no new branch/reference governed facade accepts a generic
`AuthorityMarker`; production-backed owner lowering/currentness remains in
Phase 3 (Relational) and Phase 11 (Signal), with live
admission/readmission proved in the owner phases that implement it. Existing
unrelated Proof migration surfaces remain separately tracked debt and are not
silently reclassified as branch-reference evidence.

### Phase 2: Declare The Supply Chain Semantic World

Implement the immutable Supply Chain definition, semantic keys, schema
contracts, profiles, named baselines, and scenario-delta vocabulary. Implement the pure
semantic oracle and canonical expected observations without importing
disputed Relational behavior.

Exit proof: every named handle and delta has one meaning; illegal definitions
fail before runtime construction; independently predicted observations disagree
with intentionally perturbed production results; and the same vocabulary is
sufficient for branch-local MVCC and the declared later merge cases.

### Phase 3: Compile And Audit A Production-Backed Supply Chain World

Compile the immutable program into a fresh Relational runtime through public
schema and transaction facades. Bind semantic handles only to owner-issued
identities, mint an owner-issued snapshot, observe the installed baseline
through public reads, and issue `CertifiedSupplyChainBaseline` only after
comparison with the independent oracle. The Phase-3 baseline carries a
descriptive branch envelope for traceability; it must not be presented as an
admitted branch basis or as proof of branch currentness.

Exit proof: the empty installation, Court, and Standard baselines install
causally through public schema, bulk-transaction, snapshot, and read-view
facades (the empty lane requires a public no-op commit and its owner-issued
snapshot; no optional/current fallback is admitted); declaration,
installation, budget, transaction, binding, observation, oracle, and
comparison failures are distinct typed outcomes; no private constructor or
raw-id reconstruction is used; relation handles resolve by owner-issued
semantic client-key correspondence across normal, relation-aspect, and bulk
creation paths with endpoint/kind/duplicate/incomplete negative twins;
foreign-runtime snapshot identities are denied; and the
existing Fintech world remains green as a preservation suite. Exact
branch-basis issuance, fork/reference currentness, and branch-local MVCC
remain later-phase exit proofs.

The schema compiler's deterministic kind numbers are declaration-level schema
keys, not runtime identity arithmetic. Entity and relation record identities
must still be obtained only from the owner-sealed commit correspondences.

### Phase 4: Separate Immutable Commits From Mutable References

Phase 4 is the currentness-authority cutover. It separates the immutable
commit fact from the mutable branch-reference fact before the later semantic
transaction rewrite. It introduces the canonical immutable commit artifact,
ordered parentage, branch-local truth version, checked reference generation,
the owner-issued **fork-only** admitted source basis, and exact fork
provenance. The fork-only basis may be consumed by branch creation, but it
cannot open a snapshot, begin a transaction, publish a candidate, or be
reconstructed from a descriptor. General admitted read bases, boundary
readmission, repeatable reads, and retention leases remain Phase 6/10
surfaces.

#### Transitional compatibility inventory (not Phase-4 authority)

The repository already has public historical-read, Bridge, replay, and
application-execution adapters used by existing Query and certification
consumers. They remain callable during this cutover only as compatibility
surfaces, not as Phase-4 branch authority. The following inventory is
explicit and temporary:

- `RelationalApplicationCommitBasisSource::admit_application_commit` accepts
  only an owner-validated `RelationalCommitReceipt` and runtime identity. It
  validates an already-published immutable commit and mints a historical
  execution lease; it does not select a mutable branch head, move a branch
  cell, fork, begin a transaction, or publish a candidate. It is a later
  exact-commit/read-basis adapter and is not used by the Phase-4 Supply Chain
  certification target.
- `RuntimeBridgeRelationalSource::admit_execution_basis_for_identity` and
  `admit_truth_view_execution_basis` are Bridge compatibility joins. They
  require owner/runtime-affine Bridge evidence and mint only the later
  execution lease; they cannot construct a fork basis or enter the Phase-4
  transaction/publication path. Their owner readmission cutover belongs to
  the later Bridge/read-basis phase.
- `VisibilityReadContext::project_version` and the public
  `*_at_version`/bounded historical readers are legacy immutable historical
  projections. A supplied `VersionId` is a read selector only: these methods
  cannot mutate branch references, admit a current transaction, fork, or
  publish. Phase 6 replaces their consumer-facing current-read path with an
  exact owner basis; until then they are compatibility reads, not a second
  Phase-4 currentness authority.
- `HistoryAuthority::retain_version_for_replay`, `RelationalRuntime::replay`,
  and the replay commit/range methods remain cert/maintenance-lane
  reconstruction APIs. They are not imported by ordinary transaction or
  Supply Chain Phase-4 code, cannot mint a fork basis, and cannot write a
  branch cell. Replay and reconstruction remain cert-only; their later
  retention/read-basis ownership is tracked by Phases 6 and 10.
- `HistoryAccess::historical_branch_head`,
  `VisibilityAuthority::historical_snapshot`,
  `historical_snapshot_for_identity`, `historical_snapshot_for_branch`, and
  `HistoryAccess::historical_merge_branch_basis` are pre-existing Query
  consumer adapters renamed during this cutover. `historical_snapshot` opens
  the already-published catalog-latest version as a read-only projection;
  `historical_snapshot_for_*` and `historical_branch_head` resolve through an
  owner/runtime-affine identity or an exact cell-plus-catalog lookup.
  `historical_merge_branch_basis` is a merge-planning projection. None of
  them may move a branch cell, mint a fork basis, or enter transaction or
  publication admission. Phase 6 owns their exact-basis replacement. They
  are not imported by the Phase-4 Supply Chain target.

The compatibility list is a bounded exception, not permission to add another
raw branch selector or lease constructor. Each entry must have an owner/runtime
check and direct negative coverage showing that the call leaves branch-cell
generation and truth unchanged and cannot reach transaction or publication
admission. Its owning later phase removes the entry rather than silently
widening it.

The target descriptor used by this phase is descriptive identity only. It may
carry the immutable commit identity and an owner-produced root descriptor for
comparison and provenance, but it is not the canonical `RelationalBranchRoot`
and cannot select visibility, snapshots, current reads, or publication. Phase
5 owns the visible immutable root and copy-on-write storage. No Phase 4 test
may claim semantic sibling isolation, physical sharing, atomic root
publication, or retention/reclamation from this descriptor.

The existing transaction engine may retain its broad `&mut RelationalRuntime`
borrow and legacy overlay mechanics until Phase 7, but its public branch
authority changes in this phase: `TransactionOptions` has one required,
owner-resolved `RelationalLegacyBranchBinding`; optional `target_branch`,
ambient-main defaults, `ExpectedBranchHead::{Empty, Commit}`, and commit-id-only
currentness checks are removed from the public facade. The binding is
runtime-affine, privately minted by the branch owner, non-serializable,
non-forgeable, and distinct from both the fork-only basis and the later general
admitted read basis. It has no `Default`/`None` construction path and carries
no partial-head comparison. A private, mechanically quarantined adapter may
route the legacy executor through the new branch cell during this phase, but
it accepts only that binding. Detached transactions, exact-basis reads,
footprints, prepared candidates, and compare-and-publish remain later
semantic phases.

Every other branch-bearing transaction input is quarantined by the same rule.
`merge_parent_branches` and merge planning selectors are either private legacy
execution data resolved from owner-issued branch bindings or explicitly
non-operational provenance; public raw `BranchId` values cannot resolve a
current head. `TransactionOptions` loses its `Default`, `Serialize`, and
`Deserialize` construction lanes in this cutover, and every constructor
requires the owner-resolved binding. Bulk/provenance reports may carry a
descriptive branch name for diagnostics only; they cannot select, compare, or
mint branch authority.

The old broad `HistoryAuthority::publish_commit` and
`publish_metadata_only_commit` doors, post-insertion
`append_index_generations` path, public combined `CommitReference`, public
`BranchHead`/`VersionNode` authority, and publication "latest" fallback are
removed. A diagnostic catalog-latest query may remain
only as a commit-identity report and must not feed currentness, visibility,
validation, or basis admission. Canonical artifacts are sealed before catalog
insertion; post-insertion `Arc::make_mut` or derived-index augmentation is not
permitted. Derived index material either completes before insertion or lives in
a separately named diagnostic sidecar.

Dependency-ordered work is fixed:

1. Inspect the public/private boundary and add compiler and behavior checks for
   `CommitReference`, branch-head maps, `ExpectedBranchHead`, optional branch
   routing, ambient-main fallback, `latest_published_commit_ref`, broad
   `publish_metadata_only_commit`/`append_index_generations` paths, generic
   authority bounds, and public raw target constructors.
2. Create the owner-private immutable commit identity, sealed canonical
   artifact, ordered parentage, fork provenance, and append-only catalog.
   Parent order is immutable; authoring branch is provenance and never target
   branch authority. Existing derived-index mutation is moved before catalog
   insertion or into an explicitly non-authoritative sidecar.
3. Create one owner branch-reference cell containing the Foundational exact
   observation (branch identity, explicit `Empty`/`Basis` target, generation)
   plus the owner-local branch truth version and minimal head-retention
   obligation needed to keep a fork source available. The target/root
   descriptor is not duplicated as a second currentness field. Truth movement
   increments local version and generation; metadata movement increments only
   generation; all overflow is typed.
4. Add the private Proof-backed fork-only basis and fork/lifecycle transition.
   Fork consumes an exact live source observation, allocates a fresh
   runtime-affine target identity and generation line, starts target local
   truth version at zero, points at the same immutable catalog artifact, and
   records the exact source observation as provenance. Source movement cannot
   mutate the target reference. Empty state is explicit. Runtime cloning must
   rebind every branch observation to the new runtime identity or return a
   typed non-operational clone; stale foreign observations cannot operate.
5. Migrate commit plumbing and immutable-history readers to the artifact and
   branch cell without moving visibility onto the Phase-5 root. Snapshot,
   visibility, replay, durability, lineage, merge, inspection, and Bridge
   consumers may use a private immutable commit-selection projection during
   this transition, but no such projection is a currentness or read-root
   source and it is removed when their owning later phase cuts over.
6. Replace public transaction branch inputs with the required owner branch
   binding while retaining legacy execution mechanics; migrate all call sites
   and preservation tests so no `None` means main. Remove the old broad
   publication entry and expose separate immutable commit identity and branch
   observation results.
7. Extend the existing Supply Chain behavior suite with fork-only causal
   proofs, exact counter scopes, and documentation. No COW bytes, root visibility, semantic
   sibling reads, readmission, or retention/reclamation claim advances.

Phase 4 proof matrix:

- immutable artifact fields are private and sealed; ordered parentage and
  authoring provenance are stable; catalog lookup returns one canonical
  artifact and no branch-head copy;
- an owner-issued fork basis is concrete Proof authority, cannot be forged or
  deserialized into operation, and cannot be substituted across owner/runtime;
- initial empty/main and forked references have checked generation zero;
  forked local truth version is zero; truth and metadata movement obey their
  separate increment laws; overflow denies before effects;
- a Supply Chain Court/Standard baseline is observed through the real branch
  owner, then forks `storm` and `maintenance`; both target the same immutable
  source commit/artifact, have distinct branch identities and generation
  lines, and produce one catalog artifact with zero envelope cloning;
- stale source generation, foreign-runtime twin, duplicate target, empty
  source, authoring-provenance substitution, raw target construction, and
  descriptor-without-owner-basis cases deny with distinct typed outcomes and
  leave the catalog/reference registry unchanged;
- `RelationalRuntime::fork` is an operational clone with a fresh runtime
  identity and freshly rebound owner branch cells; source-runtime observations
  fail as foreign in the clone even when commit ordinals match; and
- named Phase 4 counters cover branch-cell lookup, catalog lookup/append,
  artifact clone count, fork-reference allocation, and branch-cell contact.
- A mandatory 1/64/512 fan-out probe records setup separately and asserts
  constant per-fork catalog lookup, artifact clone, and branch-cell contact
  counts with no branch-population-dependent scan. Full physical-sharing and
  cost slopes remain Phase 5/12.

Exit proof: a causal Supply Chain fork consumes an owner-issued exact source
basis and targets one shared immutable source artifact with a distinct
runtime-affine reference and local truth version zero; source authoring
provenance cannot operate as target authority; generation/version laws and
runtime affinity are checked; canonical artifact identity is singular; public
combined `CommitReference`, secondary head sources, optional/ambient branch
selection, partial expected-head authority, and every broad publication door
are absent. No Phase 4 result is described as a visible immutable root, a
repeatable-read basis, a retained external lease, or a complete MVCC
publication proof.

### Phase 5: Install Immutable Branch Roots And Sharing Inspection

Create the complete immutable branch root, persistent copy-on-write storage
posture, independently addressable branch coordination cells, and stable
sharing/cost observations. Fork retains a root; it does not clone truth.

Exit proof: 1, 64, and 4,096 forks report zero copied truth and commit
envelopes; logical branch bytes diverge from unique physical bytes; mutable
branch fate remains isolated; and inspection cannot mutate or mint authority.

### Phase 6: Admit Exact Observations And Boundary Readmission

Status: Closed on 2026-08-24. Exact owner observations now bind reads to the
selected reference root, descriptor weakening cannot mint operating authority,
and boundary readmission distinguishes stale, foreign, unavailable, archived,
restored, and mixed-axis substitutions. Independent QA loop, test-quality,
code-quality, and final-gate reviews reported clean.

Implement exact owner basis observation, descriptor weakening, readmission,
repeatable branch observation, and explicit external retention. Cut snapshots,
visibility, presentation, and history lookup to the reference-selected root.

Exit proof: foreign, stale, unavailable, archived, restored, and mixed-axis
substitutions deny distinctly; copied descriptors do not operate; and reads of
admitted Supply Chain bases remain repeatable while references move.

### Phase 7: Detach Transactions And Make Footprints Branch-Qualified

Status: Closed on 2026-08-24. Detached transactions now consume exact admitted
bases, own branch-qualified overlays and footprints, and progress through the
single validated-proposal publication path. Independent QA loop, test-quality,
code-quality, and final-gate reviews reported clean.

Replace optional branch selection and broad runtime borrowing with exact-basis
transaction binding, detached overlays, read-your-writes, and authoritative
read/write footprints. Migrate schema and invariant consumers.

Exit proof: no governed transaction starts without an admitted basis; no live
transaction owns `&mut RelationalRuntime`; cross-branch assembly fails before
effects; and Storm and Maintenance overlays never cross over.

### Phase 8: Prepare Validated Candidates Before Effects

Status: Closed on 2026-08-24. Fallible preparation now produces an opaque,
runtime-affine, branch-bound, single-use candidate without changing public
roots, history, patches, references, diagnostics, or durable state. Explicit
and implicit discard release live obligations, detached-root materialization
matches the declared write footprint, and compiler boundaries deny candidate
self-publication and raw-proposal publication lanes. Independent QA loop,
test-quality, code-quality, and final-gate reviews reported clean.

Move planning, schema checks, invariants, footprint validation, immutable-root
construction, budget checks, and canonical commit assembly into fallible
preparation. Produce only an opaque single-use candidate and complete all
cleanup/discard paths.

Exit proof: preparation changes no public root, history, patch stream, or
reference; candidates cannot publish themselves; losing and discarded work
releases every obligation; and touched-region materialization matches the
declared Supply Chain delta footprint.

### Phase 9: Linearize Branch-Local Publication

Status: Closed on 2026-08-25. Complete-root publication is branch-local,
same-reference races select one winner, readers observe only complete roots,
and unrelated branches remain independently available. Performed-but-unsettled
application, branch-merge, scalar-effect, and batch-effect outcomes now carry
opaque domain-typed recovery through their owning public surfaces; durability,
Query publication, current ancestry, and index refresh are repaired under the
required serialization boundary. Independent QA loop, test-quality, and
code-quality reviews reported clean.

Implement complete-reference compare-and-publish over one bounded branch-local
critical section. Atomically select the next complete root and make the one
canonical performed commit available to history, patch, receipt, visibility,
and correctness-index consumers.

Exit proof: a paused Storm publication does not stop Maintenance; same-
reference races have one winner; concurrent readers see only complete old or
new roots; unrelated wait/contact counters are zero; and no old global/broad
publication door remains.

### Phase 10: Close Retention, Lifecycle, Cancellation, And Reclamation

Status: Closed on 2026-08-27. Snapshot, observation, transaction, candidate,
performed-settlement, and external component obligations are bounded and
single-terminal; archive/delete and branch-name retirement preserve retained
ancestors, post-linearization cancellation remains performed, and ordinary
publication performs no reclamation scan. Independent QA loop, test-quality,
code-quality, and final-gate reviews reported clean.

Install typed head, observation, transaction, candidate, and external-pin
obligations. Implement archive, delete, cancellation/timeout, and maintenance-
lane reclamation at the exact linearization boundary. Keep every retained root,
candidate, and obligation explicitly bounded in memory; introduce no backend or
serialized owner artifact.

Exit proof: every terminal path restores or transfers obligations once;
retained shared ancestors survive branch deletion; only unique unretained
regions become reclaimable; post-linearization cancellation returns performed;
no ordinary path scans history to reclaim; and acknowledged Relational
publication remains available through its live owner basis without allowing a
foreign branch descriptor to substitute.

### Phase 11: Cut Signal Over To The Shared Reference Vocabulary

Status: Closed on 2026-08-27. Signal now exposes one owner-admitted exact basis
grammar for observation, readmission, fork, capture, restore, advance, merge,
retention, and retirement; legacy head tuples remain private engine mechanics,
and transported descriptors open no authority until owner readmission.
Independent QA loop, test-quality, code-quality, and final-gate reviews
reported clean.

Adapt Signal's branch catalog, basis artifact, fork/restore, targeted
transactions, retention, and public facade to the same Foundational reference
grammar while retaining Signal-owned live authority and bounded in-memory
residency.

Exit proof: Signal has one engine and one head truth source; admitted basis
sharing performs no graph work; owner and definition substitutions remain
distinct; the parallel public tuple/basis dialect is absent; and a weakened
descriptor remains non-authoritative until Signal owner readmission.

### Phase 12: Verify Supply Chain MVCC Semantics And Cost

Status: Closed on 2026-08-27. All named Supply Chain deltas execute through
the public branch-local MVCC path against the independent oracle; seeded model,
schema-transition replay/recovery, cancellation, retention, structural-sharing,
and cost courts prove the ordinary and scheduled contracts. Independent QA
loop, test-quality, code-quality, and final-gate reviews reported clean.

Run causal-baseline, semantic-isolation, ancestry, independent-progress,
same-reference, atomicity, retention, cancellation, seeded-model, structural-
sharing, and cost-slope cases at their declared profiles. Run required
compiler-boundary checks. Cross-branch Relational descriptors and cross-
definition/runtime Signal basis substitutions are required red controls.

Exit proof: production observations match the independent oracle for every
accepted delta trace; branch-local differences and shared history are both
exact; and fork and write amplification meet their counters.

### Phase 13: Cutover, Documentation, And Handoff Freeze

Status: Closed on 2026-08-28. Predecessor and ambient-main authority paths are
absent from the public boundary; executable Relational owner-flow documentation
and both component-owner ports now define the 9.17.2 handoff. The full
Relational library, Supply Chain certification, existing Query preservation
slices, compiler courts, formatting, line-cap, boundary, and generated-context
lanes pass. Independent QA loop, test-quality, code-quality, and final-gate
reviews reported clean.

Delete predecessor authority paths, finish executable owner/world docs, keep
Fintech and generic preservation suites green, run boundary/generated-context/
line-cap/format/lint/focused/broader checks, and freeze the 9.17.2 owner port.

Exit proof: retired paths and ambient governed `"main"` are absent from the
public boundary; documentation teaches the same public flows the tests execute; all scoped
constitutional checks pass; and later merge certification can add cases to
the Supply Chain world without changing its baseline, delta, observation, or
oracle authority rules.

## Performance And Resource Contract

### Ordinary complexity

- Exact basis observation/readmission: O(1) fixed-axis lookups and comparison.
- Transaction open: O(1) plus one branch-local retention acquisition.
- Snapshot read: O(1) root selection plus requested result work.
- Branch fork: O(1) reference/cell/retention work, zero copied authoritative
  truth bytes, and zero copied commit envelopes, independent of world size.
- Preparation: O(read footprint + write footprint + touched invariant/schema
  work), independent of unrelated branches and total history.
- Publication: O(touched immutable-region/root paths + emitted canonical
  patch) plus O(1) branch comparison and bounded global id/stream reservations;
  it does not copy the complete Supply Chain world.
- Signal admitted-basis share/reuse: O(1), zero graph/snapshot copy and zero
  evaluation.
- Retention acquire/release: O(1); reclamation scans are maintenance work.
- Owner publication touches only the selected branch's roots, history, and
  declared indexes; unrelated branches contribute zero synchronous writes.
- Historical reconstruction is an explicit maintenance lane, not cost hidden
  in ordinary observation or commit.

### Required scale axes

Measure the Supply Chain Court, Standard, and Scale profiles separately from
fixture installation. Measure at least branch populations 1, 64, and 4,096;
retained histories 1,
1,024, and 65,536 commits where practical in the cost harness; transaction
footprints 1, 64, and 4,096 records; and future-holder fan-out 1, 64, and 1,024
for immutable component bases. If CI cost requires smaller absolute fixtures,
the same logarithmic scale relationship and fitted slope must be retained and
the reduction documented.

Ordinary transaction/open/publication work must remain flat as unrelated
branch population and unrelated history grow. Footprint growth may be linear
only in the declared footprint. No ordinary path performs SHA/canonical
re-encoding if the admitted basis already carries the required digest.
For 4,096 unchanged forks, unique physical authoritative bytes must scale with
reference/cell/retention metadata rather than 4,096 copies of baseline truth.
A single-record or single-relation delta may materialize only its touched
immutable regions and root paths; it cannot clone the complete world on first
write. Logical per-branch byte totals are reported separately from unique
physical authoritative bytes so sharing cannot be hidden in accounting.

### Required counters

Counters distinguish:

- basis resolution, readmission, denial, and stale outcomes;
- branch-cell acquisition, branch-local wait, and unrelated-branch wait;
- snapshot/transaction/candidate/external retention acquire and release;
- read/write validation and conflict classes;
- global id and patch-position atomic reservations;
- candidate preparation and discard;
- publication attempts, performed movements, stale losses, and
  failure-before-movement;
- canonical history/patch append;
- correctness-index root installation and fallback use;
- fork-materialized entities, relations, authoritative bytes, and commit
  envelopes;
- shared-root acquisitions and unique canonical commit artifacts;
- publication touched regions, reused regions, and newly materialized
  authoritative bytes;
- logical branch bytes, unique physical authoritative bytes, and reclaimable
  unique bytes;
- cancellation at each named phase;
- reclamation work; and
- explicit history/maintenance reconstruction work.

In the controlled A/B court, branch B's unrelated-branch wait and branch-A
coordination-contact counters are exactly zero. Atomic allocator contention is
reported separately and must be bounded; it cannot be relabeled branch wait.

Resource budgets bound transaction overlay bytes, read/write footprint size,
prepared-root bytes, candidate lifetime, snapshot/pin count, and owner-local
backpressure queueing. Exhaustion is typed before publication and cannot
silently widen into history scans or global eviction.

## QA Considerations

Review the implementation and its direct evidence, not a second proof system
describing the evidence. The material risks are authority substitution,
stale or mixed-axis selection, sibling leakage, partial publication,
retention or cleanup escape, false owner readmission, and ordinary work that grows
with unrelated branch population.

Use focused owner and integration tests for these risks. Keep expensive scale,
fuzz, and environment-backed cases in scheduled lanes
unless they are needed to reproduce an active defect. Important authority
seams receive one economical compiler-boundary check plus runtime denial where
that adds independent evidence. Fixtures use production runtime builders and
public owner facades.

An independent reviewer examines the final diff and executed results for
material defects. No closure ledger, source fingerprint, evidence registry,
test inventory, or test-of-tests is required.

## Documentation Deliverables

The implementation closes only with these continuing documents:

- `crates/worth-foundational/docs/branching-merging-and-commit-vocabulary/branch-references.md`
  — shared descriptive grammar, immutable target versus mutable reference,
  encoding, locators, and non-authority examples;
- revise the Foundational vocabulary README and relevant Milestone 5 docs to
  point to that canonical reference;
- revise `crates/worth-proof/docs/features/authority-and-workflow-contracts.md`
  and `crates/worth-proof/docs/features/witnesses.md` with the concrete-carrier/
  owner-sealed-marker placement rule, including why defining the seal in Proof
  or accepting a generic marker at a governed facade is incorrect;
- `crates/worth-relational/BRANCH_LOCAL_MVCC.md` — isolation, branch root,
  linearization, conflict, cancellation, retention, structural sharing,
  inspection, and cost contract;
- `crates/worth-relational/TESTING_WORLDS.md` — Supply Chain semantic model,
  profiles, named handles, causal compiler, independent oracle, scenario
  deltas, failure reproduction, cost lanes, and the contract retained for
  later merge certification;
- revise `crates/worth-relational/API_OVERVIEW.md`, `DAILY_WORKFLOWS.md`,
  `QUICKSTART.md`, and `README.md` so examples use exact admitted bases and the
  public branch/MVCC facades;
- `crates/worth-signal/BRANCH_BASES.md` — exact basis, shared reference grammar,
  fork/restore/readmission, reuse, and retention;
- revise `crates/worth-signal/DOCS.md` and `README.md` to remove the parallel
  public head/basis dialect; and
- `crates/worth-relational/OWNER_COMPONENT_PORT.md` and the corresponding
  Signal section in `BRANCH_BASES.md` — the exact artifacts, outcomes,
  cancellation, and retention ports 9.17.2 may consume; and
- an explicit note in both owner guides that state is memory-resident and
  restart durability is deferred to Worth Store integration.

Examples must compile through public facades. The AI entry README and generated
crate context must name the same authority and topology as the code.

## Must Preserve

- all 9.16.1 provider-session branch-affinity and no-ambient-main guarantees;
- all 9.16.2 package identity, fresh-validation, archive, and release-boundary
  guarantees, with no branch/component authority added to package records or
  archive bytes;
- Relational authoritative truth, schema, identity, lineage, history, merge,
  patch, replay, inspection, and durability semantics;
- Signal definition-bound branch, snapshot, restore, merge, targeted
  transaction, and derived-state meaning;
- one canonical artifact for every performed owner commit;
- Foundational as runtime-neutral meaning rather than authority;
- Proof as progression law beneath concrete owner types;
- ordinary versus reconstructive cost-lane separation; and
- existing Fintech and generic certification behavior as preservation evidence,
  while Supply Chain becomes the 9.17.1 closure world; and
- Query and Bridge inability to mutate component internals directly.

Preservation does not require source compatibility for the retired authority
paths. Supported behavior migrates to the new facade; wrappers or aliases that
keep old authority alive are forbidden.

## Explicit Non-Goals

- Relational conflict-aware disjoint-head rebasing or automatic merge;
- cross-branch serializability;
- composite Relational-plus-Signal correspondence or basis authority;
- composite commits, product branch references, or product currentness;
- coordinated cross-owner publication or rollback;
- Query public branch/history workflow;
- semantic merge, rebase, undo/redo, persistence, distributed recovery, or
  offline sync; and
- a new Signal concurrency engine beyond the basis/reference cutover required
  here.

## Allowed Debt

Store-native graph persistence, restart recovery, replication, distributed
reference movement, cross-owner atomicity, and automatic rebase remain later
work.

The following may not remain as debt:

- global ordinary Relational commit coordination;
- broad mutable-runtime transaction ownership;
- combined immutable-commit/mutable-reference authority;
- optional or ambient governed branch selection;
- raw owner-basis construction or generic marker authority;
- equal-ordinal/digest currentness;
- missing trust-boundary readmission;
- early retention release;
- split visible branch roots;
- a fixture that bypasses production construction or uses production-derived
  expectations as its semantic oracle;
- eager fork cloning, whole-world first-write cloning, or pointer-only sharing
  claims;
- parallel Signal branch-reference dialects; or
- test-only MVCC/concurrency paths.

## Parallelization And Store Dependency

After Phase 1 freezes the grammar, Signal basis cutover may proceed alongside
Phases 2-3 world construction. Relational commit/reference decomposition cannot
begin until the causal Supply Chain baseline and independent oracle close,
because those proofs constrain the implementation rather than adapting to it.
After Phase 4, observation/detached-overlay work may proceed beside retention
primitives, but atomic publication cannot close until branch root, candidate,
history, sharing inspection, and retention contracts agree. Certification
follows the real public cutover, not a shadow implementation.

This milestone is not blocked on `worth-store` because it claims only live
in-memory authority. Discovery that a requirement needs restart durability or
cross-process atomicity moves that requirement to Store integration; it is not
permission to simulate Store behavior here.

## Acceptance

Milestone 9.17.1 closes when the phase behavior and owner boundaries exist in
the final source, focused and affected integration tests pass, repository
architecture and quality gates pass, and independent review finds no material
defect. Expensive scale cases run in their scheduled lanes.

## Retained Handoff To Relational Merge Certification

Supply Chain remains the canonical Relational merge-certification world after
9.17.1. A later merge milestone receives the immutable world definition,
production compiler, certified baselines, semantic handles, named deltas, pure
oracle, public observation adapter, comparator, scale profiles, and failure-
reproduction format. It does not receive permission to weaken or replace them
with a merge-specific fixture.

The minimum later merge matrix combines:

- disjoint Storm Reroute and Atlas Maintenance adoption;
- Competing Aurora Arrival on the same voyage/call fields;
- Atlas deletion versus Aurora inspection/reference retention;
- Aurora port-call endpoint rewiring versus schedule change;
- Hazard Classification V2 across compatible and incompatible schema bases;
- unchanged cargo/infrastructure sharing across merge preparation, success,
  conflict, and discard; and
- exact common-ancestor selection after fan-out and retained-history pressure.

9.17.1 proves that those inputs are causally installed, branch-isolated,
ancestry-exact, and structurally shared. It does not implement merge selection,
conflict policy, multi-parent commits, or merge publication.

## Exact Owner-Basis Handoff Toward Milestone 9.17.2

Milestone 9.17.1.1 corrects Relational concurrency/settlement and retention;
Milestone 9.17.1.2 completes Relational lifecycle/basis ports and Signal
independent services. Neither changes owner meaning. After
both corrections close, 9.17.2 receives only these public owner capabilities:

- obtain or readmit an owner-issued exact component-basis descriptor;
- retain and release that exact basis for a named external composition
  obligation;
- ask Relational to prepare an opaque branch-bound candidate from already
  admitted owner-local work;
- consume that candidate through Relational compare-and-publish and receive a
  typed performed/stale/denied/cancelled/failure outcome;
- ask Signal to perform its existing owner-local advancement/fork/restore and
  receive a new exact Signal basis where a product operation requires it; and
- observe the exact committed component bases and safe canonical descriptors
  returned by performed owner operations.

A prepared candidate is pre-effect. `PerformedRelationalCommit` is post-effect.
An admitted component basis says only that its component owner recognizes the
exact state. None of them says that a product branch moved.

9.17.2 may order and correlate these owner calls through `worth-runtime-world`,
retain successful owner results, and mint composite authority after its own
rules succeed. It may not:

- inspect or mutate candidate internals;
- reconstruct currentness from branch id, commit id, version, generation, or
  digest;
- fabricate owner admission or retention;
- invoke private storage, graph, head-map, or transaction mechanics;
- claim rollback by deleting an already performed owner commit;
- reinterpret a Relational-current or Signal-current result as product-current;
  or
- reintroduce a global cross-owner lock.

If 9.17.2 needs an additional owner operation, the applicable owner contract
must be corrected and proved at that owner boundary. Runtime World- or
Bridge-local imitation is not an allowed extension mechanism.
