# Milestone 3.10.1: DSL Ownership, Runtime Subsystem Boundaries, and Facade Closure

> Historical QA policy (2026-08-22): proof, closure, migration, acceptance,
> and phase ledgers described below are frozen historical records. They are not
> active implementation or release gates, are not updated or reopened, and a
> ledger-only failure does not block current work. Current evidence follows
> [the QA review guide](../../docs/coding-guidelines/qa_review_guide.md) and
> [testing laws](../../docs/coding-guidelines/testing_laws.md): specifications state QA
> considerations in prose, tests and repository checks run against the current
> commit, and code review decides whether the evidence is adequate. This note
> does not retire product-domain ledgers that are part of runtime behavior.

## Status

Status: Complete (2026-07-25)

## Placement

Milestone 3.10.1 is a corrective architecture gate after Milestone 3.10 and
before Milestone 3.11.

The milestone completed before the human-visible Platform Pulse requirement was
adopted. Milestone 3.10.2 subsequently proved that the pulse traverses the
DSL-owned sealed handoff and condensed public facade established here in both
the human workflow and automated in-process integration. Milestone 3.10.3 now
owns the later-discovered requirement to prove the same route through the exact
product binary and native event loop. That successor gate is mandatory before
3.11, but neither successor is retroactive evidence for this already-completed
milestone.

Milestone 3.10 establishes the complete mounted-frame and host-contract truth
that later visual, observation, rebind, interaction, service, and appearance
work will consume. The implementation review performed after that milestone
found three remaining structural risks in the path those later milestones
would extend:

1. authored syntax, parsing, source AST, legality, and semantic lowering are
   physically concentrated under `worth-ui-runtime` even though
   `worth-ui-dsl` is the named language owner;
2. `worth-ui-runtime` contains too many authority families behind one broad
   physical and conceptual boundary, making future insertion and review
   increasingly dependent on repository archaeology; and
3. the product facades still expose transitional phase and mounted-lifecycle
   vocabulary beyond the narrow ordinary application path.

These risks must close before Milestone 3.11 binds visual snapshots and
hit-test identity to mounted truth. Otherwise every later consumer will harden
the wrong source owner, expand the runtime macro-boundary, or depend on public
intermediate authority that must later be removed.

This milestone is not a reopening of mounted receipt meaning or host mechanics.
Milestone 3.10 remains authoritative for those semantics.

## Adversarial Constraint

A valid file-authored or Rust-authored application, under repeated hot
replacement and mixed-lane high-frequency frame demand, must cross exactly one
DSL-owned authored-source-to-semantic boundary and exactly one product-owned
mounted-frame execution facade.

A hostile contributor must be mechanically unable to:

- add a parser, source AST, authored-language legality decision, or semantic
  source lowerer under `worth-ui-runtime`;
- construct the runtime handoff artifact without passing the DSL-owned
  admission transition;
- expose preparation, assembly, attempt, publication, reconciliation, identity
  index, storage, or internal cost-construction authority through the ordinary
  product facade;
- call an older public lifecycle route that bypasses
  `execute_mounted_frame(...)`;
- make an ordinary frame parse source, validate authored syntax, scan broad
  source/declaration collections, or allocate merely because ownership moved;
  or
- satisfy the milestone with renamed folders, re-export aliases, compatibility
  wrappers, token scans, or tests that manufacture the evidence they claim to
  verify.

## Problem Statement

The platform's semantic laws already describe a clean architecture:

```text
authored source
  -> DSL-owned parse / legality / semantic lowering
  -> canonical admitted application package
  -> runtime-owned graph / plan / allocation / mounted execution
  -> host-contract-owned mechanical adapter exchange
```

The current physical topology does not yet make the first transition honest.
The runtime's source tree owns substantial authored-language mechanics while
the DSL crate is comparatively thin. That location makes the runtime the
practical insertion point for future expressions, modules, imports, fragments,
source diagnostics, and lowering rules even though the roadmap assigns those
features to the language.

The runtime has also grown into the dominant implementation boundary. Size is
not itself a defect and this milestone does not prescribe arbitrary crate
splitting. The defect is that contributors cannot reliably infer which
subsystem owns a transition, which state may be retained, which authority may
cross a boundary, or where the next feature should be inserted without
tracing a very large module graph.

Finally, the narrow mounted-frame entry point exists, but the surrounding
public facades still allow ordinary callers to see or assemble too much of the
phase progression. A narrow function beside broad re-exports is not a narrow
contract.

## Goal

Close the architecture gap between the documented ownership model and the
physical/public code topology by:

- making `worth-ui-dsl` the sole production owner of authored syntax, source
  AST, language legality, source diagnostics, and authored-to-canonical
  semantic lowering;
- defining one sealed, generation-bound handoff from DSL authority into runtime
  preparation;
- partitioning runtime responsibilities into autonomous named subsystem
  contracts with explicit state, transition, and borrowing boundaries;
- preserving `worth-ui-runtime` as the owner of active application, graph,
  planning, allocation, execution, mounted publication, and runtime inspection
  truth without letting it reinterpret authored syntax;
- reducing the ordinary product surface to the minimum application lifecycle,
  mounted-frame request/outcome, and typed recovery or continuation handles
  required by real callers;
- removing predecessor public routes and broad intermediate re-exports instead
  of retaining parallel compatibility lanes; and
- proving the resulting topology mechanically, behaviorally, and with honest
  performance/build evidence.

## Non-Goals

- changing Milestone 3.10 mounted receipt, identity, host observation, native
  effect, publication, or reconciliation semantics;
- implementing Milestone 3.11 visual snapshots or hit-test identity;
- implementing Milestone 3.12 semantic observation admission or rebind
  planning;
- implementing Milestone 3.17 expressions or Milestone 3.18 modules,
  composition, and imports;
- moving filesystem watching, debounce, atomic file acquisition, or operating
  system mechanics into `worth-ui-dsl`; those are source-transport mechanisms,
  not language meaning;
- splitting `worth-ui-runtime` into crates merely to reduce file or line counts;
- creating a generic `core`, `common`, `shared`, `helpers`, `util`, or
  `types` layer;
- changing the single product-crate posture of `worth-ui`;
- preserving source or API compatibility when compatibility would retain a
  second authority path; or
- treating documentation, module renames, or re-export shuffling as sufficient
  closure.

