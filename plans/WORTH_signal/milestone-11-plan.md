# Milestone 11 Engineering Spec: Observation Policies And Extensible Delivery Strategies

> **Status:** Completed
>
> **Closeout:** [milestone-11-closeout.md](./milestone-11-closeout.md)
>
> **Roadmap parent:** [signal_architecture2.md](./signal_architecture2.md)
>
> **Vision parent:** [worth_signals2.md](./worth_signals2.md)
>
> **Test requirements:** [test-requirements.md](./test-requirements.md)
>
> **Prerequisite milestone:**
> - [milestone-10-plan.md](./milestone-10-plan.md)
>
> **Primary architectural driver:** close the missing runtime-local observation
> category without collapsing `worth-signal` into bridge publication, frontend
> ergonomics, or truth ownership
>
> **Related implementation surfaces:**
> - [runtime_state.rs](../../crates/worth-signal/src/logic/transaction/runtime/state/runtime_state/mod.rs)
> - [builder.rs](../../crates/worth-signal/src/logic/transaction/runtime/state/builder.rs)
> - [observer.rs](../../crates/worth-signal/src/logic/transaction/runtime/state/observer.rs)
> - [transaction_types.rs](../../crates/worth-signal/src/logic/transaction/runtime/transaction/transaction_types.rs)
> - [transaction_mutation.rs](../../crates/worth-signal/src/logic/transaction/runtime/transaction/transaction_mutation.rs)
> - [commit_path.rs](../../crates/worth-signal/src/logic/transaction/runtime/transaction/transaction_commit/commit_path.rs)
> - [rollback_path.rs](../../crates/worth-signal/src/logic/transaction/runtime/transaction/transaction_commit/rollback_path.rs)
> - [runtime.rs](../../crates/worth-signal/src/logic/events/runtime.rs)
> - [effect_mapping.rs](../../crates/worth-signal/src/data/effect_mapping.rs)
> - [runtime.rs](../../crates/worth-signal/src/easy/runtime.rs)
> - [compute.rs](../../crates/worth-signal/src/easy/compute.rs)
> - [facade.rs](../../crates/worth-signal/src/facade.rs)
> - [worth_signals2.md](./worth_signals2.md)
> - [MENTALITY.md](../../docs/coding-guidelines/MENTALITY.md)
> - [arch_laws.md](../../docs/coding-guidelines/arch_laws.md)
> - [perf_laws.md](../../docs/coding-guidelines/perf_laws.md)

## Summary

Milestone 11 makes observation in `worth-signal` explicit, transactional, and
extensible.

This milestone is not "add subscriptions."

It is:

- runtime-owned observation semantics for committed derived-state change
- explicit separation between dependency topology, event subscribers, and
  runtime-local observers
- policy-bearing classification of touched, recomputed, and meaningfully
  changed state
- extensible delivery and coalescing strategies resolved before notification
  dispatch
- rollback-safe and replay-honest observer delivery
- diagnostics-visible observation provenance
- a clean substrate for easy-mode watchers/effects and later wasm or React
  adapters without forcing frontend concepts into the foundational crate

The governing rule is:

`classify once, lower once, deliver once, never notify from rolled-back state`

If observation meaning is still discovered ad hoc by adapters after commit, the
milestone is incomplete.

## 1. Goal

Make runtime-local observation a first-class, domain-agnostic capability of
`worth-signal` so that the crate can expose honest watchers/effects and support
host adapters without stealing responsibility from `worth-relational` or
`worth-runtime-bridge`.

The implementation goal is not to introduce a UI-facing subscription model.
The goal is to freeze the core runtime contract for:

- what is observable
- what counts as delivery-worthy change
- when observers fire relative to transaction commit and rollback
- how multiple changes coalesce within one transaction
- how strategy variability is declared, frozen, lowered, and explained

## 2. Why This Milestone Exists

`worth-signal` already has several reaction systems:

- dependency propagation through the graph
- event subscribers through the event bus
- effect routing through `EffectMapping`
- diagnostics observation through graph and runtime observers

What it does not yet have is a first-class app-local observation surface over
committed derived-state change.

