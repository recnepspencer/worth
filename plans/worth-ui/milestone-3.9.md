# Milestone 3.9: Execution-Plan Lowering, Equivalence, and Frame-Cost Surfaces

> Historical QA policy (2026-08-22): proof, closure, migration, acceptance,
> and phase ledgers described below are frozen historical records. They are not
> active implementation or release gates, are not updated or reopened, and a
> ledger-only failure does not block current work. Current evidence follows
> [the QA review guide](../../docs/coding-guidelines/qa_review_guide.md) and
> [testing laws](../../docs/coding-guidelines/testing_laws.md): specifications state QA
> considerations in prose, tests and repository checks run against the current
> commit, and code review decides whether the evidence is adequate. This note
> does not retire product-domain ledgers that are part of runtime behavior.

**Status:** Closed. Milestone 3.8.1's structural and test-topology work is
the prerequisite. Phases 1 through 18 are closed. The hostile reopening of
Phases 9 through 13 is resolved: public real-filesystem canvas and realtime
replacement now prove hook/resource-generation retirement and stale denial;
the public large-canvas and complete cross-lane paths seal through activation;
the executable-equivalence field matrices, equal-digest hostile case, real
Query no-op lifetime, and late fallible-interruption matrix are present. Phase
14's persistent regional plan, stable-slot, lane-local successor, regional
Query succession, and allocation-catalog successor work is real and hostile-
tested. Public application replacement now admits an exact candidate-owned
changed/removal delta, derives the affected predecessor closure, carries all
unaffected allocation truth through persistent storage, and incrementally
updates graph, invalidation, scroll, portal, Query, host, and durable indexes.
The public successor receipt exposes changed, inserted, removed, and carried
postures plus delta-local work counters; real carry, removal-only, scaled
storage-retention, and sustained-churn tests close the prior locality gate.
Phase 15 now publishes application, plan, allocation, state, scheduler, Query,
host, planning, and inspection truth through one prepared infallible commit.
Exact frame-boundary session authority rejects equal-looking foreign sessions;
transient boundary denials return the intact candidate for explicit retry; and
the public receipt plus real Query lifetime tests prove simultaneous successor
coherence, predecessor survival, and candidate-only cleanup. Phase 16 binds
reload and steady-frame receipts to independent scale and allocator evidence
across the complete lane set. Phase 17 leaves named `app` and `runtime` facades,
compact inspection, and final lifecycle documentation without a parallel plan
authority. Phase 18 removes the predecessor and test-only authority paths and
closes real filesystem/watcher, Query, egui, headless, allocator, compile-
contract, topology, boundary, line-cap, and quality proof. Comparable closing
timing evidence records reviewed cost amendments without moving real-mechanism
waits or storms into the ordinary fast lane. Every closure gate in this
specification is green.

## Purpose

Milestone 3.9 makes a lowered execution plan the only operational input to UI
frame execution. Canonical declaration, graph, capability, Query-binding,
host-support, and committed-allocation truth may participate while a candidate
plan is lowered. They must not be rediscovered by an executor during an
ordinary frame.

This is a consolidation and authority cutover, not a greenfield planner. The
workspace already contains committed-allocation lowering input, typed runtime
handles, plan topology, lane-specific plans, plan equivalence, reload and frame
counters, Foundational bridges, and atomic replacement infrastructure. Those
surfaces currently form overlapping partial paths. This milestone must leave
one phase chain, one active plan owner, one regional replacement/equivalence
kernel, one minimal sealed host-output envelope, and no operational predecessor
lane.

## Why this milestone belongs here

Milestone 3.8 committed allocation truth and Milestone 3.8.1 closed the active
application, replacement, Query, host-session, framework-turn, and inspection
authority boundaries. Milestone 3.9 can therefore lower from exact committed
truth without inventing a second application authority. Milestone 3.10 will
add mounted receipts and a stricter host contract; it needs a coherent active
plan to consume first. Deferring this cutover would force mounted receipts to
depend on caller-assembled or digest-only plan state.

## Governing summaries

- `MENTALITY.md` protects foundation-first work under the adversarial
  constraint; regional replacement/equivalence storage must exist before lane
  representations make it expensive to correct.
- `arch_laws.md` protects proof-carrying phase progression, exact identity
  authority, lowered-only execution, and atomic lifecycle propagation.
- `composition_laws.md` protects one named semantic responsibility per file and
  requires lowerers, comparators, transactions, evidence projectors, and
  executors to remain distinct named steps.
- `domain_structure_laws.md` protects physical separation between authored/
  graph authority, derived plan storage, Query binding, host translation, and
  inspection evidence.
- `perf_laws.md` protects semantic-delta-bounded work, explicit equivalence,
  locality, lifecycle-fit allocation, and honest separation of reload,
  executor, adapter, renderer, and diagnostic cost.
- `worth_ui_roadmap.md` protects the sequence from committed allocation to
  lowered plans, minimal host output, complete mounted receipts, observations,
  broader Query binding, and product surfaces without reopening prior
  authority.
- Worth Query's orientation protects one sealed consumed-projection authority,
  Query-owned installed/live lifecycle, and typed support/outcome posture; Worth
  UI may lower admitted consequences but may not become a second Query planner
  or resource manager.

## Architectural inputs reconciled

This specification reconciles the current public facade and runtime APIs with
the active Worth UI orientation, DSL vision, AI-diagnostics architecture,
Milestones 1 through 3.8.1, and Worth Query's runtime mental model. The
load-bearing inherited contracts are:

- canonical declaration, graph, obligation, measurement, allocation, and
  committed-receipt authority remain distinct inputs; the execution plan is
  derived frame truth and cannot replace any of them;
- the DSL authors semantic lanes and lowers once; it never authors runtime
  handles, lane strategy, or host-specific execution contacts;
- Query crosses the UI boundary as installed, runtime-affine authority and one
  sealed consumed-projection outcome, not a tuple of copied status, basis,
  facts, labels, or digests;
- Foundational native aspect values remain native through Query consumption and
  UI lowering; plan construction may index them but may not round-trip them
  through JSON, strings, widened numerics, or UI-local mirrors;
- inspection and AI diagnostics observe the same receipts and indexes as human
  tools and may explain lowering, equivalence, activation, and frame cost, but
  may not mint or promote plan authority; and
- egui is an adapter substrate. Canonical execution-plan meaning is
  host-neutral even when the first production executor is egui.

## Governing adversarial constraint

> Under a 240 Hz hostile frame cadence and continuous replacement churn,
> execution work must remain bounded by the active lowered plan and the
> pre-execution admitted semantic delta. Equivalent replacements produce no
> executable swap, non-equivalent replacements are never suppressed, stale
> plans and handles open no authority, and executors never rediscover decisions
> that were available during lowering. The deterministic proof is structural:
> exact counters and authority receipts, not wall-clock hope inside CI. A test
> double may not stand in for a claimed filesystem, watcher, Query runtime,
> active application lifecycle, egui frame, or allocator boundary.

This constraint is more important than any proposed type or module name. If a
phase discovers that the cheapest implementation violates it, the
implementation must change rather than weakening the constraint.

## Current API disposition

The following inventory is the starting architecture, not an endorsement that
every current visibility or name survives.

| Current surface | Current role | Milestone disposition |
| --- | --- | --- |
| `WorthUiSourceProvider::filesystem_root`, `with_file`, `WorthUiSourceWatcher`, and `WorthUiWatcherEvent` | Model file-authored source and debounce, but currently allow a filesystem label over injected modules and caller-manufactured events | Split real filesystem acquisition/OS notification from explicitly synthetic providers in Phase 2. Actual disk bytes and production watcher observations must reach the same frozen candidate-composition pipeline; the runtime never writes authored `.wui`. |
| `UiCommittedAllocationLoweringInput` | Frozen 3.8-to-3.9 receipt/report/transaction handoff with freshness admission | Preserve as the allocation constituent of the sole sealed lowering authority. Do not let it become a standalone parallel authority. |
| `WorthUiExecutionPlanInput`, `WorthUiExecutionPlanInputPreparer`, and `WorthUiExecutionPlanInputWitness` | Reconstruct node, topology, reconciliation, Query-rebind, hook, and digest inputs from pending activation | Consolidate into the canonical lowerer or remove. No duplicated reconstruction, witness-only assembly, or predecessor input path may remain. |
| `WorthUiPlanLoweringBasis`, `WorthUiPlanLoweringContext`, node inputs, and lowering counters/denials | Describe provisional lowering inputs and their evidence | Retain only facts that belong in the canonical sealed input; replace raw digest identity with exact authority where identity matters. |
| `WorthUiInstalledQueryView`, lifecycle-specific `WorthUiQuerySnapshotProjectionOutcome` / `WorthUiQueryLiveProjectionOutcome`, `WorthUiQueryAuthorityHandle`, Query prerequisite/measurement settlements, and native aspect contracts | Carry installed Query authority, consumed projection authority, and binding-owned native facts into Worth UI | Preserve the sealed binding-owned handoff. Plan rows may retain compact admitted references, but may not split the outcome into UI-owned basis/status/fact/digest fields or recreate Query lifecycle authority. |
| `WorthUiPlanningLaneInput` and `WorthUiAllocationPlanning` | Admit graph/measurement locality and produce upstream allocation candidates | Preserve as the 3.8 allocation-planning phase. Execution-plan lowering consumes the committed receipt it produces, never re-enters allocation planning or accepts planning input as execution truth. |
| `WorthUiRuntimeHandleAllocation` and typed component, command, token, child-range, view-binding, lane, and state-slot handles | Allocate `(plan_index, plan_generation)` handles from committed allocation | Preserve the typed families and sealed construction. Cut over to direct indexed resolution, exact session-arena and slot-generation authority, stable reuse where proven, and explicit exhaustion denial. |
| `WorthUiExecutionLaneInput`, public `allocate_runtime_handles*`, `admit_execution_lanes`, and `assemble_execution_plan_topology*` methods | Let callers advance individual allocation, lane-admission, handle, and topology steps | Internalize them behind the sealed candidate lowerer. Preserve explicit host-support admission as a named phase, but do not expose independently composable plan constituents to product consumers. |
| `WorthUiExecutionPlan`, topology, lane partitions, lookup index, child ranges, region structure, and render-resource refs | Represent a shared topology plan | Make this an internal constituent of one sealed candidate/active plan bundle. It must not be independently submitted to execution. |
| `WorthUiEguiBoundaryInput`, `WorthUiEguiBoundaryContact`, and `WorthUiEguiPlanBoundary` | Carry egui-named contact meaning inside runtime plan topology | Replace with one minimal sealed host-neutral output envelope and confine egui translation to the adapter. The envelope is the foundation Milestone 3.10 extends into complete mounted receipts; it may not expose owned plan or authored meaning. |
| `WorthUiOrdinaryLanePlan`, `WorthUiVirtualizedDataPlan`, `WorthUiCanvasSpatialPlan`, and `WorthUiHudPlan` | Store lane-ready rows, indexes, counters, support digests, and certifications | Preserve useful lane-ready representations as sealed constituents of the plan bundle. The virtualized constituent is execution substrate for already-admitted references, not new Query collection semantics. Remove public/caller assembly and arbitrary cross-plan composition. |
| `WorthUiRuntime::prepare_*_plan` and `WorthUiFrameworkTurnExecution::execute_*_frame(&plan, ...)` | Let callers prepare and pass lane plan objects into executors | Replace with active-session or active-runtime execution capabilities that borrow the one active plan generation. A caller must not inject a stale or foreign plan. |
| `WorthUiActiveExecutionPlan` | Record only a derived active-plan digest | Replace with ownership of the actual sealed active plan bundle plus exact generation/session authority. A digest remains evidence, never the plan or its authority. |
| `WorthUiExecutionPlanEquivalenceBasis`, digestor, counters, and `Reusable`/`RebuildRequired` | Compare whole-plan fingerprints that currently include generation-bearing allocation data | Replace early with the regional predecessor-proof/equivalence kernel before lane cutover. Separate executable semantic equivalence from activation freshness, retain fingerprints only for narrowing/diagnostics, and report a decision rich enough to drive no-op, bounded reuse, rebuild, or denial. |
| `WorthUiPlanSwapReceipt`, committed-allocation preflight/publication, last-valid state, frame scheduler, and invalidation ledgers | Publish a replacement atomically after validation | Extend the existing transaction to publish the complete plan bundle and plan decision with the application generation. Denial must preserve the complete predecessor. |
| `WorthUiReloadCounterBoundary`, `WorthUiSteadyFrameCounterBoundary`, lane-cost certification, diagnostics projection, and Foundational bridges | Expose provisional cost and claim surfaces | Reuse and consolidate them around the real lower/activate/execute lifecycle. Delete parallel or synthetic counter truth. Foundational remains a post-boundary claim projection. |
| `WorthUiExecutionPlanInspection`, provenance, lane/node inspection, `compare_execution_plans`, and AI inspection harness | Inspect caller-held plan objects and project diagnostics | Preserve structured explanation but bind it to sealed candidate or active-plan receipts. Inspection may compare evidence; it may not become another plan-construction or activation path. |
| `WorthUiActiveApplicationSession` and prepared/lowered/staged/cutover replacement types | Own the active application lifecycle and exact candidate session/generation binding | Extend this existing owner to hold or exclusively govern active-plan execution. Plan publication and application-generation publication must be one cutover. |
| `prepare_replacement` artifact-digest early `NoOp` | Suppress a replacement before allocation and plan lowering | Remove as an operational no-op decision. No-op is legal only after complete canonical plan equivalence and exact authority checks. |

## Target authority and phase chain

The exact final type names may change, but the ownership chain may not:

```text
file-authored: external editor writes `.wui`
    -> production filesystem read + watcher/debounce settlement
    -> immutable ordered source-package revision
Rust-authored: compiled composition enters its admitted source boundary
    -> both converge into prepared candidate application authority
    -> sealed allocation-planning projection
       { only allocation-relevant candidate facts; no executable authority }
    -> committed allocation lowering input
  + exact candidate graph and capability/Query posture
  + admitted host/lane support
  + that committed allocation lowering input
    -> sealed execution-plan lowering authority
    -> admitted affected-region and predecessor-region proof
    -> regional plan storage + collision-safe equivalence kernel
    -> sealed candidate plan bundle
       { topology, typed handles, lane-ready plans, equivalence proof, counters }
    -> typed plan decision
       { initial activation | semantic no-op | bounded replacement | denial }
       semantic no-op -> optional typed derived provenance/inspection refresh
                         with active executable authority unchanged
       activation/replacement -> atomic application + runtime + active-plan publication
    -> active-session execution capability
    -> lane execution receipts
    -> minimal sealed host-output envelope
    -> optional inspection/diagnostic/Foundational projection
```

Only the sealed lowering authority may create a candidate plan bundle. Only
the activation transaction may promote one to active truth. Only an active
session capability may lend executable access to it. Inspection may observe
the active or candidate plan through typed receipts but may not promote it.

## Binding assumptions

These assumptions are explicit milestone law. Evidence that invalidates one
requires a visible specification amendment before implementation proceeds.

1. The active application session remains the one ordinary owner of a running
   application generation, host session, runtime, Query posture, graph
   authority, and active execution plan.
2. A plan is derived, reconstructable state. It may cache decisions and indexes
   but may not become the exclusive source of authored or graph meaning.
3. Launch may lower the complete initial plan. Replacement lowering must reuse
   admitted predecessor regions and be bounded by the affected semantic
   closure; an unrelated large graph may not turn a one-node change into a
   full-plan walk.
4. Ordinary frame execution receives no declaration artifact, graph snapshot,
   allocation planner, string registry, or candidate authority.
5. Executable semantic equivalence and activation authority are different
   questions. Incidental frame epochs or newly minted candidate generations do
   not by themselves make equivalent execution meaning different; stale or
   foreign authority still cannot activate or execute.
6. A digest is a lookup aid and diagnostic fact, never sufficient proof of
   identity, freshness, equivalence, or authorization.
7. Capability posture, Query-binding posture, host support, extension-hook
   admission, render-resource meaning, and lane policy participate in
   equivalence whenever changing them can change execution.
8. Diagnostic richness, counter materialization, and inspection formatting do
   not participate in operational equivalence unless they alter execution.
9. The four current execution regimes--ordinary, virtualized data,
   canvas/spatial, and realtime overlay--are the closed set for this milestone.
   Adding a regime requires an explicit plan constituent and cost contract, not
   a catch-all fallback.
10. Typed handles are sealed compact locators into one exact session-owned arena
    and one current slot generation. They are not hashes, names, global registry
    keys, or transferable capabilities between otherwise identical sessions. A
    locator may remain valid across whole-plan generations only when carried
    predecessor-region proof preserves the exact slot meaning and generation.
11. Query-free and headless applications use the same lifecycle with absent
    typed constituents. They do not require dummy Query or host adapters.
12. Foundational consumes certified boundary evidence. It does not define plan
    meaning, choose a strategy, or sit on the frame hot path.
13. Candidate denial, failed lowering, failed equivalence proof, and failed
    publication leave the predecessor application, plan, handles, counters,
    frame scheduler, and inspection authority coherent and usable.
14. Plan inspection and rich diagnostics are explicitly requested cold-path
    work. Ordinary execution emits compact receipts or references and does not
    materialize reports by default.
15. An application is not frame-executable until its initial committed
    allocation has lowered into and activated a real plan bundle. If host
    measurement makes that bootstrap asynchronous, the API exposes a typed
    non-executable planning posture; it may not synthesize an empty or
    digest-only "active plan."
16. Query projection crosses into Worth UI as one binding-owned sealed outcome.
    The plan may retain an admitted compact reference to that authority, but it
    may not decompose it into independently trusted UI-local basis, receipt,
    fact, support, source-label, or digest fields.
17. Query-installed domain handles and retained live resources are affine to
    the Query runtime and installation generation that minted them. Worth UI
    coordinates their admitted use and exact-once release; it does not become
    the activation, subscription, recovery, or disposal authority.
18. Native aspect values and struct values remain exact Foundational types at
    the Query-to-UI and UI-to-plan boundaries. Text/JSON conversion is allowed
    only at an explicitly declared authoring or presentation boundary, never as
    operational plan meaning.
19. Query-owned support visibility and runtime admission remain different
    questions. A visible installed capability is not plan-executable until the
    binding boundary supplies the required admitted authority and support
    evidence.
20. Plan storage and regional replacement semantics are chosen before lane
    representations broaden. Every lane constituent must fit the same complete-
    successor, exact-reuse, retirement, and reclamation model from its first
    active implementation.
21. The Query handoff has an explicit ownership/lifetime matrix covering the
    installed view, consumed projection authority, binding-owned handle, live
    resource, plan reference, and inspection reference across launch, denial,
    no-op, activation, candidate-preparation denial, replacement, removal, and
    failed publication.
22. Milestone 3.9 virtualized-data work closes execution-plan and visible-range
    substrate for already-admitted binding references. It does not invent
    collection-query declarations, cursor authority, result-state, or live
    collection-patch semantics assigned to Milestones 3.13 and 6. Any required
    new installed-domain collection capability requires a visible roadmap/spec
    amendment before implementation.
23. An executable semantic no-op and a provenance/inspection refresh are
    different decisions. Non-operational source-span or provenance changes may
    refresh derived observation metadata without changing application, plan,
    handle, Query-resource, or frame authority.
24. Every phase that introduces a runtime artifact also introduces its compact
    evidence family, relevance index, and summary/evidence-reference inspection
    path. Rich materialization remains cold and explicit; Phase 17 curates the
    public inspection DX rather than retrofitting explainability.
25. Milestone 3.9 produces a minimal sealed host-output envelope so the egui
    adapter never receives authored declarations, owned plan data, or authority
    to choose UI meaning. Milestone 3.10 completes mounted-node and mounted-frame
    semantics by extending this boundary rather than replacing a temporary
    contact lane.
26. Authored `.wui` files are external source truth. Tests may write them to a
    temporary workspace as an editor would; Worth UI may read, watch, snapshot,
    and lower them, but the runtime does not author or rewrite them.
27. A provider named filesystem-backed reads actual bytes from its declared
    root, and a watcher claim observes the production operating-system watcher
    adapter. Injected source text and caller-manufactured watcher events remain
    valid only for explicitly named in-memory/editor/generated test lanes.
28. A host path named egui executes at least one real `egui::Context` frame from
    `RawInput` through the production adapter. An egui capability enum, profile,
    or hand-built contact is not egui execution evidence.
29. End-to-end Query claims install and consume through the real Query Consumer
    Kit/runtime. Locally constructed handles, copied posture, or certification-
    only Query mirrors may prove malformed-input rejection below authority but
    cannot close a Query lifecycle claim.
30. Structural counters and receipts are necessary but not self-certifying.
    Every milestone-level cost or behavior claim pairs them with an independent
    observation of the named production mechanism, such as filesystem state,
    egui output, active generation, Query lifecycle, or an armed allocator.
31. Allocation planning and execution-plan lowering are consecutive authorities,
    not one circular input. Before allocation commit, a sealed allocation-
    planning projection may carry only the candidate facts required to choose
    allocation. It cannot construct plan rows, carry executable Query/host/hook
    authority, or be reconstructed into an execution-plan input. After commit,
    the exact candidate authority and committed allocation lineage seal the one
    legal execution-plan lowering authority; the allocation projection alone
    opens no lowering or execution door.

## Scale and cost model

The implementation must name its scale terms and keep them visible in counters
and adversarial tests:

- `P`: rows and edges in the active plan;
- `A`: rows and edges in the admitted affected closure for a replacement;
- `T_req`: rows admitted by the target contract before execution--ordinary
  subtree width, visible rows/cells, spatial query region/visible primitives,
  or explicitly targeted realtime rows;
- `T_exec`: rows actually touched by the executor;
- `L`: active execution-lane count, bounded by the closed lane set;
- `D`: diagnostic or inspection rows explicitly requested for materialization.

Required envelopes:

- initial lowering is `O(P)` time and `O(P)` retained plan memory;
- replacement lowering and equivalence are `O(A)` plus the changed output,
  after existing impact and allocation locality have admitted `A`;
- a handle lookup is direct indexed `O(1)` work with exact session-arena,
  slot-generation, and family validation--binary searching a copied row index
  is not the target design;
- a steady frame is `O(T_req)`, proves `T_exec <= T_req`, and is independent of
  unrelated plan, graph, registry, and declaration width;
- active-plan execution and compact receipt/envelope production perform zero
  source parses, artifact validations, registry string lookups, broad graph/
  plan/registry scans, rich diagnostic materializations, and general-purpose
  heap allocations; host-adapter and renderer allocations are separately
  attributed and may not be hidden inside this executor claim;
- candidate construction may allocate off the active frame path, but a small
  replacement may not rebuild every handle and lane vector merely for
  implementation convenience;
- diagnostic projection is `O(D)` and runs only when requested;
- denial work stops at the earliest boundary that has enough evidence and does
  not continue into later plan phases.

If exact structural equivalence would otherwise require a whole-plan scan, the
implementation must carry exact predecessor-region proof through lowering and
compare only changed regions. Merkle-style fingerprints may narrow comparison,
but a hash collision must not be able to authorize a no-op.

## Test-program and iteration-time constitution

Test topology is the first milestone priority because a correct architecture
that makes every development iteration recompile fixture workspaces is not a
scalable implementation process.

### Hard structural budgets

1. Keep exactly one checked-in external compiler-contract owner and exactly two
   Cargo compiler sessions: one invocation for all negative architectural
   targets and one for all positive targets. `trybuild` and its private duplicate
   target graph are not part of the closing design.
2. Freeze executable compile-contract ceilings independently by source owner:
   14 Worth UI targets, 6 certification targets, and 3 host-contract targets.
   These targets cover the unchanged 283/62/3 physical case inventories through
   compiler-checked, architecture-grouped `include!` batches. A genuinely new
   compile-time invariant must replace or consolidate an existing representative
   in the same proof class. Any budget increase requires a reviewed spec/roadmap
   amendment and new cold/warm timing evidence; a phase implementation may not
   silently raise it.
3. Freeze physical compile-fixture inventory ceilings at 283 Worth UI cases, 62
   certification cases, and 3 host-contract cases. Inventory should shrink when
   a topology audit replaces a fixture; moving a case out of the executable CSV
   does not authorize leaving a new dormant fixture behind.
4. Keep the configured integration-target sets and per-package ceilings. New
   tests enter existing compiled targets as modules or replace weaker proof.
5. No ordinary test may launch nested Cargo, create a temporary crate/workspace,
   compile generated Rust, or invoke `rustc` directly. The dedicated lane runner
   may invoke Cargo only for the two frozen groups against the checked-in fixture
   manifest and the caller's ordinary target directory; it may not generate a
   workspace or create a second dependency cache.
6. No new per-phase, per-type, or milestone-numbered compile harness is allowed.
   Compile-fail proof is reserved for a public type-system impossibility that
   cannot be established by an existing compile representative or a mechanical
   source/topology audit.
7. Every executed negative target has one canonical `.stderr` diagnostic
   contract, and every directly or transitively included source must emit its
   own primary compiler error. No new snapshot or compile fixture may be added
   without a reconciliation row naming its proof class and replacement. Net
   executable case count remains non-increasing.
8. No test-only production constructor, enum variant, public escape hatch, or
   fake authority path may be added to provoke a denial. Test the real public
   or crate boundary, or inject data below the authority decision inside a
   test-only support module.
9. Production and test Rust files remain at or below the workspace 400-line
   cap. Test support obeys the same responsibility and naming laws as
   production code.

### Preferred proof order

For every invariant, select the earliest adequate and cheapest proof:

1. ordinary unit test for a local algorithm or denial;
2. ordinary compiled-once integration/scenario test for lifecycle behavior;
3. source/topology audit for visibility, dependency, or module-placement law;
4. an existing compile-contract equivalence class;
5. only then, a replacement compile fixture under the frozen budget.

One reusable scenario authority should create canonical applications, admitted
graphs, committed allocation, host support, and replacement deltas through
production APIs. Variants must be typed scenario operations, not cloned setup
forests. Shared support must not mint sealed internals, skip freshness checks,
or turn into a god fixture that hides which authority a test is exercising.

### Required timing evidence

Structural budgets are the mechanically enforced gate; wall-clock time is
recorded evidence rather than a flaky per-run assertion. At milestone open and
close, record the median of three isolated runs for:

- a targeted warm ordinary-test iteration;
- the warm fast proof lane;
- the existing `application_contracts` target with its real-boundary modules;
- the cold and warm compile-contract lane.

The aggregate `full` command is a release smoke that composes independently
reported proof jobs; it is not itself a repeated timing benchmark. CI retains
per-job duration artifacts and history, which is the actionable enterprise
signal. Do not block a developer on three synchronous full-suite repetitions.

The closing implementation must not regress a comparable median by more than
10 percent without removing the regression before closure or recording a
reviewed budget amendment with a concrete reason. Compiler sessions, package
targets, executed compile cases, and nested-Cargo count remain hard gates even
when timing noise is present.

Opening and closing measurements must be written to one schema-versioned
evidence artifact that records the command, toolchain, target directory,
platform, cache/sccache posture, cold/warm classification, three raw samples,
median, and comparison result. Runs are comparable only when those execution
conditions match; an incomparable run is evidence to repeat, not permission to
waive the structural budgets.

### Real-boundary proof contract and compiled homes