## Governing Decisions

1. **The DSL crate owns language meaning.** Tokenization, parsing, source AST,
   authored legality, source-span diagnostics, normalization required to
   understand authored constructs, and authored-to-canonical lowering belong
   to `worth-ui-dsl`.
2. **Source transport is not source semantics.** Filesystem acquisition,
   watcher/debounce mechanics, stable-byte snapshots, and replacement
   scheduling remain with their current application/runtime mechanism owners.
   They deliver immutable source inputs to the DSL boundary.
3. **Runtime accepts semantic packages, never syntax.** Runtime preparation may
   validate runtime capability, world, generation, graph, allocation, host, and
   activation requirements. It may not parse or reinterpret authored syntax.
4. **Rust-authored and file-authored composition converge before runtime.**
   Both authoring modes produce the same canonical semantic package and carry
   provenance describing how it was authored; neither gets a privileged
   runtime lane.
5. **Subsystem boundaries follow authority and lifecycle.** Runtime partitioning
   is justified by independently owned state, transition rules, failure
   ownership, and future insertion. Line count alone does not justify a crate.
6. **The active application session is the runtime composition root.** It owns
   subsystem aggregates and coordinates transitions; it does not become a
   semantic implementation bucket.
7. **The product facade is an audience contract.** Ordinary application
   callers see lifecycle inputs/outcomes, not internal phase artifacts.
   Host mechanics and inspection use separate named audience facades.
8. **One public mounted-frame path survives.** The ordinary execution route is
   `execute_mounted_frame(...)`. Advanced in-flight, retry, recovery, or
   reconciliation behavior is reached only through typed handles returned by
   that path, not independent free-entry lifecycle routes.
9. **Internal authority is not public vocabulary.** A type is not eligible for
   the product facade merely because it is useful to tests, adapters, or another
   runtime module.
10. **Removal is part of migration.** Deprecated aliases, forwarding wrappers,
    old free functions, and transitional re-exports that preserve predecessor
    authority must be deleted in the same milestone.

## Target Ownership Topology

```text
worth-ui
  facade::app
    application construction and replacement
    active-session lifecycle
    mounted-frame request / outcome
    typed continuation and recovery handles
  facade::inspection
    compact read-only evidence queries

worth-ui-dsl
  source input and provenance
  lexer and parser
  source AST
  authored-language legality
  source diagnostics
  semantic normalization
  canonical authored-to-runtime package sealing

worth-ui-runtime
  application admission and generation publication
  graph and identity authority
  planning, measurement, and allocation authority
  execution-plan and mounted-frame authority
  observation transport intake
  inspection evidence production
  active-session composition root

worth-ui-host-contract
  host-neutral mechanical request/report contracts
  operational adapter capability

worth-ui-host-*
  native mechanics only
```

Within `worth-ui-runtime`, the required subsystem families are:

- `application`: candidate preparation, active generation, replacement
  transaction, and application-session ownership;
- `graph`: graph identity, committed topology, dependency/impact, and graph
  transition authority;
- `planning`: measurement basis, allocation planning, execution-plan lowering,
  and affected-neighborhood authority;
- `mounting`: prepared-frame, mounted identity, host exchange, presentation,
  publication, reconciliation, and mounted retention authority;
- `observation`: bounded raw/structurally validated host report transport and
  framework-turn intake, without pulling Milestone 3.12 semantic admission
  forward;
- `inspection`: relevance indexes and read-only projections over authoritative
  runtime state; and
- `session`: the thin orchestration owner that borrows the preceding subsystem
  capabilities in declared transition order.

The exact directory names may differ if the Phase 1 inventory proves a more
accurate domain term. The ownership families, allowed directions, and
prohibition on catch-all peers do not differ.

## Canonical Progression

```text
stable source bytes or Rust-authored declarations
  -> UiAuthoredSourceInput
  -> UiDslParseOutcome
  -> UiDslSemanticPackage
  -> UiDslRuntimeHandoff
  -> UiApplicationCandidate
  -> UiPreparedApplication
  -> WorthUiActiveApplicationSession
  -> UiMountedFrameRequest
  -> UiMountedFrameOutcome
```

Required transition properties:

- each arrow has one production owner and one failure owner;
- phase values are sealed and cannot be freely reconstructed from downstream
  fields;
- source spans and authoring provenance survive the DSL handoff without
  granting runtime source-interpretation authority;
- runtime generation, graph, plan, allocation, host session, surface, and frame
  identity remain runtime-owned and are not minted by the DSL;
- `UiMountedFrameOutcome` carries only public result posture, receipts/evidence
  intentionally promised to the caller, and sealed next-action handles; and
- denial before publication preserves the complete predecessor application and
  mounted truth already required by the roadmap.

Names in this progression are specification names. Phase 1 must adjudicate
existing names and record rename/retain/remove decisions rather than introduce
aliases blindly.

## Public Surface Budget

Phase 1 must establish a checked-in, mechanically enforced public-surface
manifest for the ordinary `worth-ui::facade::app` audience.

The manifest may include only:

- application source/declaration inputs and typed preparation or replacement
  outcomes required by app authors;
- the active application session handle;
- `UiMountedFrameRequest`;
- `UiMountedFrameOutcome`;
- the public stop/denial postures a caller must branch on;
- compact public mounted receipt/evidence references explicitly promised by
  Milestone 3.10; and
- sealed continuation, retry, or recovery handles returned from the ordinary
  path when the protocol requires another call.

The manifest must exclude:

- candidate-internal, prepared-frame, assembly, attempt, publication,
  reconciliation, retention, index, catalog, storage, and cost-construction
  types;
- constructors for graph, plan, allocation, mounted, host-session, application,
  or publication authority;
- test-support or certification authority;
- raw host adapter mechanics;
- raw Query authority or identity;
- independent lifecycle free functions that can begin in the middle of the
  mounted-frame progression; and
- glob or root re-exports whose future growth silently changes the audience
  contract.

No numeric symbol target is declared before the inventory because a small but
dishonest facade is worse than an explicit one. Closure is measured against
the adjudicated allowlist and caller journeys, not an arbitrary count.

## Performance and Build Budgets

The migration must preserve the established cost shape:

- steady frames perform zero source tokenization, parsing, authored-language
  legality, semantic lowering, or source-span reconstruction;
