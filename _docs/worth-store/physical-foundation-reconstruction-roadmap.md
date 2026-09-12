# Worth Store Physical Foundation Reconstruction Roadmap

## Purpose

This is the mandatory reconstruction program inside Part I of Worth Store.
It reopens the physical-foundation claims currently attributed to `S.1`
through `S.9`, repairs the development proof loop first, then joins the real
physical mechanisms into one sealed production database runtime before `S.10`
operational recovery continues.

This roadmap does not discard useful page, WAL, buffer-pool, integrity,
isolation, scheduling, layout, blob, model, or certification work merely
because the current composition is not a database. It also gives none of that
work closure credit merely because its vocabulary, local tests, or isolated
mechanisms are strong. Existing code is substrate to inspect and either admit,
refactor, or delete. Only execution through the reconstructed production path
earns a physical claim.

## Roadmap Position

```text
Physical Database Roadmap S.9 implementation state reopened
  -> C.1 direct test execution and iteration cleanup
  -> C.2 historical audit (retired; current inspection is phase-local)
  -> C.3 sealed physical runtime authority and lifecycle
  -> C.4 production media boundary and stable store namespace
  -> C.5 durable page, segment, extent, and manifest path
  -> C.5.1 physical Store instance and Signal-orchestrated work
  -> C.6 buffer pool and bounded physical access join
  -> C.7 WAL, checkpoint, root publication, and acknowledgment join
  -> C.8 fresh-process recovery and reopen
  -> C.9 physical integrity, corruption localization, and offline truth
  -> C.10 stable reads, scheduled I/O, and maintenance interference
  -> C.11 layout, index, and native blob adoption
  -> C.12 formal protocol rebinding to executable owner transitions
  -> C.13 joined physical-platform integration and S.10 entry
  -> Physical Database Roadmap S.10
  -> S.11
  -> S.12
  -> Runtime And Query Integration Roadmap Milestone 1
```

The `C.*` labels are reconstruction sequence numbers used only in planning,
specification, evidence, and closeout. Production modules, types, functions,
tests, scenarios, counters, and errors must use responsibility names rather
than `C.*`, phase, cleanup, or roadmap provenance.

No new `S.10` implementation receives closeout credit while this program is
open. Work already present in `S.10` may be preserved as unadmitted substrate,
but it must later bind to the reconstructed runtime and repeat its production-
path proof.

## Relationship To The Two Store Roadmaps

The [Physical Database Roadmap](physical-database-roadmap.md) owns Part I. It
must produce the platform that makes bytes survive through real pages,
segments, WAL, checkpoints, stable reads, bounded memory, corruption handling,
maintenance, blobs, recovery, and declared media assumptions. This cleanup is
therefore a Part I correction program, not a bridge feature and not a new
product layer.

The [Runtime And Query Integration Roadmap](runtime-integration-roadmap.md)
owns Part II. It starts from an already real physical Store instance and builds
one composition root that owns the existing Query runtime, that physical Store
instance, and a narrow Store-Query adapter between them. Part II owns semantic
lowering, hydration, durable commit publication, Store-backed reads, branch
concurrency, semantic residency, recovery readmission, and Query parity. This
reconstruction must expose the physical contracts Part II needs, but it must
not import Query, decide MVCC visibility, persist runtime authority, or create
a second semantic Store runtime in anticipation of Part II.

The dependency is deliberately asymmetric:

```text
                         Product / API
                              |
                       Worth Query facade
                              |
                 Store-backed composition root
                    /                    \
          existing Query runtime       physical Store instance
          Relational / semantic       physical authority / physical Signal /
          Signal / Runtime Bridge     scheduler / buffer / WAL / media
                    \                    /
                     narrow Store-Query adapter
```

The physical Store instance knows nothing about Query. Its physical Signal
graph is a private, reconstructible work graph for dependency readiness and
async completion; it is not crash authority and is not the semantic Signal
graph. The Part II adapter knows both public contract families and owns only
their join.

The future branch-concurrency law sharpens that boundary. Part II may plan or
queue several mutations for one branch, but Relational permits exactly one
mutation at a time to own active branch-write authority for a branch-head
generation. Different branches may own that authority concurrently, and
shared global authority coordinates independently. Part I must make this
possible without learning branch identity: it exposes independently borrowable
physical submission capabilities, exact physical scopes, generation-fenced
operation identities, bounded read capabilities, and exact effect fate. A
branch name, branch generation, semantic writer token, or caller-declared lane
is never a physical disjointness proof.

## Governing Summaries

- `MENTALITY.md` protects hard-problem-first foundations. The strongest
  constraint is that iteration speed and a real write/reopen vertical slice
  must be repaired before more operational surface is added.
- `arch_laws.md` protects proof-carrying authority and explicit lifecycle. The
  strongest constraint is that the production runtime must be sealed,
  non-cloneable as authority, decomposed into independently borrowable
  subsystems, and progressed through real durable effects rather than supplied
  replay representations.
- `composition_laws.md` protects one named responsibility per unit. The
  strongest constraint is that test support, runtime orchestration, media I/O,
  recovery, and certification cannot remain broad bags or duplicated execution
  paths.
- `domain_structure_laws.md` protects ownership and truth-source topology. The
  strongest constraint is that local tests construct only the owner they
  falsify, focused integration tests cross real owner boundaries, and physical
  boundary crossings remain spatially locatable.
- `perf_laws.md` protects visible cost and semantic locality. The strongest
  constraint is that test execution cost, resident memory, I/O breadth,
  recovery work, amplification, and interference all require named boundaries
  and counters rather than elapsed-time folklore.
- `physical-database-roadmap.md` protects byte survival. This program must make
  its S.1 through S.9 mechanisms true on one production path before S.10 can
  safely manipulate damaged or restored stores.
- `runtime-integration-roadmap.md` protects one Store-backed Query runtime.
  This program must finish with one physical Store instance whose authority,
  derived work graph, scheduler, and filesystem effects are internally
  coherent, plus stable facades that the Part II adapter can consume without
  duplicating persistence, recovery, access planning, or byte authority above
  Store.
- `test-requirements.md` protects proof of the declared behavior. Each
  reconstruction milestone must retain control, hostile, reopen, semantic-
  parity, and forbidden-shortcut lanes where applicable.
- `test-requirements-2.md` protects the physical proof mechanism. Real process
  death, real persisted bytes, production-boundary fault delivery,
  independent verification, memory envelopes, and mutation sensitivity are
  mandatory; simulation labels and live heap reuse are not evidence.
- `storage-foundation-s10.md` protects operational recovery without trusting
  the live store. Its workflow design remains downstream and unclosed until
  the underlying store can actually write, die, reopen, and be independently
  inspected through the production physical platform.

## Global Adversarial Constraint

The reconstruction must survive this hostile condition:

> An engineer changes one physical owner and receives a trustworthy local
> result quickly; CI then drives the same production runtime through real file
> writes, page and WAL publication, process death, fresh-process reopen,
> corruption, memory pressure, concurrent reads, maintenance, layout access,
> and blob streaming. Physical dependency work may be derived and scheduled
> through one runtime-owned Signal graph, but only physical authority may
> admit or settle effects. No lane may obtain persisted truth from a replay object,
> copied heap layout, surviving process state, test-owned oracle, test-only
> authority, mock-only backend, or an isolated mechanism that the production
> runtime does not call. Every admitted claim must name the exact bytes,
> authority transition, cost boundary, and independent observer that proved it.

The program has failed if it merely makes the current tests faster while
preserving fake physical proof, or makes the physical runtime real while
leaving ordinary iteration so expensive that broad verification is avoided.

## Product Decision Lock

1. Test cleanup is the first milestone and blocks all reconstruction work that
   would otherwise inherit the current ten-minute feedback loop.
2. Faster testing means less repeated compilation and correctly separated
   proof lanes. It never means deleting hostile assertions, weakening physical
   scenarios, or calling a smoke lane certification.
3. The physical Store instance represented by `PhysicalStoreRuntime` or its
   responsibility-named replacement is the sole production physical
   composition authority. It owns one physical authority, one reconstructible
   physical Signal graph, and the admitted scheduler/executor boundary. It is
   not `Clone`, does not expose constructors that mint duplicate authority, and
   does not reopen from caller-supplied heap layouts or replay artifacts.
4. A stable store root plus admitted configuration and platform authority are
   sufficient to open or recover the physical store. Callers do not supply the
   bytes that recovery is supposed to discover.
5. Real file/media operations are reached through one production storage
   boundary used by ordinary execution and wrapped by the adversarial harness.
6. Existing file-writing mechanisms in backend, WAL, isolation, replication,
   or operations crates earn no database claim until the canonical runtime
   invokes them in the required order.
7. Physical format owns byte encoding and decoding; the backend owns media
   effects; the buffer pool owns residency; recovery owns source precedence
   and replay; isolation owns stable physical visibility; physical Signal owns
   derived dependency readiness and async completion posture; the I/O scheduler
   owns resource admission. No one mechanism absorbs or outranks these laws.
8. Certification may observe, inject faults, compare, and report. It cannot
   mint production authority, supply expected persisted state, or become the
   only caller of a mechanism claimed as production behavior.
9. Compile-fail proof remains mandatory for authority and dependency
   boundaries, but it runs through consolidated, cache-sharing UI suites rather
   than hundreds of cold nested Cargo projects.
10. S.1 through S.9 closeout is reopened. Current direct evidence from each
    owning phase and the joined C.13 integration scenario decides which claims
    are restored; historical status does not.
11. S.10, S.11, and S.12 remain in their existing conceptual order after this
    program. Part II still begins only after the complete physical roadmap
    closes.
12. Query, Relational, semantic Signal, and Runtime Bridge remain consumers of
   the final platform through Part II. C.5.1 may install a distinct physical
   Signal instance only in the Store-owned physical composition layer. No Part
   I milestone imports or partially integrates Query, Relational, or Runtime
   Bridge.
13. A physical phase type and its operation methods land only in the same
    change that installs the concrete production owner and its required real-
    machine acceptance journey. No milestone predeclares operational shells
    backed by flags, optional owners, mocks, memory implementations, or typed
   `Unavailable` responses.
14. Physical Signal state is derived and disposable. WAL, pages, manifests,
   root generations, media ownership, and exact effect outcomes remain the
   sources from which physical work is reconstructed after process death.
15. Format, media, buffer-pool, WAL, recovery, isolation, integrity, layout,
   and blob mechanism crates remain Signal-agnostic. Only the Store-owned
   physical composition layer may bind their contracts into a physical Signal
   graph.
16. The physical Signal graph determines dependency readiness and async
   lifecycle. The I/O scheduler admits scarce resources and the executor
   performs effects. Neither may create a second operation registry, settle
   authority from counters, or bypass the physical owner.