Unit, integration, and end-to-end proof answer different questions. A phase may
use cheap unit proof for its local algorithm, but no product-boundary claim
closes until an existing compiled integration target has exercised the real
mechanism named by the claim.

| Claim | Required production mechanism | Existing compiled home |
| --- | --- | --- |
| regional storage, exact equivalence, handles, lane algorithms, and local denials | the real owning implementation with below-authority hostile inputs where needed | existing `worth-ui-runtime` library-test target in the fast lane |
| file-authored launch/replacement | real temporary `.wui` files read through the production filesystem source adapter | a responsibility-named child module of the existing `worth-ui-certification` `application_contracts` target |
| file watching/debounce | actual create/write/remove/atomic-rename operations observed through the production watcher adapter, then frozen into one source-package revision | the same existing `application_contracts` target; one small cross-platform contract module, not a new binary |
| public application/plan lifecycle | public prepared/active application facade from source acquisition through frame execution and replacement | the same existing `application_contracts` target |
| Query-backed execution | the real Query Consumer Kit installation, consumed projection, live-resource succession, and disposal path | the same existing `application_contracts` target |
| egui host execution | real `egui::Context::run`/`RawInput` frame mechanics consuming the production sealed host-output adapter boundary | the same existing `application_contracts` target; adapter-local pure translation tests may remain in the existing host-egui library target |
| headless parity | the production headless adapter consuming equivalent host-neutral envelope meaning | the same existing `application_contracts` target |
| zero-allocation executor claim | the real active-plan execution call observed by a thread-scoped armed allocator, reconciled with production counters | the same existing `application_contracts` target |

Placement rules are binding:

- Add no `[[test]]` entry, compile-contract session, compile fixture, fixture
  workspace, generated crate, or nested Cargo invocation. New cross-crate
  scenarios are child modules of the already compiled `application_contracts`
  binary and join the existing hostile-certification lane.
- Keep OS filesystem/watcher work, real egui frames, allocator monitoring, and
  long replacement storms out of the fast lane. The fast lane retains local
  deterministic algorithm, authority, and topology proof; the hostile lane
  pays for real external mechanisms once.
- During implementation, developers run a responsibility-named filtered module
  inside `application_contracts`; phase closure runs the complete existing
  target, and premerge runs the existing hostile-certification lane. Filtering
  changes executed tests, not the compiled target graph.
- A runtime-created temporary source directory is test data, not a generated
  compile workspace. It contains only ordinary `.wui` source and is cleaned up
  after the scenario.
- Shared scenario code is split by owned boundary--filesystem workspace, public
  application lifecycle, Query installation, egui frame driver, and allocator
  observation. No universal `world`, `helpers`, or milestone-numbered fixture
  may hide which real mechanism a test exercised.
- Real watcher tests use an explicit readiness/settlement handshake and bounded
  timeout. Blind sleeps, unbounded polling, and event-order assertions are
  forbidden; final frozen package meaning and typed ordering evidence are the
  deterministic assertions.
- Each phase records in a proof ledger: the claim, proof class, compiled owner,
  production entry point, independently observed result, and a plausible fake
  implementation that the test would fail. A receipt checked only against
  another receipt from the same synthetic setup does not qualify.
- Each ledger row names the first phase that must prove that exact claim. When
  one external mechanism has distinct obligations in different phases--for
  example Query lowering authority before virtualized execution, or minimal
  headless execution before cross-lane parity--record separate claim rows rather
  than one broad row deferred to the later phase.
- Use a bounded real-mechanism sequence to prove filesystem, watcher, Query, and
  egui crossings, then use larger deterministic public-lifecycle/plan scenarios
  for scale slopes and long churn. Do not repeat slow OS setup thousands of
  times when the high-volume claim is about plan lifecycle rather than watcher
  throughput.
- The small filesystem/watcher contract module runs on every supported CI
  platform using the existing platform jobs and the same test binary. Platform
  coverage may add a named test invocation, not a new target or dependency
  workspace.

## Cross-phase engineering rules

- Each phase ships a usable vertical behavior or a mechanical prerequisite for
  the immediately following slice. No phase may leave a second operational
  plan path "temporarily" available to consumers.
- Lowerers and transactions read as named semantic steps. Avoid coordinator
  files that also validate, allocate, compare, publish, and format diagnostics.
- Public plan types are minimized. Consumers should ask the active session to
  perform a typed operation, not learn how to assemble runtime internals.
- Full desired candidate truth and a replacement delta remain distinct. A
  delta is never treated as complete truth; a complete candidate is never
  rebuilt from an unbounded scan when exact predecessor proof is available.
- Denials name the failed phase and carry compact evidence/counters. Booleans
  such as `is_valid`, `same_plan`, or `supported` do not cross authority
  boundaries without a typed receipt or classification.
- Every new counter has one production increment site, a named semantic unit,
  a declared ordinary/reconstructive boundary, and an adversarial test that
  would fail if it drifted.
- Query-backed proof should reuse Worth Query Consumer Kit scenarios and
  authority checks when they establish Query semantics. Worth UI adds only the
  UI-specific registration, graph, allocation, plan, and execution assertions;
  it does not copy Query's test runtime or proof machinery locally.
- Every phase publishes compact evidence and indexes for the authority or
  execution family it introduces. Inspection queries consume those artifacts;
  they do not reconstruct the phase from logs, plan internals, or formatting.
- Existing `#[cfg(test)]` executor variants and production-side `for_test`
  constructors in a touched 3.9 execution family are migration residue, not a
  grandfathered proof lane. Move hostile injection below the authority decision
  into narrowly owned test support or test the real denial boundary.
- A fake, mock, enum label, copied receipt, injected source package, or
  manufactured host/watcher event cannot close a claim named after a production
  external mechanism. Doubles remain local proof tools only where the proof
  ledger also names the real-mechanism integration test for that contract.
- A collision-safety test does not need to discover a natural production-hash
  collision. It must force an equal narrowing key below the authority decision,
  exercise the real exact-region comparator, and pair that proof with a
  topology audit showing that no fingerprint-only branch can authorize reuse,
  no-op, publication, or execution. The hostile seam may alter derived narrowing
  evidence; it may not mint a candidate, predecessor proof, or activation
  authority.
- Tests must be capable of falsifying authority, equivalence, locality, cost,
  and atomicity claims. Passing the happy path is not milestone evidence.
- After Phase 4 establishes the publication shell and Phase 5 activates the
  first real bundle, every later phase that activates or executes a new plan
  constituent must publish the complete current-phase bundle and application
  generation through that same atomic shell. Phase 15 closes the transaction
  over the final cross-lane authority set; it is not permission for Phases 5
  through 14 to use a torn or bind-after-publication interim path.

## Phase 1: Freeze Honest Proof Topology and Reality Placement

**Vertical outcome:** Every later 3.9 slice has a named cheap proof home and a
named real-boundary proof home, so hostile evidence cannot be replaced by a fake
and cannot create a new compiler session, integration target, fixture workspace,
or duplicated application setup forest.

**Relevant subsystems and APIs**

- `scripts/ci/check_worth_ui_test_topology.py`
- `scripts/ci/worth_ui_test_topology_budget.json`
- `scripts/ci/run_worth_ui_test_lane.py`
- compile inventory/execution reconciliation CSVs
- `worth-ui-test-support` and the existing compiled-once certification scenario
  modules
- the existing `worth-ui-certification` `application_contracts` target and
  hostile-certification lane
- existing platform CI jobs and proof-lane timing evidence

**Deliverables**

- Extend the topology checker so every hard rule in the test-program
  constitution is mechanically enforced, including separate 14/6/3 executable
  target ceilings, 283/62/3 physical fixture ceilings, exactly two grouped
  Cargo compiler invocations, and the absence of generated compilation in
  ordinary tests.
- Define the schema-versioned timing-evidence artifact and record the opening
  structural/timing baseline using the prescribed lanes and comparable-run
  metadata. Its validator must already verify closing-command comparability,
  compute the 10 percent result, and require a concrete reviewed amendment for
  any over-budget closing median; a populated but unchecked comparison string
  is not evidence.
- Establish one narrow 3.9 scenario authority that creates application,
  committed-allocation, host-support, and replacement-delta inputs through
  production APIs. Split responsibilities rather than creating a universal
  fixture bag.
- Freeze the real-boundary matrix above in the topology/proof ledger. Assign
  filesystem/watcher, public application lifecycle, Query Consumer Kit, egui,
  headless, and allocator evidence to responsibility-named child modules of the
  existing `application_contracts` target.
- Keep those real-mechanism modules in hostile certification rather than the
  fast library-test lane. Add no package, `[[test]]` entry, compile case, or
  generated workspace to obtain black-box placement.
- Record the opening warm time for `application_contracts` separately so
  realistic certification cost cannot hide inside the full-lane median.
- Reconcile repeated current execution-plan/lane setup into typed scenario
  operations only where that reduces compilation or duplicated authority.
- Inventory test-only constructors, enum variants, and executor branches in the
  execution-plan, ordinary, virtualized, spatial, realtime, activation, and
  equivalence families. Assign each touched seam to real-boundary proof,
  below-authority test injection, or deletion; no existing fake operational
  path is implicitly retained.

**Warnings**

- Do not rewrite working tests merely to make them look uniform.
- Do not move production algorithms into test support or let a scenario builder
  mint private authority.
- Do not gate CI directly on noisy wall-clock duration; gate topology and record
  comparable timing evidence.
- Do not put filesystem notification waits, real egui frames, allocator probes,
  or long storms in the fast lane merely because a library test is easy to add.
- Do not centralize every real mechanism behind one scenario method whose body
  pre-solves source, Query, host, plan, and expected-output truth.

**Test requirements**

- Prove every retained pre-cutover proof maps to an equal or stronger ordinary,
  topology, or compile-contract proof; the migration may not silently drop a
  denial class.
- Seed checker fixtures for an extra compile Cargo session, nested Cargo call
  from an ordinary test, generated compile fixture, uninventoried fixture
  target, unexpected integration target, and compile-case budget increase; each
  must fail with a specific violation.
- Seed a malformed or incomparable timing-evidence artifact and prove the
  evidence validator rejects it without treating wall-clock noise as a topology
  failure.
- Prove two scenario variants with equivalent semantic input produce equivalent
  setup authority without recompiling a second fixture crate.
- Prove a scenario operation cannot bypass freshness, session, or graph
  authority even when all visible digest/count fields match.
- Seed the proof ledger with a fake-only filesystem claim, fake-only egui claim,
  receipt-only allocation claim, and real-boundary module assigned to the fast
  lane; the checker must reject each placement.
- Run the empty real-boundary scenario shell and prove it adds no compiler
  session, integration target, executable compile case, fixture workspace, or
  nested Cargo invocation.

**Engineering decisions**

- The one external compiler owner, two Cargo invocations, current target sets,
  and 14/6/3 executable targets are ceilings, not targets to fill. The compiler
  owner reuses the ordinary Cargo target graph instead of rebuilding the same
  platform through a private harness workspace.
- Runtime behavior is proven primarily in ordinary compiled-once tests.
- Cross-crate product claims are proven in the already compiled
  `application_contracts` binary; local algorithmic failures remain in the
  existing library-test targets.
- Source and topology laws use mechanical audits instead of compiler fixtures
  when the compiler is not the actual authority being tested.

**Open questions**

- None. The compiled target and proof-lane placement are frozen here; child
  module names follow the production responsibility they certify.

## Phase 2: Close Real Filesystem Source Ingress

**Vertical outcome:** An external editor can create or replace real `.wui`
files under a temporary workspace, the production filesystem/watcher boundary
freezes their bytes into one ordered source-package revision, and the public
application lifecycle consumes that revision without an injected source-text or
manufactured-event substitute.

**Relevant subsystems and APIs**

- `WorthUiSourceProvider`, filesystem provider identity, and source package
  acquisition
- `WorthUiSourceWatcher`, debounce, watcher events, and candidate ordering
  receipts
- file-authored lowering and inseparable candidate composition
- public prepared/active application facade
- existing `application_contracts` certification target and platform CI jobs

**Deliverables**

- Separate filesystem-root description from in-memory module injection. A
  filesystem provider reads its modules from the declared root through a
  production source-acquisition boundary; `with_file`-style text injection is
  limited to explicitly named in-memory/editor/generated providers.
- Add the real filesystem reader and production operating-system watcher
  adapter at a spatially obvious external-boundary location. The watcher emits
  candidate causes and ordering evidence; it does not decide semantic impact.
- Bound notification ingress independently of the settlement event batch.
  Overflow must coalesce into a retained final-tree resnapshot trigger rather
  than grow memory without limit or silently drop the only change indication;
  backend failure remains a distinct typed denial.
- Freeze one immutable source-package snapshot only after debounce/settlement.
  Parsing and application preparation consume that snapshot rather than
  rereading changing files piecemeal.
- Preserve the existing inseparable file/Rust candidate-composition authority.
  Real file acquisition changes the source mechanism, not the canonical
  lowering or activation path.
- Add responsibility-named real-filesystem and real-watcher child modules to
  the existing `application_contracts` test binary. Tests create ordinary
  temporary `.wui` files using OS file APIs; production Worth UI never writes
  authored source.
- Add a narrow cross-platform invocation of those same modules to the existing
  platform CI jobs, with no new test target or workspace.

**Warnings**

- A provider tagged `Filesystem` whose bytes came from `with_file` is an
  in-memory provider with a dishonest name, not filesystem proof.
- A test that passes handcrafted `WorthUiWatcherEvent` values proves debounce
  logic only. It cannot prove the production watcher adapter.
- Filesystem events are triggers, not authoritative impact or final source
  truth. Atomic rename, duplicate, missing, and reordered events converge by
  resnapshotting and typed ordering, not by trusting notification order.