- unchanged steady frames perform no broad graph, declaration, source, plan,
  mounted-receipt, or facade-manifest scan;
- DSL work is charged to initial preparation or admitted replacement, with
  named counters for bytes/tokens/nodes/declarations actually processed;
- replacement runtime work remains bounded by admitted affected scope where the
  existing runtime contract already provides that guarantee;
- moving code across crates may not introduce per-frame cross-crate heap
  allocation, cloning of heavyweight authority, string-key lookup, or dynamic
  dispatch;
- inspection projection remains read-only and budgeted rather than becoming a
  second hot execution path;
- the existing two-session compile-contract posture and consolidated
  integration-target topology remain unchanged; and
- implementation closeout records comparable warm targeted, warm fast-lane,
  and isolated cold compile-contract measurements, with any material regression
  adjudicated before closure.

Phase 1 records the current baseline commands and values before migration.
Phase 7 records comparable closing values. Measurement methodology must not
change between them merely to make the result look favorable.
## Phase 1: Adjudicated Inventory and Destination Freeze

### Relevant Subsystems and APIs

- `worth-ui-dsl`
- `worth-ui-runtime::source`
- `worth-ui-runtime::facade`
- `worth-ui::facade::{app,mounted,runtime,inspection}`
- active application session and mounted-frame lifecycle
- topology, public-surface, compile-contract, line-cap, and agent-context guards

### Required Work

Create two checked-in, manually adjudicated ledgers.

The **source-semantics inventory** must classify every production source file,
module, public symbol, and boundary edge involved in:

- source input and provenance;
- tokenization and parsing;
- source AST representation;
- authored legality and diagnostics;
- semantic normalization and lowering;
- filesystem acquisition and watcher/debounce mechanics;
- Rust-authored composition;
- canonical package sealing;
- runtime candidate preparation; and
- replacement scheduling.

Each row records current owner, semantic responsibility, authority produced or
consumed, lifecycle, failure owner, cost lane, destination owner, disposition
(`move`, `retain`, `split`, `rename`, `remove`, or `unrelated`), and forbidden
shortcut.

The **facade/runtime inventory** must classify:

- every public symbol exported by ordinary app, mounted, runtime, inspection,
  host, and certification audiences;
- every callable lifecycle entry point;
- every runtime subsystem state owner and cross-subsystem edge;
- every existing compatibility alias or forwarding route; and
- every planned later-milestone insertion point affected by this correction.

Every row must have one disposition and, for transitional rows, one exit phase
within this milestone. Ambiguous rows block Phase 2.

Freeze:

- the canonical progression and transition owners;
- the ordinary product-facade allowlist;
- the runtime subsystem dependency matrix;
- baseline performance and build commands/results; and
- the deletion list for predecessor routes.

### Engineering Decisions

- Search and dependency tools seed the inventories; a human/agent semantic
  adjudication closes each row.
- A clean token search is never accepted as proof that a responsibility moved.
- Current behavior is not automatically the correct destination. Each row is
  adjudicated against semantic authority and lifecycle.
- Runtime crate extraction is permitted only when an inventory row proves
  autonomous state, a stable directional contract, an independent failure
  owner, and no cyclic dependency. Otherwise the boundary is a named internal
  subsystem.

### Warnings

- Do not classify all of `runtime::source` as DSL work. File acquisition,
  watcher mechanics, application replacement scheduling, and runtime
  publication have different owners.
- Do not classify every public type as app-facing merely because current tests
  import it.
- Do not use line count as a proxy for ownership.
- Do not begin moves while ambiguous or transitional rows lack a named exit.

### Test Requirements

1. **Second-parser hostility:** seed the inventory fixture with a parser-shaped
   runtime module that avoids obvious names such as `parser` and `ast`; semantic
   adjudication must still classify it as DSL-owned.
2. **Mechanism/meaning hostility:** seed a filesystem watcher module that emits
   stable bytes and a neighboring module that decides authored legality; the
   inventory must retain the former with its mechanism owner and move the latter
   to DSL.
3. **Public-by-accident hostility:** include a type publicly re-exported only
   because certification imports it; the facade inventory must route it to
   support authority rather than the app allowlist.
4. **Crate-split hostility:** propose a runtime crate split with bidirectional
   state ownership; the dependency matrix must reject it instead of blessing a
   cycle behind facades.

### Open Questions

- Which current names already express the target responsibility accurately
  enough to retain?
- Which runtime source modules combine transport mechanics and language meaning
  and therefore require semantic decomposition before movement?
- Which mounted receipt/evidence values are genuinely required by an ordinary
  app caller rather than inspection or host audiences?
- Does any runtime subsystem satisfy the autonomous-crate extraction test, or
  are explicit internal boundaries sufficient for this milestone?

## Phase 2: DSL-Owned Source and Semantic Authority

### Relevant Subsystems and APIs

- `worth-ui-dsl` source input, lexer, parser, AST, legality, diagnostics,
  normalization, and lowering modules
- source-span and authored provenance types
- canonical DSL package and sealing authority
- filesystem/Rust authoring gateways
- runtime source modules listed `move`, `split`, or `remove` in Phase 1

### Required Work

Move the production ownership of authored-language meaning into
`worth-ui-dsl`.

The DSL crate must own named modules for:

- immutable authored source input and stable source identity;
- lexical analysis;
- parse tree/source AST;
- language-level legality;
- source-span-preserving diagnostic production;
- semantic normalization;
- canonical identity derivation for authored declarations where language rules
  determine that identity;
- lowering into the canonical semantic package; and
- sealing the handoff artifact consumed by runtime.

Decompose mixed modules before moving them. A file that both reads the
filesystem and parses bytes becomes:

- a transport-side producer of an immutable source snapshot; and
- a DSL-side consumer that parses that snapshot.

A file that both lowers language constructs and admits runtime capabilities
becomes:

- DSL-owned semantic lowering; and
- runtime-owned capability/world admission over the sealed result.

Preserve source spans and authoring provenance through lowering. Provenance is
diagnostic evidence, not authority to reopen or reinterpret the source in
runtime.

Delete runtime-local copies, mirrors, aliases, forwarding constructors, and
alternate parsers once callers migrate. The final runtime tree may contain
source transport and replacement orchestration, but no language semantic
owner.

### Engineering Decisions

