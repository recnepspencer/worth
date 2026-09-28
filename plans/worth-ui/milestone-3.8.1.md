# Milestone 3.8.1: Test-Program Topology and Runtime Authority Closure Gate

> Historical QA policy (2026-08-22): proof, closure, migration, acceptance,
> and phase ledgers described below are frozen historical records. They are not
> active implementation or release gates, are not updated or reopened, and a
> ledger-only failure does not block current work. Current evidence follows
> [the QA review guide](../../docs/coding-guidelines/qa_review_guide.md) and
> [testing laws](../../docs/coding-guidelines/testing_laws.md): specifications state QA
> considerations in prose, tests and repository checks run against the current
> commit, and code review decides whether the evidence is adequate. This note
> does not retire product-domain ledgers that are part of runtime behavior.

**Status:** Complete. Phases 1 through 14 and every closure gate are green.
Milestone 3.9 is unblocked.

**Phase 1-4 closeout:** The 161-target legacy proof program is reconciled into
12 explicit integration binaries, three trybuild sessions, zero ordinary
nested Cargo invocations, one compiled-once repeated scenario, and one
suite-owned source inventory. On the same Windows/toolchain posture, cold
no-run compilation fell from 502.58 seconds to 157.49 seconds (31.34%), and
median warm full closure fell from 976.0 seconds to 210.401 seconds (21.56%).
The machine-readable evidence is
`milestone-3.8.1-test-topology-closeout.json`; the 161-row authority ledger is
`milestone-3.8.1-test-proof-migration.csv`.

**Phase 5-7 closeout:** Application freeze now seals one move-only prepared
authority containing the capability, origin-typed application artifact,
declaration, graph, Query, host-session-plan, lifecycle, and derived-index
truth for one generation. Declaration-authored applications retain their real
DSL package; file-backed and Rust-authored ingress retain the admitted
candidate artifact and declaration meaning as one candidate composition
through preparation. Freeze returns typed, phase-local denials without
publishing partial authority. Runtime, hostile certification,
compile-contract, dependency-contract, topology, line-cap, Clippy, and
constitutional checks cover the cutover.

**Phase 8-10 closeout:** Prepared authority now launches by consumption into
one active application session with an opaque session identity. Replacement
preparation carries that exact session identity and a sealed application-
generation basis through lowering and staging; equal semantic digests cannot
move prepared, lowered, or pending authority between sessions. Candidate graph
touches, mounted-to-layout transitions, successor indexes, and allocation
catalogs are candidate-owned and committed before cutover. Exact graph
ownership is carried by an opaque authority identity, so an independently
committed graph with the same semantic digest cannot supply cutover proof. One
frame-boundary transition then publishes the artifact, declaration, graph,
catalog, Query, runtime, and inspection generation together; denial and no-op
paths retain the prior generation and host session. The configured host
adapter is retained as the actual operational adapter behind a session-scoped
capability, with real headless and egui lifecycle parity. Raw runtime launch,
source ingress, staging, catalog activation, and host-submission bypasses are
not ordinary facade surfaces. Cross-session hostile tests, a 1,000-step mixed
replacement storm, host-adapter integration tests, and compile-fail contracts
cover these boundaries.

**Phase 11-12 closeout:** Framework turns now seal a mutation-free transition
plan before execution. The plan carries source order, application generation,
policy family, invalidation narrowing, neighborhood selection, receipt basis,
resize facts, and exact breadth counters. Ordinary allocation, viewport,
preview, durable-resize, and drag-resize mutations live in policy-specific
executors with narrow authority, while one thin owner retains source
collection, ingress closure, exactly-once pumping, execution dispatch, and
completion publication. Planning denials and execution denials publish no
partial ledger, invalidation, or durable-state truth.

**Phase 13-14 closeout:** Uncompiled backdrop, metal, shader-pipeline, and
related GPU/theme residue are removed. Boundary-check now rejects production
Rust sources outside every declared module/target graph while understanding
ordinary modules, explicit path selection, additional targets, and narrowly
reviewed generated-source exemptions. Production-path certification exercises
equivalent file/Rust composition, an operational host session with admitted
measurement evidence, valid, invalid, and no-op replacement, Query admission,
representative allocation, and active inspection under one application
generation. Structural audits pin
one preparation, launch, host-activation, and replacement-cutover owner; the
workspace test, strict Clippy, topology-budget, line-cap, boundary, and
agent-context gates are green.

## Goal

First make the Worth UI test program mechanically efficient enough to support
honest repeated certification, then close the remaining composition seams in
the shipped 3.8 substrate so one prepared application authority, one active
runtime generation, one inspection truth, and one admitted host session cannot
drift apart.

## Why This Milestone Exists

Milestone 3.8 shipped real declaration, graph, Query-binding, allocation, and
runtime machinery, but its test program accumulated a structural execution
problem: `worth-ui` and `worth-ui-certification` expose 160 integration-test
binaries, compile contracts are spread across 58 trybuild-bearing harness
files, repeated certification scenarios are rebuilt instead of owned once,
and nested Cargo invocations create parallel build economies. The accumulated
trybuild target already exceeds 12 GiB. This makes the proof program too slow
to run as the ordinary development loop and rewards narrow or skipped QA.

The same code-level end-to-end audit found five production structural gaps:
candidate and graph truth can be separated, configured host posture is not
operationally enforced, public freeze converts admission denial into panic,
the ordinary framework-turn owner contains too much policy execution, and
dormant GPU/theme files exist outside the compiled module graph.

Milestone 3.9 would make both the slow proof topology and the production seams
more expensive by lowering additional execution authority onto them. This
milestone therefore repairs test execution first, without weakening proof,
then closes the production seams before execution-plan work continues.

## Governing Summaries

- `MENTALITY.md` protects foundation-first, adversarial, mechanically enforced
  correctness. The strongest effect here is that the proof infrastructure must
  be repaired before dependent implementation continues, then split
  application truth must become unrepresentable.
- `arch_laws.md` protects contractual facades, proof-bearing phase progression,
  typed denials, and pre-resolved execution. Launch, freeze, host admission, and
  framework-turn transitions must consume types carrying the exact proof each
  preceding phase established.
- `composition_laws.md` protects named semantic steps and reviewable
  responsibilities in production and tests. Test binaries may aggregate a
  coherent semantic suite, but test modules and scenario builders must still
  own one predictable responsibility; neither giant harnesses nor generic
  fixture bags are acceptable.
- `domain_structure_laws.md` protects physical boundaries that preserve
  authority, truth source, lifecycle, failure meaning, and test strategy. Test
  topology must falsify production topology, and shared support must live at
  the narrowest real semantic authority rather than behind a global test world.
- `perf_laws.md` protects bounded execution, amortization across honest batch
  boundaries, and measured claims. Compiler sessions, integration binaries,
  source scans, and fixture construction are test execution breadth and must
  not scale with individual assertions when one semantic suite can share them.