That missing category matters because the crate vision now explicitly promises
watchers/effects in the easy surface and first-class observation semantics in
the runtime vision. Without a real substrate:

- easy-mode watchers are forced to invent semantics on top of internal details
- wasm must guess what "change" means
- React or other host adapters have to reconstruct commit semantics themselves
- rollback-safe delivery becomes a convention instead of a contract
- diagnostics can explain recomputation, but not observer delivery

This is exactly the kind of missing middle layer that later code will fill with
adapter-local heuristics unless the core runtime owns it first.

## 3. Hard Part

The hard part is not registering callbacks.

The hard part is freezing one exact truth-preserving relationship among five
different things that naive designs constantly blur together:

- dependency subscribers inside the graph topology
- event subscribers on the event bus
- runtime-local observers of committed derived state
- host-level adapters built on top of those observers
- diagnostics and replay artifacts that explain why delivery happened

The design fails if:

- observers can fire from state that later rolls back
- "recomputed" and "meaningfully changed" are not distinct contracts
- adapters have to infer delivery semantics from touched-node bags or event-bus
  side effects
- observation strategy is resolved dynamically during callback dispatch instead
  of being lowered before delivery
- the runtime cannot explain why an observer fired
- extensibility widens commit-path breadth into whole-graph rescans

Milestone 11 therefore has to make observation rich enough for later hosts, but
structurally incapable of becoming truth ownership, bridge publication, or
frontend framework policy.

## 4. Explicit Assumptions

- `worth-relational` remains the sole owner of truth-state identity, mutation,
  history, diffs, and traversal semantics.
- `worth-runtime-bridge` remains the owner of cross-runtime coordination,
  patch-to-invalidation mapping, snapshot evaluation coordination, and stream or
  publication-oriented integration semantics.
- `worth-signal` already owns computed/effect semantics, transactional
  evaluation, invalidation, rollback, diagnostics, history, and replay for
  derived state.
- event subscribers remain a distinct integration surface; this milestone does
  not collapse event bus subscribers into value observers.
- dependency topology subscribers remain a distinct graph concern; this
  milestone does not reinterpret graph edges as app-facing observer
  registrations.
- frontend hooks, JS callback ergonomics, store adapters, and form/resource
  abstractions remain higher-layer work even if they later build directly on
  the new observation substrate.
- the runtime may add new artifacts, packets, and registries for observation,
  but it must not reopen truth authority boundaries or diagnostics-vs-truth
  separation.

## 5. Governing Summaries

- `MENTALITY.md`
  The most important thing it protects here is solving the hostile boundary
  first. This milestone therefore starts from rollback-safe committed delivery
  and change classification, not from "callbacks are easy to add."
- `arch_laws.md`
  The most important laws here are 17, 18, 21, 27, 33, 34, 35, and 41.
  Observation policy resolution must be structurally isolated from delivery,
  observation must be phase-typed and scoped, diagnostics must remain derived,
  lowerings must exist before execution, authoritative and derived state must
  remain distinct, managed resources must stay lifecycle-owned by the
  framework, producers must be decoupled from consumers by typed effect shape,
  and proof-bearing types must carry what has been established about a staged
  observation event.
- `perf_laws.md`
  The most important thing it protects is breadth honesty. Observation cannot
  hide whole-graph rescans, repeated rediscovery, or broad coordination behind
  a cheap-looking watch API. Delivery work must be bounded by semantic delta and
  explained by named counters.
- `worth_signals2.md`
  The most important thing it protects is the runtime boundary: `worth-signal`
  owns derived computation and observation semantics, not truth ownership or
  app-specific ergonomics.
- `milestone-10-plan.md`
  The most important thing it protects is the pattern for extensibility:
  registration, freeze, lowering, execution, and replay-visible identity must
  already be familiar and must be reused structurally instead of reinvented.
- `test-requirements.md`
  The most important thing it protects is hostile certification. Observation is
  not closed until commit/rollback behavior, change classification, coalescing,
  and boundedness are all machine-checked. In particular, Milestone 11 now
  depends directly on:
  - `1A. The adversarial observation and delivery equivalence test`
  - `8A. The observation and managed-resource long-session extension`
  - `9A. The future abstraction lifting rule`
  - `9B. The future abstraction workload grammar`
  - `10A. The substrate boundedness and lifting test`

