# Milestone 4 Engineering Spec: Transition And Outcome Algebra

> **Status:** Closed
>
> **Closeout:** [milestone-4-closeout.md](./milestone-4-closeout.md)
>
> **Roadmap parent:** [worth_proof_roadmap.md](./WORTH_proof_roadmap.md)
>
> **Vision parent:** [worth_proof_vision.md](./WORTH_proof_vision.md)
>
> **Test requirements:** [test-requirements.md](./test-requirements.md)
>
> **Adjacent milestone:** [milestone-3.md](./milestone-3.md)
>
> **Adjacent milestone closeout:** [milestone-3-closeout.md](./milestone-3-closeout.md)
>
> **Impacted later milestones:**
> - `Milestone 5` (`Lowering And Execution Readiness`)
> - `Milestone 6` (`Static Fork And Join Progression`)
> - `Milestone 7` (`Certification And Cross-Crate Migration Closure`)
>
> **Primary architectural driver:** make typed transition and branching outcome law canonical now that Milestone 3 has made freshness, downgrade, and re-admission states explicit

## Goal

Define the canonical transition substrate for `worth-proof` so proof-bearing
values can move through typed success, denial, defer, stale, and
rebind-required
progression without collapsing into one generic `Result` story or forcing each
WORTH crate to keep inventing its own transition algebra.

## Why This Milestone Exists

Milestone 1 established the core zero-cost carrier family.
Milestone 2 sealed proof minting and witness authority.
Milestone 3 made validity basis-scoped and trust-shift-honest.

That leaves the next structural gap:

- WORTH still lacks one shared way to express what a progression step returns
  once it becomes richer than "next type or error"
- domain crates repeatedly need to distinguish:
  - admitted progression
  - semantic denial
  - defer or not-yet-admissible posture
  - stale or rebind-required posture
  - hard failure that is not just a domain denial
- without a canonical transition algebra, later milestones would either:
  - reintroduce bespoke per-crate result enums and lowering wrappers, or
  - standardize too weakly around `Result<Next, Error>` and erase the real
    topology that WORTH workflows depend on

Milestone 4 therefore exists to solve the next hard shared-law problem:

- how a proof-bearing transition declares its input and output contract
- how non-terminal outcomes remain typed and machine-checkable
- how composition preserves proof-widening order
- how illegal progression can reject before expensive construction begins
- how later lowering, execution-readiness, and fork/join work can build on one
  honest transition grammar instead of several local ones

## Hard Part

The hard part is not creating another enum named `TransitionResult`.

The hard part is preserving all of these at once:

- exact domain denial and failure topology rather than a generic collapsed
  error story
- compile-time ordering law from Milestones 1 through 3
- explicit stale, rebind, and re-admission states from Milestone 3 without
  re-encoding them as ad hoc metadata
- zero-cost static dispatch rather than object-safe runtime transition
  machinery
- pre-construction rejection so expensive domain object assembly is not the
  first point of illegality detection
- enough shared structure that later milestones can compose on top of it
  honestly

The design fails if:

- `worth-proof` standardizes on one oversimplified binary result shape
- denial, defer, stale, and hard failure become observationally similar
- transition composition forces erased trait objects or hidden runtime lookup
- preconditions are checked only after expensive construction already occurred
- later lowering or execution-readiness milestones would need to replace the
  transition substrate instead of extending it

## Explicit Assumptions

- Milestone 1 carrier, proof-set, and facade law remain authoritative.
- Milestone 2 sealed minting and witness authority remain authoritative.
- Milestone 3 freshness, downgrade, and explicit re-admission states remain
  authoritative inputs to this milestone rather than special-case side paths.
- `worth-proof` still owns progression law only; it does not become a generic
  workflow runtime, planner, diagnostics engine, or domain error catalog.
- domain crates remain the semantic authorities for what counts as denial,
  defer, stale, or hard failure in their own workflows.
- Milestone 4 may define transition contracts, outcome families, and
  composition helpers, but must not smuggle in a runtime graph engine or
  dynamic registry.