- The Worth UI roadmap protects one canonical artifact pipeline,
  runtime-owned meaning, host adapters as native-mechanics translators, and
  steady-state execution free of semantic rediscovery. It requires this closure
  gate immediately after 3.8 and before 3.9.

## Adversarial Constraint

Under arbitrary test selection and repeated local or CI execution, the full
proof program must retain its hostile coverage while avoiding per-assertion
compiler sessions, per-file integration linking, repeated source discovery,
fixture-local build graphs, and repeated construction of semantically identical
certification worlds.

Under arbitrary valid and invalid source replacements, arbitrary framework-turn
source pressure, and arbitrary attempts to mix otherwise-valid application
parts, it must also be mechanically impossible for the active artifact,
declaration authority, committed graph, Query binding, inspection surfaces,
capability snapshot, or host session to describe different application
generations.

Every denial must preserve the last complete admitted authority without panic,
partial publication, adapter substitution, or derived-state residue.

## Product Decision Lock

- One sealed, move-only prepared application authority is the only ordinary
  input to runtime launch and replacement staging.
- Test-topology repair is the first milestone priority. No production-authority
  phase may begin until the baseline, compile-contract, certification-suite,
  and build-boundary phases are closed.
- Proof strength may not be traded for speed. Every moved, consolidated,
  replaced, or deleted test must appear in a mechanical migration ledger that
  names the surviving proof or explains why a stronger proof subsumes it.
- Compile contracts are batched by semantic boundary. A compiler invocation is
  not an assertion boundary, and private-field repetition is not a substitute
  for one load-bearing architectural contract.
- Shared certification setup is compiled once only when it has shared semantic
  authority. Generic `fixtures`, `helpers`, `world`, or catch-all scenario
  modules are forbidden.
- Ordinary workspace test execution launches no nested Cargo build and creates
  no fixture-local target directory. Truly distinct compile environments live
  in explicit, separately named proof lanes.
- Required premerge closure is one aggregate over independently executed
  semantic proof lanes. The serial `full` lane remains the exact local/manual
  reproduction of runtime proof closure, but CI must not rerun it after already
  executing every component lane in the same workflow.
- CI pins its Rust toolchain, does not persist the workspace target directory,
  records machine-readable lane timing and compiler-cache evidence, and checks
  at least Linux and Windows compilation posture.
- A digest may identify or compare that authority, but a raw digest is never
  authority and cannot open a launch, replacement, host, or inspection door.
- File-authored candidate truth and its source-backed declaration/graph truth
  remain inseparable through preparation and cutover.
- Active inspection always describes the same committed generation the runtime
  executes. Candidate inspection is explicitly candidate-scoped and cannot
  masquerade as active truth.
- Host selection is a real lifecycle decision. Per-turn measurement access is
  borrowed from the admitted host session; callers cannot substitute an
  unrelated adapter at the source boundary.
- Publicly constructible invalid input returns typed denial. Panic is reserved
  for unreachable internal invariant corruption, not ordinary declaration or
  topology rejection.
- The framework-turn owner remains the single clock and close/pump authority,
  while policy classification and policy-family execution move into named,
  typed transitions.
- Dormant rendering code is not preserved as speculative inventory. Uncompiled
  GPU/theme implementations are deleted; future rendering work must re-enter
  through the roadmap-owned host and appearance milestones.
- No compatibility facade, deprecated split lane, optional legacy constructor,
  or feature-gated bypass may preserve the old composition model.
- Milestone 3.9 remains blocked until every phase and acceptance gate here is
  closed.

## Phase Plan

### Phase 1: Test-Cost Baseline and Execution-Lane Contract

Freeze the current proof inventory and make its compilation, linking, process,
disk, and execution breadth measurable before any test or production topology
changes. This is the milestone's first implementation phase and blocks every
later phase.

**Relevant subsystems**

- Worth UI workspace manifest and Cargo profiles
- Cargo test-target metadata
- local developer test commands
- CI orchestration
- isolated target-directory measurement
- test-proof migration inventory

**Relevant APIs and source surfaces**

- `workspaces/worth-ui/Cargo.toml`
- root and Worth UI CI configuration
- `cargo metadata` test-target inventory
- trybuild harness and fixture inventory
- new machine-readable test-topology budget and proof-migration ledger

**Required design**

- Record the current test inventory by package, target, semantic suite,
  trybuild fixture, nested Cargo invocation, scenario-support owner, and proof
  family. Every later move or deletion must reconcile against this inventory.
- Measure cold no-run compilation, warm full-suite execution, compile-contract
  execution, generated-artifact size, and process/target breadth in a fresh
  isolated `CARGO_TARGET_DIR`. Record three comparable runs where warm/cold
  posture requires a median rather than one anecdotal sample.
- Add Worth UI-owned `dev` and `test` profiles that avoid unnecessary Windows
  debug/PDB amplification while retaining useful failure backtraces. The
  nested workspace must not rely on root-workspace profiles it cannot inherit.
- Define explicit fast developer, compile-contract, hostile-certification,
  documentation, and dependency-contract lanes plus one serial `full`
  reproduction. The required CI aggregate's union and the serial full runtime
  lane must cover the same reconciled runtime proof inventory; lane naming
  cannot hide omitted tests.
- Install hard topology budgets: at most 6 `worth-ui` integration targets, at
  most 10 `worth-ui-certification` integration targets, at most 4
  compile-contract binaries workspace-wide, and zero nested Cargo invocations
  in ordinary workspace test execution.
- Preserve `worth-ui-runtime` unit tests as a library-test compilation unit
  unless a real external-consumer boundary requires otherwise. Moving unit
  tests into new integration binaries is not optimization.

**Warnings**

- Do not begin by deleting tests or introducing an alternative test runner.
  Scheduling cannot repair compilation and linking amplification.
- Do not measure against the existing accumulated target directory; it cannot
  distinguish current-run work from historical artifact churn.
- Do not set an elapsed-time budget without recording machine, profile, target
  posture, and exact command. A performance claim is valid only at its named
  boundary.
- Do not let a fast lane become the only lane run by automation. Fast feedback
  and aggregate complete closure are separate obligations.

**Test requirements**

- Inventory convergence: the union of declared lanes resolves to every current
  test target and proof family exactly once, with no silently unowned target.
- Closure equivalence: the required parallel aggregate and serial `full`
  reproduction retain the same reconciled runtime proof families, including
  all-feature library, integration, documentation, and external dependency
  contracts.
- Budget rejection: an intentionally added integration target, trybuild
  harness, or ordinary nested Cargo invocation causes the topology check to
  fail with the responsible package and path.
- Measurement replay: repeated isolated measurements preserve the same target
  and process counts and classify cold versus warm posture explicitly.
- Profile honesty: the Worth UI workspace proves its own dev/test profile
  settings without depending on the parent workspace manifest.

**Engineering decisions**

