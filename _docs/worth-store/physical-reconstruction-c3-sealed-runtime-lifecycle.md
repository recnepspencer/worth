# C.3 Sealed Physical Runtime Authority And Lifecycle

## Status And Roadmap Position

**Status:** implemented and acceptance-verified engineering specification.

**Roadmap position:** C.3 establishes the only runtime authority into which
C.4 through C.13 may bind real
media, page, residency, durability, recovery, integrity, isolation, layout,
blob, and certification behavior. C.3 must close before C.4 implementation
begins.

## Goal

Establish one Store-owned, non-forgeable, non-duplicable physical runtime
composition authority whose construction, installed lifecycle ownership,
observation rights, and transitions are enforced by Rust ownership, visibility,
and phase-typed APIs.

After C.3, there is one obvious place to attach every later physical
mechanism. There is still no claim that bytes survive process death.

## Boundary

C.3 owns runtime admission, identity, composition, authority partitioning,
phase-scoped handles, lifecycle progression, shutdown posture, public facade,
and structural enforcement.

C.3 does **not** implement filesystem effects, store locking, durable format,
buffered page access, WAL publication, recovery, semantic MVCC, Query
integration, or production readiness. A declared store root is identity input,
not evidence that the directory exists or is a valid database.

`AdmittedPhysicalRuntime` is a lifecycle composition authority, not an opened
database. Its public surface contains no record, page, byte, WAL, checkpoint,
publication, recovery, maintenance, or generic mutation method. C.3 exposes
typed capability availability as observation; it does not expose future
operations merely so they can return `Unavailable`.

## Governing Decisions

1. `worth-store` owns the sole public physical runtime facade and the sole
   admission function. Internal physical crates remain autonomous owners; none
   may independently mint an equivalent composition authority.
2. The runtime authority is move-only. It is never `Clone`, `Copy`,
   serializable, reconstructible from ids, or recoverable from a supplied
   layout/replay representation.
3. A runtime id identifies one admitted in-process incarnation. A store-root
   identity identifies the intended physical namespace. Neither is authority,
   and neither may reconstruct the other.
4. Store's sealed admission transition is the C.3 construction authority. C.3
   does not accept a caller-supplied platform witness because no real external
   platform owner exists yet. A later platform boundary may add authority only
   by carrying ownership from that real predecessor; a public zero-input
   witness mint, generic marker trait, boolean flag, or test-only promotion path
   is forbidden.
5. Typestate carries lifecycle truth. Runtime checks may report environmental
   failure, but they do not compensate for APIs that allow an invalid
   transition to compile.
6. Failed consuming transitions return the still-owned prior phase where safe;
   callers are not forced to leak or recreate authority after an ordinary
   failure.
7. Observation is cloneable only where it carries immutable, phase-scoped
   visibility and explicit revocation/staleness behavior. It cannot mutate,
   publish, recover, allocate physical identity, or mint stronger handles.
8. C.3 creates no public mutation handle. Lifecycle changes consume or mutably
   borrow the admitted runtime itself. Physical mutation authority first exists
   only when C.4/C.5 install concrete media and record-owning transitions.
9. `close` is an explicit successful lifecycle transition. `abort` is an
   explicit in-process abandonment transition. Panic, process death, and
   unexpected drop are not relabeled as successful close or durable crash
   recovery.
10. C.3 uses ownership and visibility directly. It does not introduce a
    lifecycle ledger, proof registry, receipt graph, or recursively attested
    evidence hierarchy.
11. Later milestones consume `AdmittedPhysicalRuntime` into a different concrete
    phase type that owns the newly real authority. They never unlock physical
    methods on the admitted type through a flag, optional field, trait object,
    feature, or runtime capability check.

## Adversarial Constraint

Assume hostile callers can run concurrently, retain stale handles, catch
panics, call every public constructor, import every public module, enable test
features, copy all public ids, and supply plausible replay
or persisted-layout values. They must still be unable to create a second
composition authority, operate in the wrong lifecycle phase, promote
observation into authority, preserve authority after close, obtain any physical
mutation surface, or make a media-dependent operation callable before C.4.

## Authority And Lifecycle Model

The following distinctions are mandatory:

```text
PhysicalRuntimeAdmission        untrusted declaration; not runtime authority
        |
        | PhysicalStore::admit (sealed Store construction boundary)
        v
AdmittedPhysicalRuntime         sole in-process composition authority
        |  \
        |   \ close/abort
        |    v
        |   ClosedRuntime / AbortedRuntime
        |
        | C.4 real namespace/media ownership
        v
MediaOwnedPhysicalRuntime
        |
        | C.5 real page/record open
        v
ServingPhysicalRuntime -> DrainingRuntime -> ClosedRuntime
        |
        | C.8 recovery admission
        v
RecoveringRuntime -> ServingPhysicalRuntime
```

C.3 makes `AdmittedPhysicalRuntime`, explicit close/abort outcomes, and the
sealed transition boundary real. The future physical phase types are specified
here but are not defined or constructible in production code until their owning
milestones install real transitions. There must be no placeholder
`MediaOwnedPhysicalRuntime` or `ServingPhysicalRuntime` backed by heap state.

The completed C.3 authority hierarchy is ownership, not a token collection:

```text
AdmittedPhysicalRuntime
  owns RuntimeIdentity
  owns process-local root admission
  owns LifecycleCoordinator once Phase 4 installs real transitions
  owns observation/resource lifecycle
  issues scoped ObservationHandle values
  exposes immutable installed-capability status
```

Phases 1 through 3 install only `RuntimeIdentity` and process-local root
admission. Later C.3 phases add the remaining fields only in the same change
that makes their named behavior real. A config holder, constant counter, or
status flag is not an installed owner.