- `worth-ui-dsl` remains independent of runtime execution, host, Query, and
  mounted authority.
- The DSL may depend on pure canonical declaration/schema meaning if permitted
  by the workspace dependency laws; runtime must not be pulled upward as a
  convenience dependency.
- Diagnostics produced while understanding authored syntax are DSL-owned.
  Diagnostics produced while admitting runtime worlds, capabilities, graphs,
  plans, allocations, or hosts remain runtime-owned.
- Source identity has separate transport identity and semantic provenance where
  those meanings differ; one digest must not impersonate both.
- The DSL handoff is immutable, sealed, and generation-agnostic. Runtime binds
  it to application/runtime generations during preparation.

### Warnings

- Moving files while leaving constructors or decision-making in runtime does
  not transfer ownership.
- Do not make `worth-ui-dsl` a bag of shared structs while runtime continues to
  parse and lower.
- Do not import runtime types into DSL to avoid designing a real handoff.
- Do not make source spans the identity or equality authority for lowered
  semantic artifacts.
- Do not move watcher/debounce, replacement publication, or active-generation
  state into DSL.

### Test Requirements

1. **Runtime parser rejection:** a compile/boundary fixture adding a production
   runtime module that tokenizes authored source must fail even if the module is
   private and never re-exported.
2. **Runtime lowerer rejection:** a fixture that receives a source AST in
   runtime and constructs semantic declarations directly must fail the
   ownership guard.
3. **Diagnostic ownership:** malformed file source and equivalent malformed
   in-memory source must produce the same DSL diagnostic identity, spans, and
   stop class through production entry points.
4. **Mechanism preservation:** real filesystem acquisition and watcher
   replacement must still begin with actual files and OS-backed events; the
   migration may not replace them with injected source or manufactured watcher
   events.
5. **Dependency hostility:** `worth-ui-dsl` must fail topology checks if it
   imports runtime, host, Query, mounted, or active-session authority.
6. **Shadow-path hostility:** a renamed runtime module using generic terms such
   as `decode`, `compile`, or `normalize` must be detected by an AST/dependency
   ownership audit, not only filename tokens.

### Open Questions

- What is the narrowest pure canonical declaration dependency the DSL lowerer
  requires?
- Which existing diagnostic identities must remain stable for stored evidence
  or tooling consumers?
- Which current source-derived identities are language-owned, and which become
  runtime generation identities only after admission?
- Can all runtime-owned source semantic modules move directly, or do any require
  a temporary split within this phase before deletion?

## Phase 3: Canonical Authored-to-Runtime Handoff

### Relevant Subsystems and APIs

- file-authored source gateway
- Rust-authored composition gateway
- DSL semantic package and runtime handoff
- application candidate preparation and replacement admission
- source diagnostics and runtime denial evidence
- application generation and provenance inspection

### Required Work

Define exactly one production handoff accepted by runtime preparation.

The handoff must carry:

- canonical semantic declarations and their stable semantic identities;
- canonical graph/composition inputs already resolved at the language layer;
- source-span and expansion-ready provenance required for diagnostics;
- authored-mode posture (`file`, `Rust`, or a future admitted mode) as evidence,
  not as divergent runtime behavior;
- the exact DSL schema/protocol identity needed to reject incompatible packages;
- a collision-safe package identity over canonical semantic content;
- the completeness/sealing proof that all required language phases succeeded;
  and
- no runtime generation, active-session, host, Query, allocation, mounted, or
  publication authority.

File-authored flow:

```text
real source acquisition
  -> immutable source snapshot
  -> DSL parse / legality / lowering
  -> sealed runtime handoff
  -> runtime candidate preparation
```

Rust-authored flow:

```text
typed Rust composition
  -> DSL-owned semantic admission / canonicalization
  -> sealed runtime handoff
  -> runtime candidate preparation
```

Both flows must use the same runtime preparation function and produce
equivalent canonical identities for semantically equivalent applications,
except for explicitly diagnostic provenance.

Runtime preparation must consume the handoff by value or through a sealed
affine transition. It may not accept a source AST, raw token stream, loose
declaration collection, or caller-constructed equivalent.

Inspection must be able to report:

- handoff identity and protocol;
- authoring provenance;
- language stop or success posture;
- runtime preparation stop or success posture; and
- the exact transition where ownership changed.

### Engineering Decisions

- The handoff type is owned and sealed by `worth-ui-dsl`; the runtime-facing
  consumption function is owned by the runtime audience boundary.
- Semantic equality excludes source formatting and non-semantic provenance.
- Diagnostic provenance remains attached by compact references so equality and
  hot-path execution do not scan spans.
- Rust-authored composition is not allowed to mint the sealed package through a
  public struct literal or unrestricted constructor.
- Runtime denial never mutates or retroactively invalidates the DSL package; it
  denies binding that package to a runtime application generation.

### Warnings

- Do not preserve a second runtime preparation overload for loose declarations.
- Do not make file-authored and Rust-authored parity a test-only conversion.
- Do not flatten DSL and runtime failures into one generic error.
- Do not hash printable debug output, source spans, map iteration order, or
  memory addresses as semantic package identity.
- Do not retain a convenience method that unwraps the handoff into source AST
  for runtime consumers.

### Test Requirements

1. **Constructor hostility:** downstream code attempting to construct or forge
   the sealed handoff from public fields must fail to compile.
2. **Loose-input hostility:** a runtime caller supplying a source AST or
   declaration vector directly must have no production preparation route.
3. **Authorship parity:** equivalent real-file and Rust-authored applications
   must produce the same canonical semantic identity and equivalent runtime
   preparation, while preserving distinct provenance.
4. **Formatting hostility:** whitespace, comments, and source ordering declared
   semantically irrelevant must not change semantic identity.
5. **Equal-digest hostility:** test support must force equal local digest
   representations for different semantic content and prove exact package
   authority prevents aliasing.
6. **Wrong-protocol hostility:** a sealed package from an unsupported DSL
   protocol/schema must deny before runtime candidate mutation and preserve the
   prior active generation.
7. **Failure-owner separation:** malformed syntax must stop in DSL; a valid
   package with unsupported runtime capability must stop in runtime, with
   distinct typed evidence.

### Open Questions

- Which existing package type should become the canonical handoff rather than
  creating a new nominal wrapper?