- Target, process, and fixture breadth are first-class test-performance
  counters, not incidental Cargo details.
- The migration ledger is authoritative for proof preservation during Phases
  2 through 4 and is retired only after every row maps to the final topology.

**Open questions**

- None.

### Phase 2: Compile-Contract Batching and Proof Deduplication

Collapse compile-time boundary proof into a small number of semantic harnesses
without weakening construction, visibility, typestate, or authority denials.

**Relevant subsystems**

- `worth-ui` public facade compile contracts
- `worth-ui-certification` compile contracts
- `worth-ui-host-contract` public boundary contracts
- trybuild fixtures and stderr snapshots
- public-surface and dependency-boundary structural audits

**Relevant APIs and source surfaces**

- `worth-ui/tests/*compile*.rs`
- `worth-ui-certification/tests/*compile*.rs`
- both `tests/trybuild_support.rs` copies
- `tests/ui/{pass,fail}` semantic fixture trees
- compile-contract rows in the proof-migration ledger

**Required design**

- Replace per-fixture and per-family harness fragmentation with no more than
  four workspace-wide compile-contract binaries, organized by real authority
  boundary rather than milestone provenance.
- Run one `trybuild::TestCases` session per semantic suite and batch compatible
  pass/fail fixtures through stable directory globs. An individual assertion
  must not create a fresh compiler session merely to preserve a test name.
- Remove runtime `RUSTFLAGS` mutation from test support. Warning posture must
  remain explicit and must not fork the test dependency fingerprint from the
  ordinary workspace build.
- Reconcile every existing compile fixture through the migration ledger.
  Retain distinct relational type/phase incompatibilities; replace repeated
  private-field or non-export assertions only when one stronger structural or
  representative compiler proof subsumes the same invariant.
- Preserve semantic discoverability through fixture paths and module/test
  names even when compilation is batched. A failing fixture must still name
  the violated boundary directly.
- Keep pass fixtures only where compilation proves a capability not already
  exercised by an ordinary external-consumer journey.

**Warnings**

- Do not turn all compile failures into one opaque golden stderr blob. Each
  fixture remains independently attributable.
- Do not replace compiler-enforced authority denials with source-string scans
  when the compiler can prove the real relation.
- Do not preserve hundreds of structurally equivalent privacy assertions out
  of fear. Preserve the invariant with the smallest stronger proof family and
  record the substitution explicitly.
- Do not use glob batching across fixtures that genuinely require different
  features, target platforms, or dependency graphs; such a distinction must
  earn a separately named compile environment.

**Test requirements**

- Proof-ledger parity: every old compile-contract row maps to a retained
  fixture, a stronger replacement proof, or a reviewed redundancy deletion;
  no row disappears implicitly.
- Hostile authority matrix: raw IDs, digests, candidate witnesses, host
  adapters, and lower-phase artifacts remain unable to satisfy higher-authority
  APIs after batching.
- Session-budget rejection: adding a new standalone trybuild harness or
  per-fixture `TestCases` construction fails the topology gate.
- Diagnostic localization: each deliberately broken fixture reports its
  semantic boundary and expected compiler denial without depending on another
  fixture's ordering.

**Engineering decisions**

- Compiler proof is retained where invalid states should be unrepresentable;
  batching changes execution topology, not architectural enforcement.
- Public-surface inventory checks may complement trybuild but cannot replace
  relational compiler proof.

**Open questions**

- None.

### Phase 3: Certification Suite and Scenario Authority

Reduce integration-link amplification and give repeated certification setup
named, compiled-once ownership while preserving each hostile responsibility as
a small, independently navigable test module.

**Relevant subsystems**

- `worth-ui` facade and registry journeys
- `worth-ui-certification` runtime journeys
- certification scenario construction
- `worth-ui-test-support` synthetic authority
- domain-specific test modules and assertions

**Relevant APIs and source surfaces**

- `worth-ui/tests/*.rs`
- `worth-ui-certification/tests/*.rs`
- `worth-ui-certification/src`
- `worth-ui-test-support/src`
- repeated declaration, graph, Query, measurement, obligation, allocation,
  host, and inspection scenario builders

**Required design**

- Consolidate `worth-ui` integration targets into no more than six coherent
  facade, registry, and public-runtime suites. Child modules preserve one
  responsibility per file and remain subject to the workspace line cap.
- Consolidate `worth-ui-certification` into no more than ten domain suites,
  with distinct homes for declaration, graph, obligation, measurement,
  inspection, allocation/runtime, application lifecycle, and topology where
  their authority or failure fate differs.
- Move genuinely shared certification scenario construction into named
  modules compiled once as part of certification support. Scenario modules
  must describe the authority they assemble, such as declaration application,
  graph touch, or measurement admission; `fixtures`, `helpers`, `common`,
  `shared`, and global test-world bags are forbidden.
- Keep synthetic authority, hostile origin, and fault-injection capability in
  `worth-ui-test-support`. Ordinary scenario composition must use production
  facades and cannot mint admitted production artifacts.
- Keep the critical varying input, action, and assertion visible in each test
  body. Shared support may remove ceremony but may not conceal which production
  edge the test falsifies.
- Preserve stable domain-qualified test names so targeted developer commands
  remain possible after binary consolidation.

**Warnings**

- Do not produce one giant certification file or one global library-test blob.
  The goal is a small number of semantic compilation units, not erased test
  topology.
- Do not unify setup merely because its syntax looks similar. Shared scenario
  authority requires the same meaning, lifecycle, failure behavior, and proof
  strategy.
- Do not move production authority constructors into test support for
  convenience. Support authority can originate raw inputs and controlled
  faults, never the proof under test.
- Do not split `worth-ui-runtime` unit tests into external binaries to make the
  target list look uniform.

**Test requirements**

- Scenario convergence: the prior inline and final compiled-once construction
  of each representative declaration, graph, measurement, obligation, and
  inspection world produce equivalent production-path inputs and outcomes.
- Authority rejection: scenario support cannot construct prepared, active,
  admitted, committed, or generation-bound production authority without the
  proving production transition.
- Isolation hostility: a failure in one domain suite identifies that domain
  without requiring unrelated application worlds or cross-domain fixture
  initialization.
- Target-budget rejection: adding a new top-level integration binary when an
  existing semantic suite owns the responsibility fails the topology gate.

**Engineering decisions**

- Integration binaries are compilation boundaries and must be justified by
  external-consumer semantics or genuinely distinct build environments.
- Test modules remain fine-grained even when their containing binary is
  coarse enough to amortize compilation and linking honestly.

**Open questions**

- None.

### Phase 4: Build-Boundary Closure and Test-Program Certification

Remove parallel Cargo economies and repeated structural discovery, install the
final automated lanes, and prove that the repaired arrangement is both faster
and proof-equivalent before production-authority work begins.

**Relevant subsystems**