17. Part I owns no branch registry, branch queue, branch-head generation,
    semantic writer lease, MVCC conflict decision, or global semantic
    authority. Those are Part II Relational concerns even when physical work
    originated from a branch mutation.
18. The physical Store must admit concurrent work from independently held
    submission capabilities when exact physical scopes and admitted resource
    policy permit it. It must not impose a permanent whole-Store mutation
    borrow merely because the future semantic owner permits one writer per
    branch generation.
19. Every mutating physical submission carries a Store-owned operation
    identity, Store-incarnation fence, exact physical effect scope, durability
    request, and terminal fate of proven no effect, completed effect, or
    indeterminate effect. Part II may correlate that evidence with a semantic
    mutation; Part I never decides whether a branch head advances, releases,
    or remains fenced.
20. Shared physical coordination such as root publication, WAL barriers,
    allocation metadata, or namespace mutation is declared as physical scope
    and scheduled by the physical owners. It is not a surrogate for Part II's
    shared global semantic authority, and neither authority may be inferred
    from the other.

## Non-Fake Physical Acceptance Test Contract

Every `C.3` through `C.13` engineering spec must contain a
`Non-Fake Acceptance Setup` section. C.1 is the test-execution foundation and
uses its own direct truth contract. C.2 is retired historical context, not a
current audit, report, or gate. Current phases inspect their causal production
paths directly and use focused owner, boundary, and process tests. A physical
test requirement in C.3 through C.13 is incomplete unless it fixes all of the
following before implementation begins:

### Production subject

- Name the exact production facade, executable, and owner methods under test.
- Name every real crate expected on the call path.
- Name the physical artifacts expected to exist after execution.
- Name the exact test-only layers allowed around the production boundary.

### Initial world

- State whether the store root begins absent, empty, or pre-populated by a
  separately identified producer process.
- State the backend profile, format version, page size, memory budget, workload
  seed, and fault profile.
- State which artifacts must not exist before the action.
- Generate semantic expectations independently of the runtime under test.

### Execution topology

- Drive work only through the named public production facade.
- Deliver faults through the production storage boundary or named production
  yieldpoints, never by mutating private runtime state.
- Record process identity, runtime identity, storage-root identity, and every
  authority transition.
- State which process must terminate without normal cleanup and which fresh
  executable performs reopen or verification.

### Independent observation

- Reopen from the store root and admitted configuration only.
- The verification process must not receive `PhysicalStoreRuntime`,
  `PersistedPhysicalLayout`, `PlatformPhysicalReplayArtifact`, cached pages,
  runtime registries, decoded records, or expected state from the writer.
- The independent oracle may share stable format declarations but must not
  share live recovery decisions, caches, normalization, or authority paths.
- Expected outcomes and expected corruption locations must be fixed before the
  verifier runs.

### Assertions

- Assert semantic equality or intentional inequality against an independent
  model.
- Assert exact required artifacts and absence of forbidden residue.
- Assert typed failure localization rather than accepting any error.
- Assert exact or weakest-sufficient structural counters, including counters
  that must remain zero.
- Assert memory, allocation, I/O, amplification, and recovery bounds at their
  named measurement boundaries.

### Hostile sensitivity

- Name at least one controlled defect the milestone must detect.
- Protect it with a direct hostile test at the narrowest real boundary.
- A compile failure proves only a type boundary; behavioral claims require a
  behavioral test.

### Mechanical anti-substitution gates

Each spec must add dependency, source, manifest, or compile-time checks that
reject the substitutes relevant to its claim. The common forbidden set is:

- replay artifacts or persisted heap layouts crossing a process boundary
- live runtime reuse after the declared crash
- test-authority constructors on an ordinary production path
- a mock-only or certification-only backend satisfying a physical claim
- same-path producer and verifier comparison
- private-state corruption or post-hoc byte editing used to simulate a write
  seam when the storage interposer can express it
- whole-store or whole-blob materialization hidden behind a helper
- broad test-support dependencies pulled into owner-local tests
- nested Cargo builds with per-case target directories in ordinary test runs
- evidence based only on success, non-empty digests, elapsed time, or logs

### Evidence and rerun

- Git identifies the reviewed source revision and Cargo identifies the built
  targets and features.
- A failing randomized or process test prints the seed and scenario needed to
  rerun it.
- Tests and boundary checks return the verdict directly. Do not create a
  second proof bundle, source-identity layer, or validator for the test run.

This contract is a floor. Each milestone below fixes its own concrete setup so
an implementation cannot satisfy the words through a neighboring fake.

## Certification Modes And Time Budgets

The proof model has five distinct execution products:

- **Owner check**: the changed crate's unit and narrow integration tests. It
  constructs no unrelated owner and targets feedback in seconds.
- **Developer smoke**: deterministic production-path vertical specimens,
  consolidated UI smoke, and small hostile schedules. The warm target is under
  one minute on the declared reference development machine.
- **CI certification**: owner tests plus medium physical scenarios, real fresh-
  process reopen, direct adversarial regressions, and stores larger than the
  configured memory budget. Jobs are partitioned by proof family.
- **Release certification**: long crash, corruption, maintenance, blob, and
  cross-backend campaigns with independent offline verification.
- **Hardware qualification**: filesystem, flush, rename, direct-I/O, mmap,
  sector, device, and latency claims on a named deployment profile.

Time budgets are measurements and regression gates, not correctness authority.
A slower test does not become invalid merely by exceeding a target; it must be
classified into the proper lane and its structural cost explained. Conversely,
a fast smoke run cannot promote a release claim.

## Milestone Plan

## C.1: Direct Test Execution And Iteration Cleanup

Engineering spec:
[physical-reconstruction-c1-test-execution-architecture.md](physical-reconstruction-c1-test-execution-architecture.md)

### Goal

Restore a fast, structurally honest feedback loop before physical runtime
reconstruction begins.

### Boundary

This milestone changes how tests are selected, compiled, linked, and scheduled.
It does not certify currently fake physical behavior. Cargo, Git, the test
executables, and CI remain the truth sources; C.1 does not construct a parallel
authority hierarchy around them.

### Must Ship

- real, documented `store-owner`, `store-smoke`, `store-ui`, and `store-ci`
  entrypoints backed by direct command planning
- Worth Store-local development and test profiles that avoid full Windows PDB
  generation where it is not required
- consolidation of the scenario explosion into a small number of
  responsibility-named suite binaries, with separate executables retained only
  where process identity is itself part of the proof
- consolidation of compile-fail fixtures behind cache-sharing UI runners with
  stable expected diagnostics and no per-case cold target directory
- removal of nested Cargo invocations from ordinary behavioral tests;
  repository structural checks remain explicit direct commands
- owner-local or subsystem-local fixture/support surfaces replacing the broad
  `worth-store-test-support` dependency where local tests do not need the full
  physical platform
- explicit feature hygiene so certification authority is not unified into the
  ordinary production graph by default
- partitioned CI jobs with cache identity bound to OS, toolchain, profile,
  feature lane, and lockfile
- concise elapsed and unit-count console output; successful local commands
  produce no report or evidence files
- deletion of recursive behavior fingerprints, preservation ledgers, plan/run
  seals, source-edit authority, custom CI aggregates, closeout bundles, and C.2
  readiness tokens

### Non-Fake Acceptance Setup

- **Production subject:** the Worth Store workspace manifests, suite entry
  points, UI runners, CI workflow, and test-support dependency graph.
- **Initial world:** start from a clean target directory for cold measurement
  and a completed identical run for warm measurement. Record Rust toolchain,
  OS, CPU, storage, source revision, features, and exact command.
- **Execution:** change one leaf owner source file and run owner check; change
  one shared physical contract and run developer smoke; change one test/UI
  expectation and run its owning product. Run complete UI and CI products from
  the committed revision.
- **Independent observation:** Cargo manifests name targets, Cargo/test process
  results name behavioral outcomes, ordinary target-directory observation
  distinguishes cold from warm work, and GitHub matrix status names CI
  completion. No runner-produced artifact certifies another runner artifact.
- **Assertions:** owner check does not build certification or unrelated owner
  crates; developer smoke runs the declared vertical specimens; UI fixtures
  share compilation roots; no ordinary lane creates a per-case Cargo target;
  each dedicated scenario, UI, and formal target is selected by its direct
  lane, while ordinary tests run through owner CI.
- **Controlled defects:** invert one UI expectation and one ordinary assertion.
  Each direct owning product must fail for the intended reason. Planner tests
  reject duplicate units, and nextest rejects stale zero-match filters.
- **Forbidden substitutes:** excluding tests from all modes, replacing tests
  with source scans, timing only a no-change green run, or declaring success
  solely because `cargo nextest` is installed cannot close this milestone.

### Closeout Gate

`C.1` closes only when ordinary owner work no longer compiles broad
certification-only suites,
developer smoke exercises real declared specimens with a warm reference target
under one minute, UI proof no longer performs per-fixture cold builds, direct
dedicated targets resolve to real tests, and no recursive test-authority path remains.
Exact timing is observation, but unbounded or unexplained iteration cost blocks
closure.

## C.2: Historical Physical Reality Audit (Retired)

The one-time C.2 audit identified the original reconstruction boundary. Its
inventories, manual trace contract, probes, dispositions, and closeout gate are
retired. Git history is their archive.

Current phases inspect their own causal production paths and rely on direct
compilation, focused owner and boundary tests, process scenarios, and repository
gates. C.2 imposes no current deliverable, report, inventory, or validity state.

## C.3: Sealed Physical Runtime Authority And Lifecycle

Engineering spec:
[physical-reconstruction-c3-sealed-runtime-lifecycle.md](physical-reconstruction-c3-sealed-runtime-lifecycle.md)

### Goal

Establish one non-forgeable, non-duplicable physical runtime composition root
whose phase types and observation handles express the authority currently
installed, and whose consuming transition boundary is the only place later
physical owners may enter.

### Boundary

This milestone seals construction and lifecycle. It does not yet claim durable
page publication. `AdmittedPhysicalRuntime` exposes no physical operation or
mutation method. Absent owners are represented by absent methods/types and
immutable capability status, never by heap-backed implementations returning
`Unavailable`.

### Must Ship

- one Store-owned `admit` path consuming a declared store root, nested C.3
  runtime configuration, and concrete platform authority; C.4 owns real
  backend-capability and namespace admission
- a non-`Clone` composition authority with private construction and explicit
  close/abort/crashed lifecycle
- independently borrowable or cloneable read-only handles only where cloning
  does not duplicate mutation, publication, recovery, or allocation authority
- lifecycle and observation capabilities shaped so a future composition root
  can hold Query/Relational authority independently; C.3 owns neither a
  semantic writer token nor a whole-Store borrow that future reads and
  unrelated physical submissions must share
