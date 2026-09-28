# WORTH Proof Future Roadmap

## Purpose

This document defines the future work for `worth-proof`.

It is a future-only roadmap. It does not assume the proof substrate already
exists, and it does not treat proof-bearing progression as a thin convenience
layer over crate-local typestate helpers. It exists to sequence the work
required to make proof-bearing lifecycle progression as rigorous, zero-cost,
stale-honest, and execution-boundary-honest as the rest of the WORTH stack
needs it to be.

The operating rule for this roadmap is:

`prove once, encode permanently, execute only the proven form`

That rule governs every milestone:

1. progression law must be encoded in types rather than comments, booleans, or
   ambient convention
2. proof minting must be sealed so only the proving boundary can construct the
   stronger form
3. stale, assumption-scoped, and authority-scoped validity must be explicit
   rather than inferred from host discipline
4. lowering and execution readiness must remain distinct proof-bearing phases
5. power must come from compile-time structure, not runtime machinery
6. no milestone may compromise zero-cost hot-path behavior in exchange for
   generic runtime flexibility
7. wherever unresolved, resolved, symbolic, authoritative, or derived forms
   carry different guarantees, the roadmap should prefer distinct types over
   comments or booleans
8. trust-boundary crossings and proof downgrade paths must be modeled as real
   progression events rather than invisible adapter behavior

## Adversarial Constraint

`worth-proof` must survive the following hostile condition:

> A WORTH subsystem with branch-sensitive proof validity, authority-gated
> progression, mixed denial paths, multi-artifact transitions, and strict
> zero-cost hot-path requirements must be able to express its legal phase
> progression so that illegal progression is unrepresentable, stale-proof use
> is mechanically visible, and the compiled code introduces no material runtime
> overhead relative to bespoke handwritten domain code.

If any supported usage:

- requires runtime proof lookup for a statically knowable transition
- forces domains to erase distinct failure topologies into one generic error
- allows callers to WORTH a stronger proof-bearing form through ordinary
  construction
- permits stale proof consumption without an explicit revalidation or rebind
  boundary
- blurs lowered, execution-ready, and executed forms into one convenience type
- hides the cost difference between one-to-one and multi-artifact progression
- makes domains pay for diagnostics, lineage, provenance, or dynamic registry
  behavior they do not need

then `worth-proof` has failed.

## Roadmap Rules

Rules for every remaining proof-substrate item:

- each milestone must describe a real progression-law capability boundary, not
  just "add some helpers"
- each milestone must preserve the ownership split:
  `worth-proof` owns proof-bearing progression law, `worth-foundational` owns
  shared truth vocabulary, and domain crates own domain semantics and runtime
  behavior
- every milestone must distinguish static compile-time law from optional
  boundary-facing runtime materialization
- no milestone is complete until it has machine-checkable zero-cost and
  correctness evidence through compile-fail coverage, hostile progression
  scenarios, and exact counter or codegen-sensitive proof obligations where
  applicable
- sequence numbers express logical dependency order, not a promise that all
  migration work must wait for every later convenience surface
- every milestone must declare its own adversarial constraint
- every hot-path milestone must declare named complexity or codegen honesty
  obligations
- any knowingly incomplete first ship must be marked as explicit debt rather
  than implied completeness

## Operating Modes

The roadmap preserves these proof-substrate operating modes explicitly:

- `Static phase mode`: phase, proof, and assumption state are fully known at
  compile time and progression is entirely static
- `Boundary-bridged mode`: a static proof-bearing core artifact is converted to
  or from a canonical boundary-facing representation owned elsewhere
- `Assumption-scoped mode`: proofs remain valid only under an explicit basis
  such as schema version, branch epoch, authority digest, or policy version
- `Runtime-admitted mode`: runtime facts may mint executor-ready or executed
  witnesses, but they must terminate back into static proof-bearing forms
  rather than leaving the crate in a generic runtime-tagged progression state
- `Re-admitted mode`: values crossing serialization, restore, transport, or
  other trust-boundary shifts must pass through an explicit re-admission
  progression before regaining stronger proof-bearing status