- custom compile-fail and adapter fixtures
- nested Cargo manifests and fixture-local target directories
- topology, residue, growth-posture, and source-reachability audits
- Worth UI CI lanes
- final test-topology budget and performance evidence

**Relevant APIs and source surfaces**

- `worth-ui-host-contract/tests/public_boundary_compile_fail.rs`
- `worth-ui-certification/tests/host_replaceability_compile.rs`
- certification topology/source audit entry points
- fixture manifests under `tests/ui` and `tests/fixtures`
- explicit Worth UI CI commands and topology checks

**Required design**

- Replace custom temporary-crate compilation with the consolidated compile
  contracts when it proves a language/API relation. Dependency-isolation
  claims must use a manifest/dependency-graph proof plus one explicitly named
  compile lane rather than repeated ordinary-suite `cargo check` subprocesses.
- Ordinary `cargo test --workspace` for the Worth UI workspace must launch no
  nested Cargo process and create no target directory beneath `tests`.
- Make topology and source audits consume one named workspace source inventory
  per suite instead of repeatedly walking and parsing the same tree from
  independent integration binaries. The inventory is derived evidence, not a
  new source of architectural truth.
- Add independent CI jobs for quality/topology, fast runtime plus documentation
  proof, compile contracts, hostile certification, the external dependency
  contract, and Windows all-target/all-feature compilation. One stable required
  aggregate status must fail unless every component job succeeds.
- Retain the serial `full` command as the exact local/manual runtime-proof
  reproduction. Do not execute it after the component lanes in the same CI
  workflow; redundant serial closure spends time without adding proof.
- Pin the Worth UI Rust toolchain, disable incremental compilation in ephemeral
  CI workers, share compiler objects through a content-addressed compiler cache,
  and cache Cargo sources rather than the multi-gigabyte workspace target tree.
- Emit machine-readable per-command and per-lane wall time together with
  compiler-cache statistics, including on ordinary command failure.
- Re-run the Phase 1 isolated measurements after migration. The median warm
  full-suite wall time must be at most 50 percent of baseline, and cold no-run
  compilation must be at most 60 percent of baseline, without reducing the
  reconciled proof inventory.
- Enforce the final structural budgets mechanically and publish target count,
  compiler-session count, nested-process count, generated-artifact size, and
  lane timing beside the proof-migration ledger.

**Warnings**

- Do not use shared `CARGO_TARGET_DIR` as a disguise for uncontrolled nested
  builds. Remove the process boundary unless a distinct compile environment is
  the actual proof.
- Do not cache source-scan results across repository revisions or trust
  boundaries. Reuse is valid only within one explicit suite inventory.
- Do not accept faster timing if hostile cases, feature combinations,
  documentation, dependency isolation, platform posture, or compile denials
  silently leave either the aggregate or serial reproduction.
- Do not begin Phase 5 while any proof-ledger row, target budget, nested build,
  or timing gate remains open.

**Test requirements**

- Full-proof equivalence: the required CI aggregate executes every reconciled
  proof family, while the serial `full` reproduction covers the same runtime
  inventory and reproduces the same admitted/denied outcomes as the baseline.
- CI-contract rejection: deleting a required component job, restoring serial
  `full` execution after the component lanes, floating the Rust toolchain, or
  caching `workspaces/worth-ui/target` fails the topology contract check.
- Platform-drift rejection: Windows all-target/all-feature compilation is a
  required aggregate member and cannot be skipped while Linux closure reports
  success.
- Parallel-economy rejection: a hostile fixture that launches nested Cargo or
  writes a fixture-local target is detected with the owning test path.
- Source-inventory convergence: multiple structural audits over one revision
  consume the same enumerated production-source set while retaining their
  independent domain verdicts.
- Performance gate: fresh isolated cold and warm runs satisfy the relative
  budgets and emit enough counters to explain any remaining dominant cost.

**Engineering decisions**

- A distinct build graph is allowed only as an explicit proof lane with named
  ownership; it is never hidden inside an ordinary test function.
- Phase 4 closes test-topology work. Later phases may add proofs only through
  the installed suite and budget contracts.

**Open questions**

- None.

### Phase 5: Canonical Prepared-Application Authority

Define the single authority object from which an active Worth UI application
may be launched. This phase freezes ownership and identity before changing
ingress, freeze, or runtime behavior.

**Relevant subsystems**

- Worth UI public facade and application lifecycle
- capability snapshot authority
- canonical artifact and declaration authority
- committed graph authority
- Query binding plan
- host contract/session plan
- inspection authority

**Relevant APIs**

- `WorthUiApp`
- `WorthUiRuntimeLaunch`
- `WorthUiSourceBackedDslPackage`
- `CapabilitySnapshot`
- `UiGraphSnapshot`
- `WorthUiQueryBindingPlan`
- `WorthUiHostContract`
- new sealed prepared-application authority and identity types

**Required design**

- Introduce one sealed prepared-application authority that owns the canonical
  artifact, admitted declaration artifacts, committed graph snapshot,
  capability snapshot, Query binding plan, host-session plan, and inspection
  indexes for exactly one application generation.
- Give the authority a typed generation identity derived during preparation.
  Expose comparison-safe identity only; do not expose constructors that accept
  raw component digests.
- Make the authority move-only. Components may expose read-only projections,
  but callers cannot extract independently launchable artifact, graph, Query,
  or host parts.
- Separate candidate-scoped projections from active projections in the type
  system. A candidate identity must never satisfy an active-authority API.
- Keep derived indexes rebuildable from the authority-owned canonical inputs.

**Warnings**

- Do not solve shared observation by making the entire authority freely
  cloneable. Observation handles must borrow or carry a generation-scoped
  witness without becoming a second owner of truth.
- Do not use an aggregate digest as the only enforcement. The sealed authority
  type, private fields, and constructor visibility are the enforcement; its
  identity is evidence and comparison material.
- Do not place the new object in a generic `state`, `bundle`, or `context`
  module. Its physical home must name prepared application authority.

**Test requirements**

- Compile-fail: external callers cannot construct prepared authority, mint its
  identity, replace one constituent part, or promote candidate identity into
  active identity.
- Adversarial drift: two preparations sharing a capability snapshot but
  differing in declarations, graph, Query binding, or host plan produce
  distinct authority identities and cannot exchange launch witnesses.
- Rebuild honesty: destroying every derived inspection/index projection and
  rebuilding it from prepared authority yields equivalent projections without
  consulting runtime-local residue.

**Engineering decisions**

- The prepared application authority is the milestone's canonical application
  artifact; all launch and replacement surfaces narrow around it.
- Public inspection may borrow generation-scoped projections, but it does not
  own or reconstruct active authority.

**Open questions**

- Final public type names may follow the existing facade vocabulary, but the
  prepared-versus-active distinction and sealed ownership are not negotiable.

### Phase 6: Inseparable Candidate Composition at Source Ingress

Make source ingress produce one composition input whose runtime artifact and
declaration/graph source cannot be separated or silently discarded.