It owns no page collection, record collection, byte image, persisted layout,
WAL image, manifest model, buffer frame table, or generic physical subsystem
registry. Absence of a physical owner is represented by the absence of that
owner and its methods—not by an in-memory implementation behind an
`Unavailable` label.

Copied identities, diagnostics, counters, store-root declarations, and
availability reports remain observations. They never satisfy an authority
parameter.

## Opinionated Target Topology

The implementation boundary review may refine names, but any deviation must
preserve these responsibilities and explain why the replacement is clearer:

```text
worth-store/src/physical_runtime/
  mod.rs                    public aggregation only
  admission.rs              admission request and sole admit transition
  identity.rs               store-root and runtime-incarnation identities
  root_admission.rs         process-local declaration ownership
  runtime.rs                move-only composition root
  lifecycle.rs              phase types and legal transitions
  observation.rs            immutable scoped observation handles
  resource_lifecycle.rs     registered handle/resource ownership
  availability.rs           immutable installed-capability status
  shutdown.rs               drain, close, abort, and unexpected-drop posture
  diagnostics.rs            non-authoritative lifecycle observations/counters

worth-store/tests/
  sealed_runtime_lifecycle_journey.rs
  runtime_authority_pressure_journey.rs
  physical_runtime_authority_ui.rs
  physical_runtime_authority/     small fixed external-consumer specimens
```

Do not create `manager`, `helpers`, `common`, `shared`, `proofs`, `receipts`,
or milestone-named production modules. A file may be split when one of the
listed responsibilities develops an independently testable lifecycle or
failure surface; line count alone is not the design axis.

## Critical DX Target

The normal path must read as the lifecycle it performs:

```rust,ignore
let admission = PhysicalRuntimeAdmission::new(store_root)?;

let admitted = PhysicalStore::admit(admission)?;

assert_eq!(
    admitted.installed_capabilities().physical_media(),
    CapabilityAvailability::Absent,
);

let observation = admitted.observe();
assert_eq!(observation.runtime_identity(), admitted.runtime_identity());

let closed = admitted.close()?;
```

Critical DX requirements:

- one admission request, one public facade, and one obvious transition path
- configuration appears only beside the real subsystem that consumes it;
  unimplemented limits and future-owner tuning are absent rather than accepted
  and ignored
- typed errors for transitions that actually exist in C.3
- compile-time denial for wrong-phase methods rather than runtime `InvalidState`
  for calls the type system can exclude
- no consumer-visible ownership gymnastics for ordinary observation
- `open_existing`, `recover`, record/page mutation, publication, and maintenance
  methods do not exist on `AdmittedPhysicalRuntime`
- C.4 and C.5 add expensive operations only on the new phase types produced by
  their concrete media/page transitions

## Test Architecture: Two Journeys And One Compiler Boundary

C.3 closure is carried by **two production-facade journeys and one consolidated
external-consumer UI suite**. Phases do not create independent scenario
matrices. Their test requirements name assertions that must be added to one of
these three products.

For C.3, honest end to end means public admission through terminal lifecycle
and independent proof of zero physical effect. Real write, kill, and reopen
proof begins only after C.4 and C.5; inventing that scenario here would force
the heap-backed fiction this milestone is removing.

### Journey A: Sealed Runtime Lifecycle

Cargo target: `sealed_runtime_lifecycle_journey`.

One ordinary integration-test executable drives the public production facade
from declaration through admission, scoped observation, immutable capability
status, explicit close, and clean re-admission.
It runs once with an absent root and once with an existing empty directory.

Its fixed action script is:

1. snapshot both roots with ordinary OS APIs
2. submit one representative invalid admission and prove that no runtime
   identity, registration, allocation, or directory effect was created
3. admit the absent-root declaration as `R1`, acquire exactly two observation
   handles, and inspect the immutable installed-capability status
4. require media, page/record, WAL/checkpoint, recovery, maintenance, layout,
   and blob capability families to be absent; prove the observation performs no
   physical-subsystem initialization, broad allocation, or state mutation
5. close `R1`, prove both observations obey the terminal staleness law, then
   re-admit the same declaration as `R2` and require `R2 != R1`
6. close `R2`; admit and close the existing-empty-directory declaration as
   `R3`
7. compare final OS snapshots byte-for-byte with the initial snapshots

The aggregate causal counts are four admission attempts, three admitted
incarnations, one pre-construction denial, three explicit closes, two
observation acquisitions, four lifecycle-observation attempts, and fourteen
capability-family observations (the seven families reconciled once through the
facade and once through a retained observer). Physical-owner, physical-
operation, publication, media-operation, abort, panic, and unexpected-drop
counts remain zero. Per-incarnation snapshots must reconcile to the same
aggregate.

The journey compares the runtime with an independently declared lifecycle
table, reconciles exact causal counters, and uses ordinary OS inspection to
prove the directories remain unchanged. It must detect a controlled defect
that initializes a heap-backed physical subsystem while still reporting that
the capability is absent.

### Journey B: Authority Under Hostile Lifecycle Pressure

Cargo target: `runtime_authority_pressure_journey`.

One deterministic hostile executable, plus a child process only where process
identity is part of the proof, races same-root and different-root admission,
retains permitted observation handles, injects panic and cancellation at named
lifecycle boundaries, exercises abort and unexpected drop, and attempts close
while pressure is active.

Its fixed pressure script uses primary, different-root, abort, drop, panic, and
cancel declarations; eight observations retained across the primary close; two
same-root contenders; one different-root contender; and one child process. A
deterministic barrier
script—not sleeps—orders this single schedule:

1. admit the primary declaration, retain eight observers, and hold them at
   admitted and terminal barriers while closing the runtime
2. require all eight observers to agree on identity/lifecycle/capability truth,
   become closed together, and expose exactly zero physical counters
3. exercise explicit abort and unexpected drop on dedicated declarations,
   retain their observers, and prove both become stale without being reported
   as successful close
4. hold one primary root while two same-root admissions and one different-root
   admission race; require two typed denials and one independent success, then
   terminate every admitted incarnation explicitly
5. cancel one validated request before registration and immediately admit its
   root; catch one live-runtime panic, prove its observer becomes stale, and
   immediately re-admit its root
6. launch the child with another root declaration, wait until admission is
   externally observed, terminate it without cleanup, and prove no close event
   or physical residue exists
7. execute the entire parent/child script again and require an identical
   normalized counter/outcome record, excluding intentionally fresh runtime and
   process identities

Each parent execution has exactly sixteen admission attempts, thirteen admitted
incarnations, two typed admission denials, one explicit cancellation, eight
explicit closes, three explicit aborts, one panic termination, one unexpected
drop, eleven observation acquisitions, twenty-three lifecycle-observation
attempts, and fifty-six capability-family observations. Physical-owner,
physical-operation, publication, and media-operation counts remain zero. The
child independently admits one declaration and is killed before any terminal
claim. The replay must reproduce the parent's complete normalized record.

The public journey catches a panic while real admitted authority is live. A
narrow owner-unit unwind test separately injects panic after root reservation
but before authority return; this boundary cannot be selected through the
production API without adding the caller-controlled/test-only admission hook
that C.3 explicitly forbids. That unit proves reserved-root RAII and the
`admission_panics_before_return` counter, while the journey remains entirely on
the production facade.

The parent owns the schedule and independent transition oracle. It receives
only structured observations from the child, never runtime authority or
subsystem state. The journey proves there is never more than one composition
root per admitted incarnation, no stale handle promotes itself, no physical
mutation surface appears, false close is never reported, no managed resource is
orphaned, and C.3 creates no physical residue. It must detect controlled
duplicate-authority and false-close defects.

### Compiler Boundary C: External Authority Denial

Cargo target: `physical_runtime_authority_ui`.

One cache-sharing UI suite compiles a legitimate external-consumer journey and
a small fixed set of forbidden capability specimens. It exists only for
properties runtime execution cannot honestly prove:

1. one compile-pass specimen performs the supported admission/observation/close
   journey
2. one compile-fail specimen attempts runtime cloning/reconstruction
3. one compile-fail specimen attempts internal composition/phase construction
4. one compile-fail specimen attempts promotion from identity, observation,
   diagnostics, replay/layout, and test/certification values
5. one compile-fail specimen attempts physical operations on the admitted phase,
   wrong-phase transitions, and after-move operations
6. one compile-fail specimen attempts alternate admission through the maximal
   admitted ordinary feature profile

This is one responsibility-owned compiler boundary, not a fixture per phase or
per API. New fixtures are added only when a genuinely new authority class
appears that the existing specimens cannot express.

### Test Budget And Ownership Rules

- exactly two behavioral scenario families and one consolidated UI family own
  milestone closure
- phase-local unit tests may clarify a pure local algorithm, but do not become
  closeout evidence and do not duplicate journey assertions
- no phase-named tests, generic scenario matrix, checked-in case ledger, proof
  receipt graph, or per-feature Cartesian product
- default-profile owner compilation plus one maximal admitted authority UI
  profile is sufficient unless a specific feature changes authority topology
- assertions are numerous where necessary; executable worlds stay few
- a failure must identify the violated invariant and causal boundary inside the
  journey rather than require one test binary per failure

The direct rerun commands are:

```text
cargo test -p worth-store --test sealed_runtime_lifecycle_journey
cargo test -p worth-store --test runtime_authority_pressure_journey
cargo test -p worth-store --features certification-test-authority --test physical_runtime_authority_ui
```

## Non-Fake Acceptance Setup

### Production subject

- the public `worth-store` physical runtime facade
- the sole admission transition
- the move-only composition root and its lifecycle-only access surfaces
- lifecycle, observation, installed-capability status, and shutdown APIs
- the production manifest/visibility topology that prevents lower crates from
  exporting parallel construction authority

The behavioral subjects are Journey A and Journey B above. Compile-time
rejection is owned by Compiler Boundary C. No certification crate may
construct the runtime on the tests' behalf.

### Initial world

- one unique path whose store root is absent
- one unique path whose directory exists but contains no admitted store
- one representative invalid root declaration proving rejection precedes
  runtime construction; pure validation edges remain owner-local unit-test
  concerns
- no pages, manifests, WAL, checkpoints, layouts, replay artifacts, runtime
  registries, or test-owned physical subsystem graph

Because C.4 does not yet exist, the directory state is observed only to prove
that C.3 neither writes it nor treats it as persisted truth.

### Execution topology

1. Journey A performs the complete ordinary lifecycle over the absent root and
   repeats it over the existing empty directory.
2. Journey B performs all concurrent admission, stale-handle, panic, abort,
   unexpected-drop, and active-pressure behavior inside one deterministic
   schedule family.
3. Compiler Boundary C compiles the one legitimate consumer and the fixed
   forbidden authority classes.

No process-death recovery claim is made. A child-process kill is used only to
prove that C.3 does not emit files or a successful-close claim on process
death.

### Independent observation

- external compile-fail fixtures attempt forbidden construction, cloning,
  transition ordering, mutation through observation, authority reconstruction,
  and operation after move/close