- full lowering/execution-readiness distinction remains the next milestone,
  though Milestone 4 must leave that work with an honest transition grammar to
  build on.

## Governing Summaries

- `MENTALITY.md`
  The most important thing it protects here is solving the structural failure
  first. Milestone 4 therefore hardens typed transition and denial law now,
  before Milestone 5 or Milestone 6 pile more orchestration on top of bespoke
  result surfaces.
- `arch_laws.md`
  The most important thing it protects is that progression ordering, rejection,
  and failure topology must be encoded mechanically. Laws 5, 24, 29, 30, 37,
  39, and 41 shape this milestone most strongly.
- `perf_laws.md`
  The most important thing it protects is that transition law must carry proof
  and resolved control-plane decisions forward without hidden rediscovery,
  erased dispatch, or post-hoc rejection after expensive work already started.
- `domain_laws.md`
  The most important thing it protects is responsibility clarity. Outcome
  vocabulary, transition contracts, composition rules, and certification
  should be distinct responsibilities rather than blurred into one omnibus
  "transition helpers" module.
- `worth_proof_vision.md`
  The most important thing it protects is the identity of `worth-proof` as a
  static progression-law substrate that owns typed transitions and branching
  outcomes, but not runtime orchestration or domain semantics.
- `worth_proof_roadmap.md`
  The most important thing it protects is sequencing. Milestone 4 is the first
  place where stale-honest proof-bearing states become part of a canonical
  transition model, and Milestone 5 depends on that model being honest first.
- `worth-proof` test requirements
  The most important thing it protects is that transition law must be
  certified through a named suite with real success, denial, defer, stale, and
  pre-construction rejection evidence rather than only one happy-path API
  example.
- `milestone-3.md`
  The most important thing it protects is the typed stale, rebind, and
  readmission substrate. Milestone 4 must consume those states as first-class
  transition outcomes rather than re-describing them in weaker generic error
  channels.
- `milestone-3-closeout.md`
  The most important thing it protects is what later milestones may now assume:
  basis-scoped validity, explicit downgrade, explicit trust-boundary
  progression, and compile-time denial of strong-basis misuse.

## Adversarial Constraint

The milestone must survive the following hostile condition:

> Several WORTH subsystems with proof-bearing staged inputs, explicit stale and
> rebind states, authority-sensitive denials, and high-performance execution
> requirements must be able to express progression so that legal transitions
> remain statically ordered, denial classes remain semantically distinct, and
> rejection occurs before expensive construction or erased runtime dispatch
> becomes necessary.

The design fails if:

- a domain must flatten denial, defer, stale, and hard failure into one generic
  error channel to participate
- transition composition allows proof-ordering drift or skipped prerequisites
- transition helpers require object-safe erased dispatch on hot paths that are
  statically knowable
- pre-construction illegality is discovered only after rich domain values were
  already assembled
- stale and rebind-required outcomes are merely wrapped as text or optional
  fields rather than typed structural outcomes
- Milestone 5 would need to invent a parallel transition grammar for lowered
  and execution-ready flows

## Product Decision Lock

- transitions are proof-bearing contracts, not ad hoc functions that happen to
  return stronger forms
- outcome topology is first-class and typed; generic `Result` is insufficient
  for the representative WORTH pressure this milestone targets
- denial, defer, stale, rebind-required, and hard failure are distinct semantic
  classes whenever their remediation or observability posture differs
- pre-construction rejection is part of the substrate contract, not an
  optimization detail
- transition composition must preserve proof-widening order explicitly
- static dispatch is the default and normative posture; erased runtime dispatch
  is not the hot-path model of the milestone
- the crate facade remains the only public entry surface
- transition law may standardize structure, but not erase domain-owned meaning
  or failure categories
- Milestone 3 freshness and readmission states remain the canonical source for
  stale and rebind-sensitive outcomes

Normative consequence:

- any implementation that standardizes outcome law by collapsing it into one
  `Result<Next, Error>` shape is out of spec
- any implementation that treats stale or rebind outcomes as diagnostics-only
  side channels is out of spec