**Relevant subsystems**

- watched source ingress
- file-authored lowering
- Rust-authored composition
- canonical artifact assembly
- source-backed declaration witness
- replacement candidate admission

**Relevant APIs**

- `WorthUiWatchedCandidateSubmission`
- `WorthUiReplacementCandidate`
- `WorthUiSourceBackedDslPackage`
- `lower_to_candidate_submission`
- `into_candidate`
- file/Rust replacement parity surfaces

**Required design**

- Replace the split `candidate` plus optional source-backed package posture
  with one sealed candidate-composition type whose variants carry every input
  required to prepare coherent application authority.
- Remove `into_candidate` and any equivalent accessor that can consume or copy
  the runtime candidate while dropping declaration/graph material.
- File-authored and Rust-authored paths must both name their declaration source
  explicitly and converge before application preparation. An `Option` must not
  hide a missing semantic lane.
- Preserve source revision, ordering receipt, provenance, and ingress counters
  on the inseparable composition object.
- Make candidate artifact identity and declaration-source identity part of one
  typed preparation basis, while preserving their distinct semantic types.

**Warnings**

- Do not repair only the currently observed reload-storm caller. The old split
  must become unavailable everywhere, including certification and support
  fixtures.
- Do not fabricate source-backed declarations from a finished canonical
  artifact after the boundary. Both must descend from the admitted structured
  input that actually established their equivalence.
- Do not preserve a certification-only constructor that can recreate the
  production bypass under another feature flag.

**Test requirements**

- File/Rust convergence: semantically equivalent file-authored and
  Rust-authored inputs produce equivalent candidate-composition authority and
  prepare equivalent application generations.
- Compile-fail: callers cannot extract an independently launchable replacement
  candidate or discard the declaration source from a watched submission.
- Adversarial mismatch: an artifact from source revision A and declaration
  material from revision B cannot be represented as one admitted composition,
  even when capability snapshots match.
- Reload-storm preservation: rapid valid and invalid ingress sequences retain
  revision/order evidence without ever publishing artifact-only candidate
  truth.

**Engineering decisions**

- Candidate composition remains pre-authority input. It cannot answer active
  inspection or runtime APIs until preparation succeeds.
- Production and certification consume the same sealed ingress object; test
  support may originate inputs but cannot mint the admitted result.

**Open questions**

- None.

### Phase 7: Fallible Application Preparation and Typed Freeze Denials

Replace panic-based freeze with a typed preparation pipeline that denies before
publishing any application authority.

**Relevant subsystems**

- `WorthUiBuilder`
- capability registration freeze
- declaration lowering
- graph-handoff lowering and admission
- graph generation commit
- Query binding preparation
- host-session planning
- inspection-index assembly

**Relevant APIs**

- `WorthUiBuilder::freeze`
- `WorthUiCapabilityRegistrationFreezeCore::freeze_from_registration`
- `lower_graph_handoffs`
- `admit_graph_handoffs`
- `commit_initial_generation`
- new application-preparation denial taxonomy

**Required design**

- Make the public freeze/preparation surface return a typed result whose success
  contains prepared application authority and whose denial identifies the
  exact failed phase and structured local evidence.
- Replace every `expect` reachable from publicly constructible declaration,
  topology, Query, capability, or host input with explicit typed propagation.
- Order preparation so cheap eligibility and structural denials precede graph
  commit, derived-index construction, Query activation, host activation, or
  other expensive work.
- Commit the graph and seal prepared authority only after all prerequisite
  admissions succeed. No denial path may publish a partial graph generation or
  retain partially activated Query/host state.
- Keep internal invariant-corruption panics distinct from ordinary input
  denial, and make that distinction visible in module ownership and tests.

**Warnings**

- Do not flatten graph local denials into a generic freeze failure or string.
- Do not catch panics and translate them after mutation; ordinary invalid input
  must never panic in the first place.
- Do not keep an infallible convenience `freeze` beside the fallible ordinary
  API. There is one production preparation lane.

**Test requirements**

- Adversarial topology denial: duplicate or missing product-root topology,
  contradictory graph basis, and illegal structural handoffs return the
  correct typed preparation denial without unwinding.
- Residue rejection: failure at each preparation phase leaves no committed
  graph generation, activated Query binding, host session, inspection index,
  or launchable witness.
- Deterministic denial: replaying identical invalid input produces equivalent
  phase-local denial evidence and never changes the previously active
  application.
- Compile-fail: callers cannot bypass fallible preparation to construct
  prepared authority directly from a capability snapshot or graph snapshot.

**Engineering decisions**

- Registration reporting may remain separately inspectable, but successful
  registration alone is not prepared application authority.
- The denial topology mirrors the actual preparation phases rather than
  preserving old facade convenience.

**Open questions**

- None.

### Phase 8: Active Application Session and Launch Admission

Consume prepared authority into one active application session that owns both
runtime execution and active inspection for the same generation.

**Relevant subsystems**

- Worth UI application facade
- runtime launch and active state
- active inspection bridge
- retained allocation evidence
- Query binding activation
- generation identity and last-valid runtime state

**Relevant APIs**

- `WorthUiApp::launch_runtime`
- `WorthUiRuntimeLaunch`
- `WorthUiRuntime::launch`
- `WorthUiActiveArtifact`
- `WorthUiRuntimeQueryBinding`
- `WorthUiLastValidRuntimeState`
- new active-application session facade

**Required design**

- Replace independent `&app -> runtime` launch with a consuming transition from
  prepared application authority to one active application session.
- Make the active session the only ordinary owner of runtime execution,
  active-generation inspection, retained allocation evidence, Query runtime
  binding, and host session.
- Require a sealed launch admission witness produced from the entire prepared
  authority. Capability or artifact digests alone cannot satisfy launch.
- Remove ordinary runtime constructors that accept a canonical artifact or
  replacement candidate without the matching prepared authority.
- Make active inspection receipts carry the active generation identity so
  consumers can prove they describe the runtime being executed.
- Preserve last-valid state inside the active session, not as a substitute
  authority callers can independently activate.

**Warnings**

- Do not solve the ownership problem by cloning app truth into runtime truth.
- Do not leave `WorthUiRuntimeLaunch::from_candidate` public if it remains a
  path around prepared application authority.
- Certification-only launch construction must stay support-authority scoped and
  must not be type-compatible with the production launch witness.

**Test requirements**

- Adversarial cross-launch: a candidate prepared against application A cannot
  launch or replace application B, even when both share the same capability
  snapshot and artifact digest collision is simulated at the comparison seam.
- Active inspection parity: every active inspection receipt and runtime frame
  outcome reports the same generation identity through launch and ordinary
  frame execution.
- Compile-fail: a prepared app cannot be launched twice, inspected as active
  before launch, or split into independently owned runtime and inspection
  objects.
- Query-free/installed parity: both postures use the same active-session
  lifecycle without imposing Query ceremony on Query-free applications.