## 6. Adversarial Constraint

Milestone 11 must survive the following hostile condition:

> A long-lived runtime with branch churn, rollback-heavy transactions,
> partial-region invalidation, recompute-without-change suppression, and
> diagnostics-tier variation must support multiple host-declared observation
> strategies while keeping delivery commit-bounded, deterministic, replay-honest,
> and breadth-bounded by the actual changed derived surface rather than by graph
> size or observer count.

Concretely, the design must remain correct when all of the following are true:

- many observers overlap on the same node sets
- one transaction touches many nodes but meaningfully changes few
- multiple writes to the same source happen before one commit
- a transaction evaluates successfully and then fails during commit promotion
- event subscribers emit additional side effects inside the same transaction
- branch restore or merge rewrites large derived surfaces
- diagnostics tier changes between runs
- registry construction and process restart happen before replay or
  certification comparison

If any supported path falls back to observer-side graph scanning, rollback-time
delivery, ambient callback semantics, or adapter-local change reclassification
under those conditions, the milestone has failed.

## 7. Product Decision Lock

- observation is a first-class core runtime capability, not a wasm-only or
  React-only convenience
- runtime-local observation is categorically distinct from:
  - dependency topology subscribers
  - event bus subscribers
  - bridge publication streams
- default delivery is commit-bounded; rolled-back state must not produce normal
  observer delivery
- observation semantics are policy-driven:
  - touched
  - recomputed
  - meaningful change
  are distinct contracts
- extensibility follows the existing strategy pattern:
  - declare once
  - freeze once
  - lower once
  - deliver once
- easy-mode watchers/effects must compile down to the same observation
  substrate; they may not become a second execution engine
- frontend ergonomics are out of scope for core, but the core APIs must be
  strong enough that higher layers do not need to invent their own change
  semantics

Normative consequence:

- any implementation that exposes subscriptions by directly piggybacking on the
  event bus is out of spec
- any implementation that fires observers before commit finalization is out of
  spec
- any implementation that cannot distinguish touched from meaningfully changed
  delivery is out of spec
- any implementation that lets adapters redefine observation truth is out of
  spec
- any implementation that leaves delivery unit, ordering, or branch/restore
  semantics implicit is out of spec

## 7.1 Delivery Contract Lock

The following delivery semantics are explicit product decisions for this
milestone and are not left to adapter interpretation:

- the unit of normal delivery is one observer callback per observer per
  committed transaction boundary
- multiple matching node changes within one transaction are coalesced into one
  observer-visible delivery packet for that observer
- delivery ordering is deterministic and runtime-owned
- normal delivery happens only after commit has passed the point where the
  transaction cannot still become an ordinary rollback
- rollback produces no normal observer delivery
- branch restore, snapshot restore, and merge-driven rewrites are observation
  eligible only through explicit transaction-classified delivery; they are not
  allowed to bypass the same policy path as ordinary committed change

Normative consequence:

- any implementation that delivers per-node callbacks directly from the commit
  path without per-observer coalescing is out of spec
- any implementation that special-cases restore or merge delivery outside the
  observation policy path is out of spec

## 7.2 Reentrancy And Lifecycle Lock

Observer delivery must not reopen mutation authority or allow callback-local
semantics to redefine runtime truth.

For this milestone:

- observers are read-phase consumers of committed derived-state change
- observer callbacks are not allowed to mutate the runtime directly
- nested transactions triggered from inside delivery are out of scope
- unsubscribe during delivery must be deterministic and must only affect future
  transaction boundaries, not the in-flight delivery packet
- subscribe during delivery must not retroactively join the in-flight delivery
  packet

Normative consequence:

- any implementation that permits observer callbacks to mutate the same runtime
  through ambient access during delivery is out of spec
- any implementation that makes in-flight delivery membership depend on callback
  timing is out of spec

## 8. Scope

### 8.1 In Scope

- runtime-owned observation registry
- observer handles and lifecycle ownership
- observation policies and delivery strategies
- transaction-staged observation packets
- commit-only delivery and rollback suppression
- node and node-set observation
- touched / recomputed / meaningful-change classification
- deterministic observer ordering
- diagnostics-visible observation provenance
- easy-surface watcher/effect support built on the new substrate
- counters and certification coverage for delivery breadth and behavior