- exhaustive ownership of only the root-admission, lifecycle, observation,
  managed-resource, capability-status, and diagnostic responsibilities that
  C.3 actually installs
- no media, page, record, WAL, checkpoint, buffer, recovery, integrity,
  scheduling, layout, or blob owner field/view until its real physical
  transition is implemented
- no public mutation handle or generic physical subsystem access from the
  admitted phase
- later milestones must consume the admitted runtime into a distinct concrete
  owner phase; no flag, optional field, trait object, feature, or runtime check
  may unlock physical methods on `AdmittedPhysicalRuntime`
- typestate or equivalent sealed transitions preventing open, recover,
  current-serving, maintenance, and closed states from being confused
- exhaustive construction and lifecycle propagation so adding an installed
  owner breaks every incomplete admit, abort, close, and inspection site; later
  open/recover transitions must consume this same exhaustive root
- removal or quarantine of `PhysicalStoreRuntime` cloning, public replay-based
  reopen authority, and caller-provided persisted-layout construction
- explicit phase-scoped observation handles that cannot mutate or publish
- a narrow production facade that exposes no I/O, recovery, or maintenance
  operation until the phase type carrying that real boundary exists

### Non-Fake Acceptance Setup

- **Production subject:** the sole physical runtime facade, exercised through
  two production-facade lifecycle journeys and one consolidated external-
  consumer compiler boundary.
- **Initial world:** one absent root and one existing empty directory; no
  runtime, replay artifact, persisted layout, page, WAL, or manifest is
  preconstructed.
- **Execution:** one ordinary journey performs admission, observation,
  capability-absence inspection, close, and re-admission. One deterministic
  hostile journey races admission and access while injecting panic, abort,
  cancellation, unexpected drop, stale handles, and child-process death.
- **Independent observation:** one lifecycle table predicts both journeys;
  exact counters reconcile with their action traces; OS inspection proves zero
  physical residue; the small cache-sharing UI suite proves only the authority
  violations that cannot execute at runtime.
- **Assertions:** one composition root exists per incarnation, invalid lifecycle
  use is uncallable, observation cannot promote, physical methods do not exist
  on the admitted phase, no physical owner is allocated, and termination
  outcomes remain distinct.
- **Controlled defects:** duplicate authority, heap-backed physical-owner
  installation during admission, and false successful close must fail the
  compiler boundary, ordinary journey, and hostile journey respectively.
- **Forbidden substitutes:** a private field with a public cloning wrapper,
  runtime assertions after illegal calls, generic marker authority, a generic
  memory/backend parameter on the admitted phase, an `Unavailable` wrapper
  containing a heap owner, physical-looking methods that only return
  `Unavailable`, or certification-only construction standing in for ordinary
  open cannot close the milestone.

### Closeout Gate

`C.3` closes only when the compiler enforces one physical authority lifecycle,
callers cannot supply recovered truth or call physical work, no absent owner is
represented by a heap substitute, and subsequent milestones have one
unambiguous runtime into which real mechanisms must bind.

## C.4: Production Media Boundary And Stable Store Namespace

Engineering spec:
[physical-reconstruction-c4-production-media-boundary.md](physical-reconstruction-c4-production-media-boundary.md)

### Goal

Create the single production I/O boundary through which the physical runtime
owns files, directories, offsets, synchronization, publication, and backend
capability truth.

### Boundary

This milestone establishes media mechanics and namespace law. It does not yet
assign page, WAL, checkpoint, index, or blob semantics to every file. Those
artifact owners consume this boundary in later milestones. `worth-proof`
supplies checked capability and C.3-to-C.4 progression topology without owning
the runtime or OS effects. `worth-foundational` supplies canonical namespace
and portable evidence vocabulary only after stronger Store-owned facts exist.

### Must Ship

- one consuming `AdmittedPhysicalRuntime -> MediaOwnedPhysicalRuntime`
  transition whose output contains the concrete admitted filesystem owner;
  memory/mock backends cannot construct this production phase
- a sealed production media port covering open/create, positioned read/write,
  append, truncate, allocate, flush, file sync, directory sync, atomic rename,
  list, metadata, delete, and declared optional mmap/direct-I/O operations
- one real filesystem implementation with explicit Windows and supported POSIX
  semantics, error topology, and capability admission
- a stable store-root namespace with versioned identity, lock/ownership file,
  directory roles, temporary/staged/publication names, and no ambiguous residue
  discovery
- process-level exclusive ownership of the physical store root or an explicitly
  stronger admitted multi-process media design; this root/media exclusion is
  not branch-write authority and must still permit independent in-process
  physical submissions through later Store capabilities; read-only and offline
  access posture is separate
- storage-boundary operation context sufficient for fault scheduling and exact
  counters without giving the harness mutation authority above I/O
- durable create, replace, rename, and directory-publication protocols tied to
  backend capability evidence
- typed short-write, partial-read, ENOSPC, permission, stale-handle,
  unsupported-sync, and indeterminate-publication outcomes
- proof-backed root/profile capability qualification and consuming runtime
  admission that preserve denied, deferred, stale, rebind-required, and failed
  postures without creating a second qualification authority lane
- Store-owned stable identity plus an explicit Foundational canonical/boundary
  lowering for independent comparison; digests and boundary artifacts cannot
  open a root or promote a runtime
- migration of existing file-writing primitives onto this boundary or an
  explicit deletion decision where they duplicate it

### Non-Fake Acceptance Setup

- **Production subject:** the sealed runtime opening the real filesystem media
  implementation; the harness may wrap only the media port.
- **Initial world:** an absent store root on a real temporary filesystem. A
  second process is prepared to contend for mutation ownership. Backend and OS
  profiles are recorded.
- **Execution:** open/create the namespace, append and replace known framed
  bytes, force file and directory durability, attempt concurrent ownership,
  terminate the writer, and inspect from a fresh process.
- **Independent observation:** the fresh process uses OS file APIs and stable
  format declarations, not runtime caches or writer-returned byte buffers.
- **Assertions:** exact path set, file lengths, offsets, bytes, sync sequence,
  ownership denial, cleanup posture, zero writes outside the admitted root, and
  absence of `MediaOwnedPhysicalRuntime` before successful real namespace
  admission are checked. Short write, sync failure, and rename interruption
  localize to typed media outcomes. Capability and runtime progression preserve
  their exact proof-outcome categories, while runtime and independent-observer
  namespace meaning compares through the declared canonical basis without
  replacing direct OS observation.
- **Controlled defect:** report sync completion without invoking the backend
  barrier, or allow path escape beyond the root. The durability-sequence or
  namespace-confinement predicate must fail.
- **Forbidden substitutes:** `std::fs::write` in the test, a memory backend,
  inspecting a buffer returned by the writer, or declaring rename durable
  without namespace synchronization cannot close this milestone.

### Closeout Gate

`C.4` closes only when the canonical runtime owns a real store namespace, all
ordinary physical effects cross one fault-interposable production boundary,
backend capability claims are tied to observed media behavior, and only that
real admission can construct the media-owned runtime phase. A copied proof,
stale basis, profile report, identity projection, digest, or Foundational
artifact must open no capability or runtime door.

## C.5: Durable Page, Segment, Extent, And Manifest Path

Engineering spec:
[physical-reconstruction-c5-durable-physical-record-path.md](physical-reconstruction-c5-durable-physical-record-path.md)

### Goal

Replace the heap-shaped physical-format runtime with ordinary record append,
locate, scan, and reopen over real page, segment, extent, and manifest files.

### Boundary

Physical format continues to own byte grammar and verification. Media owns
effects. The runtime composes them. `PersistedPhysicalLayout` may remain a
bounded offline/test representation where honestly named, but it is not the
production store, reopen input, or whole-store transport.

### Must Ship

- distinct consuming initialize and open transitions from
  `MediaOwnedPhysicalRuntime -> ServingPhysicalRuntime`; initialize requires a
  proven-absent record family, open requires the authoritative catalog and never
  creates, and no heap layout or replay representation can construct either
- file-backed page, segment, extent, root-manifest, segment-manifest,
  extent-manifest, and free-space structures
- bounded bootstrap catalog sufficient to find current roots without loading
  every page or extent
- stable physical record ids distinct from private root-scoped placements,
  external Store-id-plus-record-id readmission, generation reuse detection,
  framed records, non-rebasing slot directories, allocation classes, and
  append/locate/scan operations over the real media port
- one copy-on-write publication path with exact data-sync, manifest-sync,
  catalog-candidate-sync, atomic replacement, and namespace-sync ordering;
  each batch is all-or-none visible
- separate Store-wide format declaration, evolvable append placement policy,
  and per-open access policy
- mandatory C.5 checksum coverage for bootstrap, manifest, page, and extent
  authority frames; comprehensive diagnosis and repair remain C.9
- bounded readers and writers that operate on ranges or frames rather than
  copying the complete store into `Vec` collections
- versioned format compatibility and typed unsupported-version outcomes
- explicit removal of production `persisted_layout()` round trips and replay-
  artifact reopen
- exact page, byte, allocation, manifest, record-id, copy, and scan counters
- bounded physical read and mutation surfaces whose borrows can be held
  independently once C.5.1 installs submission authority; a record operation
  may identify physical artifacts and ranges but cannot accept branch identity
  or semantic writer authority
- a typed handoff to C.6 naming `FrameLoadPort`, `CandidateFrameSet`, and
  `CandidateFramePublicationPort`, while Store retains root publication

### Non-Fake Acceptance Setup

- **Production subject:** canonical runtime append, locate, scan, close, and
  reopen operations backed by the C.4 filesystem media port.
- **Initial world:** absent root initialized explicitly, small page size forcing
  multiple pages, at least two segments, one extent-backed record, and a memory
  budget too small to hold all persisted page bytes at once.
- **Execution:** append deterministic records through the facade, publish
  manifests, close, discard the process, and reopen in a fresh executable using
  only root plus format expectation and access policy. Locate records in
  non-write order and perform a bounded scan.
- **Independent observation:** an offline format walker parses the directory
  and emits page/segment/extent/record-id-to-placement topology without
  constructing the runtime.
- **Assertions:** record parity, stable record ids, exact private placements and
  generations, exact artifact topology, bounded bytes read, no full layout
  construction, and zero backend-residue guessing. Reopening with one stale
  manifest and one unsupported version fails at the declared boundary.
- **Controlled defect:** make reopen initialize, use placement as identity,
  expose a partial batch, accept a stale generation, or load all pages into a
  vector before locate. Lifecycle, identity, atomicity, generation, or
  materialization predicates must fail independently.