**Engineering decisions**

- Prepared and active application sessions are distinct lifecycle types.
- Runtime internals remain private; the active application facade exposes only
  admitted execution, source, inspection, and replacement capabilities.

**Open questions**

- None.

### Phase 9: Atomic Replacement and Inspection Continuity

Make replacement cutover publish artifact, declarations, graph, Query binding,
derived indexes, and active inspection as one generation transition.

**Relevant subsystems**

- replacement candidate admission
- impact narrowing and reconciliation
- activation staging and gate
- declaration/graph preparation
- active inspection authority
- last-valid preservation
- reload-storm certification

**Relevant APIs**

- `WorthUiAdmittedReplacementCandidate`
- activation staging bundle and gate
- replacement impact/narrowing surfaces
- durable-state reconciliation plan
- reload-storm scenario
- new prepared-replacement and application-cutover artifacts

**Required design**

- Lower admitted candidate composition into a sealed prepared replacement that
  carries the candidate generation's artifact, declarations, committed graph,
  Query rebind outcome, derived inspection/index projections, and cutover
  evidence.
- Keep candidate inspection explicitly candidate-scoped until cutover. Active
  inspection continues to describe the prior generation while staging occurs.
- Publish the prepared replacement through one atomic cutover owned by the
  active application session. No constituent authority may become visible
  earlier than another.
- On denial, discard all candidate-derived authority and preserve the prior
  active generation, host session, runtime state, and inspection truth.
- Reconciliation and impact narrowing consume the prepared replacement basis;
  they must not independently reconstruct graph or declaration equivalence.

**Warnings**

- Do not mutate the active graph and then attempt to roll back from candidate
  source if a later Query or activation phase denies.
- Do not allow inspection generation to lead or lag runtime generation during
  cutover.
- Do not use shared raw IDs to bridge old and candidate generations. Identity
  correspondence must remain an admitted reconciliation artifact.

**Test requirements**

- Hostile cutover sequence: thousands of alternating valid, invalid, no-op,
  and structurally different replacements never expose mixed generation
  identities across runtime, graph lookup, Query binding, or inspection.
- Failure localization: denial injected at every staging/cutover phase leaves
  the prior generation byte-for-byte equivalent at its authoritative
  projections and leaves no candidate residue.
- No-op convergence: equivalent replacements retain the active generation and
  produce typed no-op evidence rather than republishing derived state.
- Observer boundary: candidate inspection can explain a denied candidate while
  active inspection continues to explain only the still-running generation.

**Engineering decisions**

- Replacement is application-authority replacement, not artifact-only swap.
- Candidate diagnostics are derived evidence and cannot affect cutover
  eligibility or active operational truth.

**Open questions**

- None.

### Phase 10: Operational Host-Session Authority

Turn configured host posture into the sole admitted source of native
observation and measurement capabilities for an active application.

**Relevant subsystems**

- `worth-ui-host-contract`
- `worth-ui-host-egui`
- Worth UI builder host configuration
- runtime host observation and measurement collection
- framework-turn source capabilities
- host capability reports and generations

**Relevant APIs**

- `WorthUiHostAdapter`
- `WorthUiMeasurementHostAdapter`
- `WorthUiHostContract`
- `WorthUiBuilder::with_host`
- `WorthUiHostMeasurementTurnSource::collect_and_submit`
- host measurement collector and evidence boundary
- new sealed host-session authority and measurement capability

**Required design**

- Prepare and activate one host session from the configured adapter/contract,
  host kind, capability report, and observation generation.
- Make the active application session own host-session authority. Framework
  turns borrow a generation-scoped measurement/observation capability from
  that session rather than accepting arbitrary adapters.
- Bind every host-produced observation and measurement receipt to host-session
  identity and observation generation before it enters allocation admission.
- Deny missing, stale, foreign, or capability-incompatible host evidence before
  measurement construction or frame submission.
- Keep native mechanics inside host adapters and semantic admission inside
  Worth UI runtime boundaries.
- Remove the unused retained `_host_contract` posture and any alternate path
  where host configuration is stored but not consumed.

**Warnings**

- Do not turn `WorthUiHostContract` into a semantic layout authority. It grants
  access to native observations; declarations and runtime plans still decide UI
  meaning.
- Do not compare only `WorthUiHostKind`. Session identity, capability posture,
  and observation generation are distinct facts.
- Do not require Query-free or headless applications to instantiate egui
  machinery. Host-session variants must preserve their actual capability
  boundaries.

**Test requirements**

- Adapter substitution denial: an adapter not admitted into the active host
  session cannot submit measurements, even when it implements the same public
  trait and reports the same host kind.
- Host-generation freshness: stale observations from the previous host
  capability generation deny before allocation ingress and cannot advance
  frame source order.
- Headless/egui contract parity: both hosts use the same session lifecycle and
  receipt authority while producing only capabilities they genuinely support.
- Compile-fail: external callers cannot mint host-session authority or obtain a
  turn-source measurement capability without borrowing the active session.

**Engineering decisions**

- `with_host` remains only if it performs this real lifecycle role; decorative
  configuration is forbidden.
- Host replacement, if supported, is an explicit application transition with
  typed invalidation. Ambient adapter swapping is not supported.

**Open questions**

- None. Host replacement does not ship in this milestone. Application
  replacement retains the configured active host session; any future host
  replacement must be a separate atomic application transition.

### Phase 11: Framework-Turn Transition Planning

Extract policy classification, narrowing, and selection from the close/pump
owner into a proof-bearing transition plan that is complete before mutation.

**Relevant subsystems**

- allocation frame scheduler and dispatcher
- stream policy resolution
- invalidation narrowing
- viewport and resize policy classification
- allocation-neighborhood selection
- allocation receipt planning
- framework-turn counters

**Relevant APIs**

- `WorthUiRuntime::execute_framework_turn`
- `close_allocation_ingress_at_framework_boundary`
- `UiAllocationFrameTurnOutcome`
- `UiAllocationFrameConsumptionDisposition`
- `UiAllocationInvalidationNarrowingDisposition`
- `UiResolvedAllocationCommitPlan`
- `select_replan_neighborhoods`
- new proof-bearing framework-transition plan family

**Required design**

- Preserve one owner for framework-turn clocking, ingress closure, and exactly
  one dispatcher pump.
- Move stream consumption, invalidation narrowing, policy-family
  classification, and neighborhood selection into named planning transitions
  that borrow immutable authority and produce a sealed plan or typed denial.
- Define exhaustive plan variants for no-ingress execution, ordinary
  allocation, viewport resize, resize preview, durable resize, drag-resize,
  and every existing backpressure or denial posture.
- Carry every proof execution needs: sealed frame identity, source order,
  narrowed invalidations, selected neighborhoods, resize identity/extent,
  expected receipt basis, and active application generation.
- Attach structural counters for admitted ingress width, invalidation breadth,
  selected neighborhood breadth, and policy classification.