- `Recipe mode`: symbolic intent, resolved bindings, lowered plans, and
  execution-admitted forms remain distinct proof-bearing stages
- `Composition-family mode`: composition-local symbolic siblings, existing
  authoritative targets, and identity-transforming family interactions lower
  into one deterministic family program before one coherent authority boundary
  closes
- `Multi-artifact mode`: one proof-bearing artifact may fork into several
  outputs or several inputs may join into one output without dynamic runtime
  graph ownership

This roadmap does not admit:

- a runtime transition registry in the core crate
- dynamic proof bags as the primary hot-path substrate
- generic plugin-defined progression graphs as a substitute for static law

## Obligation Surface Convention

Every milestone in this roadmap must be read as four separate obligation
surfaces, even when a section heading still says `Must Ship` for readability:

- `surface primitives`: the concrete phase markers, proof markers, witness
  types, transition traits, or outcome forms that must exist
- `semantic guarantees`: the meaning those primitives must preserve under
  progression, denial, staleness, authority, and lowering/execution boundaries
- `proof obligations`: the compile-fail coverage, hostile progression cases,
  equivalence checks, and exact counter or codegen assertions that must exist
  before the milestone is honest
- `migration unlocks`: the repeated bespoke machinery in domain crates that this
  milestone should make removable without semantic regression

If a roadmap line item names only API surface but not semantic or proof
obligations, it is incomplete.

## Platform Substrate Stance

`worth-proof` is not intended to become a second runtime. It is intended to
become the shared static substrate beneath several runtimes and planners.

That means:

- ordinary WORTH crate authors should be able to express most proof-bearing
  lifecycle progression through `worth-proof` instead of bespoke typestate
  scaffolding
- `worth-proof` may expose zero-cost markers, traits, witnesses, and transition
  helpers, but it must not steal ownership of diagnostics, lineage, provenance,
  storage layout, execution policy, or domain semantics
- lower crates remain the semantic authorities for truth, query meaning,
  reactive scheduling, compatibility law, and effect execution
- "shared substrate" therefore means one daily-driver proof language with
  authority-preserving boundaries, not one giant artifact engine that all
  crates are forced through

The roadmap must stay honest to both halves of that claim:

- if a proof-bearing progression pattern is repeated across WORTH, it needs an
  explicit roadmap home here
- if the lower crate remains authoritative for the semantics, the roadmap must
  say so explicitly instead of implying `worth-proof` became a second domain
  engine

## Early Cross-Feature Proof Gates

The hardest failures in `worth-proof` live at feature intersections rather than
inside isolated marker types. The roadmap therefore requires these cross-feature
proof gates before broad migration:

- `Milestone 1` must prove that phase-typed artifacts and proof-set composition
  remain zero-cost and do not require runtime proof lookup for statically known
  progression
- `Milestone 2` must prove sealed proof minting and witness-based authority
  cannot be bypassed by ordinary callers or convenience constructors
- `Milestone 3` must prove assumption-scoped proofs, freshness classes, and
  rebind-required outcomes preserve stale-proof honesty without collapsing into
  generic optional metadata
- `Milestone 3` must also prove trust-boundary re-admission and downgrade
  paths remain explicit rather than ambient
- `Milestone 4` must prove the transition model preserves failure topology,
  denial richness, and compile-time progression ordering without forcing one
  oversimplified `Result` story
- `Milestone 5` must prove lowering and execution readiness remain mechanically
  distinct so executors can consume proof-bearing lowered forms without
  rediscovering legality
- `Milestone 6` must prove static fork/join progression remains cost-honest and
  does not smuggle in a dynamic artifact graph runtime
- `Milestone 6` must also prove deterministic composition-family lowering for
  same-family symbolic and existing-target interaction without turning
  composition-local symbols into fake authoritative identities
- `Milestone 7` must prove the substrate is migration-worthy through hostile
  cross-crate reference integrations and exact counter or codegen honesty on
  representative hot paths

## Milestone 1: Core Artifact And Proof Substrate

### Goal

Establish the zero-cost core artifact model: phase-typed artifacts,
proof-bearing forms, proof-set composition, and the minimal marker/wrapper
substrate required for all later progression features.