- Moving the old target to a backup and then moving a pending file into the
  target path is not an atomic replacement: it has an observable absence
  window. Certification must use one same-filesystem operating-system replace
  operation whose platform contract overwrites the target atomically.
- A changed package digest does not prove changed canonical meaning. Where the
  test claims imported-module behavior, the imported declaration must change
  independently observed application/graph meaning and equal restored meaning
  must converge despite later notification sequence.
- A bounded local event vector over an unbounded notification channel is still
  an unbounded watcher design.
- Two immediate equal directory reads prove point-in-time consistency, not a
  settled source tree. Direct acquisition and the watcher's initial snapshot
  must cross a real quiet window; continuous startup churn must end in bounded
  typed denial rather than an indefinitely delayed or falsely settled launch.
- Do not use blind sleeps to make watcher tests pass. Establish watcher
  readiness before the external write and wait through a bounded settlement
  protocol with actionable timeout evidence.
- Do not let a temporary test directory become a generated Cargo workspace or
  compile fixture. It contains source data only.

**Test requirements**

- Write a valid `app/main.wui` to a new temporary directory, construct only the
  production filesystem-root provider, and prove the public file-authored path
  reads those exact bytes into the same canonical candidate meaning as an
  equivalent Rust-authored composition.
- Start the production watcher, establish readiness, perform a real temporary-
  file write plus one same-filesystem atomic replace over `app/main.wui`, and
  prove one settled source-package revision enters the public replacement path
  regardless of duplicate or platform-specific notification order. A
  multi-step backup/rename sequence does not satisfy this requirement.
- Perform a real partial/malformed write followed by a valid atomic replacement;
  the malformed candidate cannot change active application/runtime predecessor
  truth, and the valid settled package activates exactly once. This phase does
  not claim canonical active-plan authority before Phase 5.
- Create, modify, remove, and recreate an imported `.wui` module on disk; final
  canonical package meaning and ordering evidence must follow the frozen disk
  snapshot rather than an injected module list or raw event sequence. Assert
  canonical generation/graph consequences, not only source-package digests.
  The production package loader derives import edges from authored `import`
  declarations. Removing a still-declared target is an unresolved-dependency
  denial that preserves the last valid active generation; recreating equal
  target meaning converges despite its later source-revision sequence.
- Drive the bounded notification-queue algorithm past capacity below the real
  boundary and prove memory remains bounded while a resnapshot trigger survives;
  keep the real operating-system watcher contract as the independent proof that
  actual notifications reach that algorithm.
- Attempt to attach injected modules to a filesystem provider or route a
  caller-manufactured watcher event through the real-mechanism certification
  helper; construction must be unavailable or the proof ledger must classify
  the scenario as synthetic and insufficient for closure.
- Run the small real-filesystem/watcher contract on each supported CI platform
  and prove cleanup, path normalization, readiness, and timeout behavior leave
  no retained watcher, file handle, or temporary directory.

**Engineering decisions**

- External test code writes `.wui`; production Worth UI reads and watches it.
  Authored-source mutation is not a runtime responsibility.
- The production watcher uses `notify`'s recommended native backend, rejects
  polling/null backends, exposes readiness, and feeds a bounded coalescing queue.
  Settlement uses a nonzero quiet window within one caller-visible total
  deadline; a deadline-limited event wait may not begin another full quiet-window
  reread after the deadline expires.
- Source acquisition, OS notification translation, debounce/settlement, frozen
  package assembly, and candidate lowering remain named separate steps.
- In-memory providers remain valuable deterministic proof tools, but their type
  and proof classification state that they do not certify filesystem behavior.
- Phase 3 may not begin until a real disk edit can reach the public candidate
  pipeline and invalid disk state demonstrably preserves the active generation.

**Open questions**

- None. The native backend, readiness, bounded queue, quiet-window settlement,
  and explicit shutdown contract are frozen by this phase.

## Phase 3: Seal One Canonical Lowering Authority

**Vertical outcome:** A candidate application generation and its exact graph,
capability/Query posture, host support, and committed allocation cross one
admission boundary and become the only legal input to execution-plan lowering.

**Relevant subsystems and APIs**

- `WorthUiActiveApplicationSession`, prepared/lowered/staged replacement types,
  and prepared application generation identity
- `WorthUiPendingActivation`
- `UiCommittedAllocationLoweringInput`
- `WorthUiExecutionPlanInput`, its preparer/witness, basis, context, node inputs,
  hooks, counters, and denials
- lane admission and host-session support
- binding-owned installed Query and consumed-projection authority

**Deliverables**

- Introduce one sealed lowering authority that binds exact candidate application
  generation, graph authority, capability snapshot/Query posture, host-session
  support, and committed allocation lineage before plan construction.
- Split the existing circular predecessor path into two one-way authorities.
  Candidate admission first seals a narrow allocation-planning projection;
  allocation planning consumes it and emits committed allocation truth. Only
  then may the candidate authority and committed lineage seal execution-plan
  lowering. `WorthUiExecutionPlanInput` or any reconstruction-equivalent witness
  may not be an allocation-planner input, retained replan cache, or way to mint
  the post-commit lowering authority.
- Route launch and replacement through the same phase shape; their authority
  sources may differ, but plan construction may not.
- Fold every still-required fact from the provisional execution-plan input into
  this authority or a named derived constituent.
- Admit Query-bound constituents from the sealed binding-owned projection and
  installed-authority edge. Do not accept independently supplied Query basis,
  status, result-shape, source-label, fact-bag, or digest fields.
- Freeze a Query ownership/lifetime matrix for installed views, consumed
  projection authority, binding-owned handles, Query-owned live resources,
  compact plan references, and inspection references. Every launch, denial,
  no-op, activation, replacement, removal, and failed-publication transition
  must name whether each artifact is moved, borrowed, retained, observed, or
  released exactly once. Phase 3 implements and executes the rows reachable at
  its boundary; the first later phase that implements another transition must
  prove that row against the real lifecycle rather than a Phase-3 simulation.
  `plans/worth-ui/milestone-3.9-query-lifetime-matrix.csv` is the canonical
  mechanically checked matrix; Phase-3 rows are `proven`, while later rows
  remain `assigned` to their owning phases until their real lifecycle exists.
- Remove duplicated topology/Query/hook reconstruction and the parallel witness
  path. Delete or internalize predecessor APIs once their consumers move.
- Remove the artifact-digest early replacement `NoOp`; classification is deferred
  until the complete candidate plan is available.

**Warnings**

- Do not solve exact authority by cloning an entire graph or declaration tree
  into the lowerer.
- Do not encode candidate identity as another hand-folded digest.
- Do not permit a default host-support profile to participate invisibly; support
  that changes execution is explicit admitted input.
- Do not rename the provisional execution-plan input to “allocation projection.”
  The earlier projection is non-executable, contains only allocation-relevant
  candidate meaning, and cannot be expanded back into plan-lowering facts.

**Test requirements**

- Equivalent declaration/graph/allocation truth presented in different lawful
  input order lowers to equivalent canonical input facts.
- Equal artifact, capability, and allocation digests from a foreign graph or
  active session deny before node or handle construction.
- Missing Query posture for a Query-bound node, and unexpected Query posture for
  a Query-free node, produce typed denials without partial candidate state.
- A split or copied Query tuple with matching visible values but foreign
  installed-runtime authority denies before plan-row construction.
- Walk the Phase-3-reachable Query lifetime rows through real Consumer Kit
  installation, consumed handoff, lowering admission, lowering denial, and
  candidate discard. Mechanically prove the matrix assigns all later no-op,
  candidate-preparation denial, rebind, removal, bounded-replacement, and
  failed-publication rows to Phases 8, 13, 14, and 15 as applicable; do not
  simulate those future transitions to claim runtime proof early.
- Prove the allocation planner accepts only the sealed allocation-planning
  projection, the projection cannot construct plan rows or retain executable
  Query/host/hook authority, and a copied/reconstructed provisional plan input
  cannot satisfy either the allocation or post-commit lowering boundary.
- A candidate formerly accepted by the early digest no-op path must continue to
  lowering when host support, Query posture, or allocation meaning differs.
- In the existing `application_contracts` target, acquire one candidate from the
  real filesystem path closed in Phase 2 and one Query-bound candidate from a
  real Query Consumer Kit installation, then enter lowering only through the
  public application/session facade. Direct lowerer calls and locally minted
  Query handles cannot satisfy this integration proof.

**Engineering decisions**

- Exact sealed identities authorize; raw digests only accelerate comparison or
  support diagnostics.
- Committed allocation is a constituent of candidate plan authority, not a
  caller-provided sibling argument.
- One canonical ordered representation is established during admission so
  downstream phases do not sort or reconstruct it repeatedly.
- Query-installed and consumed authority remains owned by the binding/Query
  lifecycle. The lowerer receives only the sealed handoff and the plan retains
  only the compact admitted reference permitted by the lifetime matrix.

**Open questions**

- Whether the sealed authority borrows cold canonical artifacts during lowering
  or owns compact admitted projections should be decided from measured lifetime
  and allocation cost. It must never expose those artifacts to execution.

## Phase 4: Freeze Regional Plan Storage and Equivalence Proof

**Vertical outcome:** Before any lane representation broadens, the runtime owns
one complete-successor storage model and one collision-safe regional proof
kernel that can build, compare, reuse, retire, reclaim, and publish plan regions
within the admitted affected closure.

**Relevant subsystems and APIs**

- invalidation/impact affected-closure proof and committed allocation locality
- candidate and active plan bundle region representation
- predecessor-region correspondence and unchanged-region proof
- execution-plan equivalence schema, fingerprints, and exact comparison
- active-session activation transaction and predecessor reclamation boundary

**Deliverables**

- Define the plan-region identity and canonical region ordering consumed by
  every later lane constituent.
- Select and implement the storage model--copy-on-write regions, immutable
  arenas, or another measured representation--before ordinary, data, spatial,
  and realtime rows depend on it.
- Encode full desired successor truth separately from the admitted delta. A
  candidate may share exact immutable predecessor regions internally, but the
  published bundle is always a complete successor view.
- Carry sealed predecessor-region proof from impact/allocation authority into
  lowering. Matching names, indexes, generations, or fingerprints are never a
  substitute.
- Define the executable-equivalence schema spine and per-region exact-comparison
  contract. Fast fingerprints may reject or narrow; equality may authorize
  reuse/no-op only after exact comparison of the admitted changed region.
- Define slot preservation, changed-slot retirement, insertion, deletion,
  reparenting, lane-transition, and reclamation semantics independent of any
  one lane's row shape.
- Establish the activation transaction shell that can atomically publish one
  complete successor bundle and reclaim predecessor-only storage after commit.
  Later phases add constituents; they do not redesign publication.
- Emit compact regional-lowering/equivalence evidence and index it by candidate,
  predecessor, affected closure, and region identity.

**Warnings**

- Copying all plan vectors and reporting only changed counters is not bounded
  replacement.
- A fingerprint match is not predecessor proof and cannot preserve a slot or
  authorize a semantic no-op.
- Dense execution layout may not make unrelated insertions renumber every
  authoritative handle. Separate stable slot identity from iteration layout
  where the workload requires it.
- Structural sharing is derived immutable storage, not shared mutable authority
  between active and candidate plans.

**Test requirements**

- Express the same affected closure and successor through different lawful
  delta orderings; canonical regions, exact equivalence, reuse decisions, and
  complete successor observations must converge.
- Force equal fingerprints for structurally different changed regions and
  prove exact comparison rejects reuse/no-op without a test-only production
  constructor or forged authority path.
- Change one leaf while scaling unrelated predecessor regions by two orders of
  magnitude; region construction, comparison, and storage-copy counters remain
  bounded by the admitted closure and changed output.
- Exercise insert, delete, reparent, and lane-transition classifications before
  concrete lane payloads exist; unchanged slots retain exact predecessor proof,
  changed slots retire, and stale handles cannot resolve.
- Inject failure before and after every fallible transaction-shell step; the
  active predecessor and its storage remain coherent, and candidate-only
  regions are reclaimed exactly once.
- Run a representation-level churn storm and prove retained storage returns to
  the active `O(P)` envelope rather than growing with replacement history.

**Engineering decisions**

- The storage/reuse model is foundation, not a Phase 14 optimization.
- Executable equivalence has an early schema spine. Phase 12 closes field
  coverage after all lane constituents exist.
- Complete-successor publication and delta-bounded construction are simultaneous
  requirements; neither weakens the other.
- The transaction shell becomes infallible after its final validation boundary
  or carries mechanically complete rollback.

**Open questions**

- The concrete region storage representation remains open only until this phase
  measures representative `A/P` workloads. Phase 5 may not begin while that
  choice is unresolved.

## Phase 5: Activate and Execute the Minimal Ordinary Plan Slice

**Vertical outcome:** A root shell and one component lower from canonical truth
into a sealed plan bundle, activate atomically, and execute only through the
active application session.

**Relevant subsystems and APIs**

- `WorthUiExecutionPlan`, topology, lane partitions, and lookup index
- `WorthUiOrdinaryLanePlan` and ordinary plan builder/executor
- `WorthUiActiveExecutionPlan` and active runtime state
- `WorthUiFrameworkTurnExecution`
- committed-allocation activation preflight/publication and plan-swap receipt

**Deliverables**

- Define the sealed candidate/active plan bundle and make shared topology plus
  the ordinary root/component lane its first complete constituent.
- Implement that bundle on the regional storage, predecessor-proof, and
  transaction shell frozen in Phase 4; the minimal slice may not introduce a
  flat rebuild-only representation that later lanes must undo.
- Replace digest-only active-plan state with ownership of the actual active
  bundle and exact application/session generation binding.
- Add an active-session execution capability that borrows the active plan.
  Remove caller-supplied ordinary plan execution from the product facade.