**Warnings**

- Do not merely move the existing nested match into a differently named file.
  Planning must become a real phase with a sealed output type and no commit
  authority.
- Do not unify policy variants whose costs, failure modes, or correctness
  requirements differ.
- Do not let the executor rediscover neighborhood selection, durable extent,
  or policy family from raw frame facts.

**Test requirements**

- Replay convergence: the same sealed frame and active authority produce an
  equivalent transition plan and exact counters regardless of diagnostic
  richness.
- Mutation isolation: every planning denial leaves receipt ledgers,
  invalidation authority, durable resize state, and active frame generation
  unchanged.
- Exhaustive family matrix: each admitted source combination selects exactly
  one plan variant; ambiguous or contradictory combinations deny before
  execution.
- Breadth certification: local invalidations produce exact neighborhood and
  counter breadth without unrelated graph scans.

**Engineering decisions**

- Planning owns policy choice; execution owns only the mutation named by the
  chosen plan.
- Existing correct classifier and selector types should be retained where they
  already carry the required proof, but raw or partial outputs may not cross
  into execution.

**Open questions**

- None.

### Phase 12: Policy-Family Execution and Thin Framework Ownership

Split allocation mutation into policy-family executors and reduce the
framework-turn owner to lifecycle orchestration visible as a short semantic
sequence.

**Relevant subsystems**

- framework-turn owner and completion facade
- ordinary allocation transaction
- viewport resize commit
- resize preview publication
- durable resize commit
- drag-resize preview/terminal commit
- allocation receipt ledger and invalidation authority

**Relevant APIs**

- `close_allocation_ingress_at_framework_boundary`
- allocation transaction commit surfaces
- `UiViewportResizeOutcome`
- `UiResizePreviewOutcome`
- `UiDurableResizeCommitOutcome`
- `WorthUiFrameworkTurnCompletion`
- new policy-family execution transitions

**Required design**

- Give each policy family one named execution responsibility with a typed plan
  input, typed commit/denial output, and the minimum mutable authority it needs.
- Keep receipt publication and authority mutation transactionally aligned for
  every family. A denial cannot publish a receipt or partially mutate durable
  state.
- Make the framework-turn owner read as: collect sources, close ingress, plan
  transition, execute transition, publish completion. It must not inline
  family-specific selection or commit mechanics.
- Keep backpressure, panic-safe ingress closeout, and exactly-once pump behavior
  at the owner boundary.
- Place execution modules by policy responsibility, not under generic
  `helpers`, `handlers`, `operations`, or milestone-named folders.

**Warnings**

- Do not introduce trait abstraction merely because executors share method
  shape. Share lifecycle only where authority, cost, and failure topology are
  actually equivalent.
- Do not widen mutable borrows from family executors back to the whole runtime.
- Do not preserve the old owner body as a fallback or certification reference.

**Test requirements**

- Family parity: ordinary, viewport, preview, durable, and drag-resize scenarios
  retain their admitted outcomes, receipts, and denials after decomposition.
- Transactional hostility: injected denial at every family mutation boundary
  produces no partial ledger entry, invalidation update, durable-state update,
  or completion claiming success.
- Exactly-once lifecycle: source callback success, source callback panic,
  downstream backpressure, and no-ingress turns each close/pump exactly once
  where permitted and never double-publish completion.
- Borrow-boundary enforcement: structural/compile checks prevent a family
  executor from receiving unrelated runtime subsystems or independently
  clocking the dispatcher.

**Engineering decisions**

- Single framework authority does not require single-function implementation.
- Completion mapping is presentation over typed transition outcomes and cannot
  decide policy or mutate allocation truth.

**Open questions**

- None.

### Phase 13: Dormant Theme and Rendering Residue Removal

Remove source files that claim GPU/theme functionality but are not part of the
compiled crate graph, and add mechanical detection so orphan production Rust
cannot recur.

**Relevant subsystems**

- `worth-ui-theme`
- `worth-ui-components`
- `worth-ui-host-egui`
- workspace source-reachability enforcement
- theme/component documentation and facade claims

**Relevant APIs and source surfaces**

- `worth-ui-theme/src/lib.rs`
- `worth-ui-theme/src/backdrop.rs`
- `worth-ui-theme/src/metal.rs`
- `worth-ui-theme/src/shader_pipeline.rs`
- `worth-ui-theme/Cargo.toml`
- workspace structural certification scripts/tests

**Required design**

- Delete the unreachable backdrop, metal-paint, and shader-pipeline files and
  any dead references, claims, fixtures, or manifest residue associated with
  them.
- Do not activate their `wgpu`/`egui-wgpu` implementation in this milestone.
  GPU-backed mounted rendering belongs after the roadmap's mounted-host and
  appearance authority are available.
- Audit theme and component public documentation for claims stronger than the
  compiled implementation and correct those claims without adding historical
  legacy narrative.
- Add a workspace check that fails when a production Rust source file under a
  crate's `src` tree is absent from every compiled module graph, except for a
  narrowly named and reviewed generated-source exemption.
- Ensure the check understands ordinary `mod`, explicit `#[path]`, feature
  combinations, and platform-gated production modules so it does not reward
  false deletion or false reachability.

**Warnings**

- Do not keep the files as examples, future reference, or an uncompiled
  feature. Version control is the historical store.
- Do not make every optional target compile in the ordinary binary; the check
  must reason over declared supported feature/platform graphs.
- Do not broaden this into Milestone 3.16 appearance semantics or Milestone 10
  real-time rendering product work.

**Test requirements**

- Structural rejection: an intentionally orphaned production `.rs` fixture
  causes the source-reachability gate to fail with the exact crate and path.
- Feature/platform reachability: legitimate feature-gated and `#[path]`
  modules are recognized when their declared compilation graph is included.
- Residue audit: no backdrop, metal, shader-pipeline, `wgpu`, or `egui-wgpu`
  production reference remains in the current theme/component crates after
  deletion.
- Theme honesty: every documented public theme capability is reachable through
  the compiled facade and exercised by at least one focused test.

**Engineering decisions**

- The current semantic-token/theme surface remains; only uncompiled claimed
  functionality is removed.
- Future GPU rendering must be reintroduced from the correct host/appearance
  authority, not resurrected by copying these files back.

**Open questions**

- None.

### Phase 14: Hostile End-to-End Closure and Anti-Bypass Certification

Certify that the repaired workspace has one application authority from ingress
through active execution, inspection, host observation, replacement, and
framework-turn closeout.

**Relevant subsystems**

- `worth-ui-certification`
- public facade compile-fail suites
- source ingress and reload-storm scenarios
- active inspection harness
- host adapter certification
- allocation framework-turn scenarios
- workspace boundary and source-reachability gates

**Relevant APIs and evidence**