- ordinary OS directory inspection asserts exact absence of C.3-created files
- Cargo metadata and dependency/source boundary checks assert there is one
  exported admission facade and no production test feature widens it
- allocation instrumentation observes the real public admission boundary and
  rejects broad heap ownership by a deliberately generous fixed ceiling;
  Cargo dependency checks and closeout's manual constructor/field trace reject
  persisted-layout, replay, memory-backend, and certification composition
  without relying on a source-token blacklist that aliases can evade
- runtime behavior is compared with an independent lifecycle transition table,
  not a receipt emitted by the runtime

### Assertions and counters

- exactly one composition root exists per admitted runtime incarnation
- runtime identities are unique and cannot be converted into authority
- same-root and different-root admission behavior matches the declared C.3
  process-local policy without claiming C.4 cross-process locking
- observation exposes only immutable, phase-valid state
- every future physical capability family is observed as absent and no
  corresponding operation is callable on the admitted phase
- close, abort, panic, and unexpected drop remain distinct outcomes
- admitted, denied, transition, active-observation, capability-observation,
  close, abort, and unexpected-drop counters match the exact action trace;
  physical-operation, publication, and media counters remain zero
- directory contents remain exactly unchanged

### Anti-substitution

Forbidden substitutes include private fields behind cloneable wrappers,
generic `AuthorityMarker` bounds, a global registry treated as the sole guard,
test-only runtime construction, supplied persisted layouts, replay artifacts,
runtime assertions for statically invalid calls, a generic storage/backend
parameter on the admitted phase, an `Unavailable` wrapper that contains a real
heap owner, physical-looking methods that merely return `Unavailable`, fixture-
created subsystem graphs, and evidence based only on a non-empty id or
successful return.

### Evidence and rerun

C.3 closeout runs the direct owner, lifecycle, UI, dependency, boundary, and
workspace checks. Scenario output is disposable diagnostics; it is not hashed,
reconciled, retained as a certificate, or consumed by another test.

## Phase Plan

### Phase 1: Name Runtime Authority And Truth Status

**Responsibility:** define the vocabulary and type distinctions that prevent
declarations, identities, observations, models, and diagnostics from being
mistaken for runtime authority.

**Relevant subsystems and APIs:** `physical_runtime::identity`,
`PhysicalRuntimeAdmission`, `DeclaredStoreRoot`, `RuntimeIdentity`, and typed
admission errors.

**Engineering decisions:**

- root path/declaration, future C.4-admitted stable root identity,
  runtime-incarnation identity, platform authority, future physical mutation
  authority, and observation are distinct meanings; C.3 does not define the
  future stable-root or mutation-authority types
- identity fields are private and read-only; no identity type implements an
  authority trait or reconstructs an authority-bearing runtime
- configuration and declarations are serializable only if operationally useful;
  runtime authority and access handles never are
- C.3 introduces no platform-authority witness because there is no real
  external platform owner; Store's private constructor path is the actual
  construction boundary, and any later authority must arrive from a real
  predecessor rather than a zero-input mint
- truth status appears in names: `Declared`, `Admitted`, `Observed`, `Absent`,
  and future `MediaOwned/Serving/Recovered` are not interchangeable

**Warnings:** do not create a universal capability token, generic phase marker,
or `RuntimeProof` bag. The type graph must describe real ownership distinctions,
not mirror every sentence in this spec.

**Test requirements:**

- Journey A's independent table must classify declaration, identity,
  observation, and authority exactly as the production facade does.
- Compiler Boundary C must reject representative identity/observation/
  diagnostic promotion and any public conversion that reconstructs authority.
  These are assertions in the shared suite, not Phase 1 fixtures.

**Open question:** whether the stable root identity is derived in C.3 or only
declared for C.4. Default: C.3 carries a caller-admitted `DeclaredStoreRoot`
without claiming filesystem-derived identity; C.4 installs durable namespace
identity.

### Phase 2: Seal Admission Inputs And Runtime Identity

**Responsibility:** make one exhaustive, validated, Store-owned admission path
the only way to create a runtime incarnation.

**Relevant subsystems and APIs:** `physical_runtime::admission`,
`PhysicalStore::admit`, `PhysicalRuntimeAdmission`, nested
`AdmittedPhysicalRuntime`, `AdmissionError`, and `RuntimeIdentity` generation.

**Engineering decisions:**

- admission consumes the complete validated root declaration
- configuration is absent until an installed subsystem has real behavior to
  configure; accepting ignored future knobs is forbidden
- validation rejects before subsystem allocation or registration
- runtime identity is generated inside the sealed transition after validation
- a failed admission cannot leak a partially admitted runtime or leave a
  registry entry that blocks subsequent valid admission
- test setup calls the same facade; no public `new_for_test`, unchecked builder,
  or feature-gated constructor exists

**Warnings:** same-root process-local coordination is not cross-process store
locking. Name and document the limited C.3 guarantee so C.4 can replace it with
real namespace ownership without API deception.

**Test requirements:**

- Journey A must show equivalent admitted topology and distinct incarnation
  identity for repeated legitimate worlds, with rejection occurring before
  allocation or registration side effects.
- Journey B and Compiler Boundary C together must deny live duplicate admission
  and alternate/test construction without adding a per-input case matrix.

**Open question:** whether multiple C.3 admitted-but-not-open runtimes may name
the same root in one process. Default: reject duplicate live admission now to
avoid ambiguous composition ownership, while clearly reserving durable and
cross-process lock semantics for C.4.

### Phase 3: Build The Exhaustive Installed Composition Root

**Responsibility:** create one move-only root that exhaustively owns only the
responsibilities actually installed by C.3 and forces later concrete owner
additions through every lifecycle site.