- Make launch publish a real initial plan rather than a digest synthesized from
  artifact and capability digests. The chosen lifecycle must either deny launch
  until committed allocation can produce that plan, or retain a typed
  non-executable bootstrap posture until first plan activation; it may not hand
  out an active session backed by a fabricated empty plan.
- Emit compact ordinary execution receipts without per-frame general-purpose
  heap allocation.
- Introduce the first production form of the minimal sealed host-output envelope
  for the ordinary root/component slice and make the headless adapter consume
  it. Phase 11 extends this same boundary across all lanes; it may not replace a
  temporary receipt-to-host or contact lane introduced here.
- Add the thread-scoped armed allocator observer to the existing
  `application_contracts` binary now that the first real executor exists. Arm it
  only around active-plan execution; Phase 16 reuses it to reconcile the final
  cross-lane counter schema rather than introducing the first independent
  allocation proof after the claim has already shipped.
- Index minimal-plan lowering, activation, and ordinary-frame receipts by exact
  candidate/active generation for summary and evidence-reference inspection.

**Warnings**

- The active plan bundle is derived runtime state, not a second application or
  graph authority.
- Do not leave public plan-preparation methods as an alternate product path.
- A root-shell target may intentionally touch its admitted ordinary subtree; it
  may not scan unrelated lanes or cold topology.

**Test requirements**

- Two lawful initial applications with equivalent execution meaning produce
  equivalent minimal ordinary bundles and equivalent frame receipts.
- A stale plan object, foreign session, forged visible digest, or predecessor
  component handle cannot execute through the active session.
- Attempt to obtain or use frame execution before initial committed
  allocation/plan activation. A lifecycle that requires initial publication at
  launch must deny before yielding an active session; a lifecycle that permits
  a pre-publication session must return a typed bootstrap denial. Neither branch
  may fabricate an empty plan.
- Inflate unrelated non-ordinary topology by two orders of magnitude and prove
  component-target work/counters do not grow.
- Prove an ordinary frame performs zero forbidden cold work and zero
  general-purpose heap allocations while still touching the intended component.
  The allocation assertion uses the armed observer around the real public
  active-plan call, not an always-zero runtime counter or test executor.
- Inspect the minimal active plan and frame receipt through the formal harness;
  the query stays budgeted and cannot borrow executable plan ownership.
- Launch a real `.wui` application from the Phase 2 filesystem boundary through
  the public facade, activate its initial minimal plan, and execute it through
  the production headless host. Independently observe active generation and
  host-neutral output so a digest-only or no-op executor would fail the test.
- Prove the production headless adapter receives only the Phase-5 sealed
  envelope, and that the envelope has a typed extension/refinement path for the
  additional lane constituents in Phase 11 rather than a disposable host seam.

**Engineering decisions**

- The session lends execution; it does not export an owned active plan.
- Lane-ready rows are built during lowering/activation, never on first use in a
  frame.
- Frame receipts carry compact indexes/ranges or refs rather than allocating
  vectors of touched handles.

**Open questions**

- The exact borrow shape between framework-turn execution and the active plan
  should be chosen to preserve transactional source-turn semantics without
  widening plan visibility.

## Phase 6: Complete the Compact Handle Arena

**Vertical outcome:** Every current handle family resolves directly against one
exact active session arena and current slot generation, cannot cross family,
slot-generation, or session boundaries, and survives a whole-plan replacement
only when predecessor proof preserves the exact slot meaning.

**Relevant subsystems and APIs**

- runtime handle allocation, basis, receipt, family widths, session-arena
  identity, slot generation, and counters
- component, command, token, child-range, view-binding, lane, and state-slot
  handle types
- plan topology and lane indexes

**Deliverables**

- Back typed handles with dense plan-owned arenas or equivalent direct-indexed
  storage; remove binary-search lookup tables where the handle already carries
  an admitted plan index.
- Separate exact session-arena and slot-generation authority from semantic
  equivalence fingerprints and whole-plan generation evidence.
- Preserve unchanged handles across a replacement only when carried
  predecessor-region proof establishes the same slot meaning; otherwise mint a
  new generation or slot.
- Add explicit capacity, index-conversion, and generation-exhaustion denial.
- Prevent any public constructor, raw integer conversion, or global registry
  lookup from minting authority.
- Emit compact handle-resolution/denial evidence indexed by session arena,
  family, slot, slot generation, and target without exposing raw minting parts.

**Warnings**

- Stable handle reuse must not become stable authority across changed meaning.
- A hash-shaped handle or digest-derived generation is not collision-safe
  authority.
- Do not allocate a second vector per family if one tagged arena plus compact
  family views is measurably simpler and cheaper; choose from evidence.

**Test requirements**

- The same slot and slot generation under a foreign session arena, and the same
  arena/slot generation under a wrong family, both deny before target touch.
- Reordering equivalent admitted source facts preserves semantic plan
  equivalence without making handles publicly interchangeable between sessions.
- Exercise `u32` index/capacity and generation boundaries without allocating an
  enormous fixture; exhaustion must deny before truncation or wraparound.
- Scale unrelated families and prove lookup counters for one handle remain
  constant and registry/string counters remain zero.
- Inspect a valid, stale, wrong-family, and foreign-session handle outcome; the
  evidence localizes the failed check without allowing the inspection receipt
  to resolve or recreate the handle.

**Engineering decisions**

- Compact handles remain typed and non-mintable.
- Resolution validates session arena, slot generation, and family before
  dereferencing executable meaning.
- Direct index access is the ordinary path; reconstructive scans are not an
  accepted fallback.

**Open questions**

- Choose arena partitioning after measuring memory layout and borrow complexity;
  the choice must fit the Phase 4 region/reclamation model, and the public
  handle contract must not expose that internal layout.

## Phase 7: Lower Commands, Tokens, Children, and State Through Ordinary Execution

**Vertical outcome:** The ordinary plan executes component shells, child ranges,
commands, token/style support, and state slots as one coherent lowered slice
without consulting authored identifiers during a frame.

**Relevant subsystems and APIs**

- ordinary lane plan/node/target/receipt/counters/certification
- command, token, child-range, component, and state-slot handles
- plan child ranges, region structure, lane partitions, and render refs
- durable state reconciliation receipts

**Deliverables**

- Lower all ordinary families into canonical lane-ready rows and compact child
  ranges with direct handle resolution.
- Carry admitted durable-state succession into state-slot plan meaning without
  letting the plan decide reconciliation.
- Remove repeated family index vectors or maps that duplicate arena authority.
- Make ordinary receipts distinguish intentional subtree breadth from forbidden
  full-plan scanning.
- Publish ordinary-family lowering and target-breadth evidence through the same
  plan/handle indexes established in earlier phases.

**Warnings**

- Token/style support must carry native admitted meaning, not recover names from
  a registry.
- Child ranges must be canonical and bounds-checked; overlapping or unsorted
  ranges cannot be repaired silently.
- A command or state transition may not execute through a component-shaped
  fallback.

**Test requirements**

- Equivalent ordinary meaning with reordered declarations produces equivalent
  lane behavior and receipts.
- Swap command/token family tags, overlap child ranges, or bind state succession
  to a foreign predecessor; each must deny before active publication.
- Change one leaf command in a large ordinary tree and prove replacement work is
  bounded by the admitted affected closure while unrelated handles remain
  stable.
- Execute component, command, and token targets repeatedly and prove zero source
  parsing, string lookup, broad scan, and per-frame allocation.
- Request summary inspection for child-range, command, token, and state-slot
  targets; each response is bounded by the requested target and preserves the
  owning authority distinction.

**Engineering decisions**

- Child traversal uses plan-owned ranges, not graph traversal.
- Durable state authority remains in reconciliation; the plan carries only the
  admitted slot/reference needed to execute.
- Family-specific behavior is explicit, not hidden behind a generic helper bag.

**Open questions**

- If root-shell traversal legitimately touches all ordinary rows, its receipt
  must name that target breadth distinctly from a broad-scan violation.

## Phase 8: Lower Admitted View References Into the Virtualized Data Slice

**Vertical outcome:** An already-admitted Query view reference lowers into a
range-aware execution slice and executes through an active view-binding handle;
the phase proves plan/handle/cost substrate without claiming the broader
collection-projection and live-patch product semantics owned by Milestones 3.13
and 6. A Query-free application follows the same lifecycle without synthetic
Query state.

**Relevant subsystems and APIs**

- Worth Query installed snapshot/live declaration types, binding identities,
  admitted posture, preservation receipts, and opaque Query-owned managed-live
  resources already exposed by the binding boundary
- view-binding handles
- `WorthUiVirtualizedDataPlan`, visible ranges, binding lifecycle posture,
  targets, receipts, counters, and certification
- active application Query status and inspection links

**Deliverables**

- Lower already-admitted Query view references and execution requirements into
  plan-owned view-binding rows without storing raw Query registries,
  interpreting collection meaning, or rediscovering bindings per frame.
- Consume one sealed lifecycle-specific
  `WorthUiQuerySnapshotProjectionOutcome` or
  `WorthUiQueryLiveProjectionOutcome` binding-owned authority envelope. Retain
  only compact references whose provenance and installation generation can
  still be checked; do not trust copied basis digests, support flags,
  projection bags, or source labels. The snapshot-only admission surface must
  be unable to accept the live envelope.
- Preserve Query's lifecycle distinction in Worth UI types. An installed
  snapshot view may expose one-shot read/projection; an installed live view may
  expose only Query-owned `open`/managed-resource entry. A lifecycle-erased
  registration envelope may exist only after that distinction has selected the
  legal operation; it may not offer a shared convenience method that silently
  executes live meaning as a snapshot read.
- Admit a live resource and its consumed projection authority as one affine
  binding transaction. Any denial returns the still-owned resource or routes it
  into Query's abandonment/cleanup lane; it may not consume half the pair,
  publish a settlement without its resource, or leave caller folklore to infer
  whether cleanup is required. Query-owned provenance must prove that the
  projection came from that exact managed-resource generation; matching view
  definition and installed-domain authority are insufficient.
- Preserve exact Foundational native aspect and struct values through
  refinement and plan admission. Any derived plan scalar must have a named,
  lossless contract from the native value rather than a string/JSON/widened
  numeric conversion.
- Bind Query plan rows to exact candidate/active application generation and any
  already-admitted opaque managed-resource or continuation evidence exposed by
  Query. This phase may route and retain that evidence but may not manufacture
  patch/rebind semantics or define their meaning.
- Preserve Worth UI visible-range semantics and make execution proportional to
  the pre-admitted requested window. Query cursor construction, pagination
  legality, collection materialization, and query-shaped patch production stay
  Query-owned and outside this phase.
- Represent Query absence explicitly and cheaply; remove dummy bindings or
  special lifecycle paths.
- Publish compact view-row/range execution evidence and link it to Query-owned
  inspection references without copying Query explanations.

**Warnings**

- The UI plan consumes admitted Query meaning; it does not recreate Query basis
  or authority.
- Offset pagination is not a cursor substitute, but this phase does not mint or
  interpret Query cursors. A missing admitted range denies rather than widening
  to a full plan-row or collection scan.
- Query status diagnostics must not be materialized during ordinary data-lane
  execution.
- Query owns installed-domain, live-resource, subscription, result-state,
  recovery, and disposal semantics. The plan records admitted references and
  coordinates lifecycle handoff; it must not implement a local resource
  manager.
- Successor binding preparation must finish before predecessor release. Phase 8
  owns rollback of any candidate-only projection/resource it first creates;
  Phase 15 later re-proves that rule inside the complete cross-family atomic
  publication transaction rather than introducing Query cleanup for the first
  time.
- Do not add a new Worth UI installed-domain collection capability merely to
  make this phase's tests look product-real. Such a capability requires a
  visible spec and roadmap amendment.

**Test requirements**

- Equivalent already-admitted view references and visible ranges produce
  equivalent execution receipts across lawful candidate generations.
- Equal visible output with a changed Query binding identity, required surface,
  installed reference, or managed-resource generation is non-equivalent or
  denied as appropriate.
- Equal basis digests and native values from a foreign Query runtime or
  installation generation deny, while the same lawful binding expressed
  through equivalent native aspect values remains equivalent.
- A test proves native `AspectValue`/`StructAspectValue` meaning reaches the
  plan edge without JSON, text parsing, numeric widening, or a UI-local fact
  mirror.
- A stale view handle and empty/overflowing/missing admitted range deny before
  row execution. Offset-pagination and full-scan hostile cases are proven at
  the range/counter admission boundary without test-only executor variants.
- Grow the admitted row-reference population and unrelated graph independently;
  frame work remains bounded by the visible range and forbidden counters stay
  zero. This is not represented as proof of Query collection execution.
- Request summary and evidence-reference inspection for the active view row;
  the response links to Query-owned evidence, stays within budget, and cannot
  recover consumed projection or installed authority.
- Install the real Worth UI Query domain through the Query Consumer Kit, consume
  a real projection, lower its admitted reference through the public application
  lifecycle, execute a visible range, replace/remove it, and observe Query-owned
  live-resource succession and exact-once disposal. Replacement/removal must use
  the Phase-4 publication shell for the complete Phase-8 bundle and application
  generation; a torn interim cutover or certification-construction handle cannot
  close this test.
- Prove snapshot/live lifecycle typing through the existing compile-contract
  equivalence class or a cheaper source/type audit under the frozen topology:
  snapshot views cannot open managed resources, live views cannot use the
  one-shot snapshot path, and lifecycle erasure cannot restore either forbidden
  operation.
- Deny live-resource/projection admission after both affine inputs exist and
  prove the denial returns or Query-reaps the candidate resource while the
  predecessor binding remains executable. Then force candidate preparation or
  Phase-4 publication-shell denial and independently observe the same
  candidate-only cleanup. Phase 8 must close this first-path resource rollback;
  Phase 15 repeats it under the final complete transaction.