- any implementation that defers clearly knowable rejection until after
  expensive construction is out of spec
- any implementation that forces runtime-erased dispatch on representative
  static lanes is out of spec

## Required Contracts

### Outcome Family Rule

Every canonical transition surface introduced by this milestone must make the
primary outcome family explicit.

Required vocabulary:

- success or admitted-next posture
- semantic denial posture
- defer or not-yet-admissible posture
- stale-derived posture
- rebind-required posture
- hard-failure posture

Rules:

- distinct remediation paths require distinct typed outcomes
- stale-derived and rebind-required outcomes must be able to preserve Milestone
  3 proof meaning directly rather than being translated into generic failure
  text
- the milestone may permit narrower outcome families for specific transitions,
  but only when that narrowing is explicit and structurally honest

### Canonical Transition Contract Rule

The crate must define one shared way to express a proof-bearing transition from
an input posture to an output or branching outcome family.

Required vocabulary:

- typed transition input
- typed transition output or outcome family
- optional transition context posture
- transition authority or witness posture where required

Rules:

- transition signatures must encode ordering by consuming the prior proof form
  and producing only the next legal outcome family
- if context is needed, it must be explicit in the contract rather than hidden
  ambiently
- the transition substrate must support both pure progression and
  authority-sensitive progression without turning all transitions into one
  maximally broad shape

### Denial Topology Preservation Rule

Transition law must preserve real semantic distinctions instead of forcing
domains into one convenience failure bucket.

Required vocabulary:

- advisory or informational non-blocking posture when applicable
- semantic denial posture
- defer posture
- stale or rebind-sensitive posture
- hard failure posture

Rules:

- the shared substrate may standardize category shape, but not overwrite
  domain-owned semantics inside those categories
- callers must be able to match on category at the type level
- later milestones must be able to route lowering and execution-readiness
  behavior based on typed outcome family rather than re-parsing diagnostics

### Pre-Construction Rejection Rule

Eligibility and ordering failures that are knowable before expensive domain
construction must be rejected before that construction begins.

Required vocabulary:

- pre-construction input posture
- construction-admitted posture
- rejection-before-construction outcome

Rules:

- the transition substrate must leave room for cheap rejection before rich
  payload assembly, resolution, or lowering work
- representative transition APIs must make it obvious whether the caller is
  still in pre-construction space or has crossed into admitted construction
- this milestone does not need to solve every domain planner, but it must lock
  the structural law that later planners and lowerers build on

### Composition Ordering Rule

Transition composition must preserve proof-widening order and must not allow a
composed path to skip structural prerequisites.

Required vocabulary:

- sequential composition posture
- composed success posture
- early-terminal non-success posture

Rules:

- composition must preserve the same ordering law as direct hand-written
  progression
- if one step can end in denial, defer, stale, or failure, later steps must not
  run as if success had occurred
- composition helpers must remain static and monomorphization-friendly for the
  representative admitted scope

### Performance-Shaping Rule

Transition algebra must remain zero-cost honest for statically knowable
progression lanes.

Required performance-shaping surfaces:

- representative direct transition lane
- representative composed transition lane
- representative pre-construction rejection lane

Rules:

- transition composition must not require mandatory allocation
- representative static lanes must not require mandatory virtual dispatch or
  runtime graph lookup
- control-plane facts proven earlier in the transition chain must be carried
  forward rather than rediscovered during later steps

## Scope

### In Scope

- canonical typed transition contract or equivalent transition family
- typed outcome vocabulary for success, denial, defer, stale, and
  rebind-required representative surfaces
- typed hard-failure coexistence with richer non-failure outcomes
- explicit transition context posture where context is required
- sequential transition composition helpers that preserve proof order
- explicit pre-construction rejection substrate
- facade hardening needed to keep internal transition machinery private
- milestone-local certification notes that map directly onto the crate-level
  `Transition And Outcome Algebra Test`

### Explicitly Out Of Scope