- **Forbidden substitutes:** writer-supplied `PersistedPhysicalLayout`, replay
  artifacts, a test-generated manifest, or proof against a memory-backed
  `PhysicalStoreRuntimeStorage` cannot close this milestone.

### Closeout Gate

`C.5` closes only when physical records survive a real process boundary and
fresh reopen discovers them from bounded on-disk roots without complete heap
materialization or caller-supplied persisted state, and only that path can
construct the record/page-serving runtime phase.

## C.5.1: Physical Store Instance And Signal-Orchestrated Work

Engineering spec target:
`physical-reconstruction-c5-1-physical-store-work-runtime.md`

### Goal

Turn the C.3 through C.5 serving runtime into one cohesive physical Store
instance whose specialized physical authority remains the only source of
operation truth while one private Worth Signal graph owns dependency readiness,
async-resource lifecycle, and completion causality for physical work.

### Boundary

This milestone establishes the physical runtime topology that C.6 through C.11
must enter. It does not import Query, Relational, or Runtime Bridge and does not
replace physical owners with Signal nodes. Physical authority admits,
dispatches, settles, poisons, and closes operations. Signal derives which
declared work is ready and retains async completion posture. The I/O scheduler
admits resources. The executor alone invokes the C.4 media boundary. WAL,
buffer-pool, isolation, integrity, index, and blob owners remain specialized
mechanisms installed by later milestones through the same topology.

Physical semantic boundaries remain aspect-native. `worth-foundational` and
`worth-store-aspect-native` own aspect identity, contracts, validated values,
authoritative state, patches, and the distinct projection, mutation, and
diagnostic mask laws. Signal `Aspect` and `AspectMask` values are derived,
bounded dependency-routing coordinates only; Store owns the admitted binding
from physical aspect meaning to those coordinates. JSON is not an internal
work, dependency, scheduling, settlement, observation, or evidence carrier.
It is permitted only at an explicitly named external compatibility ingress or
terminal projection edge, never as authority.

C.5.1 is intentionally branch- and MVCC-agnostic. It supplies the lower half
of future multi-branch execution: independently borrowable, incarnation-fenced
physical submission capabilities; exact artifact, range, security, durability,
and coordination scopes; and terminal effect fate. It must not install a
Store-owned branch writer table, branch-labelled queue, semantic generation
lease, or global mutation lock. The future adapter correlates a physical result
with its semantic mutation, and Relational alone advances, releases, or fences
active branch-write authority.

### Must Ship

- one Store-owned physical instance composition with exhaustive construction,
  close, abort, unexpected-drop, and inspection propagation for physical
  authority, the physical Signal owner, scheduler admission, executor access,
  serving health, and managed work
- independently borrowable physical read and mutation-submission capabilities
  whose retained handles are fenced by Store incarnation and capability
  generation without duplicating settlement, publication, recovery, or media
  authority
- Store-owned physical operation identities and exact physical scopes carried
  through declaration, readiness, scheduling, dispatch, backend effect,
  settlement, observation, and recovery disposition; upstream correlation is
  metadata only and cannot widen scope or grant concurrency
- one audited binding from Store physical-work declarations into Worth Signal's
  existing async-node and resource request lifecycle; Store-specific identity,
  generation, artifact/security scope, effect class, settlement owner, and
  recovery disposition remain in sealed physical authority instead of being
  copied into a parallel Signal-like envelope
- one consuming composition of Signal request admission/readiness, scheduler
  resource admission, executor dispatch, exact effect outcome, and physical
  settlement; raw ids, digests, counters, callbacks, generic Signal completion,
  or Store-local lifecycle lookalikes cannot skip a phase
- one private physical Signal graph whose nodes model only derived work and
  async completion, whose evaluators perform no filesystem effects, and whose
  entire state may be discarded and rebuilt from current physical authority
- one frozen Store-owned binding from admitted Foundational/Store-native aspect
  contracts and physical witnesses into bounded Signal aspect slots and
  partitions; raw Signal slots cannot define meaning, Foundational mask modes
  cannot collapse into one generic mask, and internal JSON cannot carry the
  semantic basis
- one lowering boundary from ready Signal work into scheduler demand and from
  admitted scheduler demand into a concrete executor command; the scheduler
  may shape fairness, priority, queue depth, byte budgets, and locality but may
  not rediscover dependencies or settle physical truth
- exact cancellation, supersession, timeout, late-completion, stale-generation,
  partial-effect, indeterminate-effect, and health-revocation behavior; once an
  effect may have begun, consumer cancellation cannot erase its settlement
- terminal settlement evidence that distinguishes proven no effect, completed
  effect, and indeterminate effect so Part II can correlate fate without Store
  deciding semantic publication or branch-writer progression
- reuse of Worth Signal's existing resource cancellation, retry, revalidation,
  supersession, timeout, completion, diagnostics, and retention laws wherever
  their semantics match; a missing domain-neutral capability is added once to
  Signal, while physical effect fate remains Store-owned
- one lifecycle close protocol that stops admission, cancels work still safe to
  cancel, settles or terminally classifies dispatched work, drains managed
  completions, disposes the derived Signal graph, and only then releases media
  authority
- migration of the C.5 append, locate, scan, publication, and bounded-read
  journeys through the canonical work topology wherever they cross an async,
  scheduling, dependency, or physical-effect boundary; cheap local decode and
  pure format work remain direct and do not receive ceremonial Signal wrappers
- exact counters for declared, ready, resource-denied, queued, dispatched,
  completed, partially effected, indeterminate, cancelled-before-dispatch,
  cancelled-after-dispatch, superseded, stale-completion, poisoned, drained,
  and residual work
- hard dependency and visibility gates proving lower physical mechanism crates
  cannot import Signal, raw media/backend effects cannot bypass the Store
  executor, and external consumers cannot construct Signal completions or
  physical settlement receipts
- source and type gates proving Store has no branch-head, MVCC, semantic writer,
  or branch-labelled concurrency vocabulary and proving two independently held
  mutation capabilities can submit physically disjoint work concurrently
- deletion or non-production quarantine of duplicate physical runtimes,
  background envelopes, async registries, pending maps, callback effect routes,
  polling loops, queue lifecycle state machines, certification-only owners, and
  legacy residency models that would compete with the canonical instance
- a narrow C.6 handoff exposing the real physical work, scheduler-demand,
  frame-load, candidate-publication, and settlement contracts without
  predeclaring fake buffer-pool, WAL, or maintenance owners

### Strongly Preferred Implementation Shape

C.5.1 is a composition correction, not a crate-expansion milestone. Production
composition should remain in the existing `worth-store` owner, reuse
`worth-signal` and `worth-store-io-scheduler` through their public contracts,
and reach the existing physical backend only through the sealed executor. A new
general-purpose runtime, async, work-registry, or scheduler crate requires an
explicit proof that no present owner can hold the responsibility; convenience
is not sufficient.

The rough responsibility target inside the Store owner is:

```text
physical_runtime/
  instance/            construction, generation, health, close, abort
  work/
    declaration/       Store-owned physical identity and effect contract
    signal_binding/    physical dependencies to Worth Signal resources
    scheduling/        ready resource to existing scheduler demand
    execution/         sole lowering into backend effects
    settlement/        exact outcome to physical truth, then derived completion
    cancellation/      pre-dispatch stop and post-dispatch continuation law
  observation/         bounded snapshots and causal counters only
```

This is a responsibility sketch, not a requirement to create every directory.
Prefer fewer modules when one named file remains cohesive. Do not retain old
paths as compatibility wrappers: each migrated production path is deleted in
the same slice, together with obsolete features and dependencies.

### Non-Fake Acceptance Setup

- **Production subject:** the one Store-owned physical instance running C.5
  record operations through its physical Signal, scheduler, executor, and C.4
  media path where those boundaries apply.
- **Initial world:** a real C.5 store with multiple pages and segments; bounded
  scheduler capacity; two independently held mutation-submission capabilities;
  disjoint reads; physically disjoint appends; one root-publication conflict;
  one delayed effect; one cancellable pre-dispatch request; and named effect-
  boundary yieldpoints. The workload uses opaque upstream correlations, never
  branch labels as scheduling input.
- **Execution:** submit work concurrently, reorder completions, deny one resource
  request, cancel before and after dispatch, inject one short/partial effect,
  attempt close with work in every lifecycle state, terminate the process, and
  reopen from the last physically published C.5 root without carrying Signal or
  scheduler state across the boundary.
- **Independent observation:** OS/media observation records actual calls and
  bytes; the C.5 offline walker records the published physical world; a separate
  action trace predicts the allowed work transitions. None receives private
  Signal state or settlement constructors.
- **Assertions:** exactly one effect occurs per dispatched identity; no denied
  or safely cancelled request reaches media; late completion cannot settle a
  newer generation; partial effects revoke the declared health boundary;
  physically disjoint submissions can progress concurrently; shared root work
  coordinates only at its declared physical scope; no whole-Store mutation
  borrow serializes unrelated work; close leaves no unclassified managed work;
  and fresh reopen reconstructs work from physical truth rather than a
  serialized Signal graph.
- **Controlled defects:** perform a filesystem effect inside Signal evaluation,
  settle from a scheduler counter, dispatch through a raw backend handle, and
  retain a duplicate pending registry. Effect-boundary, authority, dependency,
  or residue predicates must fail independently.
- **Forbidden substitutes:** a second async runtime, a second I/O scheduler,
  callback-owned settlement, mock-only resource completion, serialized Signal
  state used as recovery authority, raw Signal aspect slots used as semantic
  identity, collapsed Foundational mask modes, internal JSON control/semantic
  envelopes, a Store-owned branch writer registry, a branch-labelled physical
  queue, a global physical submission lock, or source scans without production-
  path execution cannot close this milestone.

### Closeout Gate

`C.5.1` closes only when one physical Store instance owns every current
physical work lifecycle, Signal is demonstrably derived and effect-free, the
scheduler is only resource authority, exact backend outcomes are the only route
to settlement, shutdown leaves no unclassified work, lower mechanisms remain
Signal-agnostic, independent physical submission capabilities preserve
disjoint concurrency without semantic branch knowledge, and every duplicate
production authority or dead competing path has been deleted or mechanically
quarantined.

## C.6: Buffer Pool And Bounded Physical Access Join

Engineering spec:
[physical-reconstruction-c6-buffer-pool-runtime-join.md](physical-reconstruction-c6-buffer-pool-runtime-join.md)

### Goal

Make the C.5 file-backed store operate through the bounded buffer pool inside
the C.5.1 physical work topology for ordinary reads and writes, with leases,
dirty state, eviction, and memory admission governing real frames rather than
parallel test models.

### Boundary