### Adversarial Constraint

The core substrate must support repeated WORTH proof-bearing progression
patterns without introducing runtime proof lookup, dynamic phase tags on
statically known paths, or extra allocation relative to handwritten domain
types.

### Must Ship

- phase marker vocabulary for statically known progression states
- core proof marker vocabulary and proof-set composition model
- artifact wrapper forms capable of carrying phase, payload, proof, and
  assumption state without forcing one runtime representation
- proof-carrying collection helpers for common proven collection properties
- fixed-shape wrappers for important small-cardinality forms
- room for proof-bearing structural facts such as canonical order, uniqueness,
  disjointness, or normalization
- minimal access patterns that preserve read-only proof honesty
- zero-cost composition patterns for common proof-bearing wrapper shapes

### Must Preserve

- monomorphization-friendly codegen
- no mandatory heap allocation
- no mandatory dynamic dispatch
- no loss of crate-local payload ownership
- no forced diagnostics, lineage, or provenance baggage

### Proof Obligations

- compile-fail coverage proving later-phase operations are uncallable on
  earlier-phase artifacts
- hostile equivalence checks proving semantically identical proof-bearing forms
  do not require runtime proof lookup
- exact allocation and dispatch proof points on representative wrapper paths
- evidence that proof-set composition does not force hidden dynamic storage
- compile-fail or type-shape tests for fixed-shape and proof-carrying
  collection forms where emptiness, ordering, uniqueness, or cardinality matter
- proof that proven structural facts can be carried forward without repeated
  rediscovery or ambient side-channel markers

### Migration Unlocks

- crate-local proof-bearing wrappers that differ only by progression mechanics
- repeated "validated/lowered/admitted" marker boilerplate
- bespoke local proof bundle tuples or marker structs where the only variation
  is progression law

## Milestone 2: Sealed Minting And Witness Authority

### Goal

Make proof-bearing progression mechanically enforceable by standardizing sealed
construction, witness-based minting, and trusted authority patterns.

### Adversarial Constraint

No caller outside the proving boundary may be able to WORTH a stronger
proof-bearing form, bypass authority-bearing prerequisites, or construct an
execution-ready value without the required witness path.

### Must Ship

- sealed-construction patterns for proof-bearing outputs
- witness and token patterns for proof minting
- zero-sized authority and capability witness surfaces
- recipe progression sealing patterns for symbolic -> resolved -> lowered ->
  admitted forms
- canonical crate-level privacy patterns for progression-critical constructors
- guidance and helper surfaces for compile-fail boundary tests

### Must Preserve

- no runtime permission scaffolding where a static witness is sufficient
- no weakening of domain-specific failure semantics
- no ambient global authority state in the substrate

### Proof Obligations

- compile-fail coverage for WORTHd minting attempts and constructor bypass
- compile-fail or privacy-boundary tests for witness misuse
- hostile construction tests proving ordinary callers cannot synthesize stronger
  proof-bearing forms through convenience APIs
- evidence that witness-bearing paths compile away without runtime registry or
  ambient authority lookup
- hostile tests proving symbolic/unresolved recipe stages cannot be WORTHd into
  resolved or admitted forms by ordinary callers

### Migration Unlocks

- bespoke token/private-field/constructor sealing scattered across crates
- repeated compile-fail privacy boundary patterns
- ad hoc authority markers and one-off witness structs

## Milestone 3: Assumption, Freshness, And Staleness Law

### Goal

Make scoped validity first-class so proofs can be carried honestly across schema
version, branch epoch, policy basis, authority digest, and replay-sensitive
contexts.

### Adversarial Constraint

WORTH must not be able to accidentally consume a proof outside the basis under
which it was established, or silently treat rebind-required and stale states as
ordinary valid progression.

### Must Ship

- assumption-scoped proof vocabulary
- freshness and staleness marker model
- unresolved-to-resolved distinction helpers where resolution changes validity
  or admissibility guarantees
- trust-boundary admission/re-admission helpers
- downgrade/invalidation progression helpers for basis expiry, rebind-required,
  or authority-loss cases