### 8.2 Explicitly Out Of Scope

- React hooks
- JS or wasm callback ABI design
- bridge publication streams
- relational subscription semantics
- async resource or form abstractions
- frontend store batching and transition APIs
- broad family/partition observation beyond what the milestone needs to prove
  the substrate honestly

## 9. Current-State Assessment

The runtime is already structurally ready for this milestone in several ways:

- [`SignalRuntime`](../../crates/worth-signal/src/logic/transaction/runtime/state/runtime_state/mod.rs)
  already owns the right long-lived subsystems and is the natural owner of an
  observer registry
- [`SignalTransaction`](../../crates/worth-signal/src/logic/transaction/runtime/transaction/transaction_mutation.rs)
  already stages commit-relevant work and is the natural place to stage
  observation candidates
- [`commit_path.rs`](../../crates/worth-signal/src/logic/transaction/runtime/transaction/transaction_commit/commit_path.rs)
  already has the correct semantic boundary for post-commit delivery
- [`rollback_path.rs`](../../crates/worth-signal/src/logic/transaction/runtime/transaction/transaction_commit/rollback_path.rs)
  already gives the correct hard boundary for suppressing delivery
- the crate already has a strong extensibility pattern through frozen
  registries, policy resolution, and lowered execution artifacts
- the crate already distinguishes graph observers, runtime observers, event
  subscribers, and effect mapping structurally

However, the missing category is still real:

- no runtime-owned observer registry exists
- no public observation handle or policy surface exists
- transaction scratch does not yet stage observation artifacts
- commit does not yet finalize or dispatch observation events
- rollback does not yet know about observer suppression as a typed concern
- easy mode advertises effects/watchers in the vision, but the current easy
  surface does not expose a first-class watcher layer

This means the runtime has the raw ingredients for observation, but not an
honest observation architecture.

## 10. Architecture Rules For This Milestone

### 10.1 Observation Is A Runtime Subsystem, Not A Helper

Observation must be modeled as a first-class runtime subsystem with owned
state, lifecycle, and facade access. It must not be implemented as:

- a bag of boxed callbacks hanging off `SignalRuntime`
- a thin wrapper around event subscribers
- an adapter-local cache over `TransactionResult`

Required consequence:

- `SignalRuntime` gains an owned observation subsystem
- builder and facade surfaces expose observation configuration explicitly

### 10.2 Resolution Must Precede Delivery

Observation strategy selection must happen before notification dispatch.

Acceptable:

- choosing a built-in observation policy during subscription registration
- lowering the relevant delivery strategy before commit dispatch
- deriving observation packets from transaction-staged facts

Not acceptable:

- asking listeners how they want to classify change during callback dispatch
- rescanning graph state from each observer callback
- deciding touched vs meaningful-change separately in different adapters

### 10.2.1 Change Classification Must Be Runtime-Locked

This milestone must freeze how the observation substrate interprets existing
runtime facts.

At minimum, the contract must distinguish:

- `Touched`
  The observer's scope intersected the committed transaction's touched derived
  surface, whether or not the node ultimately recomputed or emitted meaningful
  output change.
- `Recomputed`
  The observer's scope intersected a node whose committed evaluation verdict was
  a real recomputation rather than pure reuse or untouched carry-forward.
- `MeaningfulChange`
  The observer's scope intersected a committed node whose output-change basis,
  comparator result, or changed-region classification says the derived result
  changed in a way that downstream observers are allowed to treat as
  semantically different.

Required consequence:

- these classifications must be derived from existing runtime artifacts such as
  evaluation verdicts, output-change classification, comparator policy outcome,
  and changed-region evidence
- adapters and easy-mode surfaces must consume these classifications rather than
  inventing their own meanings

### 10.2.2 Registration, Freezing, Lowering, Delivery, And Explanation Are Distinct Phases

Observation must not collapse into one open packet type shared across runtime
construction, transaction staging, commit delivery, and diagnostics.

The architecture must preserve these distinct forms:

- registration intent
- frozen runtime-owned descriptor
- lowered matching or classification plan
- committed delivery packet
- diagnostic explanation artifact

Required consequence:

- no public or internal "god packet" may represent all observation phases at
  once
- each phase transition must consume the prior form and produce a narrower,
  more proven form
- diagnostics must consume committed observation artifacts or narrow retained
  summaries rather than reconstructing semantic truth from arbitrary raw
  transaction state

### 10.3 Delivery Must Be Commit-Bounded

Delivery is a semantic consequence of committed derived state, not of
in-progress mutation.

Required consequence:

- no normal observer callbacks during active mutation
- no normal observer callbacks during rollback
- one transaction must expose one coherent delivery boundary
- one observer must receive at most one normal delivery packet per committed
  transaction boundary

### 10.4 Observation Must Stay Derived

Observation artifacts are not truth and are not allowed to redefine execution
outcomes.

Changing observation richness or diagnostics policy must not change:

- evaluation truth
- transaction outcome
- branch truth
- replay truth
- event-bus semantics

### 10.5 Resource Lifecycle Must Be Framework-Owned

Observers are managed runtime resources. Registration, disposal, and delivery
must be framework-owned rather than caller-owned conventions.

Required consequence:

- explicit handle types
- explicit unsubscribe path
- deterministic runtime cleanup on drop or shutdown boundaries

### 10.5.1 Observation Delivery Must Be Read-Capability Only

The reentrancy contract must be enforced structurally rather than only by
documentation.

Required consequence:

- observer callbacks receive a read-phase observation context, not mutation
  authority
- observer delivery surfaces must not expose `&mut SignalRuntime` or any
  equivalent write-capable handle
- if later milestones add controlled write-back or deferred action mechanisms,
  they must be introduced as separate explicit capability types rather than by
  widening the observer callback context

### 10.6 Matching Breadth Must Be Indexed Architecturally

Observer matching must consume an explicit ownership and lookup structure. It
must not depend on scanning all active observers at commit time.

Required consequence:

- node-scoped observation must maintain a node-to-observer lookup path
- observer teardown must update that lookup path through a framework-owned
  lifecycle
- commit delivery must scale with staged relevant change and matching observer
  sets, not with total active observer count

Not acceptable:

- `for changed_node in ... { for observer in all_observers { ... } }`
- callback-time graph inspection to determine observer relevance
- observer-local rescans over the transaction result to decide applicability

## 11. Required Architecture Changes

### 11.1 Add A Dedicated Observation Subsystem

Add a new runtime-owned subsystem under `logic/transaction/runtime/state`,
likely alongside the existing observer and branching subsystems.

It should own:

- observer registry
- stable observer ids
- frozen observation strategy registry or built-in policy table
- deterministic observer ordering metadata
- delivery counters and diagnostics support

It must not own:

- truth state
- event-bus semantics
- host callback ABI specifics

### 11.2 Introduce Observation Policy And Strategy Types

Add explicit types for:

- `ObservationPolicy`
- `ObservationTrigger`
- `ObservationDeliveryMode`
- `ObservationScope`
- `ObservationHandle`
- `ObservationEvent`
- `ObservationStrategyDescriptor`
- `ObservationStrategyRegistration`
- `FrozenObservationStrategyRegistry`

The exact names may evolve, but the structure must preserve the existing
strategy pattern already used elsewhere in the runtime.

Milestone 11 must make one additional scope decision explicit:

- built-in sealed observation policies are required
- fully host-registerable strategy registration is optional in this milestone
  and must not be introduced unless the implementation proves a real semantic
  need that built-in sealed policies cannot satisfy honestly

The default bias for Milestone 11 is:

- freeze the substrate
- freeze the built-in policy family
- defer open-ended host strategy registration unless and until a second
  milestone needs it

The milestone should also prefer semantic newtype wrappers anywhere a shared
primitive would otherwise blur runtime meaning. This includes, at minimum:

- `ObserverId`
- `ObservationHandleId`
- `ObservationPolicyId`
- `ObservationStrategyId`
- `DeliveryOrdinal`
- `TransactionOrdinal`
- `ObservedNodeSet`
- `MatchingObserverSet`
- `ObservationCause`