- Which semantic equivalence rules are already authoritative and which require
  explicit adjudication before parity can be claimed?
- Should runtime consume the package affinely, or may immutable packages be
  retried against multiple compatible runtime worlds?
- What is the minimum protocol/version posture required before Milestone 3.18
  adds modules and expansion provenance?

## Phase 4: Runtime Subsystem Authority Partition

### Relevant Subsystems and APIs

- active application session
- application preparation/replacement/publication
- graph/identity and dependency impact
- measurement/allocation/execution planning
- mounted-frame preparation/execution/publication/reconciliation
- host-report transport intake
- inspection evidence and relevance indexes
- runtime facades and internal module topology

### Required Work

Recompose `worth-ui-runtime` around the subsystem families frozen in Phase 1.

Each subsystem must have:

- one named state owner or explicit stateless contract;
- one public-to-runtime or internal facade that exposes transitions rather than
  fields;
- a declared set of authority inputs and outputs;
- a declared failure owner and preservation guarantee;
- an explicit cost lane;
- a future insertion statement naming which roadmap work belongs there; and
- a topology rule preventing reverse or lateral dependency shortcuts.

The active application session becomes a thin composition root that owns the
subsystem aggregates and coordinates their declared transition order. Its
ordinary mounted-frame method must:

1. validate exact session/request authority;
2. borrow only the subsystem capabilities required for the current phase;
3. execute the existing mounted-frame protocol;
4. commit or preserve predecessor truth according to the typed outcome; and
5. return the product-level outcome without leaking subsystem state.

Replace wide methods and free functions that accept the entire session/runtime
state when they need only a narrow capability. Use explicit borrowed capability
views whose names describe the permitted operation. Capability views may not
offer general field access or become generic context bags.

Move files to the subsystem that owns their primary semantic transition.
Split files that mix multiple named responsibilities. Delete obsolete
same-level buckets and forwarding modules.

For each proposed crate extraction, record:

- independent authority and lifecycle;
- stable directional API;
- dependency effect;
- compile-cost effect;
- cycle analysis; and
- why an internal module boundary is insufficient.

No extraction lands without that record. Conversely, a proven autonomous
boundary must not be rejected solely to preserve the current crate shape.

### Engineering Decisions

- The partition is semantic, not cosmetic: state fields, constructors,
  transitions, and tests move with their owner.
- Orchestration remains named business flow. It is not hidden in
  `helpers`, `common`, a service locator, or a generic context.
- Subsystems communicate through sealed receipts/handles and borrowed
  capabilities, not direct sibling field mutation.
- Inspection borrows authoritative state read-only and materializes compact
  projections; it does not own duplicate operational state.
- Host report transport may retain structurally validated batches, but semantic
  observation admission remains reserved for Milestone 3.12.
- Future-insertion rows name the exact seam a successor consumes; they do not
  assign every responsibility in a cross-boundary milestone to one runtime
  family. Milestone 3.17 keeps authored expression meaning in `worth-ui-dsl`
  while runtime evaluation enters through planning. Milestone 3.18 completes
  composition and module lowering before the sealed semantic handoff and
  therefore has no runtime-subsystem insertion.
- All touched production and test files comply with the workspace line cap
  unless an explicit existing exemption applies.

### Warnings

- Do not turn the active session into a god object with methods merely moved
  into extension traits.
- Do not create one facade per file or introduce ceremonial wrappers without
  authority value.
- Do not split mutually recursive modules into crates joined by a new shared
  type bag.
- Do not duplicate identities, counters, or state to make a boundary easier.
- Do not pull later snapshot, rebind, intent, services, or appearance meaning
  into this cleanup gate.

### Test Requirements

1. **Sibling-mutation hostility:** a subsystem attempting to mutate another
   subsystem's private state directly must fail compilation or topology checks.
2. **Whole-session hostility:** a new operation that accepts the entire active
   session despite needing only graph read authority must fail the capability-
   scope guard or review fixture.
3. **Thin-wrapper hostility:** moving a function behind a facade while it still
   reaches across multiple private subsystem trees must fail the dependency
   matrix.
4. **Atomicity:** denial at every subsystem transition must preserve the
   complete predecessor application, plan, allocation, mounted, and publication
   truth promised by the existing lifecycle.
5. **Inspection non-authority:** mutating or reconstructing operational truth
   from an inspection projection must be impossible through public types.
6. **Future insertion:** representative placeholder changes for Milestones
   3.11, 3.12, 3.17, and 3.18 must each have one unambiguous owner without
   creating reverse dependencies.
7. **Cycle hostility:** a fixture that creates a graph-to-mounting-to-graph
   dependency cycle must fail the topology guard even if each edge passes
   through a facade module.

### Open Questions

- Which subsystem operations require affine authority and which safely borrow?
- Does mounting own host report structural validation, or should observation
  own it while mounting retains frame correlation?
- Which current runtime facades become internal subsystem facades, and which
  disappear entirely?
- Does any autonomous boundary justify a new crate after cost and cycle
  analysis?

## Phase 5: Product Facade Condensation

### Relevant Subsystems and APIs

- `worth-ui::facade::app`
- existing mounted/runtime facade exports
- `execute_mounted_frame(...)`
- application preparation, launch, replacement, and active session
- mounted-frame request, outcome, stop, continuation, and recovery types
- inspection and host-contract audience facades
- product facade contract tests

### Required Work

Implement the Phase 1 public-surface manifest as an explicit product contract.

The ordinary app journey must read coherently as:

```rust
let candidate = app.prepare(authored_input)?;
let mut session = app.activate(candidate)?;
let outcome = session.execute_mounted_frame(frame_request);
```

The exact existing names may be retained when they already express the
contract. The semantic requirements are:

- application preparation/activation/replacement begins at an app-owned entry;
- the active session owns execution authority;
- a caller submits one mounted-frame request;
- one mounted-frame outcome describes success, unchanged reuse, rejection
  before effects, bounded in-flight posture, publication, or indeterminate
  presentation as defined by Milestone 3.10;
- next actions are available only from sealed handles carried by that outcome;
  and
- ordinary callers never assemble internal transition phases.

Split audiences explicitly:

- `facade::app` for ordinary application authors and embedders;
- `facade::inspection` for compact read-only evidence access;
- host-contract surfaces in `worth-ui-host-contract`, consumed by adapters and
  runtime integration rather than re-exported wholesale to applications; and