- rebind-required and authority-revalidation-required state surfaces
- explicit invalidation/revalidation boundaries in the progression substrate
- support for carrying or re-establishing proofs across trust-boundary shifts

### Must Preserve

- no fake global validity for basis-scoped proofs
- no hidden stale-proof reuse
- no generic metadata blob standing in for typed freshness law
- no silent proof preservation across trust-boundary crossings
- no hidden collapse of stronger forms into weaker forms without explicit type
  transitions

### Proof Obligations

- compile-fail or type-shape tests proving stale or rebind-required forms are
  not silently consumable as fully valid forms
- hostile basis-shift tests covering schema, branch, policy, and authority
  rebasing
- evidence that assumption-scoped validity travels with the proof-bearing form
  rather than ambient side state
- proof that revalidation/rebind boundaries are explicit in the type flow
- hostile tests proving unresolved forms cannot silently flow into resolved-only
  APIs
- hostile re-admission tests proving restored or transported values must regain
  strong proof-bearing status explicitly
- hostile downgrade tests proving expired or invalidated forms cannot continue
  flowing through stronger APIs

### Migration Unlocks

- bespoke stale-proof markers
- ad hoc freshness enums and booleans
- branch/schema/policy scoped validity surfaces that currently drift by crate

## Milestone 4: Transition And Outcome Algebra

### Goal

Define the canonical transition model: typed input/output progression, typed
denial topology, and non-terminal branching outcomes that remain richer than
generic runtime errors.

### Adversarial Constraint

The transition substrate must allow domains to preserve their real failure and
denial topology without reintroducing bespoke transition machinery or erasing
important semantic distinctions.

### Must Ship

- core transition trait or equivalent typed progression contract
- typed success, denial, defer, stale, and rebind-required outcome patterns
- transition context model that stays static where possible and explicit where
  not
- transition composition patterns that preserve proof-widening order
- explicit support for pre-construction rejection

### Must Preserve

- exact failure topology from domain crates
- no one-size-fits-all generic error collapse
- no runtime graph engine hidden under the transition API

### Proof Obligations

- compile-fail coverage proving transition ordering remains structurally
  enforced
- hostile denial-topology tests proving advisory, denial, defer, stale, and
  hard-failure paths remain distinguishable
- evidence that transition composition does not force erasure into object-safe
  runtime dispatch on hot paths
- pre-construction rejection tests proving illegal progression is rejected
  before expensive domain object construction

### Migration Unlocks

- bespoke lowering/report/denial scaffolding
- repeated local progression result enums or wrappers
- inconsistent "success vs denial vs error" patterns across crates

## Milestone 5: Lowering And Execution Readiness

### Goal

Make pure lowering and execution readiness first-class proof-bearing boundaries
so planners can produce canonical lowered forms and executors can consume only
the proven executable form.

### Adversarial Constraint

Execution must not rediscover legality, strategy, or authority that should have
been established during planning, lowering, or readiness admission.

### Must Ship

- explicit lowered-form marker surfaces
- explicit execution-ready marker surfaces
- transition patterns for lowered -> execution-ready progression
- typed progression hooks for execution-ready -> executed or
  execution-attested forms where a domain needs post-execution proof state
- support for authority- or basis-gated execution readiness
- substrate guidance for separating pure lowering from effectful execution

### Must Preserve

- plan / execute separation
- no executor-side strategy rediscovery
- no silent collapse of lowered and executed semantics into one type

### Proof Obligations

- compile-fail or type-flow tests proving lowered, execution-ready, and
  executed forms remain distinct where the domain models all three
- hostile runtime-admission tests proving runtime-gated readiness still
  terminates in static proof-bearing forms
- evidence that executors can consume proof-bearing admitted plans without
  re-deciding legality
- proof that receipts and descriptive execution artifacts remain outside
  `worth-proof` while post-execution proof state can still be modeled

### Migration Unlocks

- repeated lowered-plan versus execution-ready plan scaffolding
- bespoke execution token plumbing
- repeated plan/readiness boundary marker patterns across crates

## Milestone 6: Static Fork And Join Progression

### Goal