### 11.3 Stage Observation Packets In Transactions

`TransactionScratch` needs an observation lane rather than forcing commit-time
rediscovery.

Expected additions include:

- staged observer candidate set
- staged observation candidate packets
- staged classified observation packets
- counters for classification and coalescing work

The commit path should consume already-lowered observation work, not rebuild it
from scratch.

The staged forms must be phase-distinct:

- `StagedObservationCandidate`
- `ClassifiedObservationEvent`
- `DeliveredObservationEvent`

The exact names may vary, but the type structure must encode what has been
proven at each phase rather than carrying one open bag of observation fields.

These forms should be sealed behind private fields and constructor or lowering
functions so later code cannot synthesize "already lowered" or "already
committed" observation packets by struct literal assembly.

### 11.3.1 Matching Must Lower Through Indexed Runtime Forms

Observer registration intent must not be used directly as the commit-time
matching surface.

Required consequence:

- registration lowers into runtime-owned index entries
- commit-time matching consumes those lowered index entries
- observer applicability is not rediscovered from raw registration declarations
  during delivery

This is the observation analogue of the existing planner rule:

declare once, freeze once, lower once, execute once

### 11.3.2 Classification Should Lower Once From Runtime Facts

The commit path must not repeatedly reinterpret evaluation verdicts,
comparator outcomes, output-change records, and changed-region evidence in
multiple places.

Required consequence:

- one lowering pass derives observation-relevant classification facts from the
  transaction's runtime artifacts
- commit delivery consumes those lowered classification facts
- diagnostics explanation consumes the committed classification facts or a
  retained summary derived from them

### 11.4 Add Commit-Path Delivery And Rollback Suppression

`commit_path.rs` must:

- finalize staged observation packets after graph patches commit
- classify them through the resolved observation policy
- dispatch them in deterministic order
- record observation provenance into diagnostics-visible artifacts

`rollback_path.rs` must:

- suppress normal observer delivery
- preserve typed rollback diagnostics when observation staging existed

### 11.5 Add Easy Watchers On Top Of The Same Substrate

`easy/runtime.rs` must grow a real watcher/effect surface that compiles to the
new observation machinery rather than implementing an easy-only reaction path.

This milestone does not need to solve wasm or React ergonomics, but it must
prove that the easy layer can sit cleanly on the same substrate.

## 12. Milestone Phases

### Phase 0: Contract Freeze

Deliver:

- the core type vocabulary for observation
- the subsystem boundary
- the product-decision lock encoded in docs and public naming
- the phase model for registration, freezing, lowering, delivery, and
  explanation

Must prove:

- graph subscribers, event subscribers, and runtime observers are
  non-overlapping categories in the code structure

### Phase 1: Registry And Handle Substrate

Deliver:

- observation registry
- observer ids and handles
- subscribe/unsubscribe surface
- deterministic ordering
- node-to-observer indexing substrate
- read-capability callback context type

Must prove:

- observer lifecycle is framework-owned
- registration is not ambient callback wiring
- the runtime can match changed nodes to relevant observers without scanning all
  active observers
- observer callbacks cannot mutate runtime truth through the exposed delivery
  interface

### Phase 2: Transaction-Staged Classification

Deliver:

- staged observation packets in `TransactionScratch`
- policy-based classification for touched / recomputed / meaningful-change
- coalescing logic per transaction
- lowered classification facts derived once from runtime artifacts

Must prove:

- commit path consumes staged packets instead of rediscovering them broadly
- classification truth is not recomputed differently by delivery and
  diagnostics paths

### Phase 3: Commit / Rollback Semantics

Deliver:

- commit-bounded dispatch
- rollback suppression
- typed transaction-result counters or summaries for observation

Must prove:

- no observer delivery from rolled-back state
- deterministic delivery order holds under repeated runs

### Phase 4: Diagnostics And Easy Surface

Deliver:

- observation provenance in diagnostics
- easy watcher/effect support on the same substrate
- public facade exposure
- proof-bearing committed observation artifacts or summaries suitable for
  diagnostics consumption

Must prove:

- diagnostics richness does not alter operational observer truth
- easy watchers are not a parallel runtime

## 13. Acceptance Surface