- support/certification authority behind its existing gated owner.

Reduce or remove `facade::mounted` and `facade::runtime` as public product
surfaces. If either name remains, its allowlist must represent a real advanced
audience with a documented caller and may not expose phase constructors.

For every removed export, migrate production callers to:

- the ordinary app facade;
- the inspection audience;
- the host contract;
- an internal runtime subsystem facade; or
- support authority.

Do not replace removed exports with root-level re-exports.

Add a checked-in API manifest that records symbol name, owning audience,
stability posture, and the real caller journey requiring it. The manifest must
be compared mechanically in CI so a new public export requires deliberate
review.

### Engineering Decisions

- The app facade optimizes for progressive disclosure: the common lifecycle is
  visible without teaching internal phase vocabulary.
- A typed denial or outcome required for exhaustive caller branching is public;
  the internal evidence used to derive it is not automatically public.
- Continuation/recovery handles are affine when duplicate use would violate
  protocol or exact-once effects.
- Inspection IDs/references may be public while storage, indexes, and
  materializers remain private.
- Adapter implementers consume the host contract directly. The product facade
  does not become a convenience mirror of that contract.
- Public fields are avoided where they would permit authority reconstruction;
  read-only observations use narrow accessors.

### Warnings

- Do not measure success only by reducing export count.
- Do not hide the same broad surface behind one prelude, root module, trait
  object, or generic request enum.
- Do not force ordinary app callers to import runtime, host, Query, or
  certification crates.
- Do not expose internals solely so external integration tests can construct
  fixtures.
- Do not flatten distinct mounted outcomes to `Result<(), Error>`.

### Test Requirements

1. **Ordinary journey:** a downstream fixture must prepare, activate, replace,
   execute a frame, branch on all public outcome families, and inspect compact
   evidence using only documented product facades.
2. **Phase-construction hostility:** downstream code attempting to construct a
   prepared frame, publication attempt, reconciliation input, mounted identity
   index, or cost evidence directly must fail to compile.
3. **Mid-protocol hostility:** downstream code attempting to begin publication,
   reconciliation, or retry without the sealed handle returned by
   `execute_mounted_frame(...)` must fail to compile.
4. **Audience hostility:** ordinary app code importing raw host mechanics,
   certification authority, or internal runtime facades through `worth-ui`
   must fail the public-surface guard.
5. **Manifest growth hostility:** adding a public re-export without a manifest
   row and named real caller must fail CI.
6. **Outcome honesty:** rejection-before-effects, in-flight, complete
   presentation/publication, unchanged reuse, and indeterminate posture remain
   distinguishable through the condensed facade.
7. **No ceremony regression:** a Query-free headless app uses the ordinary
   facade without dummy Query, inspection, adapter, or recovery setup.

### Open Questions

- Does any real external caller require a public advanced mounted audience after
  the ordinary path and typed continuation handles are complete?
- Which mounted receipt fields are stable product evidence versus inspection-
  only detail?
- Should application preparation be a free facade function, a builder, or an
  application owner method based on current lifecycle authority?
- Which compatibility removals require a coordinated version boundary for
  downstream workspace crates?

## Phase 6: Predecessor-Route Removal and Mechanical Enforcement

### Relevant Subsystems and APIs

- obsolete source/runtime preparation routes
- obsolete framework-turn or mounted lifecycle entry points
- deprecated aliases and forwarding wrappers
- root/facade re-exports
- workspace topology and boundary checker
- compile-fail/compile-pass contract owner
- production-source reachability and public API manifests

### Required Work

Delete every Phase 1 row marked `remove` and close every transitional row.

Required removals include, where the inventory confirms their current
existence:

- runtime-owned parser, source AST, language legality, and semantic lowering
  modules;
- loose-declaration or source-AST runtime preparation entry points;
- independent public free functions that enter mounted execution after the
  ordinary start or before a typed continuation;
- public constructors for intermediate graph/plan/allocation/mounted/
  publication authority;
- broad glob/prelude re-exports of runtime or mounted internals;
- compatibility aliases that preserve predecessor lifecycle names;
- production constructors used only by tests; and
- unreferenced forwarding modules left behind by physical moves.

Extend mechanical enforcement with:

- a dependency/topology rule that `worth-ui-dsl` cannot depend on runtime,
  Query, host, mounted, active-session, or certification owners;
- an ownership audit that authored tokenization, parsing, source AST, authored
  legality, and semantic lowering cannot be implemented under runtime;
- an inverse audit that runtime generation, plan, allocation, mounted,
  publication, and host-session authority cannot be implemented under DSL;
- an exact ordinary-product public-surface manifest;
- a callable-entry audit proving one ordinary mounted-frame route;
- a runtime subsystem dependency matrix;
- production-source reachability;
- compile-fail cases for authority forgery and midpoint entry; and
- line-cap and no-catch-all module checks.

The ownership audit must combine syntax/AST inspection, dependency edges,
constructor ownership, and manual ledger closure. Filename/token matching alone
is insufficient.

### Engineering Decisions

- There is no deprecation window inside the repository for an authority-bypass
  route.
- Test-support authority may expose fixtures only through its existing
  non-production gate; it may not re-export production constructors that remain
  otherwise public.
- Generated agent-context files are regenerated by tooling and never hand
  edited.
- Enforcement errors name the violated semantic boundary and expected owner,
  not only a banned token.

### Warnings

- Do not leave private dead copies "for reference."
- Do not keep old routes behind undocumented feature flags.
- Do not satisfy the one-entry audit by having several functions call the same
  implementation.
- Do not add another Cargo session, generated fixture workspace, or integration
  target for each compile case.
- Do not manually edit generated context or allowlist output.

### Test Requirements

1. **Alias hostility:** an old lifecycle function reintroduced as a deprecated
   alias must fail the callable-entry/public-surface checks.
2. **Forwarding hostility:** a runtime `parse_source` wrapper that simply calls
   DSL must still fail because runtime must not present itself as language
   authority.
3. **Feature-flag hostility:** a predecessor route hidden behind a non-default
   production feature must still fail reachability and public-surface checks.
4. **Test-authority hostility:** certification code attempting to restore a
   public production constructor for fixture convenience must fail compile
   contracts.