- full lowering versus execution-readiness separation
- generic runtime workflow execution or graph orchestration
- diagnostics schema ownership, receipts, lineage, or provenance surfaces
- domain-specific taxonomies for every possible denial or failure reason
- broad cross-crate migration closure beyond what is needed to define the
  substrate shape

## Phases

### Phase 1: Outcome Vocabulary And Structural Boundary

Define the representative outcome algebra before standardizing any transition
trait or composition helper.

Must ship:

- canonical representative outcome vocabulary for:
  - success
  - denial
  - defer
  - stale-derived
  - rebind-required
  - hard failure
- clear distinction between non-terminal semantic outcomes and hard failure
- representative integration path for Milestone 3 stale and rebind states
- explicit rejection of one global "all transitions return one broad enum"
  posture when narrower typed families are more honest

Implementation guidance:

- start with the semantic categories and only then choose the carrier shape
- preserve room for transitions whose honest topology is narrower than the full
  representative family
- keep Milestone 3 state-carrying forms intact instead of flattening them into
  strings or optional status fields
- define the minimum shared vocabulary needed for later lowering and readiness
  work without pretending all domains share the same denial reasons

This phase is complete only when the crate can honestly express "success",
"semantic denial", "defer", "stale/rebind-sensitive non-success", and "hard
failure" as structurally distinct outcomes.

### Phase 2: Canonical Transition Contract

Define the core transition contract that consumes a proof-bearing input and
returns a typed outcome family in proof-order-preserving form.

Must ship:

- one canonical transition contract or equivalent family of transition traits
- explicit input and output posture
- explicit context posture where runtime or authority context is genuinely
  needed
- representative witness-sensitive transition lane proving that static
  authority-bearing progression still fits the contract
- facade-owned exposure strategy for the public transition surface

Implementation guidance:

- the contract should encode ordering by types, not by documentation
- avoid one maximally generic contract if a small family of closely related
  contracts is more honest about cost or outcome shape
- keep context explicit and narrow; do not smuggle transition environment in as
  ambient state
- solve the representative no-context lane first, then extend to explicit
  context-bearing lanes only where structurally required

This phase is complete only when a proof-bearing transition can be expressed in
shared substrate terms without losing type-enforced ordering or richer outcome
topology.

### Phase 3: Composition And Pre-Construction Rejection

Define how transitions chain safely and how cheap illegality rejection happens
before expensive construction or lowering begins.

Must ship:

- representative sequential composition helper or pattern
- early-terminal behavior for non-success outcomes
- explicit pre-construction rejection posture
- representative lane proving illegal progression can reject before rich domain
  object assembly
- representative lane proving composed transitions preserve proof-widening
  order without erased dispatch

Implementation guidance:

- composition should feel like proof-preserving progression, not like a runtime
  orchestrator
- pre-construction rejection belongs before expensive payload assembly,
  resolution, or lowering, not as a cleanup pass afterward
- preserve category distinction across composition; a denial must not be forced
  through the same machinery as a hard failure just to keep the helper generic
- if composition helpers begin looking like a workflow engine, the design has
  crossed the crate boundary and must be reduced

This phase is complete only when the crate has an honest story for "check
eligibility early, then compose only the proven next steps" without runtime
machinery creep.

### Phase 4: Hostile Certification And Closure

Prove that transition law is typed, denial-rich, ordering-preserving, and
pre-construction-honest rather than a renamed `Result` wrapper.

Must ship:

- hostile compile-fail coverage for ordering misuse and skipped prerequisites
- hostile transition tests covering:
  - success
  - denial
  - defer
  - stale or rebind-sensitive non-success
  - hard failure
- explicit pre-construction rejection certification lane
- closure record of what Milestone 5 and later milestones may now assume about
  transition law

Implementation guidance:

- the named suite should certify category distinction directly rather than
  inferring it from helper internals
- include at least one equivalence lane for semantically identical admitted
  progression through alternate but legal paths
- publish explicit residual debt if first ship leaves representative rather
  than exhaustive domain integration