**Relevant subsystems and APIs:** `physical_runtime::runtime`,
`AdmittedPhysicalRuntime`, and its direct private fields for runtime identity
and process-local root admission. Later C.3 phases add lifecycle, observation,
availability, and diagnostics fields only when those responsibilities become
operational.

**Engineering decisions:**

- the composition root is not `Clone`, `Copy`, serializable, or publicly
  constructible
- `AdmittedPhysicalRuntime` uses direct exhaustive private fields, not a nested
  subsystem bag, map, service locator, heterogeneous registry, or optional bag
- phases 1 through 3 contain exactly the generated runtime identity and the
  admitted root owner; configuration holders, constant counters, empty
  registries, and future-status shells do not count as installed owners
- C.3 contains no media, page, record, WAL, checkpoint, buffer, recovery,
  integrity, scheduling, layout, or blob owner field
- C.3 contains no `UnavailableSubsystem<T>`, mock owner, model owner, or generic
  slot implementing the same operational trait as a future physical owner
- each construction and transition site must exhaustively construct,
  destructure, or explicitly propagate every responsibility installed at that
  point in the milestone
- adding a subsystem causes compile errors at every incomplete lifecycle site
- the root does not borrow the whole world for narrow subsystem work

**Warnings:** an `Arc<Mutex<AdmittedPhysicalRuntime>>` is cloneable root
authority in disguise. Pre-allocating future owner slots would turn absence
into a swappable implementation detail and allow the old heap runtime to enter
through the back door.

**Test requirements:**

- Journey A must reconcile only the installed lifecycle responsibilities
  through construction and close, while proving every physical owner family is
  absent and unallocated.
- Compiler Boundary C must reject representative reconstruction/duplication,
  while the implementation's exhaustive field handling makes an added owner
  fail all incomplete lifecycle sites at normal compile time.

**Open question:** none. Existing physical/model owners are not admitted into
C.3. Their first eligibility decision occurs in the milestone that connects
them to a real physical transition.

### Phase 4: Encode Lifecycle Progression In Types

**Responsibility:** make valid runtime phases and transitions callable and
invalid orderings unrepresentable.

**Relevant subsystems and APIs:** `physical_runtime::lifecycle`,
`AdmittedPhysicalRuntime`, `ClosedRuntime`, `AbortedRuntime`, and typed failure
for the lifecycle transitions that C.3 actually implements.

**Engineering decisions:**

- phase-bearing runtime types own the composition root; phase markers alone do
  not grant authority
- transitions consume the prior phase and return the next phase
- ordinary environmental failure returns a typed error plus the safely
  reusable prior phase when its invariants remain intact
- transitions that may leave state indeterminate say so explicitly and do not
  automatically return reusable prior authority
- C.3 implements real admitted-to-closed and admitted-to-aborted progression;
  it does not define production `MediaOwned`, `Serving`, `Recovering`, or
  maintenance runtime types at all
- C.4 and later add new phase types only in the same change that installs their
  concrete predecessor transition and production owner

**Warnings:** a single runtime struct plus a mutable state enum and pervasive
`InvalidState` checks fails this phase. A typestate wrapper with a public raw
runtime escape hatch fails equally.

**Test requirements:**

- Journeys A and B must match the same independent C.3 lifecycle table for
  ordinary and hostile admitted/closed/aborted sequences.
- Compiler Boundary C must reject skipped admission, physical/future-phase
  calls on the admitted phase, after-move use, and terminal use.

**Open question:** whether phase types use distinct wrappers or one generic
`PhysicalRuntime<Phase>`. Default: choose the representation that keeps public
names readable and internal access narrow; no generic phase parameter may be
publicly implementable or constructible.

### Phase 5: Keep Physical Mutation Authority Absent

**Responsibility:** make it impossible for C.3 admission to yield data mutation,
publication, allocation, recovery, or generic subsystem mutation authority.

**Relevant subsystems and APIs:** the public method inventory of
`AdmittedPhysicalRuntime`, crate visibility, installed C.3 lifecycle fields,
and the sealed transition insertion points that C.4/C.5 will later extend.

**Engineering decisions:**

- no `MutationAccess`, `WriteAccess`, raw subsystem accessor, record/page API,
  or generic operational trait is implemented for `AdmittedPhysicalRuntime`
- internal `&mut self` use is limited to root admission, resource registration,
  lifecycle transition, and counter maintenance
- no admitted field contains records, pages, byte images, WAL frames, manifests,
  buffer frames, or a model that can stand in for any of them
- C.4 may introduce narrowly named media mutation only on a media-owned phase;
  C.5 may introduce record/page mutation only on a physically opened phase
- later physical concurrency is earned from real subsystem disjointness and is
  not anticipated by a generic C.3 mutation token

**Warnings:** a handle with no current methods but a production-sounding
`MutationAccess` name is still a false capability and a future escape hatch.
Do not create it in anticipation of later work.

**Test requirements:**

- Journey B must hold the runtime under concurrent observation while proving
  physical-operation and physical-owner counters remain exactly zero.
- Compiler Boundary C must reject physical mutation on admitted authority and
  observation-to-mutation promotion because neither target type/API exists.

**Open question:** none. Physical mutable borrowing begins only after a real
owner exists; C.3 does not speculate about its access pairs.

### Phase 6: Scope Observation Without Authority Leakage

**Responsibility:** provide useful immutable runtime visibility while keeping
observation phase-scoped, revocable/stale-aware, and categorically
non-authoritative.

**Relevant subsystems and APIs:** `physical_runtime::observation`,
`ObservationHandle`, `RuntimeObservation`, `LifecycleObservation`,
`ObservationError::{Closed, Stale}`, and read-only identity/capability-status/
counter accessors.

