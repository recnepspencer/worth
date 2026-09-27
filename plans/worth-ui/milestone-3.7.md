# Milestone 3.7 Engineering Spec: Runtime Topology And Proof-Flow Cleanup Gate

> **Status:** Closed
>
> **Roadmap parent:** [worth_ui_roadmap.md](/Users/Esther/Documents/Programming/WORTH_workspace/worth-ui/plans/worth-ui/worth_ui_roadmap.md)
>
> **Primary prerequisite:** `Milestone 3.6b Allocation Neighborhood Planning And Constraint Propagation`
>
> **Follow-on sequence:** `Milestone 3.8 allocation receipts, incremental replanning, scroll, portal, and continuous interaction measurement`
>
> **Primary architectural driver:** make the shipped 3.1 through 3.6b runtime surfaces structurally auditable before allocation receipts, continuous interaction churn, and execution-plan lowering stack more semantics on top of broad facades, dumping-ground trees, and overloaded proof-flow files.
>
> **Closeout:** see [Structural Closeout And 3.8 Readiness](#structural-closeout-and-38-readiness) below.

## Goal

Milestone 3.7 is a cleanup gate, not a product-capability expansion milestone.

It closes when Worth UI's already-shipped runtime lanes read as explicit
authority transitions instead of bags of nouns, broad facades, helper swamps,
and directory topology that requires grep archaeology to understand.

The milestone exists to produce one honest result:

- future 3.8 work consumes cleaned runtime boundaries
- future 3.8 work does not reopen broad facade shape, helper placement,
  runtime/host mixing, or evidence-topology ambiguity just to land receipts,
  churn invalidation, and continuous interaction semantics

## Why This Milestone Exists

Milestones 3.1 through 3.6b built real runtime capability, but the buildout
left visible structural residue:

- large export hubs in runtime and evidence roots
- root directories with too many same-level files to teach responsibility
  quickly
- broad lifecycle files that make valid order and next capability implicit
  instead of explicit
- inspection, certification, and test-support lanes that are directionally
  correct but still at risk of helper creep or production-authority leakage

If 3.8 adds allocation receipts, incremental replanning, scroll/portal churn,
and continuous drag/resize pressure on top of that residue, the runtime will
still work but the codebase will become materially harder to reason about,
harder to audit structurally, and easier for later automation to patch in the
wrong place.

3.7 exists to stop that compounding now.

## Governing Summaries

- `MENTALITY.md`: the most important thing to protect is hard-problem-first
  design. The spec must treat structural ambiguity itself as the adversarial
  problem rather than letting later runtime milestones inherit it.
- `arch_laws.md`: the most important thing to protect is proof-bearing
  progression. Public/runtime boundaries must consume prior proof and expose the
  next valid capability instead of forcing callers to rediscover order from
  helpers, predicates, or raw fields.
- `composition_laws.md`: the most important thing to protect is visible logic
  hierarchy. Files and functions must read as named semantic steps instead of
  one broad body mixing collection, classification, verification, mutation,
  receipt construction, diagnostics, and result assembly.
- `domain_structure_laws.md`: the most important thing to protect is the tree
  as responsibility architecture. Directories, modules, and public surfaces
  must encode real lifecycle, authority, and replacement boundaries rather than
  flat storage.
- `perf_laws.md`: the most important thing to protect is semantic-breadth
  honesty. Cleanup must not hide broad scans, broad invalidation, or repeated
  rediscovery behind nicer names or reorganized files.
- `worth_ui_roadmap.md`: the most important thing to protect is milestone
  sequencing. This cleanup gate belongs after 3.6b because the planning kernel
  now exists, and before 3.8 because receipts and churn would otherwise stack
  on top of structural residue.
- `WORTH_UI_README.md`: the most important thing to protect is runtime-owned UI
  meaning. Host code may allocate pixels and report observations, but it may
  not recreate UI meaning or become the de facto owner through structure.
- `worth-ui-dsl-vision.md`: the most important thing to protect is semantic-lane
  separability. Cleanup must keep structure, identity, layout, appearance,
  content, bindings, intents, and services from collapsing back into local
  blobs.
- `ai-diagnostics.md`: the most important thing to protect is one runtime-owned
  evidence substrate. Inspection and diagnostics must consume runtime truth
  rather than becoming a second graph, a host debug lane, or presentation-only
  folklore.
- `worth-query/docs/AI_README.md`: the most important thing to protect is the
  Query runtime boundary. Worth UI cleanup may consume Query-owned public
  artifacts and bindings, but it must not restate or rebuild Query runtime
  truth locally.

## Adversarial Constraint

No Worth UI runtime flow may claim completion if a reviewer still has to
reconstruct the transition grammar from:

- broad facades
- flat dumping-ground directories
- `mod.rs` business logic
- copied receipt/evidence fields
- generic helper modules
- test/certification support that can mint production-meaning shortcuts
- host-local or renderer-local structure that effectively redefines runtime
  semantics

If the next correct edit is still harder to locate than the next convenient
edit, the milestone has failed.

## Product Decision Lock

3.7 must not add new Worth UI user-facing behavior.

Any new type, module, export, folder, helper, compile-fail surface, structural
scan, or certification lane created here must exist to make an existing runtime
lane more auditable, more bounded, more honest, or harder to misuse.

## Cleanup Evidence Standard

This milestone closes on structural evidence, not feature count.

Valid evidence includes:

- explicit cleanup maps
- final directory skeletons
- public-surface diffs
- removed exports and narrowed facades
- split files or functions with clearer logic hierarchy
- named transition families and decision tables
- compile-fail coverage for sealed construction boundaries
- focused runtime tests where cleanup changes behavior or boundary power
- structural scan checks for file-count, directory-shape, or facade discipline
- focused verification commands

Pure topology cleanup may close on structural diffs and review evidence when no
runtime behavior changed. Any cleanup that changes construction authority,
public capability, or runtime behavior must carry focused executable evidence.

## Phase Plan

### Phase 1: Structural Inventory And Concept Freeze

Freeze new 3.x runtime concept expansion and produce the cleanup map that the
rest of the milestone must consume.

**Relevant subsystems**
- `worth-ui-runtime`
- `worth-ui-inspection`
- `worth-ui-query-binding`
- `worth-ui-certification`

**Relevant APIs**
- `workspaces/worth-ui/crates/worth-ui-runtime/src/lib.rs`
- `workspaces/worth-ui/crates/worth-ui-runtime/src/facade/mod.rs`
- `workspaces/worth-ui/crates/worth-ui-runtime/src/runtime/mod.rs`
- `workspaces/worth-ui/crates/worth-ui-runtime/src/evidence/mod.rs`
- `workspaces/worth-ui/crates/worth-ui-certification/src/topology/mod.rs`

**Warnings**
- Do not count nouns as architecture. A type or file named `Receipt`,
  `Evidence`, `Authority`, `Contract`, or `Plan` is suspect until its
  transition source, denial cases, and next valid capability are visible.
- Do not widen milestone scope into 3.8 behavior under the cover of cleanup.
  This phase maps and freezes; it does not broaden runtime semantics.

**Test requirements**
- Adversarial parity test: rerunning the same structural scan over the same
  tree must produce the same cleanup map categories and blocker set.
- Adversarial rejection test: a candidate finding that is merely ugly but not
  structurally load-bearing must be rejected from the cleanup-critical set so
  the milestone does not turn into cosmetic churn.

**Engineering decisions**
- The cleanup map must classify findings by failure mode: facade leakage,
  topology sinkhole, helper swamp, authority mixing, function overload, file
  size, or test bypass.
- Findings must name the evidence needed for closeout: structural diff,
  compile-fail, focused runtime test, or certification/harness proof.

**Open questions**
- None.

### Phase 2: Public Facade And Export Grammar Cleanup

Clean the public runtime surfaces so callers can see lifecycle and authority
order instead of alphabet soup.

**Relevant subsystems**
- `worth-ui-runtime` facade
- `worth-ui-inspection` public surface
- `worth-ui-query-binding` public surface

**Relevant APIs**
- `workspaces/worth-ui/crates/worth-ui-runtime/src/lib.rs`
- `workspaces/worth-ui/crates/worth-ui-runtime/src/facade/*`
- `workspaces/worth-ui/crates/worth-ui-inspection/src/lib.rs`
- `workspaces/worth-ui/crates/worth-ui-query-binding/src/lib.rs`

**Warnings**
- A public surface that mirrors internal topology freezes accidental structure
  as API.
- A facade may aggregate and route; it must not quietly implement runtime law,
  classification, measurement semantics, or diagnostics assembly.

**Test requirements**
- Adversarial parity test: ordinary callers can follow one stable lifecycle path
  through the public facade without deep-importing internal modules.
- Adversarial rejection test: compile-fail or visibility proof blocks external
  construction or deep import of surfaces that should remain internal after the
  cleanup.

**Engineering decisions**
- Public exports should be grouped by lifecycle capability and authority class,
  not by internal module count or accidental file adjacency.
- Compatibility re-exports are allowed only when they preserve lifecycle order
  and do not expose new construction power.

**Open questions**
- None.

### Phase 3: Runtime Lifecycle Tree Cleanup

Reshape runtime directories so planning, activation, topology, reconciliation,
handle allocation, and host observation are visible lanes rather than same-level
 file swamps.

**Relevant subsystems**
- `worth-ui-runtime/src/runtime`
- `worth-ui-runtime/src/host`

**Relevant APIs**
- `workspaces/worth-ui/crates/worth-ui-runtime/src/runtime/*`
- `workspaces/worth-ui/crates/worth-ui-runtime/src/host/*`
- runtime entry surfaces that lower from planning into activation and mounted
  execution preparation

**Warnings**
- A root containing dozens of same-level files is storage, not architecture.
- Reorganizing filenames without splitting the underlying lifecycle bags is a
  false cleanup. The implementation beneath the folders must also become easier
  to audit.

**Test requirements**
- Adversarial parity test: equivalent runtime lifecycle inputs still lower to
  the same planning/activation outcome after topology cleanup.
- Adversarial rejection test: no host-local or runtime-helper shortcut may be
  able to bypass the intended lifecycle order after the refactor.

**Engineering decisions**
- The top-level runtime tree should teach lifecycle and authority axes such as:
  planning, plan identity, activation staging, atomic swap, reconciliation,
  handle allocation, and host observation intake.
- Broad lifecycle owners such as runtime host orchestration should split into
  orchestration plus named semantic steps or child modules.

**Open questions**
- None.

### Phase 4: Evidence And Planning Proof-Flow Topology Cleanup

Turn evidence/planning residue into named transition families rather than one
big vocabulary warehouse.

**Relevant subsystems**
- `worth-ui-runtime/src/evidence`
- planning and graph-adjacent evidence families introduced by 3.5, 3.6a, and
  3.6b

**Relevant APIs**
- `workspaces/worth-ui/crates/worth-ui-runtime/src/evidence/*`
- planning and measurement receipts consumed by inspection and certification

**Warnings**
- Evidence folders must not become the new place where every residual semantic
  fragment goes when the real owner is unclear.
- Copied evidence fields are not proof flow. They are hidden reconstruction.

**Test requirements**
- Adversarial parity test: equivalent planning inputs still converge to the
  same retained evidence identity/order after evidence-topology cleanup.
- Adversarial rejection test: no later phase may synthesize a stronger witness
  from copied proof fields once the cleaned evidence families are in place.

**Engineering decisions**
- Group evidence by semantic transition family, not by residue history or
  broad noun class.
- Make collect/classify/verify/build responsibilities visible where broad files
  currently collapse them.

**Open questions**
- None.

### Phase 5: Graph, Planning, And Host Seam Cleanup

Clean the seams between graph truth, planning truth, and host observation so
host or helper code cannot effectively recreate layout meaning.

**Relevant subsystems**
- `worth-ui-runtime/src/graph`
- `worth-ui-runtime/src/runtime/plan_topology`
- `worth-ui-runtime/src/host`
- `worth-ui-host-contract`

**Relevant APIs**
- graph identity and neighborhood/planning handoff surfaces
- host observation intake surfaces
- planning identity handoff into runtime topology and activation

**Warnings**
- Host mechanics are evidence, not semantic authority.
- Planning identity and graph identity may be related, but they are not
  interchangeable. Cleanup must not collapse them for convenience.

**Test requirements**
- Adversarial parity test: equivalent declaration + graph + measurement inputs
  still lower to the same planning identity and runtime handoff after seam
  cleanup.
- Adversarial rejection test: a host-shaped or renderer-shaped shortcut that
  tries to reintroduce layout meaning locally must fail through visibility,
  construction, or certification fences.

**Engineering decisions**
- Keep graph truth, planning truth, and host observation as distinct lanes with
  explicit handoffs.
- Any moved helpers must land in the narrowest owner that can explain their
  role without broadening the host/runtime boundary.

**Open questions**
- None.

### Phase 6: Inspection, Certification, And Test Authority Cleanup

Make inspection, certification, and test support prove runtime law without
becoming runtime law.

**Relevant subsystems**
- `worth-ui-inspection`
- `worth-ui-certification`
- runtime-local test support and certification helpers

**Relevant APIs**
- `workspaces/worth-ui/crates/worth-ui-inspection/src/*`
- `workspaces/worth-ui/crates/worth-ui-certification/src/*`
- any runtime test-support modules exercised by 3.1 through 3.6b certification

**Warnings**
- Inspection is an evidence consumer, not a second truth graph.
- Certification helpers that mint impossible production authority are not
  testing aids; they are false lanes.

**Test requirements**
- Adversarial parity test: real runtime scenarios still inspect and certify
  through the same retained evidence after helper and topology cleanup.
- Adversarial rejection test: synthetic or test-only authority must be visibly
  labeled and fenced so it cannot satisfy production runtime contracts by shape
  alone.

**Engineering decisions**
- Production contract vocabulary lives in production-owning crates; inspection
  and certification consume it.
- Test support topology should mirror real responsibility boundaries only when
  that helps falsify production surfaces instead of bypassing them.

**Open questions**
- None.

### Phase 7: Function Decomposition And Helper Placement Cleanup

Split the remaining god functions and helper buckets that still hide semantic
steps after directory cleanup.

**Relevant subsystems**
- any touched runtime, inspection, or certification files still overloading
  collection, classification, verification, mutation, receipt construction,
  diagnostics, and counter publication

**Relevant APIs**
- remaining oversized or broad functions in the phase-scoped cleanup map
- any helper modules that currently serve multiple unrelated reasons

**Warnings**
- Smaller files are not enough if the same broad function body survives inside
  them.
- Moving logic into `helpers` or `common` is semantic exile, not
  decomposition.

**Test requirements**
- Adversarial parity test: decomposed functions preserve equivalent behavior and
  equivalent proof outputs for the same admitted inputs.
- Adversarial rejection test: the decomposition must make at least one
  previously implicit classification or transition case mechanically visible and
  testable rather than leaving it inline.

**Engineering decisions**
- Functions should read as orchestration plus named semantic steps.
- Helpers must live next to the responsibility they serve unless multiple
  callers truly share the same authority and lifecycle.

**Open questions**
- None.

### Phase 8: Structural Closeout And 3.8 Readiness

Prove the next milestone can consume cleaned runtime capabilities rather than
raw internals, facade residue, helper-only authority, or host folklore.

**Relevant subsystems**
- all crates touched by 3.7
- `plans/worth-ui/worth_ui_roadmap.md`
- 3.8 handoff surfaces for allocation receipts, replanning, and continuous
  interaction measurement

**Relevant APIs**
- cleaned `worth-ui-runtime` public surfaces
- cleaned inspection/certification/test-support entry points
- planning and runtime handoff surfaces that 3.8 will extend

**Warnings**
- Passing focused tests is insufficient if the public surface still teaches the
  wrong architecture.
- Do not close with broad debt language. Any remaining structural exception
  needs an explicit reason, scope, owner, and follow-on milestone.

**Test requirements**
- Adversarial parity test: a focused 3.6b-derived planning path still closes
  through the cleaned surfaces without reintroducing deep-import pressure or
  helper-only authority.
- Adversarial rejection test: attempts to start 3.8-style work from raw
  internals, copied receipts, or host-local semantics must be visibly blocked
  by the cleaned boundaries.

**Engineering decisions**
- The closeout artifact should name the final directory skeleton for each
  critical cleaned area and the public caller surfaces that remain.
- 3.8 readiness must be expressed as typed cleaned capabilities and boundary
  proofs, not as narrative confidence.

**Open questions**
- None.

## Must Ship

- roadmap-recognized 3.7 cleanup gate and detailed spec
- generic code-cleanup runner pipeline imported into this workspace as a
  distinct pipeline family
- narrowed public facades and export grammar where current topology exposes too
  much or teaches the wrong order
- cleaned runtime and evidence directory topology for the critical 3.1 through
  3.6b lanes
- decomposed proof-flow functions where broad semantic bags still existed
- explicit production/test/certification/inspection authority boundaries

## Must Preserve

- shipped 3.1 through 3.6b runtime behavior
- Query-owned truth and Query runtime boundaries
- host adapters as native-mechanics translators only
- runtime-owned explanation and diagnostics posture
- existing planning and measurement semantics where cleanup is structural only

## Acceptance Evidence

- `python automation/phase_runner/runner.py validate` on the 3.7 cleanup config
- focused Python runner tests covering the distinct cleanup pipeline behavior
- focused Rust checks/tests for touched crates when cleanup crosses production
  behavior boundaries
- structural diffs showing the final topology and public surface shape
- a closeout bundle proving 3.8 can extend cleaned boundaries rather than
  reopening residue

## Sequencing Notes

- 3.7 belongs immediately after 3.6b because the planning kernel now exists and
  can be cleaned honestly instead of speculating ahead of real runtime code.
- 3.7 belongs before 3.8 because allocation receipts, incremental replanning,
  and continuous interaction churn would otherwise stack on top of broad,
  hard-to-audit topology.
- 3.7 does not replace later runtime work. It makes later runtime work
  structurally readable, auditable, and harder to patch dishonestly.

## Structural Closeout And 3.8 Readiness

This section is the closeout bundle for Milestone 3.7. It freezes the cleaned
topology and public consumption surfaces that Milestone 3.8 must extend. It does
not add product behavior.

### Final directory skeleton (critical cleaned areas)

Paths relative to `workspaces/worth-ui/crates/worth-ui-runtime/src` unless noted.

```
lib.rs
  facade/     # entry â†’ lifecycle â†’ registry â†’ runtime_handoff â†’ evidence â†’ host â†’ inspection
  runtime/    # launch â†’ replacement â†’ planning â†’ activation â†’ execution â†’ host_observation
    matching/worth_ui_identity_match_graph_builder/
              # guard â†’ index â†’ classify kind â†’ match graph â†’ report
    launch/   # host_test_support is cfg(test) only
  evidence/   # construction â†’ measurement â†’ planning â†’ obligation
    measurement/projection/inspection_receipt/
              # classify failure â†’ project maps â†’ assemble view
  graph/allocation_neighborhood/constraint_pipeline/
              # collect authority â†’ classify specials â†’ admit edges â†’ verify â†’ construct set
  host/       # observation mechanics; no always-on for_test reexports at root
  certification_support/   # SUPPORT AUTHORITY; public only with feature certification-support

worth-ui-inspection/src/   # consumer vocabulary by lifecycle lane
worth-ui-test-support/     # sole public cross-crate fixture home
worth-ui-certification/    # topology audits, trybuild, certify_* consumers
```

### Public facade shape (3.8 caller surface)

Facade lifecycle order (see `facade/mod.rs`):

`entry â†’ lifecycle â†’ registry â†’ runtime_handoff â†’ boundaries â†’ evidence â†’ host_observation â†’ inspection_bridge`

Facades **route**. Domain modules **own law**. Product code must not deep-import
internal runtime topology to invent a parallel lifecycle.

### Proof-flow grammar frozen for 3.8 extension

```
declaration freeze / source ingress
  â†’ graph admit + correspondence
  â†’ replacement (admit â†’ compare â†’ impact â†’ narrow â†’ identity match â†’ node plan
                 â†’ reconcile â†’ query rebind)
  â†’ stage pending activation
  â†’ measurement basis admit + neighborhood admit + constraint pipeline
  â†’ plan_allocation
  â†’ [3.8] allocation receipts / invalidation / replan
  â†’ handle allocation â†’ lane plan â†’ execute â†’ host observe
  â†’ inspection / diagnostics consume (do not mint)
```

### Authority map

| Authority | Owns | 3.8 may |
|-----------|------|---------|
| Runtime production | Graph, planning, measurement basis, identity match, constraints, execution | Extend with receipts/replan consuming prior proofs |
| Host | Native observations, capability reports, pixels | Feed measurement inputs only |
| Query | Query truth / public bindings | Consume Query public artifacts only |
| Inspection | Projection vocabulary / support posture | Project admitted truth |
| Test-support / certification fixtures | SUPPORT AUTHORITY | Falsify production; never become production law |

### 3.8 start-here (typed cleaned capabilities)

| 3.8 concern | Consume from | Must not start from |
|-------------|--------------|---------------------|
| Allocation receipt | Post-`plan_allocation` planning artifact + admitted neighborhood identity | Synthetic neighborhood without admit path |
| Incremental replan | Impact narrow + identity match + neighborhood | Host-local invalidation of UI meaning; whole-tree default |
| Continuous measurement | Host observation â†’ admit measurement basis â†’ inspect | Host inventing semantic truth |
| Certification | `worth-ui-test-support` + facade `certify_*` | `certification_support` deep paths / production test mint |

### Anti-bypass fences retained

- Feature `certification-support`: production `certification_support` is not
  public without opt-in; preferred public home is `worth-ui-test-support`.
- `host_test_support` and measurement fact test helpers are `cfg(test)`.
- Inventory audit critical set empty (O-03/O-04/S-01 and prior phases cleared).
- Trybuild / topology audits block helper-bypass construction and facade leaks.

### Owned structural exceptions

| Exception | Scope | Owner | Follow-on |
|-----------|-------|-------|-----------|
| Some production modules remain ~400â€“423 lines as cohesive single-domain APIs or type catalogs (not phase-7 hotspots) | Individual files outside identity-match builder, inspection_receipt, and constraint_pipeline hotspots | worth-ui runtime maintainers | Split only when a god-function or multi-responsibility bag appears |
| Full `cargo test -p worth-ui-runtime --lib` still has pre-existing test-module wiring gaps (`replacement_impact_test_support` / private `tests` reexports in some test files) | Test build only | worth-ui runtime tests | Test-hygiene task; not required for 3.8 product start |

### Verification evidence (closeout run)

Commands and outcomes recorded for this closeout:

```text
python automation/phase_runner/runner.py validate \
  automation/phase_runner/config/worth-ui-milestone-3.7-code-cleanup.json
â†’ config is valid

cargo test -p worth-ui-certification --test milestone_37_structural_inventory_audit
â†’ ok (7 passed)

cargo test -p worth-ui-certification --test graph_touch_runtime_origin_receipts
â†’ ok (1 passed)

cargo test -p worth-ui-certification --test measurement_basis_determinism_runtime
â†’ ok (3 passed)

cargo test -p worth-ui-certification --test measurement_authority_compile
â†’ ok (1 passed)
```

### 3.8 non-reopen rules

Milestone 3.8 must:

1. Consume the cleaned planning, measurement, identity-match, and facade lanes above.
2. Not reintroduce broad root facades, same-level dumping-ground trees, or
   certification/test-as-production-law.
3. Attach new allocation receipts and replan outcomes to verified transition
   results (not host-frame folklore or reconstructed booleans).
4. Treat residual file-size exceptions as owned hygiene, not permission to
   restore god-functions.

### Closeout judgment

Phases 1â€“7 delivered structural inventory, facade grammar, lifecycle trees,
evidence topology, seam cleanup, authority fences, and function decomposition.
Phase 8 freezes that shape as the 3.8 foundation. **Milestone 3.7 is closed.**