Milestone 11 is not done because one callback fired in one demo.

It is done only when the runtime can certify all of the following:

- commit-bounded observer delivery
- rollback suppression
- deterministic observer ordering
- touched vs meaningful-change classification
- transaction-local coalescing
- diagnostics-visible observer provenance
- easy watcher/effect compilation onto the same substrate
- boundedness of delivery work under representative high-churn transactions

### 13.1 Required Named Test Families

- `observation_commit_boundary`
- `observation_rollback_suppression`
- `observation_meaningful_change_classification`
- `observation_transaction_coalescing`
- `observation_ordering_determinism`
- `observation_reentrancy_and_lifecycle_rules`
- `observation_easy_surface_parity`
- `observation_branch_restore_and_merge_behavior`
- `observation_boundedness_counters`

These families are not standalone inventions. They are the owning implementation
lanes for the corresponding substrate requirements now declared in
[`test-requirements.md`](./test-requirements.md), especially:

- `1A. The adversarial observation and delivery equivalence test`
- `8A. The observation and managed-resource long-session extension`
- `9A. The future abstraction lifting rule`
- `9B. The future abstraction workload grammar`
- `10A. The substrate boundedness and lifting test`

### 13.2 Hostile Conditions Required In Certification

- recompute without meaningful output change
- multiple writes to the same node before commit
- event-bus activity in the same transaction
- commit failure after staging observation work
- branch restore and merge touching many nodes
- diagnostics tier variation across repeated certification lanes

## 14. Performance Contracts

The milestone must expose named counters for at least:

- observer registrations
- active observers
- staged observation candidate count
- matching observer set width
- delivered observation count
- coalesced observation count
- suppressed-by-policy count
- rollback-suppressed delivery count
- observation classification breadth
- observation provenance materialization count
- observer index maintenance cost

The milestone must also declare named complexity contracts for:

- subscribe
- unsubscribe
- commit-time observer matching
- commit-time delivery dispatch
- diagnostics-time provenance expansion
- index maintenance under churn

Each contract must name its real cost bases explicitly. At minimum:

- subscribe cost must be stated in terms of observed-scope width and registry
  maintenance work
- unsubscribe cost must be stated in terms of owned registration footprint
- commit matching cost must be stated in terms of changed derived surface and
  matching observer-set width, not total graph size
- diagnostics-time expansion cost must be explicitly separated from operational
  delivery cost
- index-maintenance cost must be explicitly separated from commit-time matching
  cost so long-session churn cannot hide inside one blended counter

Honest cost rules:

- registering an observer may be broad within the observer registry, but commit
  delivery may not broaden to whole-graph scans
- delivery work must scale with staged relevant change and matching observers,
  not with total graph size
- diagnostics richness must degrade by policy instead of always materializing
  full observer provenance
- provenance explanation must be a richer path than operational delivery rather
  than an always-on hot-path tax

## 15. Explicit Deferrals

Milestone 11 intentionally does not include:

- JS-facing callback ABI design
- React `useSyncExternalStore` wrappers
- async resource models
- frontend-form abstractions
- bridge-stream convergence with runtime-local observers
- full family and region observation product surfaces beyond what is needed to
  preserve the architecture honestly

Those remain later adapter or higher-layer milestones.

## 16. Milestone Done When

Milestone 11 is done only when `worth-signal` can support runtime-local
observation through a frozen, typed, policy-bearing substrate that:

- preserves the authority boundary with `worth-relational`
- preserves the integration boundary with `worth-runtime-bridge`
- never delivers rolled-back derived state
- never requires adapters to redefine change meaning
- exposes bounded, measurable delivery work
- supports easy watchers/effects without a second execution engine
- leaves frontend ergonomics as a higher-layer concern instead of polluting the
  foundational runtime

And when the certification story is strong enough that later abstractions such
as forms, resources, outputs, and adapters are forced to inherit these runtime
truths through the shared substrate requirements in
[`test-requirements.md`](./test-requirements.md) rather than inventing
abstraction-local semantics.

At that point, `worth-signal` will finally own the missing middle category that
its vision already implies: not just recomputation, and not just diagnostics,
but honest observation of committed derived-state change.