**Engineering decisions:**

- cloneability is permitted only for handles whose methods remain immutable
  and non-authoritative
- handles carry runtime-incarnation and lifecycle generation identity so stale
  access is reported deterministically
- observation snapshots are derived values and may be destroyed/rebuilt from
  the live runtime; they never participate in construction or transition
- phase transitions either invalidate old handles or narrow their visible
  surface according to one explicit policy
- no observation method lazily performs I/O, recovery, maintenance, or broad
  traversal behind getter-shaped syntax
- observing current counters is O(1) at the named facade; rich diagnostic
  materialization is an explicit separate operation

**Warnings:** `Arc<InnerRuntime>` inside an observation handle leaks authority
if any reachable method mutates or publishes. Shared storage must expose a
read-only type boundary, not rely on naming convention.

**Test requirements:**

- Journeys A and B must show that simultaneous observations agree at each
  schedule point and become exactly stale/closed under the chosen terminal law.
- Compiler Boundary C must prove one observation value cannot reach any
  mutation, transition, publication, recovery, or raw-owner surface.

**Open question:** whether post-close immutable summary access remains valid.
Default: `ClosedRuntime` owns an explicit final summary; pre-close handles are
invalidated so shared ownership cannot prolong the live runtime lifecycle.

### Phase 7: Expose Only Installed Lifecycle Observation

**Responsibility:** expose narrow observation of C.3's installed lifecycle
responsibilities without manufacturing views over future physical owners.

**Relevant subsystems and APIs:** `physical_runtime::observation`,
`RuntimeObservation`, `LifecycleObservation`, immutable root-admission status,
installed-capability status, and structural counters.

**Engineering decisions:**

- each observation accessor borrows exactly one installed C.3 responsibility
- public consumers receive lifecycle and availability observations, not raw
  internal owner types
- there is no `MediaObservation`, `ResidencyObservation`,
  `DurabilityObservation`, or equivalent object until the observed owner exists
- there are no crate-private mutable accessors for absent physical owners
- no service locator, string key, `Any`, trait-object registry, or global
  context is introduced
- future milestones add observation beside the real owner they install

**Warnings:** an observation object named after a nonexistent physical owner is
still a semantic claim that the owner exists. A bag of unavailable views makes
it easy to swap in a heap implementation without changing the facade.

**Test requirements:**

- Journey A must reconcile lifecycle/availability observations with the facade
  while exact allocation and installed-owner counters prove no physical owner
  was started.
- Compiler Boundary C must reject deep imports, untyped owner lookup, and every
  attempted future-owner observation on the admitted phase.

**Open question:** none. C.3 exposes identity, lifecycle, root-admission status,
installed-capability status, and structural counters only.

### Phase 8: Represent Uninstalled Physical Work As Missing APIs

**Responsibility:** encode the absence of later-milestone physical work through
missing phase types and methods, with immutable availability observation only.

**Relevant subsystems and APIs:** `physical_runtime::availability`,
`PhysicalCapability`, `CapabilityAvailability`, and the public method inventory
of `AdmittedPhysicalRuntime`.

**Engineering decisions:**

- availability observations name the absent owner/capability without creating
  an operational object or roadmap-named production type
- physical operation denial occurs at compilation because the method and target
  phase do not exist
- availability is derived from installed production owners, not caller flags
  or test configuration
- observing absence performs no allocation, registry mutation, physical
  identity generation, model mutation, or success-counter increment
- `Unsupported` and environmental failure become meaningful only after a real
  owner/operation exists; C.3 does not prebuild their operation envelope
- a later capability adds a new phase type and its real operation surface in
  the same change that installs the production owner

**Warnings:** returning `Unavailable` from a physical-looking method is weaker
than not exposing the method and invites someone to replace the denial with a
heap implementation. C.3 must choose structural absence.

**Test requirements:**

- Journey A must observe every named future capability family as absent and
  prove zero physical-owner allocation and zero physical effect.
- Compiler Boundary C and the ordinary-feature topology gate must prove caller
  flags, replay values, test features, and direct method calls cannot install
  or invoke a capability.

**Open question:** none. Future physical operation methods are absent from C.3.

### Phase 9: Define Drain, Close, Abort, And Crash Posture

**Responsibility:** make runtime termination outcomes distinct and ensure no
live authority or managed resource silently survives a terminal transition.

**Relevant subsystems and APIs:** `physical_runtime::shutdown`,
`AdmittedPhysicalRuntime::close`, `abort`, future `begin_drain`,
`ClosedRuntime`, `AbortedRuntime`, `ShutdownError`, resource registration, and
the unexpected-drop guard.

**Engineering decisions:**

- explicit close consumes the live runtime, rejects or awaits outstanding
  managed access according to a declared bounded policy, and returns a closed
  summary
- abort consumes authority and records in-process abandonment without claiming
  durability, cleanup, or recovery
- unexpected drop/panic records an observable incomplete termination event in
  process where possible; it never emits a successful-close outcome
- child resources are framework-owned and cannot remain unregistered
- C.3 draining concerns in-process handles/tasks only; file flush, WAL drain,
  namespace cleanup, and recovery residue belong to later milestones
- `Drop` is a safety net for revocation/diagnostics, not a hidden blocking close
  path

**Warnings:** destructors must not conceal unbounded work, panic, or report
physical durability. An `Arc` retained by observation must not keep composition
authority, root admission, or child resources alive after close.

**Test requirements:**

- Journey B must cover close, abort, panic, cancellation, and unexpected drop
  inside one deterministic schedule and distinguish their exact terminal
  outcomes without orphaned resources or false success.