5. **Inverse ownership:** DSL code attempting to mint a runtime generation,
   execution plan, mounted receipt, or publication witness must fail topology
   or compile checks.
6. **Entry count:** structural inspection plus compile fixtures must prove that
   ordinary downstream code has exactly one callable mounted-frame start and no
   callable midpoint.
7. **Guard mutation:** deliberately mutate each new manifest/guard fixture and
   prove the owning CI command turns red for the intended reason.

### Open Questions

- Which existing enforcement tool should own semantic source-boundary auditing?
- Can the public API manifest be derived from rustdoc JSON while retaining an
  adjudicated audience file as authority?
- Which predecessor names require updates in examples or external workspace
  consumers before deletion?
- Are any obsolete routes required by a non-production forensic tool that
  should instead consume certification support authority?

## Phase 7: Real-Lifecycle and Cost Certification

### Relevant Subsystems and APIs

- real filesystem and watcher source ingress
- Rust-authored composition
- application prepare/activate/replace lifecycle
- headless and egui host adapters
- mounted-frame execution and typed recovery/reconciliation
- Query-free and Query-bound existing application paths
- runtime/DSL counters, allocator observer, and compile/build measurements
- consolidated application and compile-contract owners

### Required Work

Prove the corrected architecture through production mechanisms and independent
oracles.

The primary hostile scenario must:

1. create a real file-authored application on disk;
2. have production filesystem/watcher code acquire and observe it;
3. cross the DSL-owned parse/legality/lowering boundary;
4. prepare and activate it through the condensed app facade;
5. execute ordinary, virtualized, canvas, and realtime mounted work across the
   existing host-neutral contract;
6. perform a semantically equivalent Rust-authored replacement;
7. perform a valid local file edit;
8. perform an invalid syntax edit;
9. perform a valid syntax edit that runtime capability admission denies;
10. exercise unchanged reuse, pre-effect denial, bounded in-flight work, and
    reconciliation/indeterminate behavior already supported by Milestone 3.10;
11. inspect compact provenance, transition, and mounted evidence; and
12. prove cleanup and predecessor preservation.

Use existing consolidated test owners. Add cases/modules rather than new
integration targets or nested Cargo invocations.

Required independent evidence:

- an authored semantic model independent of production lowering for selected
  fixtures;
- a public-facade downstream compile owner;
- headless transcript comparison;
- at least one real egui context frame;
- an independent thread-scoped allocation observer;
- structural counters for DSL preparation/replacement and steady frames;
- dependency/topology and public-surface manifests; and
- comparable baseline/closing build timing.

Record costs separately for:

- initial file acquisition and DSL lowering;
- Rust-authored canonicalization;
- valid local replacement;
- invalid syntax denial;
- runtime capability denial;
- unchanged steady frame;
- changed mounted frame;
- inspection materialization; and
- warm/cold verification lanes.

### Engineering Decisions

- Production counters are evidence but not their own oracle.
- Real filesystem/watcher and real egui/headless paths remain separate claims;
  a scripted host fault adapter certifies protocol branches but not native
  effects.
- Ordinary fast tests use deterministic production boundaries without
  substituting fake authority for the claim under test.
- Large-scale fixtures must vary source size, declaration count, graph width,
  mounted breadth, and changed scope independently enough to expose hidden broad
  work.
- Any cost amendment names the changed contract, measured evidence, and reason;
  it cannot silently weaken an existing guarantee.

### Warnings

- Do not certify the filesystem claim with `with_file`-style source injection
  or manufactured watcher events.
- Do not certify DSL/runtime separation with only unit tests inside either
  crate.
- Do not use production equality/hash/counters as the only equivalence oracle.
- Do not hide cross-crate allocation behind an arena reset or exclude it from
  steady-frame accounting.
- Do not expand the ordinary fast lane with flaky OS waits or retries.

### Test Requirements

1. **Hot-frame source hostility:** poison or disable all DSL parsing/lowering
   after activation; repeated unchanged and changed frames must continue without
   touching the poisoned path.
2. **Unrelated-source scaling:** grow unrelated source/declaration content while
   holding the admitted changed scope constant; local replacement and frames
   must not scale with unrelated breadth beyond the existing declared contract.
3. **Invalid edit preservation:** malformed source observed through the real
   watcher must stop at DSL and preserve the prior active and mounted
   generation.
4. **Runtime denial preservation:** syntactically valid source requiring an
   unsupported runtime capability must pass DSL, deny in runtime, and preserve
   prior truth.
5. **Facade-only lifecycle:** the complete scenario must compile and run without
   importing internal runtime or mounted phase modules.
6. **Adapter parity:** headless and real egui execution consume the same sealed
   mounted meaning; neither can access DSL source or semantic package authority.
7. **Allocation hostility:** steady unchanged execution must meet the existing
   allocation claim under an independent observer after code movement.
8. **Build-budget hostility:** compile contracts remain within the existing two
   Cargo invocations and target budget; a proposed extra target must fail the
   budget guard.
9. **Interrupted replacement:** interruption at each handoff/preparation/
   publication boundary must leave one complete predecessor generation and no
   mixed subsystem authority.

### Open Questions

- Which existing consolidated scenario is the best owner for the full
  file-to-mounted journey?
- What scale factors expose source-width, graph-width, mounted-width, and
  changed-scope complexity independently with acceptable test time?
- Which existing allocator and timing harnesses already provide comparable
  baseline evidence?
- Which Milestone 3.10 in-flight/indeterminate branches can be exercised by
  production headless/egui mechanisms versus protocol fault injection?

## Phase 8: Documentation and Closeout

### Relevant Subsystems and APIs

- subsystem `AGENT_CONTEXT.md` generation inputs
- public product lifecycle documentation and examples
- DSL ownership and runtime handoff documentation
- runtime subsystem topology documentation
- roadmap and milestone ledgers
- boundary, quality, line-cap, compile-contract, and certification commands

### Required Work

Update documentation only after the code and enforcement describe one closed
architecture.

Required documentation:

- one source-to-mounted architecture overview naming every authority owner;
- one DSL authoring/lowering guide showing file and Rust convergence;
- one ordinary application lifecycle example using only `facade::app`;
- one advanced outcome/recovery example using only typed handles returned by the
  ordinary path;
- one inspection example that does not import operational internals;
- one runtime subsystem map with allowed dependency directions and future
  insertion points;