Support compile-time honest many-input and many-output progression for the
subset of WORTH flows that split one proof-bearing artifact into several
artifacts or join several artifacts into one result.

### Adversarial Constraint

The substrate must support fork/join progression without forcing domains into a
dynamic artifact graph engine, hidden allocation, or cost-dishonest generic
container model.

### Must Ship

- static fork helpers for one-to-many progression
- static join helpers for many-to-one progression
- proof-preserving composition patterns across several inputs
- explicit cost-honest APIs for small fixed-arity joins
- disjoint-pair / non-empty / exactly-one style helper patterns where those
  cardinalities materially change correctness or execution shape
- composition-family helpers for symbolic sibling declaration, family-local
  resolution, and deterministic family lowering
- progression markers that distinguish composition-local symbolic references
  from resolved authoritative family members

### Must Preserve

- no dynamic transition registry in the core
- no hidden broad coordination for small fixed-arity operations
- no representation lies about cardinality or cost
- no symbolic family reference may masquerade as stable authority identity
- no same-family interaction path may depend on caller-owned ordering folklore
  once the family has entered lowered proof-bearing form

### Proof Obligations

- fixed-arity compile-time composition tests for representative fork/join
  shapes
- hostile cost-honesty tests proving small joins do not smuggle in broad
  generic containers or hidden allocation
- evidence that multi-artifact helpers preserve input/output cardinality
  explicitly rather than through erased collections
- proof that the API does not drift into a dynamic artifact graph engine
- compile-fail or type-shape tests proving invalid empty or wrong-cardinality
  forms do not type-check where fixed shape is required
- hostile same-family composition tests covering symbolic siblings, existing
  targets, identity-preserving rewrites, retirement, and supersession within
  one deterministic lowered family
- proof that composition-local symbols cannot flow into authority-identity APIs
  without explicit family resolution

### Migration Unlocks

- bespoke multi-input lowering glue
- bespoke split-plan progression wrappers
- repeated small fixed-arity join scaffolding in merge, bridge, and
  certification flows

## Milestone 7: Certification And Cross-Crate Migration Closure

### Goal

Prove that `worth-proof` is genuinely fit to replace bespoke progression
machinery in WORTH by closing hostile substrate certification and reference
migration lanes.

### Adversarial Constraint

The substrate must hold up under cross-crate migration pressure without hidden
runtime cost, semantic drift, stale-proof dishonesty, or loss of failure
richness on representative real WORTH flows.

### Must Ship

- hostile compile-fail suites for illegal progression, WORTHd minting, and
  stale-proof misuse
- exact counter or codegen-honesty proof points on representative hot paths
- reference migrations from at least one proof-heavy family in
  `worth-signal`, one in `worth-relational`, and one in `worth-query` or
  `worth-store`
- explicit migration guidance for when to use `worth-proof` versus
  `worth-foundational`
- closure criteria for retiring bespoke local progression scaffolding

### Must Preserve

- no regression in canonical semantics on migrated surfaces
- no hidden cost cliffs in migrated hot paths
- no expansion of `worth-proof` into diagnostics, lineage, or storage ownership

### Proof Obligations

- hostile compile-fail suites for illegal progression, stale misuse, WORTHd
  minting, and witness bypass
- exact counter, allocation, or codegen-honesty proof points on representative
  migrated hot paths
- cross-crate parity tests proving migrated proof-bearing surfaces preserve
  semantics and failure topology
- explicit residual-debt inventory for any remaining bespoke progression
  machinery left outside the shared substrate

### Migration Unlocks

- broad replacement of local progression boilerplate
- predictable proof-bearing API shape across WORTH
- faster new-subsystem authoring because the progression pattern becomes
  canonical rather than re-invented

## Outstanding Future Debt

The roadmap intentionally does not yet admit:

- a dynamic runtime transition registry for the core crate
- cross-language proof-substrate portability as a first-class milestone
- proof-owned diagnostics, provenance, or lineage schemas
- a generic artifact runtime or persistence system

Those concerns may be legitimate later, but admitting them now would blur the
crate boundary and compromise the zero-cost static core this roadmap exists to
establish first.