- prepared and active application facades
- candidate composition and replacement cutover receipts
- application/host generation identities
- typed preparation, launch, replacement, host, and frame denials
- allocation breadth counters
- boundary-check, agent-context, line-cap, lint, and reachability gates

**Required design**

- Add one production-path certification scenario that starts from both
  file-authored and Rust-authored composition, prepares an application,
  activates a real host session, executes representative framework turns,
  performs valid/no-op/invalid replacements, and inspects the active result.
- Assert one generation identity across runtime execution, declaration lookup,
  graph lookup, Query binding, host evidence, allocation receipts, and active
  inspection at every published boundary.
- Add anti-bypass compile-fail and structural checks for every removed split
  constructor, raw-digest promotion, arbitrary turn adapter, infallible freeze,
  independent runtime launch, and dormant source lane.
- Require clean workspace warnings, strict Clippy, dead-code posture, Rust file
  line caps, boundary check, agent-context check, and documentation truth before
  closure.
- Update public docs and `AI_README.md` discovery surfaces to teach only the
  final ordinary path and current authority vocabulary.

**Warnings**

- Certification must consume production facades. Support authority may provide
  source/host inputs and fault injection, but it cannot mint prepared, active,
  host-session, or cutover authority.
- A green behavioral suite is not sufficient if the old split APIs still
  compile or orphan production files remain invisible to the compiler.
- Do not retain historical migration instructions in discovery documentation.

**Test requirements**

- Mixed-generation attack matrix: systematically attempt every artifact,
  declaration, graph, Query, host, inspection, and launch cross-product and
  prove all foreign combinations are unrepresentable or deny before mutation.
- End-to-end convergence: equivalent file/Rust composition and replayed source
  sequences converge to the same active authority, transition outcomes,
  receipts, inspection evidence, and exact structural counters.
- Failure preservation: fault injection at every preparation, launch,
  replacement, host-observation, planning, and execution boundary preserves the
  last complete active authority without panic or residue.
- Anti-bypass topology: production source and public facade scans prove there
  is exactly one preparation lane, one launch lane, one host-session lane, one
  replacement cutover, and no orphan production Rust files.

**Engineering decisions**

- This phase closes discovered architecture, not only tests changed during the
  milestone. Substrate failures exposed by certification are in scope.
- Milestone 3.9 may begin only after the hostile scenario and all mechanical
  gates are green with no named debt.

**Open questions**

- None.

## Must Ship

- a machine-readable test inventory, proof-migration ledger, and enforced
  target/process/session budgets
- Worth UI-owned dev/test profiles; explicit fast, compile-contract,
  hostile-certification, documentation, dependency-contract, and serial-full
  lanes; and one proof-equivalent parallel premerge aggregate
- no more than 6 `worth-ui` and 10 `worth-ui-certification` integration targets,
  with no more than 4 compile-contract binaries workspace-wide
- batched compile contracts with no runtime warning suppression or
  per-fixture compiler-session topology
- named, compiled-once certification scenario authority without generic test
  worlds or production-authority minting
- zero nested Cargo invocations in ordinary workspace tests and one derived
  source inventory per structural-audit suite
- one sealed prepared application authority and distinct active application
  session
- inseparable source candidate composition for file- and Rust-authored paths
- fallible application preparation with typed phase-local denials
- launch and replacement cutover bound to complete application authority
- operational host-session authority with generation-bound observation access
- proof-bearing framework-turn transition planning
- policy-family allocation executors under one thin framework clock owner
- deletion of unreachable GPU/theme source and mechanical orphan-source
  detection
- production-path hostile certification and anti-bypass compile/structure gates
- current discovery and feature documentation for the final ordinary path

## Must Preserve

- the canonical UI artifact remains the source of runtime UI meaning; the new
  application authority binds it to related truth without replacing it with an
  aggregate hash or facade-owned shadow model
- Worth UI remains above host adapters and does not give native code authority
  over layout, operability, Query meaning, or visible semantic state
- Query binding remains Query-owned and installed-authority-backed; this
  milestone binds its lifecycle to the application generation without
  reimplementing Query locally
- invalid reloads preserve the last admitted active generation and typed
  candidate diagnostics remain observational
- allocation work stays delta-bounded, counter-visible, and free of per-frame
  semantic rediscovery
- Query-free and headless applications remain ceremony-free within the same
  honest lifecycle
- support authority remains distinct from production authority
- hostile compiler, runtime, topology, and boundary proof remains complete
  after test consolidation; speed may not come from silently shrinking proof
- all production and test Rust files remain within the workspace line-cap rule
  unless explicitly and narrowly exempted

## Acceptance Evidence

- the proof-migration ledger reconciles every pre-migration test and fixture to
  a retained proof, stronger replacement, or reviewed redundancy deletion
- final test topology satisfies the integration-target, compile-contract, and
  nested-process budgets mechanically
- isolated measurements show median warm full-suite time at no more than 50
  percent of baseline and cold no-run compilation at no more than 60 percent,
  with the same reconciled proof families
- explicit Worth UI CI lanes cover quality/topology, fast runtime,
  documentation, compile contracts, hostile certification, dependency
  isolation, and Windows compilation behind one stable required aggregate;
  the serial full lane remains the exact local/manual runtime reproduction
- the pinned CI toolchain, source-only Cargo cache, content-addressed compiler
  cache, timeouts, cancellation policy, lane reports, and CI topology checker
  make execution cost and configuration drift visible
- external callers cannot compile any path that launches an artifact,
  replacement candidate, graph, Query binding, or host adapter independently of
  the matching prepared application authority
- invalid public declaration/topology input returns typed preparation denial
  without unwind or partial publication
- active runtime, graph, declarations, Query binding, host observations,
  allocation receipts, and inspection receipts expose one matching application
  generation across launch and replacement
- foreign/stale host adapters and observations deny before measurement or frame
  ingress
- framework-turn planning is mutation-free, family execution is transactional,
  and the owner performs exactly one permitted close/pump per turn
- local allocation changes retain exact bounded breadth counters after
  decomposition
- equivalent file- and Rust-authored applications converge through the same
  prepared and active lifecycle
- every production Rust source file participates in a declared compiled module
  graph and the dormant theme/GPU files and claims are absent
- workspace tests, strict Clippy, warnings/dead-code checks, line-cap checks,
  boundary check, agent-context check, compile-fail suites, and hostile
  certification are green

## Sequencing Notes

- Phase order is mandatory. Phases 1 through 4 repair and certify test topology
  before any production-authority implementation begins. Candidate ingress
  cannot be narrowed honestly before canonical prepared authority exists;
  launch and replacement cannot be repaired before fallible preparation exists;
  frame execution cannot be split before a complete transition plan exists.
- This milestone is a closure gate for 3.8, not an early implementation of 3.9,
  3.10, or 3.16. It does not lower the future execution plan, mount host render
  receipts, or introduce runtime appearance semantics.
- Milestone 3.9 is blocked until Phase 14 closes with no compatibility lane or
  named architectural debt.