**Engineering decisions**

- Query execution identity participates in the operational equivalence basis;
  diagnostic presentation does not.
- Query binding equivalence consumes binding-owned exact authority/equivalence
  evidence; UI code never derives it from labels or digest equality.
- Snapshot and live installed views are different lifecycle types; their common
  registration form carries definition identity, not executable convenience.
- Live-resource plus projection admission is atomic and recoverable. Published
  binding state never contains one without the other.
- View binding resolution is direct through the active plan.
- Query-free parity is ordinary supported behavior, not a test-only exception.
- Worth UI visible-window metadata belongs in the plan-owned virtualized row;
  the direct handle index resolves that row without a separate caller-owned
  side arena. Query cursor representation is not part of this decision.

**Open questions**

- None. Phase 8 freezes exact-reference preservation, successor preparation,
  candidate-only cleanup, and Query-owned retirement handoff; Phase 14 adds
  sustained-churn locality proof and Phase 15 closes the final cross-family
  transaction without reopening those ownership decisions.

## Phase 9: Lower Canvas and Spatial Execution End to End

**Vertical outcome:** Canvas draw, hit-test, viewport, overlay, and tool-state
meaning are admitted during lowering and execute from one active spatial plan
without ordinary-widget fallback.

**Relevant subsystems and APIs**

- canvas/spatial plan, node, viewport, overlay, hit-test, tool-state, admitted
  extension-hook evidence, targets, receipts, counters, and certification
- extension-hook and lane admission
- render-resource refs and lane handles

**Deliverables**

- Lower admitted spatial hook meaning into the lane-support digest and
  generation-bound certification alongside viewport meaning, hit-test plan,
  overlay plan, tool-state access, and render-resource references. Do not mint
  duplicate hook-token objects from facts already sealed by host admission.
- Bind every extension hook and resource ref to the exact host support and plan
  generation that admitted it.
- Execute spatial targets directly from the active plan without graph search,
  hook registry lookup, or ordinary layout reconstruction.
- Make spatial scale terms explicit: visible primitives, queried hit-test
  region, and touched overlay/tool rows.
- Index spatial lowering and frame receipts by plan region, viewport, admitted
  hook/resource reference, and target for budgeted inspection.

**Warnings**

- A hook is an admitted extension point, not an alternate semantic authority.
- Missing spatial support must deny during lowering, not quietly reclassify the
  node into an ordinary lane.
- Renderer resources must not be validated by string or global registry on the
  frame path.

**Test requirements**

- Equivalent spatial declarations, hook admissions, and viewport facts produce
  equivalent plan behavior despite lawful source reordering.
- Foreign/stale hook admission, wrong resource generation, missing hit-test
  support, and ordinary-lane fallback each deny before candidate publication or
  frame work.
- Add a large unrelated ordinary/data plan and prove one spatial target's work
  and counters do not grow.
- Mutate one spatial region in a large canvas and prove only its admitted
  affected closure is rebuilt while untouched plan regions remain reusable.
- Inspect a spatial denial and a successful target without materializing the
  full spatial index or giving inspection access to renderer-resource authority.

**Engineering decisions**

- Spatial strategy is selected during lowering and represented explicitly in
  the plan.
- Hook admission and resource authority remain distinct even when one operation
  needs both.
- Frame receipts expose compact touched-region evidence, not materialized node
  lists.

**Open questions**

- The internal spatial index may be dense, tiled, or hierarchical; select it
  from scale evidence while preserving the same host-neutral plan contract.

## Phase 10: Lower Realtime Overlay Execution End to End

**Vertical outcome:** HUD and high-frequency overlay work executes from an
admitted active plan and renderer-surface capability under an explicit frame
policy, never through a hidden ordinary pass.

**Relevant subsystems and APIs**

- HUD plan/node/builder, realtime overlay lane, targets, receipts, counters, and
  certification
- high-frequency frame policy
- admitted realtime extension-hook evidence
- renderer-surface admission and handles

**Deliverables**

- Lower HUD rows, admitted overlay-hook meaning, renderer-surface references,
  and frame policy into the active plan bundle; the sealed support digest is the
  hook receipt rather than a second inert hook-token type.
- Bind renderer surfaces to exact platform/session and plan generations.
- Keep the high-frequency path free of ordinary layout, declaration/graph
  access, report materialization, and general-purpose allocation.
- Make budget exhaustion and unsupported policy explicit typed outcomes.
- Index realtime lowering, budget, renderer-surface, and target receipts for
  summary/evidence-reference inspection without materializing frame reports.

**Warnings**

- Realtime priority cannot suppress correctness counters or skip authority
  checks.
- A renderer-surface handle is not platform identity.
- A high-frequency policy must not become a generic callback that can perform
  unmeasured work.

**Test requirements**

- Equivalent HUD meaning, support, and policy produce equivalent plan and frame
  receipts.
- Stale/foreign renderer surfaces, wrong overlay hooks, zero/overflowing budget,
  and ordinary-widget fallback deny before draw work.
- A hostile custom hook that attempts hidden layout, broad scan, or counter
  suppression must fail certification.
- Scale unrelated plan lanes and HUD row count separately; execution work grows
  only with the explicitly targeted realtime rows.
- Inspect a realtime budget denial and successful receipt; evidence retains the
  exact plan/surface generation while opening no draw or renderer authority.

**Engineering decisions**

- Realtime policy is immutable plan input for a generation unless a typed
  replacement changes it.
- Renderer and hook validation occurs off the high-frequency frame path.
- Realtime receipts remain compact enough to aggregate without allocation.

**Open questions**

- If host surface lifetime can end independently of application replacement,
  define the smallest typed invalidation transition rather than adding a frame
  lookup.

## Phase 11: Close the Cross-Lane Bundle and Minimal Host-Output Boundary

**Vertical outcome:** One candidate/active plan bundle contains all admitted
lanes, and active-session execution produces one minimal sealed host-output
envelope. The host adapter receives that envelope, never an owned plan, authored
meaning, lane builder, or authority to choose execution strategy.

**Relevant subsystems and APIs**

- all four lane plans and lane-meaning parity certification
- lane admission/support descriptors and extension hooks
- egui-named boundary input/contact/plan types
- minimal host-output envelope identity, active-plan binding, and lane payload
  references
- runtime handoff and Worth UI product facades
- active-session/framework-turn execution surfaces

**Deliverables**

- Seal shared topology, handle arenas, all lane-ready plans, support basis,
  resource refs, equivalence basis, and counters into one internally coherent
  bundle.
- Validate cross-lane ownership and parity once during candidate construction;
  the frame path consumes the result.
- Replace egui-specific canonical-plan meaning with a host-neutral minimal
  output-envelope contract, extending the production ordinary envelope
  introduced in Phase 5 across every admitted lane, and lower its sealed
  contacts into egui only inside the adapter.
- Bind the envelope to the exact active application/plan generation and compact
  lane execution receipts. Its fields are the minimum needed to prevent a
  throwaway contact lane; complete mounted-node facts, participation, geometry,
  accessibility, and observation contracts remain Milestone 3.10 work.
- Make the envelope the sole runtime-to-host execution output. Milestone 3.10
  must be able to extend/refine it into mounted node/frame receipts without
  changing active plan ownership or reopening adapter plan access.
- Add a real egui frame driver to the existing `application_contracts` target.
  It supplies `RawInput`, calls `egui::Context::run`, and routes only the sealed
  host-output envelope through the production `worth-ui-host-egui` adapter.
  Add only the dependencies needed by that existing target; do not create an
  egui-specific integration binary.
- Remove public/caller plan builders and plan-accepting frame methods after all
  consumers use active capabilities.
- Add mechanical visibility/dependency checks that prevent reintroduction of
  raw topology/lane-plan execution through product facades.

**Warnings**

- A generic `AnyLane` or default branch is not forward-compatible design; it
  hides missing cost and failure semantics.
- Host neutrality does not mean an untyped property bag.
- The minimal envelope is not permission to pre-build the complete mounted
  receipt graph or move Milestone 3.10 observation semantics forward.
- Do not duplicate common topology into every lane merely to simplify builders.

**Test requirements**

- Equivalent meaning routed through headless and egui hosts yields equivalent
  host-neutral plan constituents and envelope meaning before adapter lowering.
- Conflicting lane ownership, missing support, duplicate plan index, foreign
  hook/resource admission, and egui-only meaning in the canonical plan each
  fail before publication.
- A node transition between lane regimes is classified as non-equivalent and
  cannot leave residue in the predecessor lane.
- A stale/foreign envelope, envelope bound to the wrong active plan, and an
  adapter attempt to recover owned plan or authored meaning each deny before
  native host work.
- Summary/evidence-reference inspection links the envelope to its active plan
  and lane receipts without materializing complete mounted-node evidence.
- Source/topology audits prove product consumers cannot import builders, mint
  bundles, or call an executor with an owned plan object; use no new compile
  fixture for these placement laws.
- Execute equivalent host-neutral meaning through a real egui context and the
  production headless adapter. Assert Worth envelope/generation parity before
  host lowering and independently inspect the real egui `FullOutput`, input,
  measurement, or contact consequence admitted by this phase. Complete mounted-
  node rendering remains Milestone 3.10; an egui enum, capability profile, or hand-
  built contact cannot satisfy this test.
- Drive at least two real egui frames across a public application replacement
  and prove the minimal native input, measurement, or contact consequence owned
  by this phase changes only after atomic activation; stale/denied replacement
  leaves both active generation and that egui-observed consequence on the
  predecessor. Complete mounted-node output remains Milestone 3.10 work.

**Engineering decisions**

- The closed lane set is represented exhaustively.
- Host adapters translate contacts and observations; they never choose UI
  meaning or plan strategy.
- The minimal envelope is the narrow authority foundation that 3.10 extends
  into complete mounted receipts; it must be refined, not replaced by a
  parallel output path.
- Cross-lane shared facts have one owner and are referenced, not copied into
  competing authorities.

**Open questions**

- Freeze only the minimal envelope fields required for safe 3.9 adapter
  execution. Every field must either remain valid in 3.10 or have an explicit
  typed refinement into a mounted receipt field; a disposable compatibility
  contact is not allowed.

## Phase 12: Close Canonical Executable Equivalence

**Vertical outcome:** The regional proof kernel from Phase 4 now covers every
execution-affecting constituent in the complete cross-lane bundle, and the
runtime can classify complete executable meaning without treating a digest or
incidental generation as proof.

**Relevant subsystems and APIs**

- execution-plan equivalence basis, digestor, counters, and reuse classification
- candidate/active plan bundle
- per-region predecessor proof and replacement locality
- lane support, Query, hooks, resources, frame policy, and durable-state slot
  meaning

**Deliverables**

- Close the complete executable-equivalence schema introduced in Phase 4 and
  document why every lane, Query, support, hook, resource, state, policy, and
  host-output field participates or is excluded.
- Separate semantic fingerprints from exact activation/session/generation
  authority.
- Compare changed regions exactly using carried predecessor proof; use digests
  only for narrowing and diagnostics.
- For Query-backed rows, compare the binding-owned executable contract and
  installed/live authority posture. Do not compare reconstructed UI-local
  summaries of Query meaning.
- Replace the two-state reuse classification with a typed decision that can
  distinguish exact semantic no-op, bounded changed regions, required rebuild,
  and denial without conflating them.
- Make canonical ordering part of construction so equivalence never depends on
  map iteration or source order.
- Publish plan-equivalence summary/evidence-reference inspection over the same
  regional receipts used by the decision; rich explanation remains explicit
  `O(D)` materialization.

**Warnings**

- Including every generation-bearing handle in semantic equivalence makes
  lawful no-op replacements impossible; excluding changed support or resources
  makes unsafe no-ops possible.
- Digest equality followed by `Reusable` is not collision-safe.
- Diagnostic/counter formatting changes should not churn an operational plan.

**Test requirements**

- A table-driven ordinary test varies source order, incidental candidate
  generation, diagnostics policy, each execution-affecting lane/support/Query/
  hook/resource field, and proves the expected equivalence classification.
- Force equal test fingerprints for structurally different changed regions and
  prove exact comparison rejects the no-op without a production test-only
  authority bypass.
- Equivalent plans with freshly minted handle generations classify semantically
  equivalent while stale predecessor handles still cannot execute.
- Omit one equivalence field in a seeded test schema and prove the schema/field
  coverage audit detects drift.

**Engineering decisions**

- Equivalence is a typed proof over complete executable meaning.
- Authority validation precedes comparison; equivalence never authorizes a
  foreign candidate.
- Per-region proof is the scalability mechanism; whole-plan equality is not the
  ordinary replacement path.

**Open questions**

- None. The exact changed-region representation was selected in Phase 4; this
  phase may refine constituent encoding only if the same collision-safety and
  `O(A)` contracts remain mechanically proven.

## Phase 13: Make Semantic No-Op a Complete Plan Decision

**Vertical outcome:** A fully lowered, exactly admitted candidate with equivalent
executable meaning produces a typed no-op receipt and leaves every active
authority and resource unchanged.

**Relevant subsystems and APIs**

- application replacement preparation and current early no-op surface
- execution-plan equivalence decision
- plan-swap receipt, frame scheduler, invalidation and allocation ledgers,
  durable state, active inspection, and application generation
- reload counters and diagnostics

**Deliverables**

- Classify no-op only after canonical candidate plan construction, exact
  candidate authority validation, and complete executable equivalence.
- Return a typed no-op receipt that identifies the admitted candidate, active
  generation, equivalence proof, and measured work without pretending a swap
  occurred.
- Preserve the active application generation, plan bundle, handle generation,
  ledgers, scheduler, state, host session, Query binding, and inspection
  authority on no-op.