- migration notes listing removed public routes and their intended audience
  replacements;
- generated per-crate agent context from the enforcement tool; and
- the final Phase 1 ledgers with every row closed.

Perform a fresh adversarial review of the exact closing source. Reopen any
acceptance guarantee affected by the final fixes and rerun its causal evidence.

Mark the milestone complete only after:

- no inventory row remains ambiguous or transitional;
- all planned removals are gone;
- all new guards have a proved red mutation;
- all required verification is green;
- baseline/closing cost evidence is recorded and adjudicated;
- roadmap status is updated; and
- Milestone 3.11 can begin using only the new ownership and public surfaces.

### Engineering Decisions

- Documentation names semantic owners and caller journeys, not directory tours.
- Removed APIs are not documented as deprecated alternatives.
- Generated agent context is accepted only from the enforcement command.
- Closeout evidence records exact commands and source revision.
- If the final implementation differs from a specification name, the closeout
  ledger records the adjudicated equivalent and why it better expresses the
  same responsibility.

### Warnings

- Do not close on green tests if the inventories still contain transitional
  rows.
- Do not document private internals as a workaround for a missing public
  journey.
- Do not hand-edit generated agent context.
- Do not begin Milestone 3.11 against a compatibility facade scheduled for
  later removal.
- Do not claim performance preservation without comparable measurements.

### Test Requirements

1. **Fresh-reader journey:** a downstream fixture derived only from the
   lifecycle documentation must compile and execute the real ordinary path.
2. **Stale-doc hostility:** examples using a removed predecessor route must fail
   documentation/API checks rather than compiling through an alias.
3. **Insertion audit:** reviewers must place representative Milestone 3.11,
   3.12, 3.17, and 3.18 changes on the subsystem map without ambiguity or
   reverse dependency.
4. **Ledger closure:** CI must reject `ambiguous`, `transitional`, missing
   disposition, or missing evidence fields in the closing inventories.
5. **Generated-context drift:** modifying a generated agent-context file by hand
   must be detected by the agent-context check.
6. **Exact-source rerun:** all boundary, compile, certification, cost, and
   quality evidence must be rerun after the last causally relevant fix.

### Open Questions

- Which public guide becomes the canonical ordinary lifecycle entry point?
- How should removed routes be communicated to downstream workspace consumers
  without preserving compatibility authority?
- Which runtime subsystem map format is both human-readable and mechanically
  comparable?
- What exact evidence bundle is required for Milestone 3.11's start gate?

## Cross-Phase Acceptance Ledger

The implementation ledger must track each guarantee below from first
establishment through final closeout. A later phase that changes a causal
dependency reopens the guarantee.

| Guarantee | Established | Reopened by | Closing evidence |
| --- | --- | --- | --- |
| DSL is the sole authored-language semantic owner | Phase 2 | Phases 3, 6 | ownership audit, topology, hostile compile cases |
| File and Rust authoring converge before runtime | Phase 3 | Phases 5, 7 | parity model and real lifecycle |
| Runtime consumes one sealed semantic handoff | Phase 3 | Phases 4, 6 | constructor/loose-input compile failures |
| Runtime subsystem ownership is explicit and acyclic | Phase 4 | Phases 5, 6 | dependency matrix, sibling-mutation tests |
| Active session is a thin composition root | Phase 4 | Phases 5, 7 | capability-scope audit and lifecycle evidence |
| Ordinary callers have one mounted-frame route | Phase 5 | Phase 6 | callable-entry audit and downstream fixture |
| Product facade exposes no intermediate authority | Phase 5 | Phases 6, 8 | API manifest and compile-fail suite |
| Predecessor routes are absent | Phase 6 | Phase 8 | reachability, alias/feature hostility |
| Steady frames perform no source work | Phase 7 | Phase 8 | poisoned DSL path and independent counters |
| Existing frame/build cost posture is preserved | Phase 7 | Phase 8 | comparable baseline/closing measurements |
| Later milestones have one honest insertion point | Phase 4 | Phase 8 | representative insertion audit |
| Documentation describes only the closed path | Phase 8 | — | doc fixture and exact-source review |

## Required Verification

The implementation closeout must run, at minimum:

```text
cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .
cargo run --manifest-path tools/agent-context/Cargo.toml -- check
```

It must also run the workspace's authoritative commands for:

- DSL/runtime semantic ownership auditing;
- runtime subsystem topology;
- ordinary product public-surface manifest;
- production-source reachability;
- workspace Rust line caps;
- formatting and linting;
- workspace compilation and tests;
- the existing batched negative/positive compile-contract owner;
- application contract certification;
- real filesystem/watcher scenarios;
- headless and real egui mounted execution;
- independent allocation observation; and
- comparable warm targeted, warm fast-lane, and isolated cold build timing.

The Phase 1 inventory must record the exact repository commands rather than
inventing parallel scripts when an owner already exists.

## Completion Standard

Milestone 3.10.1 is complete only when all of the following are true:

- `worth-ui-dsl` physically and semantically owns every production authored
  syntax, source AST, language legality, source diagnostic, normalization, and
  authored-to-canonical lowering responsibility;
- runtime owns no parser, source semantic lowerer, or callable facade that
  presents it as the language owner;
- file-authored and Rust-authored composition produce one sealed canonical
  runtime handoff through production paths;
- runtime preparation accepts no loose source/AST/declaration bypass;
- runtime subsystem state, transitions, failure owners, costs, and dependency
  directions are explicit and mechanically enforced;
- any crate extraction is justified by autonomous authority/lifecycle, and no
  arbitrary split or new cycle was introduced;
- the ordinary product lifecycle uses one `execute_mounted_frame(...)` route;
- public callers cannot construct or enter intermediate mounted/runtime phases;
- predecessor aliases, wrappers, and compatibility routes are deleted;
- real filesystem, Rust-authored, headless, egui, replacement, denial,
  publication, and inspection scenarios pass through the condensed facades;
- steady frames prove no source work, broad source scan, or migration-induced
  allocation;
- build/test topology and measured iteration cost remain within adjudicated
  budgets;
- every inventory and acceptance-ledger row is closed on the exact final source;
  and
- Milestone 3.11 can consume mounted truth without importing DSL internals,
  runtime internals, or transitional public authority.