The buffer pool owns residency and frame lifecycle. The C.5.1 physical Signal
graph derives fault and writeback readiness, its scheduler admits the required
memory and I/O resources, and its executor reaches the media port. Physical
format owns decoding and the media port owns effects. None of those layers may
settle another layer's authority. Stable semantic MVCC and Query residency
remain outside Part I.

Store uses `worth-proof` for residency-policy admission and retains the
admitted policy at the physical-instance boundary. The lower pool imports
neither `worth-proof` nor Signal; it reports physical state while Store's
existing Signal path carries effectful work. Physical pressure is not a
Foundational fact. C.6 Phase 5 introduces the dedicated Foundational
frame-writeback basis above the pool boundary.

### Must Ship

- physical-instance-owned buffer-pool construction through the C.5.1 lifecycle
  with hard resident, pinned, dirty, prefetch, writeback, and operation-
  allocation budgets
- page fault, pin, lease, decode, dirty transition, writeback, eviction, and
  stale-generation flows against C.5 files
- zero-copy or explicitly bounded-copy record views with lifetime tied to a
  frame lease
- rejection or backpressure before a request can exceed memory, pin, or dirty
  limits
- separation of foreground, recovery, scrub, maintenance, verifier, and blob
  allocation scopes
- page faults, prefetch/read-ahead, dirty publication, and write-behind lowered
  into the existing C.5.1 work declaration, Signal-readiness, scheduler-demand,
  executor, and settlement progression without a second queue or pending map
- exact resident-byte, allocation, pin, dirty, fault, hit, eviction, copy,
  writeback, and denial counters
- compile-time prevention of record views outliving their frame authority
- adapter-consumable bounded physical read leases and pressure evidence that
  reveal basis, generation, bytes, and retry posture without treating cached
  frames as semantic residency or allowing Part II to reach into pool internals

### Non-Fake Acceptance Setup

- **Production subject:** the C.5 runtime read/write facade with its real
  buffer-pool and filesystem path.
- **Initial world:** fresh-process reopen of a store at least eight times larger
  than the configured resident-byte budget; deterministic hot, cold, sequential,
  and pinned access sets.
- **Execution:** interleave reads, appends, forced evictions, dirty-page
  pressure, prefetch, and a denied over-pin request while continuously sampling
  admitted memory at the allocation boundary.
- **Independent observation:** process-level allocation instrumentation and
  media-port counters are compared to buffer-pool receipts. Final data is
  checked after a second fresh-process reopen.
- **Assertions:** peak admitted bytes never exceed the hard budget; exact pins
  and dirty pages respect limits; cold reads cause real file I/O; hot reads do
  not; evicted records re-fault correctly; forbidden whole-store allocation and
  stale record views fail.
- **Controlled defect:** hide one complete-store copy behind a fixture helper or
  permit eviction of a pinned frame. Allocation or lease predicates must fail
  at the causal boundary.
- **Forbidden substitutes:** a synthetic frame table not used by the C.5
  runtime, final RSS alone, data smaller than the memory budget, or nonzero-only
  counters cannot close this milestone.

### Closeout Gate

`C.6` closes only when all ordinary file-backed record access is mediated by
bounded residency, stores materially larger than memory remain operational,
frame-lifetime and budget claims are mechanically falsifiable, and the bounded
read surface can be consumed without transferring pool or semantic-residency
authority.

### Current Contract And Successor Links

Phase 10 closes C.6 on one Store-owned bounded-residency contract with the
parallel S.2 authority graph removed. Current validity comes from the checked-out
code, direct Cargo tests, and repository gates; historical campaign reports and
mutation catalogs are retained only in Git history. Current developer guidance
and completed contracts:

- [C.6 engineering specification](./physical-reconstruction-c6-buffer-pool-runtime-join.md)
- [bounded physical record access guide](./bounded-physical-record-access.md)
- [buffer-pool owner README](../../workspaces/worth-store/crates/worth-store-buffer-pool/README.md)

C.6 hands bounded physical truth forward through these exact successor
boundaries. These are insertion contracts, not empty successor modules or
permission to publish C.6 internals early.