- Classify non-operational source-span/provenance drift separately. When the
  candidate carries newer derived observation metadata, publish at most a
  typed provenance/inspection refresh that cannot change application, plan,
  handle, Query-resource, scheduler, or frame authority.
- Ensure candidate-only allocations and diagnostics are discarded or released
  exactly once.
- Prove the Query lifetime matrix's no-op row against the real complete-plan
  decision: installed authority, live resources, compact plan references, and
  inspection references remain on their exact predecessor lifetimes unless a
  separately typed non-operational observation refresh is admitted.

**Warnings**

- Equal source or artifact digest is not enough to skip lowering.
- No-op must not publish a fresh plan generation merely to report success.
- Candidate inspection remains candidate-scoped even when its plan is
  equivalent.
- A provenance refresh must not turn candidate inspection into active
  operational truth or let source metadata participate in plan equivalence.

**Test requirements**

- Equivalent candidates across source order and incidental candidate generation
  produce a no-op with identical active observations and zero swap/publication
  counters.
- Equal artifact digests with changed Query posture, lane support, hook,
  resource, frame policy, or allocation meaning must not produce a no-op.
- Repeat thousands of equivalent replacements and prove no handle-generation,
  retained-memory, registry, scheduler, or inspection-authority drift.
- Apply source-formatting and source-span-only changes whose executable meaning
  is identical; prove the plan/application generation remains stable while the
  typed derived provenance view either refreshes atomically or explicitly
  reports that the active source mapping remains the prior admitted mapping.
- Attempt to attach foreign executable facts to a valid provenance refresh;
  the refresh denies without changing active operational or observation truth.
- Interrupt no-op processing at every fallible pre-decision boundary and prove
  the active predecessor remains usable.

**Engineering decisions**

- No-op is a plan decision, not an admission shortcut.
- Candidate work is observable through reload counters even when activation work
  is zero.
- Executable activation and derived provenance refresh are different outcome
  families and counter rows. Observation metadata cannot smuggle an operational
  publication.
- The no-op receipt is separate from `WorthUiPlanSwapReceipt` unless the latter
  is refactored into an exhaustive activation outcome.

**Open questions**

- Decide whether the public application replacement outcome is one exhaustive
  enum or separate receipts; it must make operational no-op, derived provenance
  refresh, and executable publication impossible to confuse.

## Phase 14: Apply Bounded Structural Replacement and Stable Reuse

**Closure note (2026-07-19):** Closed. Regional plan storage, exact stable-slot
reuse, canvas/realtime/virtualized lane-local successor sealing, Query-owned
regional succession, and the allocation-catalog successor all preserve one
complete active truth while doing ordinary replacement work only for the exact
affected closure. Application replacement accepts a candidate-graph-owned
changed/removal delta; the runtime derives affected predecessor scopes, denies
missing, overlapping, unjustified, or stale authority, replans changed rows,
and carries unaffected rows through immutable persistent indexes. Graph,
invalidation, scroll, portal, Query, host, and durable lookup truth update from
that same delta and fail closed if predecessor-derived indexes have diverged.
The public successor receipt reports row dispositions and delta-local counters.
Public real-source tests prove unchanged-row carry and a removal-only empty
successor, a 128-row activated-catalog test proves retained predecessor storage,
and the replacement storm proves no carried-row scan or historical growth.

**Vertical outcome:** A non-equivalent candidate replaces only its admitted
affected plan regions while producing one complete desired successor plan.

**Relevant subsystems and APIs**

- impact lookup and invalidation narrowing
- committed allocation replan and allocation truth delta
- candidate/active plan region representation and handle arena
- lane-ready plan constituents and equivalence decision
- durable-state and Query rebind succession

**Deliverables**

- Carry affected-closure proof into plan lowering so unchanged regions are
  structurally reused without rescanning or rebuilding them.
- Lower changed regions into a complete successor view; never publish a delta as
  if it were complete truth.
- Preserve handle slots and lane storage for exact unchanged meaning, retire
  removed regions, and mint fresh authority for changed/replaced meaning.
- Carry Query-owned live-resource succession/disposal receipts through the
  replacement transaction. Reused bindings retain their lawful Query-owned
  resource; replaced or removed bindings are released exactly once only after
  successor publication can no longer fail.
- Define insertion, deletion, move/reparent, lane transition, Query rebind,
  resource change, and support-policy change behavior.
- Keep retained memory `O(P)` across sustained churn; obsolete candidate and
  predecessor-only storage is reclaimed at an explicit lifecycle boundary.

**Warnings**

- Copying all vectors and calling the result incremental is not acceptable.
- Index stability may not override semantic correctness after reparenting or
  family changes.
- Overlapping affected regions must be canonicalized once, not processed twice.

**Test requirements**

- Change one leaf in plans of increasing unrelated size and prove plan-lowering,
  allocation, comparison, and lane-rebuild counters remain bounded by the same
  affected closure.
- Equivalent deltas expressed through different lawful invalidation order
  produce the same successor plan and reuse receipt.
- Adversarial insert/delete/reparent/family/lane/Query/resource changes preserve
  only exact unaffected handles and reject every stale affected handle.
- Run a long mixed replacement storm and prove retained plan memory, live arena
  slots, and region counts return to the active-plan envelope rather than
  growing with history.
- Rebind, remove, deny, and retry Query-backed regions under a replacement storm
  and prove no double activation, double disposal, leaked live resource, or
  UI-local recovery path.
- Reprove the Query ownership/lifetime-matrix rebind/removal rows first exercised
  in Phase 8 under bounded regional replacement and sustained churn; the added
  claim here is locality and lifecycle stability, not first-path existence.

**Engineering decisions**

- Existing graph/allocation locality proof defines the maximum legal replacement
  scope; the plan lowerer may narrow further but may not widen silently.
- Structural sharing is internal derived storage and does not share mutable
  authority between candidate and active plans.
- Stable reuse requires exact carried predecessor proof, not matching names or
  hashes.
- Phase 14 consumes the region storage, slot-retirement, comparison, and
  reclamation model frozen in Phase 4. It does not select a second lane-specific
  replacement representation.

**Open questions**

- None about storage or reuse authority. Phase-local tuning may change layout
  only when the Phase 4 representation contracts and evidence remain intact.

## Phase 15: Publish Application and Plan Authority Atomically

**Vertical outcome:** A successful replacement publishes the successor
application generation, complete plan bundle, allocation/invalidation truth,
state, scheduler posture, Query/inspection authority, and host binding as one
frame-boundary transition.

**Relevant subsystems and APIs**

- `WorthUiActiveApplicationSession` cutover
- committed-allocation activation preflight, transaction, publication, and
  commit resources
- active runtime state, last-valid observation, frame scheduler, transient
  admission, durable resize, Query and planning inspection registries
- plan-swap and application-cutover receipts

**Deliverables**

- Extend and close the atomic activation transaction shell established in Phase
  4 and required for every active slice since Phase 5 so the final cross-lane
  plan bundle, application generation, and remaining authority families are
  prepared and committed together.
- Delete any residual legacy path that binds application generation after
  runtime publication. Such a path is migration residue, not an interim lane
  that earlier phase tests may use.
- Validate every predecessor and candidate authority before acquiring the final
  commit boundary.
- Make commit infallible after the last validation or provide rollback that is
  itself mechanically complete.
- Return one receipt/envelope whose generation, plan, allocation, state,
  scheduler, Query, host, and inspection facts all describe the same successor.
- Fold the Query binding succession first shipped in Phase 8 into the final
  complete-authority transaction, re-prove its simultaneous observation with
  every remaining authority family, and schedule any predecessor Query-resource
  release from the successful commit receipt, never from candidate preparation
  or failed publication.
- On every failed-publication path, release candidate-only Query resources and
  references exactly once while leaving predecessor-owned Query authority and
  live resources untouched; this is the runtime proof of the matrix's failed-
  publication row.
- Keep last-valid predecessor observation available for typed denial reporting
  without retaining competing active authority.

**Warnings**

- Sequential field assignment inside one method is not automatically atomic
  authority publication.
- Do not let inspection observe the successor before execution can, or vice
  versa.
- A frame-in-progress denial must not consume a candidate irreversibly unless
  the API explicitly returns a retryable pending handle.

**Test requirements**

- Inject failure at every fallible precommit stage and prove all active
  observations, handles, ledgers, state, Query bindings, and inspection remain
  the predecessor.
- For each such failure stage, independently observe candidate Query-resource
  cleanup and predecessor Query-resource survival; matching receipts alone do
  not close the lifetime proof.
- At successful cutover, simultaneously inspect application generation, active
  plan, allocation, Query, host, and state facts and prove exact successor
  parity.
- Cross two identical-looking sessions/candidates with equal digests and counts;
  exact session/graph/generation authority must deny before mutation.
- Race a frame-boundary posture change against prepared activation in the
  deterministic scheduler harness and prove no torn or double commit.

**Engineering decisions**

- Publication owns the complete successor bundle and consumes its one-shot
  authority.
- Expected contention and stale-boundary conditions are typed denials, not
  unwinds.
- Plan swap and application cutover are views of one transaction, not nested
  independently authoritative commits.

**Open questions**

- If retry after a transient frame-boundary denial is supported, define a
  session-bound pending activation handle with explicit freshness and disposal.

## Phase 16: Bind Honest Reload and Steady-Frame Cost Surfaces

**Vertical outcome:** The actual launch/replacement/lower/compare/activate/frame
paths emit exact counters that certify their declared complexity and lower into
Foundational only at the measurement boundary.

**Relevant subsystems and APIs**

- reload counter boundary, builder, stop stages, receipt, and Foundational bridge
- steady-frame counter boundary, lane receipts, report planner, diagnostic
  policy, and Foundational bridge
- lane/frame cost certification and scale-variation proof
- measurement counter packets and complexity contracts

**Deliverables**

- Reconcile existing provisional counters with the final canonical lifecycle;
  remove synthetic, duplicate, or never-authoritative rows.
- Define exact counter schemas for initial lowering, semantic no-op, derived
  provenance refresh, bounded replacement, denied replacement, minimal
  host-output envelope production, and each steady execution lane.
- Carry active plan/generation identity and affected-scope evidence in compact
  receipts so counter packets cannot be attached to foreign work.
- Carry `T_req` from the pre-execution target admission and record `T_exec`
  independently. Certification rejects `T_exec > T_req`; an executor may not
  define its own requested breadth after doing the work.
- Enforce forbidden steady-frame counters at the production boundary before
  Foundational lowering.
- Attribute allocations separately to active-plan execution/compact receipt
  production, host-adapter translation, and renderer/native mechanics. Only the
  named executor boundary carries the zero-general-purpose-allocation claim.
- Reuse the thread-scoped armed allocator observer introduced with the first
  real executor in Phase 5. Extend its coverage across the final lane set and
  reconcile its independent observation with the final counter schema; it
  remains inactive for unrelated tests and measures only the named active-plan
  execution interval.
- Lower certified Worth UI evidence into shared Foundational claim vocabulary
  without importing Foundational into plan meaning or control flow.

**Warnings**

- Always-zero counters that no production path can increment are theater.
- Counter increments inside diagnostics projection do not measure the operation
  they claim to describe.
- Foundational readiness cannot certify a malformed or foreign Worth UI receipt.

**Test requirements**

- Equivalent executions produce equivalent certified cost meaning even when
  diagnostic richness differs.
- Attach a valid-looking counter packet to a foreign plan/generation, omit a
  required row, duplicate a lane, add an unattributed bucket, and overflow a
  counter; every case must deny before Foundational projection.
- Scale `P`, `A`, and each lane's `T_req` independently and prove the measured
  slopes match the declared envelopes for launch, replacement, and frame work;
  assert `T_exec <= T_req` at every sampled width.
- Seed each forbidden frame operation through a lower-level counter-schema test
  rather than a test-only production executor branch; certification must fail.
- Execute the public active-plan frame path while the independent allocator is
  armed and reconcile its observed count with the certified executor allocation
  row. Then run host-adapter/egui work outside that interval and prove its
  allocations cannot be hidden inside the executor claim.
- Produce closing schema-versioned timing evidence under conditions comparable
  to the opening artifact. Malformed metadata, changed toolchain/cache posture,
  or missing raw samples must classify the run as incomparable rather than
  silently passing the 10 percent review.

**Engineering decisions**

- One production event owns each increment.
- Counter schemas distinguish ordinary frame work from reconstructive reload
  work and stop-stage denial.
- Allocation claims are boundary-specific; adapter or renderer work cannot be
  charged to or hidden behind the active-plan executor boundary.
- Foundational claim bundles are derived evidence, not active runtime state.

**Open questions**

- Retain only counters that answer an architectural or operational question;
  remove ornamental row families during schema reconciliation.

## Phase 17: Curate the Scalable Developer and Inspection Surface

**Vertical outcome:** The common API reads as application intent, expensive
lowering/replacement is visibly phased, and the evidence/inspection surfaces
shipped with Phases 2 through 15 are curated into one scalable public experience
without exposing a second execution authority.

**Relevant subsystems and APIs**

- Worth UI product `app` and `runtime` facades
- active application session and typed replacement outcomes
- plan inspection, diagnostics projection, AI harness, and Query inspection links
- runtime handoff exports and documentation/`AI_README.md`

**Deliverables**

- Curate the public surface around active-session operations and exhaustive typed
  outcomes; remove raw plan builders, internal digests, and assembly plumbing
  from ordinary consumers.
- Unify the already-shipped compact plan summary/cost/equivalence inspection
  before activation and typed active-plan observation after activation, without
  exposing owned executable plan data or creating a new evidence source.
- Link Query-backed plan rows to Query-owned inspection/evidence handles instead
  of copying Query explanations into UI diagnostics.
- Make expensive replacement/lowering calls visibly expensive and leave cheap
  active observations as ordinary borrows.