This phase is complete only when the milestone has machine-checkable evidence
that ordering is enforced, denial classes stay distinct, and illegal
progression rejects before expensive construction.

## Must Ship

- one canonical typed transition contract or equivalent transition family
- one canonical representative outcome vocabulary richer than generic binary
  success/error
- explicit typed coexistence of denial, defer, stale-derived, rebind-required,
  and hard-failure outcomes where their semantics differ
- explicit transition context posture where needed
- representative sequential composition support
- explicit pre-construction rejection support
- compile-fail coverage for ordering misuse
- milestone-local implementation notes that map directly onto the crate-level
  certification bar in `worth-proof/test-requirements.md`

## Must Preserve

- Milestone 1 carrier and proof-set law
- Milestone 2 sealing and witness authority posture
- Milestone 3 freshness, downgrade, and readmission semantics
- zero-cost hot-path posture after monomorphization
- no mandatory heap allocation
- no mandatory dynamic dispatch on representative static lanes
- no runtime graph engine hidden under the transition surface
- no generic error collapse that erases real denial topology
- no delayed rejection of clearly knowable illegality until after expensive
  construction
- no drift of `worth-proof` into diagnostics, provenance, lineage, planner
  ownership, or runtime orchestration semantics

## Acceptance Evidence

Milestone 4 is complete only when `worth-proof` satisfies the named milestone
suite:

- `Transition And Outcome Algebra Test`

Required machine-checkable outputs:

- `transition_digest`
- `failure_digest`
- `compile_fail_bundle`
- `codegen_honesty_report`

Milestone-specific proof obligations:

- transition ordering remains structurally enforced
- semantically distinct denial classes remain typed and machine-checkable
- equivalent admitted transition lanes match exactly
- intentionally different denial, defer, stale, and hard-failure lanes diverge
  explicitly
- representative illegal progression rejects before expensive construction
- representative composition lanes do not force erased runtime dispatch or
  hidden allocation
- the suite includes hostile misuse lanes rather than only happy-path
  progression

Milestone 4 is not closed by "there is a transition trait" or "callers can
pattern match some enum" arguments.

## Architectural Notes

- Milestone 4 is transition law, not execution runtime.
- Outcome categories are shared substrate vocabulary; domain crates still own
  the semantic reasons inside those categories.
- Narrower typed transition families are preferable to one giant maximally
  generic contract if the giant contract would cross cost or failure-topology
  boundaries.
- Pre-construction rejection is part of progression law because construction
  timing affects both cost honesty and invalid-state prevention.

## Implementation Topology

This milestone should extend the existing topology without replacing the facade
or Milestone 3 assumption surfaces.

Preferred additions:

```text
crates/worth-proof/src/
  facade.rs
  transition/
    mod.rs
    contract.rs
    outcomes.rs
    composition.rs
    rejection.rs
```

This is not a forced final topology, but the ownership boundaries are
intentional:

- `transition/contract.rs`
  - core transition trait or equivalent typed progression contract
- `transition/outcomes.rs`
  - canonical outcome vocabulary and category families
- `transition/composition.rs`
  - sequential composition helpers and early-terminal composition law
- `transition/rejection.rs`
  - pre-construction rejection surfaces and representative cheap-admission law

The milestone should avoid:

- one giant `transition.rs` that mixes outcome categories, contract law,
  composition, and pre-construction rejection
- hiding Milestone 3 stale/readmission meaning behind generic transition text
- building a runtime orchestrator or graph executor under the name of
  transition composition
- exposing internal helper machinery publicly instead of through the facade

## Sequencing Notes

- This milestone belongs immediately after Milestone 3 because transition law
  needs typed stale, rebind, and readmission states before it can standardize
  branching outcomes honestly.
- Milestone 5 depends on this milestone because lowering and execution
  readiness need a canonical way to express "admitted", "deferred",
  "rebind-required", and "hard-failed" progression without inventing a second
  result grammar.
- Milestone 6 depends on this milestone because fork/join and same-family
  progression need a canonical non-success and composition story before they
  widen from scalar transitions to multi-artifact transitions.