| Successor | C.6 authority it may consume | Authority it must not acquire | Current insertion or adapter boundary |
| --- | --- | --- | --- |
| [C.7 durability ordering](#c7-wal-checkpoint-root-publication-and-acknowledgment-join) | Store-composed dirty/writeback typestate and `PhysicalWritebackSettlement` identity, effect fate, recovery disposition, and Signal settlement | pool construction, frame-table or clean-transition control, or a durability decision inferred from dirty state; C.7 supplies WAL and durability policy | `record_serving/residency/dirty/`; the Store-owned `FrameWritebackPort` remains private and the successor typestate is not ordinarily exported before C.7 installs its adapter |
| [C.8 fresh-process recovery](#c8-fresh-process-recovery-and-reopen) | `RecoveryPhysicalAllocation`, runtime/generation binding, and public physical effect-fate and recovery-disposition evidence | live pool contents, frame-table constructors, eviction control, or a hidden unlimited reconstruction mode | `worth-store-recovery-physics::RecoveryMemoryAllocation` consumes the runtime-borrowed Recovery allocation; reconstruction must enter through a future Recovery-owned port |
| [C.9 physical integrity](#c9-physical-integrity-corruption-localization-and-offline-truth) | frame-owned exact-source admission and Verification/Scrub-scoped allocation | pool, pin, eviction, frame-loading, or integrity authority inferred from unverified bytes | Store's private resident admission binds validated results to the C.6 frame lifetime; scrub emits descriptive observations only |
| [C.10 stable reads and scheduled I/O](#c10-stable-reads-scheduled-io-and-maintenance-interference) | lease-bound chunks, `PhysicalRecordPressureEvidence`, Maintenance-scoped allocation, and Store-composed scheduler-ready physical work | semantic MVCC, Query authority, direct scheduler ownership, or a second scheduler | `worth_store::physical_runtime::stability::PhysicalByteGuard::from_record_chunk` owns the live lease adapter; lower isolation retains local plan completion, and future scheduling enters through the Store composition boundary |
| [C.11 layout, indexes, and native blobs](#c11-layout-index-and-native-blob-adoption) | `BlobPhysicalAllocation` and bounded `RecordReadSession` chunk streaming with Store/runtime/generation evidence | whole-blob materialization, pool control, or semantic liveness inferred from physical residency | `worth-store-blob-chunks::streaming` consumes exact Blob allocations and bounded source/observation streams; whole-object vector substitutes remain explicit denials |

## C.7: WAL, Checkpoint, Root Publication, And Acknowledgment Join

Engineering spec:
[physical-reconstruction-c7-durable-publication-join.md](physical-reconstruction-c7-durable-publication-join.md)

Current caller and operator contract:
[physical-durability-and-checkpoints.md](physical-durability-and-checkpoints.md)

The public durability contract and focused C.7 tests are the current cutover
authority. Historical closeout details remain available in Git history.

### Goal

Join the existing WAL, durability-ordering, checkpoint, pageLSN, manifest, and
root-publication mechanisms into one canonical durable write progression
executed through the C.5.1 physical work topology.

### Boundary

This milestone owns physical transaction ordering, not semantic MVCC or Query
acknowledgment. WAL and publication owners decide ordering and durability;
physical Signal derives dependency readiness, the scheduler admits resources,
and the executor performs the exact effects. A physical write acknowledgment
means exactly the declared physical durability boundary was reached under the
admitted backend profile. It does not serialize semantic mutations, advance a
branch head, release a branch writer, or confer shared global semantic
authority.

### Must Ship

- one sealed progression from admitted physical batch through WAL append,
  required WAL barrier, page/extent mutation, pageLSN assignment, checkpoint or
  root publication, namespace durability, and physical acknowledgment
- canonical WAL frame and transaction identities bound to exact physical
  effects and idempotency keys
- Store-issued durable-generation idempotency leases, bounded checkpoint-side
  attempt-binding compaction, expired-key denial, and proof that WAL reclamation
  never deletes the last authoritative live binding
- preservation of the C.5.1 physical operation identity and exact submitted
  scope across grouping, WAL ordering, root publication, acknowledgment, and
  indeterminate fate; group commit may share barriers but cannot merge caller
  identities or invent semantic commit order
- WAL-before-data enforcement and typed barriers for backend capability tiers
- group-commit-compatible batching without changing individual mutation
  identity or allowing premature acknowledgment
- fuzzy or non-blocking checkpoint capture with a bounded WAL tail and exact
  source range
- bounded page causality carrying one certified prior-page basis plus only the
  newly applied redo delta, never a page's lifetime WAL history
- atomic current-root publication and old-root retention sufficient for C.8
  recovery
- explicit indeterminate physical outcome when failure occurs after possible
  durability but before the caller can observe completion
- a Store-owned mutation handle whose drop abandons observation rather than
  cancelling effects or abandoning settlement, with typed cancellation and
  close-time draining
- sealed physical settlement receipts sufficient for a future adapter to
  correlate proven no effect, completed effect, or indeterminate effect with
  its own mutation while remaining incapable of deciding branch progression
- exact frames, bytes, fsyncs, directory syncs, page writes, pageLSNs,
  checkpoints, root swaps, grouped mutations, and acknowledgment counters
- dependency edges for WAL-before-data, barriers, checkpoints, and root
  publication expressed through the C.5.1 derived work graph without moving
  ordering authority into Signal evaluation
- elimination of isolated WAL/file durability demonstrations that do not feed
  the canonical runtime progression

### Non-Fake Acceptance Setup

- **Production subject:** canonical runtime batch append and physical
  acknowledgment facade using real C.4 media, C.5 artifacts, and C.6 residency.
- **Initial world:** one real store with a published root, an admitted durable
  backend profile, four requests of which one cancels before group sealing and
  three share a group-commit opportunity, and named yieldpoints before and
  after every persistence boundary.
- **Execution:** run the control batch, then independently produce a fresh Store
  root from the same deterministic workload seed for each crash seam: before
  WAL append, during WAL append, after WAL write/before sync, after WAL
  sync/before data, during data write, after data/before root publication,
  after root publication/before directory sync, and after durability/before
  observed acknowledgment. Do not copy a post-run Store, workspace, archive,
  serialized runtime, cache, or writer memory between seams.
- **Independent observation:** capture files after abrupt process termination;
  a raw WAL/page inspector records durable prefixes, pageLSNs, roots, and sync
  evidence. C.8 will later decide full recovery, but C.7 must already prove
  impossible acknowledgment states.
- **Assertions:** no acknowledged batch lacks its required barriers; no data
  page outruns its WAL basis; torn tails are distinguishable; group commit
  preserves all four request identities, allocates WAL/data effects only to the
  three sealed members, and does not propagate the pre-seal cancellation; exact
  barrier counters match the injected seam.
- **Controlled defect:** acknowledge before WAL sync and separately write a
  pageLSN ahead of durable WAL. The acknowledgment and ordering predicates must
  fail at distinct locations.
- **Forbidden substitutes:** an in-memory WAL, simulated durability receipt,
  calling the file durability executor without the canonical runtime, or a
  crash represented by `Err` while the process survives cannot close this
  milestone.

### Closeout Gate

`C.7` closes only when one real production write progression owns every
physical durability edge and no acknowledgment can be manufactured without
the exact file, directory, ordering, and root-publication effects required by
its backend profile. Idempotency authority must remain bounded without allowing
expired-key reuse or reclaiming the last live binding. Its receipts must remain
physical facts: no C.7 type or transition may advance a branch, release a
semantic writer, or impose semantic serialization across operations that
merely share a durability barrier.

## C.8: Fresh-Process Recovery And Reopen

C.8 is governed by the
[standalone fresh-process recovery specification](physical-reconstruction-c8-fresh-process-recovery-and-reopen.md).
It starts from the bounded crash-recovery contract described by the
[C.7 successor handoff](physical-reconstruction-c7-durable-publication-join.md#c8-successor-handoff),
not from that in-memory closeout value or a live runtime. The caller-facing
boundary and current exclusions are summarized in the
[C.7 current limits and C.8 handoff](physical-durability-and-checkpoints.md#current-limits-and-c8-handoff).

### Goal

Recover the canonical physical runtime deterministically from checkpoint,
WAL tail, pages, and manifests after real process death.

### Boundary

Recovery decides physical source precedence and reconstructs physical current
state. It does not readmit Query, Relational, Signal, or Bridge authority. It
may produce typed physical truth and handoff evidence for later Part II
readmission. Recovery resolves physical operation fate by Store identity; it
does not know which branch submitted the work or whether Relational may admit a
successor writer.

Worth Proof supplies only private up-front contract substrate beneath concrete
Store types: sealed lane markers, exact per-axis bindings, one-terminal session
law, owner-sampled generation freshness, and performed-effect evidence. A bare
witness, binding match, freshness carrier, terminal receipt, or performed value
opens no C.8 public door. Foundational versions the recovery and independent
observer report protocols for compatible cross-process interpretation; those
descriptive protocols never admit persisted Store formats or recovery truth.

### Must Ship

- bounded bootstrap from current checkpoint plus WAL tail, never history from
  genesis or a supplied heap layout
- deterministic recovery-source precedence across current and previous roots,
  checkpoints, WAL segments, partial pages, compaction products, and residue
- torn-tail rejection, idempotent redo, pageLSN comparison, incomplete
  publication handling, and closed-work quiescence
- exact resolution of `AcknowledgedDurable`, `ProvenNoEffect`,
  `DurableUnacknowledged`, and `Indeterminate` physical operation fates;
  `ProvenNoEffect` requires positive persisted no-effect evidence
- recovery reconciliation indexed by stable physical operation identity and
  Store incarnation, returning the weakest exact fate rather than promoting an
  unresolved effect to success or no-effect
- a separate reconstruction-band composition facade; recovery physics remains
  pure selection, redo, fate, and cost law and ordinary Store crates do not
  import replay
- concrete sealed recovery authority with private exact bindings for root,
  Store, session, backend profile, media generation, configuration, and budget,
  including one-axis-at-a-time drift proof
- one owner-tracked linear recovery session reaching exactly one top-level
  terminal outcome
- owner-sampled checkpoint-generation freshness for retained idempotency
  bindings and current-root-generation freshness for cleanup retry
- concrete Store evidence recorded only after each C.4 staging, publication,
  namespace, reopen, and cleanup effect completes
- new runtime identity and fresh handles after every recovery
- recovery time/work bounded by checkpoint interval, tail, and damaged scope,
  not total store size
- post-recovery checkpoint/root cleanup policy that preserves evidence until
  current truth is namespace durable and independently reopened, with cleanup
  failure reported as deferred maintenance rather than recovery rollback
- a narrow recovery handoff exposing verified physical roots, bounded access
  capabilities, reconciled physical operation fates, and unsupported or
  quarantined scope; no semantic checkpoint, branch head, or writer lease is
  reconstructed here
- exact bootstrap bytes, manifests, WAL frames, page redo, skipped redo,
  rejected tails, residue denials, and recovery allocation counters
- version-1 `store.physical.recovery-report` and
  `store.physical.recovery-observer-report` families with exact consumer
  windows, typed incompatibility, and no Store authority

### Non-Fake Acceptance Setup

- **Production subject:** a dedicated writer executable and a distinct recovery
  executable both using the canonical runtime facade.
- **Initial world:** deterministic operations with periodic checkpoints over a
  store larger than the memory budget. The harness knows expected semantic
  record values from its independent history model.
- **Execution:** the parent harness starts the writer, waits for a named
  production yieldpoint, terminates it without cleanup, records writer process
  identity, then launches the recovery executable with only root,
  configuration, backend profile, physical recovery limits, and an
  output-evidence path. Concrete platform authority is minted inside the
  recovery process and is never a launch input.
- **Independent observation:** a third offline process parses persisted
  artifacts without constructing the runtime. Neither recovery nor verifier
  receives writer heap objects, replay artifacts, decoded values, or expected
  truth.
- **Assertions:** recovered record/model parity, new process and runtime
  identities, deterministic classification for identical bytes, exact
  acknowledged/indeterminate outcomes, bounded tail work, and disagreement
  visibility between runtime and verifier. Every submitted physical identity
  resolves to exact fate or remains explicitly indeterminate; no branch or
  semantic-writer state is reconstructed. Bare lane witnesses cannot cross
  recovery bindings, freshness cannot be caller-sampled, admitted work cannot
  stand in for performed effects, the recovery session has one terminal
  outcome, and report protocol admission cannot promote Store bytes.
- **Controlled defect:** preserve one writer registry across restart or ignore
  pageLSN during redo. Crash-isolation or redo predicates must fail and
  localize separately.
- **Forbidden substitutes:** same-process recovery, graceful close, replay-
  artifact reconstruction, copied `PersistedPhysicalLayout`, global singleton
  state, or calling runtime decode from the offline verifier cannot close this
  milestone.

### Closeout Gate

`C.8` closes only when a killed writer can be replaced by a genuinely fresh
process that reconstructs one deterministic physical truth from persisted
authority alone inside declared memory and recovery-work bounds, and exposes
exact physical reconciliation without making a semantic readmission decision.
Every governed transition must also retain its exact dynamic binding, sample
generation freshness through its Store owner, advance past effects only from
concrete performed evidence, terminate the recovery session exactly once, and
keep versioned cross-process reports descriptive.

The C.8 successor value remains `RecoveredPhysicalRuntimeHandoff`. C.9 joins
the C.8 recovery architecture at two seams: integrity ingress before
owner-specific recovery interpretation, and the recovered-runtime handoff for
resident admission, quarantine observation, and scrub. The handoff does not
mint integrity admission. C.10 consumes the handoff to add stable reads,
epochs, reclaim, scheduled I/O, and maintenance interference. Descriptive
recovery and observer reports cannot be promoted into either successor's
authority.

## C.9: Physical Integrity, Corruption Localization, And Offline Truth

The governing [C.9 specification](physical-reconstruction-c9-integrity-and-offline-truth.md)
freezes the current family matrix and implementation boundaries. Caller and
operator behavior is documented in the
[integrity guide](physical-integrity-and-offline-verification.md).

### Goal

Bind checksums, framing, generation validation, quarantine, scrub, and offline
verification to the real C.5 through C.8 artifacts before damaged bytes can
enter logical or recovery interpretation.

### Boundary

Integrity owns physical validity and localization. Recovery owns source
precedence. Artifact owners decide whether damaged derived material can be
rebuilt. The offline verifier is read-only observation and cannot mint current
runtime or repair authority. Integrity reports artifact facts; it does not
decide semantic visibility, branch health, Query readmission, or whether a
logical mutation may be retried.

### Must Ship

- checksum and structural validation on every ordinary page, extent, WAL,
  checkpoint, root/segment/extent manifest, free-space, index, and blob path
  introduced so far
- decode refusal before logical or owner-specific interpretation after physical
  failure
- structurally aware corruption targets and typed localization by artifact,
  field, identity, range, and expected blast radius
- online scrub and independent offline walk over the same stored format through
  distinct authority paths
- quarantine observations separated from reachability mutation and repair
  authorization
- separate validator-outcome, owner-disposition, and quarantine-posture axes
  preserving intact authority, damaged authority, rebuildable derived state,
  quarantined region, unsupported version, unknown, and indeterminate
- verifier/runtime disagreement as explicit evidence rather than hidden
  reconciliation
- descriptive integrity observations stable enough to cross the future adapter boundary:
  exact artifact identity, generation, damaged range, authority/derivation
  class and quarantine posture, without recovery options, mutation authority,
  or semantic policy conclusions
- exact checked, failed, skipped-decode, quarantined, rebuildable, unknown,
  and bytes-read counters

### Non-Fake Acceptance Setup

- **Production subject:** real files produced by C.7 and recovered by C.8;
  corruption is delivered through the C.4 storage interposer at declared write
  seams or by an offline artifact editor only after the process is dead and the
  target field is predeclared.
- **Initial world:** independently recorded clean artifact manifest containing
  every current authoritative family, including pages/extents, WAL, checkpoint,
  root protocol, and free-space metadata. C.9 has no current derived family;
  dormant index/blob formats cannot stand in for one.
- **Execution:** apply checksum, length, generation, pointer, payload, removal,
  duplication, and stale-version operators to isolated copies; run runtime
  reopen and offline verification independently.
- **Independent observation:** expected target and blast radius are fixed before
  either observer runs. The verifier shares stable format declarations only,
  not runtime recovery, cache, normalization, or decision code.
- **Assertions:** exact localization, decode refusal, distinct authority versus
  derived outcomes, deterministic repeated classification, and explicit
  disagreement evidence.
- **Controlled defect:** ignore one checksum and separately make the verifier
  call runtime recovery parsing. Mutation and structural-independence predicates
  must fail.
- **Forbidden substitutes:** arbitrary byte scribbling with `is_err()`, private
  struct mutation, corruption after decode, or the same parser/decision path
  returning the same answer twice cannot close this milestone.

### Closeout Gate

`C.9` closes only when every current physical authority family is rejected or
localized before semantic use when damaged, and an independent read-only
process can disagree with live recovery without sharing its authority path.
Its verdict remains physical evidence for later semantic policy, not that
policy itself.

## C.10: Stable Reads, Scheduled I/O, And Maintenance Interference

Engineering specification:
[Stable reads and I/O coordination](physical-reconstruction-c10-isolation-and-io-coordination.md).
It fixes the production test-case matrix, root-protection/retirement interlock,
exact physical conflict scopes, bounded scheduler service and retained storage,
record-preserving rewrite recovery, destination topology, and parallel-work
start triggers. Its first implementation checkpoint is a protected ordinary
reader surviving real root publication with exact membership and live-registration
proof. Seven phases retain that path: scheduling includes capacity-starvation
prevention; Phase 3 installs retention admission and pending-publication exclusion
before rewrite effects; Phase 4 adds actual reclamation and pressure recovery.
Each effect-bearing phase owns its lifecycle/recovery contract; later phases
complete the combined interference, crash and handoff evidence.

Phase 1's protected-reader checkpoint is implemented: `records()` acquires
bounded, root-indexed protection under the sole publication owner's lock;
descendant sessions retain it through final release. Deterministic capture/
publication races and real cold-read/membership journeys cover that boundary.
Isolation now supplies local planning contracts without Store/Recovery imports
or synthetic publication execution/recovery receipts. Phase 2 next joins exact
effect scopes and bounded class-aware dispatch to the existing scheduler.

GitHub CI currently runs the Rust line-cap check only. C.10's focused,
integration and release/manual test products remain direct implementation and
closure obligations; authoring this spec does not restore automated CI jobs.

### Goal

Join physical leases, epochs, latches, copy-on-write publication, reclaim
barriers, and foreground/background policy into the existing C.5.1 scheduler
and work graph so concurrent maintenance cannot destabilize bytes or hide
unbounded interference.

### Boundary

This is physical isolation and I/O coordination, not semantic MVCC. It evolves
the C.5.1 scheduler's resource policy; it does not create another scheduler,
async lifecycle, completion registry, or effect route. A Store-issued protected
reader retains its selected physical root; a lower local plan or completion is
not live protection or byte-execution evidence. Neither decides which semantic
version a user may observe. Physical concurrency is derived from exact artifact,
range, publication, reclaim, and resource scopes. Branch labels and semantic
writer generations are neither accepted inputs nor substitutes for those
proofs.

### Must Ship

- stable read plans bound to exact roots, generations, physical references,
  security scope, and lease lifetime
- physical work-scope intersection and disjointness proofs carried into the
  scheduler so independent submissions may overlap when safe while shared root,
  allocator, WAL, or namespace work coordinates only at the exact physical
  authority it touches
- real latch/epoch/hazard or equivalent protection integrated with C.5 pages,
  C.6 frames, C.7 root publication, and C.8 recovery
- copy-on-write maintenance publication with protected-old-generation
  retention and safe reclaim
- foreground and background work classes, reservations, queue-depth admission,
  pacing, cancellation, and backpressure installed in the one C.5.1 scheduler
  and executor path over the C.4 media port
- checkpoint, scrub, compaction, reclaim, backup-read, verifier-read, and blob
  work classes with explicit cost and interference posture
- typed stale-plan retry/rejection, deadlock prevention or detection,
  starvation bounds, and unsupported-QoS outcomes
- explicit separation between Store-owned shared physical coordination and
  future Relational-owned shared global semantic authority; neither token,
  queue, or lock may be reused as proof of the other
- exact latch, lease, retry, blocked-reclaim, queue, yield, sync-delay,
  foreground-wait, and maintenance-debt counters

### Non-Fake Acceptance Setup

- **Production subject:** canonical runtime read and write facades plus real
  checkpoint, scrub, rewrite, reclaim, and scheduler operations.
- **Initial world:** store larger than memory, one pinned old-root reader, one
  current reader, foreground writes, and bounded background queues under a
  deterministic interleaving schedule.
- **Execution:** force root rewrite, checkpoint, scrub, and attempted reclaim
  while readers pause at named production yieldpoints; inject I/O latency and
  crash during publication; reopen fresh afterward.
- **Independent observation:** a serial physical reference model predicts
  allowed roots and generations; media counters explain actual I/O ordering;
  post-crash verifier checks reachable generations.
- **Assertions:** no half-published root, protected bytes remain readable,
  stale plans retry or deny typed, reclaim waits exactly where required,
  foreground reservations remain visible, and scheduler work corresponds to
  actual backend operations.
- **Controlled defect:** reclaim despite a live lease and separately route
  background sync outside the scheduler. Lease-safety and scheduling-bypass
  predicates must fail independently.
- **Forbidden substitutes:** simulated schedules over fake pages, sleeps used
  as correctness, branch labels standing in for physical disjointness, or
  scheduler receipts without backend I/O cannot close this milestone.

### Closeout Gate

`C.10` closes only when readers and real maintenance interleave over persisted
bytes without unstable visibility, and foreground/background cost is governed
and explained at the actual I/O boundary. Physically disjoint submissions must
not acquire a global mutation lock, and shared physical coordination must not
be represented as branch or global semantic authority.

## C.11: Layout, Index, And Native Blob Adoption

### Goal

Move B-tree/LSM access paths, rebuildable indexes, chunk trees, blob streaming,
dedupe, and reclaim onto the canonical runtime rather than parallel fixture or
sidecar paths.

### Boundary

Layout and indexes are derived physical access structures unless explicitly
classified otherwise. Blobs may be authoritative artifacts, but chunk
placement and dedupe indexes do not redefine blob identity. Query pushdown and
semantic graph traversal remain Part II. Semantic liveness, retention, tenant
policy, key policy, and blob meaning arrive later as admitted proofs or requests;
Part I may enforce their physical scope and execute their effects but may not
derive or decide them.

### Must Ship

- artifact-family registry binding each admitted family to physical layout,
  source authority, access operations, rebuild basis, format version,
  integrity class, physical retention mechanics, and recovery participation;
  semantic retention and liveness remain external inputs
- B-tree and LSM point/range/prefix/scan paths over C.5 pages and C.6 residency
  with C.10 stable plans and scheduling
- deterministic index rebuild and corruption fallback whose cost and support
  posture are explicit
- native content-addressed chunk-tree storage through the same media, WAL/root,
  integrity, lease, C.5.1 work, scheduling, and recovery boundaries
- constant-memory streaming ingest/read/verify/export/import and interrupted
  ingest recovery
- collision handling plus physically scoped dedupe, reachability traversal,
  orphan classification, tier movement, and reclaim execution with exact
  effects; semantic liveness, tenant/key authorization, retention, and reclaim
  permission must be presented by their future authority rather than inferred
  from physical absence or age. Store may independently reclaim residue proven
  to belong only to a failed/aborted physical operation or rebuildable physical
  derivation when that proof is complete
- broad-scan denial where an admitted indexed path is required
- exact page touches, probes, ranges, amplification, chunk bytes, resident
  bytes, copies, dedupe, reachability, and reclaim counters

### Non-Fake Acceptance Setup

- **Production subject:** canonical runtime family registration, B-tree/LSM
  access, and blob streaming facades over the reconstructed physical platform.
- **Initial world:** indexed data and a blob each materially larger than the
  memory budget, repeated content for dedupe, at least two segments, and one
  active reader lease during rewrite/reclaim.
- **Execution:** ingest, perform physical point lookup and range scan, stream
  ranges, interrupt and resume blob ingest, corrupt derived index and one chunk,
  rebuild, rewrite layout, attempt reclaim, crash, and fresh-process reopen.
- **Independent observation:** canonical key/value and blob-digest models are
  built from the input generator; offline traversal measures pages, chunks,
  roots, reachability, and orphan sets without using runtime access APIs.
- **Assertions:** input-model byte parity, constant-memory slope, exact access and
  amplification counters, rebuild parity, corruption localization, safe lease
  preservation, dedupe scope honesty, and no sidecar or whole-object path.
- **Controlled defect:** hide a full blob materialization and separately accept
  a corrupted derived index as authority. Allocation and rebuild-basis
  predicates must fail.
- **Forbidden substitutes:** tiny blobs, in-memory indexes, test fixture files
  not produced through the runtime, or access receipts unsupported by observed
  page/chunk I/O cannot close this milestone.

### Closeout Gate

`C.11` closes only when every retained S.8/S.7 access and blob mechanism runs
through the one reconstructed platform with bounded memory, recoverable
publication, and destroy/rebuild honesty for derived structures. Physical
reclaim must consume admitted liveness/retention proof rather than inventing
semantic reachability from its own artifact graph.

## C.12: Formal Protocol Rebinding To Executable Owner Transitions

### Goal

Run S.9 checked law against the reconstructed runtime's actual durability,
recovery, visibility, quarantine, admission, and publication transitions
without a certification intermediary.

### Boundary

Formal models define finite checked law; production owners execute and decide.
Focused tests map concrete owner outcomes to model actions, and the pinned TLC
command checks the model directly. Model actions and checker verdicts never
become runtime authority. C.12 models only transitions with a production
physical owner in this reconstruction sequence. Semantic commit, branch
admission, MVCC visibility, recovery readmission, Query pushdown, and
replication remain outside its model authority.

### Must Ship

- direct TLC execution for every current checked protocol family
- focused owner integration tests for each claimed mapping from concrete owner
  behavior to WAL/checkpoint, recovery-source, stable-read/reclaim,
  compaction/publication, quarantine/recovery admission, or physical import law
- compiler-exhaustive mapping where a production outcome is a closed enum;
  explicit capability gaps where no production owner exists
- explicit backend, atomicity, durability, clock, scheduling, and bounded-state
  assumptions
- transient counterexample diagnostics that identify the checked protocol and
  illegal edge when TLC supplies that information
- fresh-process or media-interposed tests only where the claim actually crosses
  a process or media boundary
- explicit negative mapping proving that branch writers, semantic generations,
  global semantic authority, Query visibility, and replica meaning have no
  C.12 production witness and therefore cannot be smuggled in as modeled
  physical actions

### Non-Fake Acceptance Setup

- **Production subject:** named reconstructed owner transitions exercised by
  focused tests, not catalog rows or generated receipts.
- **Checked subject:** the committed TLA+ artifact, configuration, explicit
  bounds, and backend assumptions invoked by the pinned runner.
- **Assertions:** exact expected owner outcomes in focused tests; distinct typed
  success, counterexample, unsupported-backend, tool-failure, and
  bound-exhaustion verdicts from the runner.
- **Diagnostics:** counterexamples are printed for the failed run and may be
  retained by CI; Store does not build a report, digest, inventory, matrix,
  trace-lifting layer, localization receipt, or evidence archive around them.
- **Forbidden substitutes:** fictional owner cases, generated mapping
  inventories, coverage matrices, bound exhaustion reported as proof, or model
  verdicts used as production witnesses cannot close this milestone.

### Closeout Gate

`C.12` closes only when the pinned command checks every current S.9 model,
focused tests exercise every claimed production mapping directly, unsupported
owner cases remain explicit, and no modeled branch, semantic commit, Query, or
replication transition is promoted in advance of its production owner.

## C.13: Joined Physical-Platform Integration And S.10 Entry

### Goal

Exercise the reconstructed S.1 through S.9 owners together through the ordinary
production runtime, remove obsolete paths exposed by that integration, and
provide the bounded handoff needed to begin S.10.

### Boundary

This milestone adds no new physical feature. It is a joined integration and
source-cutover phase. S.10, S.11, S.12, and Part II retain their own future
obligations. C.13 may expose the physical facade shape those programs consume;
it cannot issue Part II semantic readiness or pre-authorize branch, Query, or
replica behavior.

### Must Ship

- hard deletion or non-production quarantine of heap runtimes, replay-based
  reopen, duplicate backends, fake physical fixtures, obsolete certification
  paths, and shadow authority discovered by the program
- dependency checks proving ordinary product paths reach only the canonical
  runtime facade and physical owners depend in the admitted direction
- direct public-facade compilation and focused integration tests proving it
  exposes independently borrowable bounded reads, independently borrowable
  generation-fenced mutation submission, exact physical scope, exact terminal
  and recovery fate, capability negotiation, lifecycle/drainage, and pressure
  evidence while exporting no Query, Relational, branch, MVCC, or semantic
  writer vocabulary
- cross-milestone hostile execution combining real writes, stores larger than
  memory, checkpoint/WAL recovery, corruption, stable readers, scheduled
  maintenance, index rebuild, and blob streaming
- focused regressions at the owning boundaries for acknowledgment inversion,
  live-state reuse, checksum bypass, generation bypass, reclaim-with-live-lease,
  scheduler bypass, broad scan, full materialization, derived-authority
  promotion, model/owner drift, accidental whole-Store submission
  serialization, and branch-label-based physical admission
- typed `S10PhysicalPlatformReadiness` or equivalently responsibility-named
  handoff whose private construction requires every restored claim and whose
  payload exposes the exact S.10 owner ports
- direct adapter-facing compilation proving the lower contracts required by the
  runtime-integration roadmap remain available and branch-agnostic, without a
  carried compatibility report or second readiness token
- explicit list of remaining non-platform-grade or unsupported capability
  profiles; no unnamed debt

### Non-Fake Acceptance Setup

- **Production subject:** one release-built canonical runtime and the distinct
  offline verifier executable. No milestone-local runtime or fixture backend
  may satisfy an aggregate predicate.
- **Initial world:** real store at least eight times the memory budget with
  pages, extents, checkpoints, WAL tail, B-tree/LSM indexes, multi-segment
  blobs, derived and authoritative artifacts, and declared backend assumptions.
- **Execution:** foreground reads/writes from multiple independent submission
  capabilities continue during checkpoint, scrub,
  rewrite, reclaim, index rebuild, and blob streaming; inject crash and
  corruption at declared seams; start a fresh recovery process; run offline
  verification; run direct regressions for known failure modes.
- **Independent observation:** an external history model supplies semantic
  expectations; the offline verifier supplies physical classification; OS/media
  observation supplies actual artifact and barrier evidence. None receives
  runtime heap state.
- **Assertions:** all restored S.1 through S.9 predicates, exact resource and
  interference counters, deterministic recovery, independent classification,
  zero forbidden paths, concurrent progress for disjoint physical submissions,
  exact coordination for shared physical scope, and absence of semantic branch
  authority below the adapter boundary.
- **Forbidden substitutes:** combining milestone receipts without rerunning the
  joined system, using a memory backend, reusing a live process, omitting larger-
  than-memory pressure, or granting readiness through a public constructor
  cannot close this milestone.

### Closeout Gate

`C.13` closes only when S.1 through S.9 work together in one real physical
database runtime, every obsolete substitute is unreachable from production,
the joined hostile scenario and focused owner regressions are green, and the
sealed S.10 readiness handoff is constructible only from the completed runtime
progression.

## Required Engineering Specs

Each reconstruction milestone receives a separate engineering spec before
implementation:

- `physical-reconstruction-c1-test-execution-architecture.md`
- focused owner-path inspection and boundary-check output
- `physical-reconstruction-c3-sealed-runtime-lifecycle.md`
- `physical-reconstruction-c4-production-media-boundary.md`
- `physical-reconstruction-c5-durable-physical-record-path.md`
- `physical-reconstruction-c5-1-physical-store-work-runtime.md`
- `physical-reconstruction-c6-buffer-pool-runtime-join.md`
- `physical-reconstruction-c7-durable-publication-join.md`
- `physical-reconstruction-c8-fresh-process-recovery-and-reopen.md`
- `physical-reconstruction-c9-integrity-and-offline-truth.md`
- `physical-reconstruction-c10-isolation-and-io-coordination.md`
- `physical-reconstruction-c11-layout-index-blob-adoption.md`
- `physical-reconstruction-c12-formal-owner-rebinding.md`
- `physical-reconstruction-c13-platform-integration-and-s10-entry.md`

C.3 through C.13 specs, including C.5.1, inherit the Non-Fake Physical
Acceptance Test Contract,
then make their setup more concrete. Repeating only “control, hostile, and
reopen lane” is not sufficient; the spec must name executables, initial files,
process deaths, forbidden inputs, independent observers, exact counters, and
controlled defects. C.1 instead closes through direct Cargo/test execution and
ordinary CI status. C.2 is retired historical context and imposes no current
closeout work; each current phase inspects and tests its own causal boundaries.

Every C.3 through C.13 spec also inherits the Part I/Part II handoff law in
`Relationship To The Two Store Roadmaps` and Product Decision Lock items 17
through 20. Each spec must name (a) the exact physical authority it installs,
(b) the branch-agnostic facade or evidence it contributes to later integration,
and (c) the semantic, branch, MVCC, Query, or global-authority decision it is
forbidden to make. A generic statement that integration happens later is not a
boundary proof.

## Must Preserve

- Physical Store owns byte survival and physical access.
- Physical format remains meaning about bytes, not the owner of media effects.
- Relational remains semantic MVCC, visibility, branch, and transaction
  authority.
- Part II Relational remains the sole owner of active branch-write admission:
  many submissions may be planned or queued, exactly one mutation may own a
  branch-head generation at a time, different branches may write concurrently,
  and shared global semantic authority coordinates independently.
- Physical Store remains branch-agnostic and exposes concurrent submission,
  exact physical scope, durability, settlement, and recovery fate rather than
  branch queues, semantic writer leases, or semantic commit decisions.
- Query remains the future ordinary domain-facing language through Part II.
- semantic Signal and Runtime Bridge retain derived and causal authority in
  Part II; physical Signal owns only reconstructible physical-work derivation
  inside the Part I composition layer.
- Existing strong mechanisms and tests are preserved when they can bind to the
  production path without weakening ownership.
- Test and certification infrastructure never becomes a production authority
  source.
- Store larger than memory, crash, corruption, stable reads, maintenance,
  blobs, and independent verification remain non-negotiable physical claims.

## Acceptance Evidence

Current validity is queried from the current tree. Each phase runs the focused
owner tests, affected integration or process scenarios, compile-time authority
checks, and repository boundary gates proportionate to its change. Git records
history; Cargo metadata and current callers expose the live graph. Generated
catalogs, audit CSVs, proof bundles, mutation reports, source fingerprints, and
report-to-report reconciliation are not program deliverables.

The final joined evidence is a real physical database scenario covering
larger-than-memory access, durability and crash boundaries, fresh-process
recovery, corruption localization, stable reads, maintenance interference,
layout/index rebuild, and blob streaming, with focused owner tests providing
local failure diagnosis. The sealed S.10 handoff remains a production typestate
boundary, not a certification receipt.

## Sequencing Rules

- C.1 closes before any later milestone begins implementation.
- Current caller, Cargo-graph, and owner-boundary inspection precedes any phase's
  decision to preserve or delete an existing mechanism.
- C.3 and C.4 establish the only runtime and media seams later milestones may
  consume.
- C.5 precedes buffer, WAL, recovery, integrity, isolation, layout, and blob
  claims because those claims require real physical artifacts.
- C.5.1 follows C.5 and precedes C.6 through C.11 because every later physical
  owner must enter one work, scheduling, execution, settlement, and shutdown
  topology rather than grow its own background runtime.
- C.6 through C.12 extend C.5.1's branch-agnostic physical contracts; none may
  replace them with a whole-Store mutation borrow, branch-labelled lane,
  semantic admission table, or milestone-local adapter.
- C.6 precedes greater-than-memory recovery and access claims.
- C.7 precedes C.8 because recovery must consume one real durability law.
- C.8 precedes C.9 operational classification because corruption evidence must
  be tested against an actual recovery path.
- C.9 precedes C.10 and C.11 closeout so maintenance and artifact families
  cannot publish unchecked bytes.
- C.10 precedes C.11 closeout so index and blob rewrite/reclaim use real stable
  reads and scheduled I/O.
- C.12 follows the executable transitions it models.
- C.13 is last and is the joined production integration route into S.10.

## Completion Standard

This roadmap is complete only when Worth Store can honestly say:

- changing one owner has a fast, narrow, trustworthy feedback loop
- all proof lanes are explicit and expensive proof is expensive for a named
  reason rather than repeated compilation
- the physical runtime is sealed and cannot be cloned or reopened from supplied
  heap truth
- ordinary writes create real files through one production media boundary
- records live in real pages, segments, extents, and manifests
- one physical Store instance owns operation lifecycle through one disposable,
  effect-free physical Signal graph and one scheduler/executor boundary, with
  no duplicate work authority or raw effect path
- independent, incarnation-fenced physical capabilities admit disjoint work
  concurrently, coordinate shared physical scope exactly, and report proven
  no-effect, completed, or indeterminate fate without learning branch identity
- the public physical facade gives Part II everything needed to correlate
  physical work while leaving active branch-write admission, branch-head
  advancement, MVCC visibility, and shared global semantic authority entirely
  above Store
- ordinary access is buffer-pool bounded for stores larger than memory
- WAL, page, checkpoint, root publication, and acknowledgment form one durable
  progression
- killed processes recover from persisted bytes in fresh processes
- physical damage is rejected before semantic decode and independently
  classifiable offline
- stable readers survive real maintenance and scheduled I/O interference
- layouts, indexes, and blobs use the same canonical physical platform
- formal laws map to executable owner transitions
- S.1 through S.9 have been revalidated over the joined production runtime
- S.10 receives a sealed readiness handoff rather than another vocabulary claim

Only then may operational recovery resume. The later runtime-integration
roadmap can subsequently build the existing Worth runtime on top of this
physical platform without inheriting a fake database boundary.
