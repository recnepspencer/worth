# WORTH Signal Condensation Map

## Purpose

This document records where raw public capability should be collapsed into
guided higher-level forms.

This is required because "condense" is not specific enough on its own.

For each workflow family, we need to choose the exact target shape.

---

## Workflow Families To Specify

The following families require an explicit condensation decision:

- runtime setup and configuration
- computation definition
- batch invalidation
- planning and execution orchestration
- diagnostics access
- branch / snapshot / restore
- merge / reconciliation

---

## Required Fields Per Family

For each family, record:

- current raw flow
- current pain
- target canonical public flow
- target abstraction shape
  - builder
  - session
  - request object
  - prepared operation
  - preset bundle
  - layered raw + guided split
- raw APIs retained
- raw APIs contained or hidden
- migration notes

---

## 1. Runtime Setup And Configuration

### Current raw flow

- create `SignalGraph`
- call `SignalRuntime::builder(graph)`
- optionally switch generic dimensions with `with_tiers`, `with_context`,
  `with_events`, `with_domains`, `with_impacts`
- independently set `runtime_policy`, `checkpoint_barrier`, `checkpoint_policy`,
  fallback comparator, tier policies
- build runtime
- continue mutating runtime with additional setup calls

### Current pain

- configuration is powerful but still somewhat bag-like
- some settings feel like they belong to one coherent runtime declaration but
  are split across builder generics, policy objects, and post-build methods
- users can configure the runtime correctly, but the shape is not yet memorable

### Target canonical public flow

- `SignalRuntime::builder(graph)` remains the root
- runtime setup should be expressed as a small number of coherent builder
  sections:
  - operational policy
  - optional typed capabilities
  - optional advanced sections

### Target abstraction shape

- guided builder with nested config sections

### Raw APIs retained

- typed builder dimension switches
- advanced policy structs
- post-build specialist controls where architecturally justified

### Raw APIs contained or hidden

- low-level plumbing that does not belong in the default setup story

### Migration notes

- prefer guided builder additions over more free-floating setters

---

## 2. Computation Definition

### Current raw flow

- define node structurally with `graph.node()...build()`
- separately wire dependencies
- separately choose conditions / comparator / partitioned output
- separately define runtime-level computation families or keyed nodes where
  needed
- separately provide evaluation closure at read/evaluate time

### Current pain

- semantic declaration is spread across graph authoring, runtime registration,
  and evaluation call sites
- the framework knows a lot, but the declaration boundary is not singular

### Target canonical public flow

- one coherent runtime-owned computation declaration path for production use
- graph-level node authoring remains as the explicit low-level path

### Target abstraction shape

- declaration builder around `Recipe`

### Raw APIs retained

- `graph.node()`
- explicit dependency APIs
- explicit plan/evaluate closures

### Raw APIs contained or hidden

- scattered coordination calls that should be derivable from the declaration

### Migration notes

- the guided computation declaration should become the recommended production
  path once it can express the majority of serious workflows
- the public noun should stay human and product-shaped, not config-packet
  flavored

---

## 3. Batch Invalidation

### Current raw flow

- `mark_dirty(graph, node, aspect)`
- `mark_dirty_with_regions(...)`
- `mark_dirty_batch(...)`
- transaction-level scalar dirty calls

### Current pain

- both scalar and batch paths exist, but the production story is not
  forcefully batch-first
- users can still think in scalar orchestration too easily

### Target canonical public flow

- transaction-owned batch invalidation as the production path
- graph-level scalar invalidation remains the low-level convenience path

### Target abstraction shape

- guided batch/session object under transaction

### Raw APIs retained

- `mark_dirty`
- `mark_dirty_with_regions`
- `mark_dirty_batch`

### Raw APIs contained or hidden

- none immediately, but batch-first guidance must dominate docs and examples

### Migration notes

- preserve scalar APIs for simple cases, but subordinate them in docs and
  examples

---

## 4. Planning And Execution Orchestration

### Current raw flow

- `build_evaluation_plan(...)`
- `execute_prepared_plan(...)`
- executor selection
- runtime-level variants of the same pattern

### Current pain

- planning and execution are explicit, which is good
- but the normal path still feels like users manually shuttle artifacts between
  stages

### Target canonical public flow

- guided request/session flow over plan + execute while preserving explicit
  planning capability

### Target abstraction shape

- prepared operation / execution session

### Raw APIs retained

- explicit plan building
- explicit prepared execution
- explicit executor selection

### Raw APIs contained or hidden

- stage/task-level detail types from the main path

### Migration notes

- explicit planning remains a first-class advanced feature
- the guided path should make the happy path memorable
- current guided shape:
  - `runtime.target(node).read(...)`
  - `runtime.target(node).run(...)`
  - `tx.target(node).read(...)`
  - `tx.target(node).run(...)`

---

## 5. Diagnostics Access

### Current raw flow

- many direct inspect/compare/render helpers
- many summary/diff/history types
- separate direct diagnostics module and facade diagnostics namespace

### Current pain

- powerful but flat
- export families are more discoverable than user jobs

### Target canonical public flow

- one diagnostics access point from graph/runtime
- job-oriented operations:
  - explain
  - compare
  - health
  - history

### Target abstraction shape

- diagnostics session / access object

### Raw APIs retained

- diff, summary, replay, lineage primitives

### Raw APIs contained or hidden

- flat helper discovery as the main user experience

### Migration notes

- the new diagnostics access point should become the canonical docs entry

---

## 6. Branch / Snapshot / Restore

### Current raw flow

- many snapshot, branch, restore, replay, and lineage types are publicly visible
- users compose history flows from several separate concepts

### Current pain

- legitimate power, but weak guided orchestration
- history machinery can overshadow the core runtime story

### Target canonical public flow

- runtime-owned history surface with explicit guided operations:
  - capture
  - restore
  - branch
  - replay

### Target abstraction shape

- runtime-owned history/session surface

### Raw APIs retained

- underlying snapshot and lineage types

### Raw APIs contained or hidden

- restore and lineage internals from the default path

### Migration notes

- keep specialist state-history types available, but reachable through a more
  coherent history surface

---

## 7. Merge / Reconciliation

### Current raw flow

- broad family of merge plans, witnesses, conflict records, adoption policies,
  and result types

### Current pain

- architecturally meaningful but too raw
- high chance of exposing specialist truth because the runtime internally thinks
  in those terms

### Target canonical public flow

- one guided merge orchestration path for specialists:
  - configure
  - plan
  - inspect conflicts
  - execute

### Target abstraction shape

- specialist merge builder plus planned merge operation

### Raw APIs retained

- conflict/result/plan primitives for specialist tooling and bridge work

### Raw APIs contained or hidden

- broad raw merge vocabulary from the main runtime namespace

### Migration notes

- the bar for keeping raw merge concepts prominent should be brutal
- keep only what external specialists are genuinely meant to think in directly

---

## 8. Policy Surface Unification

### Current raw flow

- multiple policy knobs across runtime, diagnostics, tiers, comparators,
  conditions, checkpoints, and merge

### Current pain

- policy control is real but at risk of fragmentation
- several concepts are structurally related without being presented as such

### Target canonical public flow

- users choose one named runtime policy
- then optionally refine bounded policy sections

### Target abstraction shape

- policy bundle plus sectioned advanced refinements

### Raw APIs retained

- low-level policy structs where they encode legitimate expert control

### Raw APIs contained or hidden

- scattered low-level knobs from the default story

### Migration notes

- every new policy knob must justify why it is not part of an existing policy
  bundle or section