- Compiler Boundary C proves moved/terminal runtime authority cannot be reused;
  Journey B proves retained observation handles become inert and no OS residue
  appears afterward.

**Open question:** whether C.3 supports asynchronous draining. Default: keep
the phase model capable of a later explicit async boundary, but implement only
bounded in-process coordination needed by current owners; do not introduce an
executor or background thread solely for lifecycle ceremony.

### Phase 10: Seal The Public Facade And Dependency Topology

**Responsibility:** make the canonical path mechanically unavoidable from
ordinary product code and spatially locate every authority boundary.

**Relevant subsystems and APIs:** `worth-store` crate root exports,
`physical_runtime::mod`, crate visibility, lower physical crate manifests,
boundary-check configuration, generated `AGENT_CONTEXT.md`, and consolidated
UI fixture ownership.

**Engineering decisions:**

- `physical_runtime::mod` aggregates public types but implements no behavior
- the public surface exports capabilities and lifecycle types, not internal
  owner topology
- lower crates may expose narrow subsystem contracts to `worth-store`; they do
  not export public composition constructors that product callers can combine
- ordinary production features cannot enable certification/test constructors
- replay/reconstruction surfaces remain outside ordinary lanes and cannot
  satisfy admission
- boundary configuration, not comments, enforces dependency and facade law;
  generated agent-context files are regenerated rather than edited manually

**Warnings:** a clean directory tree with deep public imports or re-exported raw
owners is decorative architecture. Likewise, source grep alone is not enough;
compile/dependency gates must reject the bypass.

**Test requirements:**

- Journeys A and B must both enter through `PhysicalStore::admit`; their traced
  production call paths must contain no alternate composition entrance.
- Compiler Boundary C plus the existing boundary checker must reject internal,
  lower-crate, replay/layout, certification, and test-feature substitutes as
  one facade-authority class.

**Open question:** whether any existing lower-crate constructor must remain
public for owner-local tests. Default: keep it crate-private or test-local at
the narrowest owner scope; public compatibility requires an explicit bounded
deprecation and cannot mint composition authority.

### Phase 11: Make Lifecycle Cost And Decisions Observable

**Responsibility:** expose enough non-authoritative measurement to explain
admission, observation, capability absence, and termination without pulling rich diagnostics
onto ordinary paths.

**Relevant subsystems and APIs:** `physical_runtime::diagnostics`, structural
counter cells, `RuntimeObservation`, `ClosedRuntimeSummary`, typed admission and
transition errors, and optional rich lifecycle trace materializer.

**Engineering decisions:**

- counters live at admission, lifecycle transition, observation,
  capability-status, resource-lifecycle, and terminal boundaries
- ordinary counters are monotonic or phase-scoped with documented reset law;
  each has one causal increment site
- ordinary identity/phase/counter observation is O(1) and allocation-free after
  handle acquisition
- rich traces are derived on explicit request and cannot authorize, reconstruct,
  or keep the runtime alive
- measurement distinguishes attempted, admitted, denied, completed, aborted,
  and indeterminate outcomes rather than counting only success
- counter snapshots name their runtime incarnation and phase generation so
  cross-runtime aggregation cannot silently imply sameness

**Warnings:** logs, ids, elapsed time, and nonzero counts are not correctness
evidence. Diagnostics must not duplicate lifecycle state or become a second
decision source.

**Test requirements:**

- Journeys A and B must reconcile all exact counters against their independent
  action traces; no separate counter-by-counter scenario family is allowed.
- Journey A must prove ordinary identity/phase/counter observation performs no
  scan, physical operation, or post-acquisition allocation, while Compiler
  Boundary C rejects diagnostic promotion.

**Open question:** whether rich decision traces belong in C.3 production code
or only in later operational diagnostics. Default: ship typed errors and exact
structural counters now; add a trace only if a concrete C.3 ambiguity cannot be
localized without one.

### Phase 12: Prove Authority And Lifecycle Adversarially

**Responsibility:** falsify the complete C.3 authority model through compiled
misuse attempts and difficult joined behavioral schedules, not isolated happy
paths.

**Relevant subsystems and APIs:** the consolidated UI suite, ordinary
`worth-store` integration tests, child-process test executable, deterministic
schedule driver, OS directory observer, and Cargo/public-topology gates.

**Engineering decisions:**

- compile-fail fixtures are grouped by responsibility and share one Cargo
  compilation root
- behavioral scenarios call only the production facade; test support may
  schedule calls and observe boundaries but cannot construct internal owners
- the lifecycle oracle is an independent finite transition table fixed before
  execution
- scenario seeds, action traces, counter expectations, and direct directory
  observations are reproducible

**Test requirements:**

- run Journey A and Journey B as the only behavioral closeout worlds and show
  that their combined invariant set covers every phase's referenced assertion
- compile the ordinary product under default features, then run Compiler
  Boundary C once under the maximal admitted authority feature; do not duplicate
  the trybuild campaign without a known authority difference
- assert duplicate authority is compiler-rejected, heap-backed owner
  installation is absent at runtime, and unexpected drop remains distinct from
  successful close

The two journeys collectively provide equivalence, parity, rejection, drift,
residue, and leakage coverage. Compiler Boundary C adds only the negative space
that ownership and visibility make impossible to exercise at runtime.

**Warnings:** randomized concurrency without deterministic replay, sleeps as
ordering, source search as the only authority check, or a broad `is_err()` does
not satisfy this phase.

**Open question:** which lifecycle yieldpoints must remain in production.
Default: retain stable, low-cost named boundaries needed by later crash and
coordination milestones; test-only scheduling wraps those boundaries without
gaining private mutation access.

### Phase 13: Cut Over And Quarantine Duplicate Authority