- Keep common Query-free/headless usage free of optional-system ceremony while
  retaining explicit advanced Query/host/lane controls at their responsible
  boundary.
- Update feature documentation, examples, discovery docs, and `AI_README.md` to
  teach only the final phase chain and its cost/authority model.

**DX target (illustrative shape, not frozen spelling)**

```rust
let mut session = app.launch()?;

let prepared = session.prepare_replacement(submission)?;
let candidate = session.lower_replacement(prepared)?;

candidate.summary();       // local, compact observation
candidate.cost_envelope(); // planned affected scope; no execution

match session.activate(candidate)? {
    ReplacementOutcome::NoOp(receipt) => observe(receipt),
    ReplacementOutcome::Activated(receipt) => observe(receipt),
}

session.execute_frame(|frame| {
    frame.component(component_handle).execute()
})?;
```

The ordinary caller never constructs a plan, chooses internal lane strategy,
passes a plan to an executor, or compares digests.

**Warnings**

- A friendly one-call replacement helper may compose the phases, but it cannot
  erase typed no-op/denial outcomes or hide expensive work.
- Explanation is a structured projection of receipts, not log scraping.
- Do not preserve old public names as aliases if they keep the predecessor
  authority model usable.

**Test requirements**

- Common query-free, Query-bound, headless, and egui applications follow the
  same phase progression and produce parity at shared boundaries.
- Product-facade source audits reject raw plan construction, digest-based
  comparison, executor plan injection, and internal lowerer imports without new
  compile fixtures.
- Candidate inspection cannot activate or execute a plan; active inspection
  cannot observe candidate-only facts before cutover.
- Plan inspection cannot promote a Query receipt, basis digest, native aspect
  value, or consumed-projection reference into installed or executable Query
  authority.
- Rich explanation/materialization changes neither equivalence nor steady-frame
  counters and runs only on explicit request.

**Engineering decisions**

- Organized truth and proof-carrying progression define the DX; shorthand is
  secondary.
- Public errors/outcomes preserve failure topology rather than collapsing to a
  boolean or string.
- Advanced APIs expose the next responsible boundary, not unsealed internals.

**Open questions**

- Decide which one convenience method, if any, composes prepare/lower/activate
  for simple applications after the explicit surface is proven and measured.

## Phase 18: Hostile Certification, Legacy Removal, and Closure

**Vertical outcome:** Only the canonical plan lifecycle remains, its claims
survive adversarial end-to-end scenarios, and the milestone closes without test
or compile-time debt.

**Relevant subsystems and APIs**

- every API in the current-disposition table
- source/topology audits, boundary checker, agent-context checker, line-cap gate,
  Clippy, test lanes, and timing evidence
- reload storm, lane parity, identity/state/Query drift, and cost certification
- documentation and public exports

**Deliverables**

- Delete predecessor execution-plan input/witness paths, digest-only active plan,
  caller-built/caller-passed lane execution, egui meaning in canonical topology,
  raw digest no-op, redundant counters, obsolete tests, and compatibility aliases.
- Delete touched 3.9 test-only executor variants and production-side `for_test`
  authority constructors identified in Phase 1; hostile proof must use real
  public/crate boundaries or narrowly owned below-authority test support.
- Add anti-bypass source/topology rules at the narrowest authoritative boundary.
- Run the final adversarial matrix against real public lifecycle operations and
  production authority, reusing compiled-once scenario support.
- Reconcile the real-boundary proof ledger: every filesystem, watcher, Query,
  egui, headless, allocator, and public-lifecycle claim names both its actual
  production entry point and an independent observation that would fail a fake.
- Record closing structural/timing evidence and reconcile every spec claim to a
  test, audit, counter receipt, or documented engineering decision.
- Leave strict Clippy, warnings, dead-code, Worth UI line caps, test topology,
  boundary-check, agent-context, and all Worth UI proof lanes green.

**Warnings**

- A deprecated alias is still an operational predecessor path.
- Do not add last-minute compile fixtures to prove closure; use the frozen proof
  topology.
- Passing isolated module tests is not end-to-end authority evidence.
- A long deterministic storm below an already-proven external boundary may
  certify plan churn, but it cannot certify filesystem, watcher, Query, or egui
  crossing. Those claims require the bounded real-mechanism sequence here.

**Test requirements**

- Run a mixed launch/frame/replacement storm containing semantic no-ops, bounded
  changes, denials at every phase, Query/no-Query transitions, host-support
  changes, lane transitions, stale handles, equal-digest foreign authority, and
  inspection requests; active truth must remain coherent after every step.
- The Query portion of that storm uses the Query Consumer Kit/public installed
  domain path and includes foreign installation generation, equal digest,
  native-aspect, live-resource succession, and exact-once disposal cases.
- The file-authored portion writes and atomically replaces real `.wui` files
  under the production watcher while the same public application session drives
  real egui frames. Valid, semantic-no-op, malformed, partial, removed-import,
  and restored-import edits must produce externally observable output and
  generation behavior consistent with the canonical receipts. A removed target
  whose `import` declaration remains is denied and cannot silently lower a
  smaller application generation.
- After that bounded real-mechanism sequence has proven ingress/host crossings,
  run the high-volume churn/scale portion through the same public session using
  production-frozen candidate inputs. This keeps `A/P/T_req` evidence
  deterministic without paying OS watcher settlement on every iteration or
  mislabeling the deterministic loop as watcher-throughput proof.
- Repeat the storm at multiple unrelated plan widths and prove replacement/frame
  work follows `A`/`T_req`, not `P`, with `T_exec <= T_req` throughout.
- Mechanical audits prove every removed API/path is absent and cannot be reached
  through another facade or alias.
- Mechanical audits prove the egui adapter consumes only the minimal sealed
  host-output envelope and cannot import active plan bundles, lane builders,
  authored declarations, or candidate authority.
- Re-run proof-parity reconciliation and demonstrate the frozen one-owner,
  two-invocation compiler topology, zero lost diagnostic classes, zero net new
  physical compile fixtures, no ordinary nested Cargo invocation, and no
  generated fixture workspace.
- Compare opening/closing medians for the targeted ordinary contract, fast,
  `application_contracts`, and cold/warm compile-contract lanes; run the full
  lane once as a closure smoke rather than turning it into a median benchmark.
  Real-boundary work may lengthen hostile execution,
  but it may not move OS waits/storms into fast iteration or exceed the reviewed
  10 percent comparable-median gate without architectural correction.

**Engineering decisions**

- Closure means one principled path, not one preferred path plus compatibility
  residue.
- Any remaining provisional API must be justified as part of the canonical
  lifecycle in the disposition table; otherwise remove it.
- Timing, counter, topology, and behavioral evidence are all required because no
  one proof form establishes the whole milestone.

**Open questions**

- None may remain about authority ownership, equivalence meaning, operational
  plan input, or test topology. A non-blocking representation choice may be
  recorded only if both choices already satisfy every binding invariant.

**Closure note (2026-07-20):** Closed. The canonical active-session lifecycle is
the only public operational path; every real-boundary ledger row and Query
lifetime row is proven; the frozen product, certification, and host compile-
contract owners remain complete and nonredundant; and the final timing artifact
records raw samples plus reviewed amendments for the intentional proof cost.

## Phase ordering rationale

The sequence is intentional:

1. Phase 1 protects iteration speed and freezes the real-mechanism proof homes
   before either synthetic or end-to-end tests proliferate.
2. Phase 2 closes the already-claimed filesystem/watcher prerequisite with real
   `.wui` files and production OS/source adapters before plan work can hide
   behind injected packages.
3. Phase 3 establishes the one legal lowering boundary and freezes the Query
   ownership/lifetime handoff through a real Consumer Kit installation. It also
   breaks the existing allocation/lowering cycle into a narrow pre-allocation
   projection followed by the sole post-commit execution-plan authority.
4. Phase 4 solves the hard replacement problem first: regional storage,
   predecessor proof, collision-safe equivalence, reclamation, and the atomic
   publication shell exist before lane representations depend on them.
5. Phases 5 through 11 grow the actual plan bundle as working vertical execution
   slices: minimal ordinary plus the durable headless envelope and independent
   allocator proof, handles, complete ordinary, admitted data-reference
   substrate, spatial, realtime, then cross-lane host-output closure with real
   headless and egui mechanics.
6. Phase 12 closes equivalence field coverage after every execution-affecting
   constituent exists; it does not retrofit the regional proof architecture.
7. Phases 13 through 15 use that proof for semantic no-op/provenance refresh,
   bounded replacement, and complete atomic publication.
8. Phase 16 binds counters to independent allocator/scale evidence over the final
   lifecycle rather than provisional functions.
9. Phase 17 curates DX and inspection over evidence shipped continuously by the
   preceding phases.
10. Phase 18 removes all predecessor paths and certifies the whole system through
    real filesystem, Query, headless, egui, allocator, and public-session paths.

This avoids horizontal half-completion: each executable lane works through the
same active-plan owner before optimization and public-surface closure depend on
it.

## Milestone acceptance evidence

Milestone 3.9 is complete only when all of the following are true:

- an external test writes actual `.wui` files, the production filesystem reader
  and operating-system watcher freeze them into ordered candidate composition,
  and malformed/partial/atomic-rename cases preserve or replace active truth as
  specified without injected file contents or manufactured watcher events;
- one sealed authority lowers exact candidate application, graph,
  capability/Query, host-support, and committed-allocation truth into one
  candidate plan bundle;
- allocation planning consumes a distinct sealed non-executable projection and
  cannot consume, cache, or reconstruct the execution-plan input; only committed
  allocation joined back to exact candidate authority can open plan lowering;
- one regional storage/equivalence kernel carries exact predecessor proof,
  produces complete successor truth with `O(A)` changed work, and prevents a
  fingerprint collision from authorizing reuse or no-op;
- the active application session owns or exclusively governs the complete
  active plan, and callers cannot construct or submit plans to executors;
- launch and every successful non-no-op replacement publish a real plan bundle
  atomically with application/runtime authority;
- identical executable meaning produces collision-safe equivalence and a typed
  no-op even across incidental candidate generations; non-operational source/
  provenance drift is separately typed and cannot publish executable authority;
- every execution-affecting change is classified non-equivalent, and stale or
  foreign authority cannot activate or execute even when digests/counts match;
- component, command, token, child-range, state-slot, view-binding, lane, and
  render-resource access uses sealed compact handles with direct indexed
  resolution and explicit exhaustion denial;
- ordinary, virtualized-data substrate, canvas/spatial, and realtime execution
  consume lane-ready active-plan constituents and satisfy their separate cost/
  failure contracts; 3.9 does not claim broad Query collection projection or
  live collection-patch product semantics;
- initial lowering is `O(P)`, replacement work is `O(A)` plus changed output,
  target execution is `O(T_req)` with `T_exec <= T_req`, and retained plan
  memory is `O(P)` under long replacement storms;
- steady frames prove zero parsing, artifact validation, string registry lookup,
  broad scan, rich diagnostic materialization, and general-purpose heap
  allocation inside the named active-plan execution/compact-envelope boundary;
  adapter and renderer allocations remain separately attributable;
- reload and steady-frame counter receipts are exact, foreign-proof, and lower
  into Foundational only after Worth UI certification;
- host-neutral plan meaning reaches egui only through one minimal sealed
  host-output envelope, with no egui-owned semantic decision or plan access in
  canonical runtime topology;
- at least one production `egui::Context::run` frame and one production headless
  frame consume that envelope through their real adapters; enum/profile/contact-
  only simulations do not count;
- Query-backed plan rows preserve one sealed installed/consumed authority edge,
  exact native aspect meaning, support-versus-admission posture, and Query-owned
  live-resource lifecycle without UI-local mirrors;
- the Query acceptance path uses an actual Query Consumer Kit/runtime
  installation and proves live-resource succession/disposal rather than relying
  on certification-constructed handles;
- plan and AI inspection link to Query-owned evidence without promoting
  receipts, digests, or observed native values into authority;
- every phase-added runtime artifact has compact indexed evidence and a budgeted
  summary/evidence-reference inspection path before the phase closes;
- test topology retains proof parity through the frozen one-owner,
  two-invocation compiler topology with no lost diagnostic class, no new
  physical fixture, no ordinary nested Cargo invocation, and no generated
  fixture workspace; closing timing evidence satisfies the iteration budget;
- real filesystem/watcher, public lifecycle, Query, egui/headless, allocator,
  and long-storm tests live as responsibility-named child modules of the existing
  `application_contracts` target in hostile certification; the fast lane remains
  free of OS waits, real-host frames, allocator probes, and long storms;
- documentation and `AI_README.md` teach only the canonical lifecycle; and
- strict Worth UI tests, Clippy, warnings/dead-code, line caps, boundary-check,
  agent-context, topology budgets, and hostile certification are green.

## Non-goals

- Milestone 3.9 does not make the runtime an author of `.wui` files. Real tests
  mutate authored files as an external editor; production Worth UI owns reading,
  watching, snapshotting, lowering, and typed denial only.
- Milestone 3.9 does not define the complete mounted-node or mounted-frame
  receipt contract. It defines only the minimal sealed host-output envelope that
  Milestone 3.10 must extend/refine into those receipts without reopening plan
  ownership.
- It does not add general Query collection projection, cursor, result-state, or
  live collection-patch semantics to Worth UI; those remain Milestones 3.13 and
  6 unless the roadmap is visibly amended.
- It does not implement the full host observation/rebind planner scheduled after
  mounted receipts, though its active plan must be ready to consume those later
  transitions.
- It does not add an open-ended third-party lane plugin system.
- It does not persist or replay executable plans as a new source of truth.
- It does not optimize compiler performance outside the Worth UI test program.
- It does not preserve predecessor APIs for source compatibility when they keep
  a competing authority path alive.