**Responsibility:** make the new facade the only production composition path,
then delete or explicitly quarantine paths that still imply heap-backed,
replay-backed, or certification-backed physical authority.

**Relevant subsystems and APIs:** old `PhysicalStoreRuntime` construction and
clone surfaces, persisted-layout/replay reopen paths, certification factories,
production manifests/features, deprecation boundaries where deletion cannot
yet occur, and current authority blockers found in C.3's causal call paths.

**Engineering decisions:**

- migrate production callers and tests by responsibility, never by aliasing the
  old constructor to the new authority
- delete duplicate composition APIs when no legitimate non-production owner
  remains
- honestly named offline/model representations may remain only behind a
  boundary that cannot be promoted into runtime authority
- any temporary compatibility surface is non-authoritative, has a named owner
  and deletion milestone, and is unreachable from ordinary admission
- C.3 resolves authority blockers in source and direct tests without creating
  an audit artifact or runtime evidence wrapper
- later milestones extend this composition root; they do not create successor
  runtimes beside it

**Warnings:** type aliases, wrapper renames, deprecated public constructors,
feature-hidden factories, and broad test-support exports can preserve the exact
duplicate authority this phase is meant to remove.

**Test requirements:**

- rerun Journeys A and B after cutover and require the same lifecycle/counter
  outcomes through the canonical facade
- rerun Compiler Boundary C and the targeted boundary trace to prove the old
  clone, replay/layout, certification, and lower-owner construction class is
  unreachable without adding compatibility-specific test worlds

**Open question:** whether any model-only constructor shares the old production
name. Default: rename it by its actual in-memory/model truth status now; do not
preserve semantic ambiguity for compatibility.

## Must Ship

- one sole Store-owned production admission facade
- one move-only, private-construction runtime composition authority
- sealed Store-owned admission with no caller-mintable authority substitute
- distinct declared-root and runtime-incarnation identity types; C.4—not C.3—
  defines stable namespace identity
- exhaustive ownership of only the lifecycle responsibilities installed by C.3
- real admitted/closed/aborted typestate progression with no physical future
  phase type defined early
- no physical mutation handle, physical operation method, future-owner view, or
  heap-backed physical subsystem in the admitted runtime
- cloneable observation only where structurally immutable and stale-aware
- immutable capability status showing every physical owner family as absent
- explicit close, abort, panic, unexpected-drop, and future drain posture
- narrow public facade plus mechanical dependency/feature enforcement
- exact lifecycle, observation, capability-status, and termination counters
- two production-facade lifecycle journeys, one consolidated compiler boundary,
  and three representative controlled defects
- deletion or non-production quarantine of every C.3 duplicate authority path

## Must Preserve

- Physical Store owns byte survival and physical access; C.3 does not absorb
  Query, Relational, Signal, or Runtime Bridge authority.
- Physical format owns byte meaning, while the future C.4 media boundary owns
  actual effects.
- Existing subsystem owners remain autonomous and independently testable.
- Replay/reconstruction remains outside ordinary lanes and never becomes an
  admission source.
- Certification observes and attacks the runtime; it never constructs or
  promotes it.
- Runtime authority remains singular without forcing all subsystem work
  through one global mutable borrow or lock.
- Absent capability means no operational API, no owner allocation, no physical
  effect, and no simulated heap success.
- C.4 retains ownership of real namespace identity, locking, filesystem
  operations, and backend capability qualification.

## Acceptance Evidence

C.3 closes with:

- direct owner and integration test commands
- one consolidated UI report covering construction, cloning, conversion,
  lifecycle order, stale use, and facade bypass
- boundary-check and generated agent-context validation
- Cargo feature checks proving ordinary lanes cannot enable test,
  certification, or replay authority
- exact independent lifecycle action trace and matching runtime counters
- unchanged absent-root and empty-directory observations proving zero physical
  effects
- results for Journey A, Journey B, and Compiler Boundary C
- direct source and test corrections for authority/lifecycle blockers

No runtime-produced receipt certifies another runtime-produced receipt. The
compiler, direct process outcomes, OS observation, dependency topology, and
independent transition oracle are the evidence sources.

## Sequencing Notes

1. Implement phases 1 through 4 as the authority/lifecycle foundation.
2. Implement phases 5 through 7 as the physical-authority-absence and
   lifecycle-observation slice.
3. Implement phases 8 through 11 as honesty, termination, facade, and
   observability enforcement.
4. Implement phases 12 and 13 as joined falsification and source cutover.
5. Run owner tests during each logical slice and the joined hostile scenarios
   after the implementation is coherent.
6. C.4 may begin only after the canonical runtime exists, duplicate production
   authority is unreachable, and the future media transition has exactly one
   sealed insertion seam.
7. C.4 must replace the C.3 process-local root-admission posture with real
   namespace and cross-process ownership law by consuming the admitted runtime
   into a new media-owned type; it must not add media methods to the admitted
   type.
8. C.5 may create the first record/page-serving phase only in the same change
   that proves real files and fresh-process reopen. Later milestones likewise
   add authority only through real predecessor transitions.

## Completion Standard

C.3 is complete only when the compiler and direct runtime behavior agree that
there is one physical composition authority, every legal lifecycle transition
preserves or consumes it exactly once, observation and identity cannot promote
themselves, no physical operation or owner exists before its real transition,
capability observation allocates no shadow store, terminal outcomes are honest,
and every later physical owner has one unavoidable runtime seam to join.

The milestone is not complete merely because a struct is private, tests are
green, ids are unique, or old heap behavior has been renamed. It closes only
when the strongest plausible external caller cannot manufacture a second path
to physical authority.
