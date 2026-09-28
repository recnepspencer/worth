# Milestone 1 Engineering Spec: Core Artifact And Proof Substrate

> **Status:** Closed
>
> **Roadmap parent:** [worth_proof_roadmap.md](./WORTH_proof_roadmap.md)
>
> **Vision parent:** [worth_proof_vision.md](./WORTH_proof_vision.md)
>
> **Test requirements:** [test-requirements.md](./test-requirements.md)
>
> **Impacted later milestones:**
> - `Milestone 2` (`Sealed Minting And Witness Authority`)
> - `Milestone 3` (`Assumption, Freshness, And Staleness Law`)
> - `Milestone 4` (`Transition And Outcome Algebra`)
> - `Milestone 5` (`Lowering And Execution Readiness`)
> - `Milestone 6` (`Static Fork And Join Progression`)
>
> **Primary architectural driver:** freeze one zero-cost artifact grammar for
> proof-bearing progression before any sealing, staleness, transition, or
> execution-boundary work can reinterpret the core shape

## Goal

Establish the zero-cost core of `worth-proof`: phase-typed artifacts,
proof-bearing wrappers, proof-set composition, fixed-shape/proven-collection
helpers, and the minimal access law needed for later sealing, staleness,
transitions, lowering, and composition-family work to remain honest.

## Why This Milestone Exists

Everything later in `worth-proof` depends on whether the core artifact model is
strong enough to encode proof-bearing progression without cheating.

If Milestone 1 is weak:

- later milestones will bolt sealing, staleness, or transition law onto a
  substrate that still leaks raw payloads
- domain crates will keep local wrapper types because the shared core will not
  express their real proof surfaces
- collection facts such as canonical order, uniqueness, or non-emptiness will
  continue to be re-proven ad hoc
- future runtime bridges will be tempted to introduce erased or dynamic proof
  bags because the static core was never made expressive enough

This milestone therefore exists to solve the hard substrate problem first:

- zero-cost phase and proof encoding
- zero-cost room for assumption state
- room for proof-bearing structural facts
- room for fixed-shape and proven-collection forms
- access patterns that preserve proof honesty without smuggling in framework
  baggage

Milestone 1 does **not** try to finish the whole crate. It creates the core
type law that later milestones can safely harden.

## Hard Part

The hard part is not making a nice generic wrapper.

The hard part is keeping five things separate that weak proof substrates
collapse together:

- the payload itself
- the phase in which the payload is legally consumable
- the proof facts already established about that payload
- the assumption basis under which those proof facts remain valid
- the structural and cardinality facts that later phases should consume without
  rediscovering them

The design fails if:

- phase is mechanically present but not actually load-bearing for API
  admissibility
- proof state can be replaced by ambient comments, booleans, or side tables
- assumption state exists only as an erased metadata bag
- structural facts such as canonical order or uniqueness still have to be
  rediscovered from raw collections
- fixed-shape forms immediately collapse back into `Vec` or slice conventions
  at the first nontrivial API boundary
- the only way to extend the substrate for sealing, staleness, or transitions
  later is to replace the core wrapper model entirely

Milestone 1 therefore has to freeze one exact core artifact law before later
milestones start depending on it.

## Explicit Assumptions

- `worth-proof` remains a static-first proof substrate, not a dynamic runtime
  registry or graph engine.
- `worth-foundational` will later own diagnostics, lineage, provenance,
  receipts, digests, aspect vocabulary, and other descriptive truth surfaces.
- domain crates will continue to own payload semantics, cost topology, data
  layout, runtime execution, and domain-specific proof meaning.
- later milestones will add sealing, witness authority, staleness, transition
  algebra, and execution-readiness law on top of the artifact substrate frozen
  here rather than redefining the core artifact shape.
- proof-bearing collections and fixed-shape wrappers belong in this milestone
  only when the carried property materially changes correctness or later proof
  consumption.

## Governing Summaries

- `MENTALITY.md`
  The most important thing it protects is solving the adversarial constraint up
  front instead of retrofitting proof law after helper APIs already spread. The
  spec therefore starts with the zero-cost artifact core rather than with
  convenience transitions.
- `arch_laws.md`
  The most important thing it protects here is that proof-bearing progression
  must be carried by types, not by conventions. Laws 30, 37, 39, 40, and 41 are
  especially load-bearing for this milestone.
- `perf_laws.md`
  The most important thing it protects is cost honesty. The core substrate may
  not hide allocation, dynamic lookup, erased dispatch, or fake collection
  uniformity behind a nice generic surface.
- `domain_laws.md`
  The most important thing it protects is responsibility separation and
  decomposition. `worth-proof` must own progression law only; it must not start
  absorbing diagnostics, provenance, storage, or runtime semantics.
- `worth_proof_vision.md`
  The most important thing it protects is the identity of the crate: a
  compile-time law layer for proof-bearing progression that stays static-first,
  zero-cost, and honest about trust-boundary, symbolic/resolved, and
  composition-family distinctions.
- `worth_proof_roadmap.md`
  The most important thing it protects is sequencing. Milestone 1 must create
  the core artifact and proof substrate that every later milestone depends on,
  including sealing, staleness, transition algebra, lowering readiness, and
  same-family composition helpers.
- `worth-proof` test requirements
  The most important thing it protects is that proof-substrate closure is
  certification-grade rather than smoke-test-grade. Milestone 1 therefore has
  to satisfy the named `Core Artifact And Proof Substrate Test` and the
  cross-milestone compile-time and zero-cost gates rather than relying only on
  local milestone notes.

## Adversarial Constraint

The milestone must survive the following hostile condition:

> Several WORTH subsystems with different payload shapes, proof sets, trust
> bases, collection facts, and fixed-cardinality requirements must be able to
> encode proof-bearing core artifacts such that phase and proof progression are
> statically visible, later phases cannot accidentally consume weaker forms, and
> the compiled code introduces no material allocation, lookup, or dispatch
> overhead relative to bespoke handwritten wrappers.

The design fails if:

- the core artifact form requires runtime proof lookup for statically known
  phases or proofs
- structural facts such as canonical order or uniqueness still require ambient
  side channels rather than traveling with the type
- fixed-shape and proven-collection cases are forced back into raw `Vec` plus
  comments
- the only way to carry assumption state is a generic metadata blob
- collection/proof helpers hide meaningful cost distinctions
- later milestones would need to replace the core wrapper model rather than
  extend it

## Product Decision Lock

- the milestone ships one canonical artifact wrapper family for proof-bearing
  progression; first ship does not admit several competing core artifact
  stories
- phase and proof state are represented by compile-time structure on statically
  known hot paths
- assumption scope is explicit and typed; it may not be replaced by one generic
  metadata map
- proven structural facts are first-class proof-bearing forms when later phases
  need them, not comments about raw collections
- fixed-shape correctness may not be expressed as "vector plus convention" on
  any admitted core surface where cardinality materially matters
- public hot-path APIs do not accept raw weaker forms when a proven collection
  or fixed-shape form is the real requirement
- the milestone may be conservative about the number of built-in structural
  fact and fixed-shape helpers, but every shipped helper must preserve
  correctness and cost honesty mechanically
- runtime-string phase identifiers, runtime-string proof identifiers, dynamic
  proof maps, and boxed proof registries are out of spec for the core
  substrate

Normative consequence:

- any implementation that represents statically knowable phase or proof state
  primarily through runtime tags is out of spec
- any implementation that relies on ambient side tables to recover structural
  facts already claimed as proven is out of spec
- any implementation that exposes raw mutable payload escape hatches that skip
  the proof-bearing surface is out of spec

## Required Contracts

### Core Artifact Authority Rule

The artifact wrapper is the only admitted first-ship carrier for proof-bearing
phase progression in the crate.

Required vocabulary:

- `Artifact`
- `ArtifactView` or equivalent read-only access surface
- phase marker family
- proof marker family
- assumption-scope carrier

Rules:

- artifact access may expose payload reads without weakening the carried phase
  or proof law
- artifact construction may not require runtime lookup to discover statically
  known phase or proof state
- artifact identity in this milestone is progression identity only; it does not
  absorb diagnostics, provenance, or receipt meaning

### Proof-Set Composition Rule

The substrate must define one canonical story for carrying several independent
proof facts together.

Required vocabulary:

- empty-proof or no-proof surface
- single-proof surface
- proof-composition surface
- proof-widening surface

Rules:

- later phases may require several proof facts without forcing ad hoc tuple
  conventions in domain crates
- proof composition may not hide dynamic proof lookup or unordered runtime bag
  semantics

### Structural Fact Carry Rule

Expensive structural facts may survive as proof-bearing forms when later phases
need to consume them directly.

Required first-ship structural-fact vocabulary:

- canonical order
- uniqueness
- disjointness
- normalization

Rules:

- structural facts must travel with the type or a proof-bearing wrapper, not
  through ambient booleans, comments, or side channels
- first ship may mark some structural fact families as explicit `Debt`, but
  only if the omission is named and no weaker fake generic placeholder is
  shipped instead

### Fixed-Shape And Proven-Collection Rule

Collection shape and cardinality must remain explicit wherever they materially
change correctness or cost topology.

Required first-ship vocabulary:

- `NonEmpty`
- `Pair`
- `DisjointPair`
- exactly-one surface
- proven-collection surfaces for canonical order and uniqueness

Rules:

- fixed-shape APIs may not accept raw `Vec` or slice input when cardinality is
  part of the contract
- proven-collection APIs may not silently recover their proof from raw
  collection inspection on the hot path
- helper surfaces must remain representation-honest; one generic collection
  helper may not pretend all cost topologies are the same

### Compile-Time Boundary Rule

The highest-risk progression boundaries in this milestone must be compiler
enforced rather than doc-only.

Required compile-time posture:

- later-phase-only APIs reject earlier-phase artifacts
- proven-collection-only APIs reject raw collections
- fixed-shape-only APIs reject unconstrained generic collections
- construction surfaces do not allow ordinary callers to synthesize stronger
  proof-bearing shapes by convention

### Performance-Shaping Types Rule

The milestone must encode the core cost decisions into the type surface so
later execution does not rediscover them procedurally.

Required performance-shaping surfaces:

- proof-bearing artifact wrapper family
- fixed-shape wrapper family
- proven-collection wrapper family
- structural-fact wrapper family

Rules:

- static proof-bearing access may not allocate by default
- composition of phase, proof, and assumption state may not require virtual
  dispatch
- representative wrapper paths must remain codegen-honest enough that the
  resulting machine code shape is materially comparable to bespoke handwritten
  wrappers

## Scope

### In Scope

- phase marker vocabulary for statically known progression states
- core proof marker vocabulary and proof-set composition model
- artifact wrapper forms that can carry phase, payload, proof, and assumption
  state
- zero-cost room for proof-bearing structural facts such as canonical order,
  uniqueness, disjointness, or normalization
- proof-carrying collection helpers where the proven collection property is the
  real law
- fixed-shape/cardinality helpers for small forms such as exactly-one,
  non-empty, pair, and disjoint pair where those distinctions materially matter
- access surfaces that preserve proof honesty and do not silently downgrade to
  raw mutable payload handling
- compile-fail and codegen-sensitive certification for the core shapes
- module topology for the core substrate

### Explicitly Out Of Scope

- sealed proof minting and witness authority beyond the minimal shape required
  for the core artifact model
- staleness/rebind/re-admission law beyond the artifact shape hooks that later
  milestones will harden
- transition traits, branching outcome algebra, or lowering/execution readiness
- diagnostics, lineage, provenance, digests, receipts, or boundary explanation
  surfaces
- generic runtime registries, dynamic proof bags, or dynamic transition graphs
- domain-specific semantics for any proof, payload, collection, or recipe shape

## Phases

### Phase 1: Core Marker And Artifact Law

Define the minimum artifact grammar that later proof-bearing progression will
stand on.

Must ship:

- phase marker patterns for statically known progression states
- proof marker patterns for statically known proof facts
- an artifact core shape that can carry:
  - payload
  - phase
  - proof state
  - assumption state
- read-only access law that preserves proof honesty
- explicit rejection of runtime-string phase/proof identifiers on the hot path

Implementation guidance:

- create `artifact/carrier.rs` first and freeze one minimal `Artifact` shape before
  adding helper wrappers
- create `phase/markers.rs` and `proof/markers.rs` as independent marker
  modules rather than embedding phase or proof definitions inside the artifact
  wrapper itself
- define the assumption carrier in `assumption/basis.rs` during this phase
  even if first ship keeps it austere; Phase 2 and Phase 3 must not have to
  invent a second basis-carrying story later
- define one read-only view or borrowing surface early enough that later phases
  are forced to build on it instead of exposing the raw payload directly
- do not start with mutable access helpers; first ship should prove the read and
  ownership story before exploring mutation posture

This phase is complete only when later phases can talk about "an artifact that
is in phase P with proof set Q under basis A" without inventing a second wrapper
model.

### Phase 2: Proof-Set And Structural Fact Substrate

Extend the core shape so it can honestly express the nontrivial proof facts
WORTH repeatedly carries across phases.

Must ship:

- proof-set composition primitives strong enough for:
  - one proof
  - several independent proofs
  - proof-bearing widening
- room for proof-bearing structural facts such as:
  - canonical order
  - uniqueness
  - disjointness
  - normalization
- proof-carrying collection helpers where the collection property itself is the
  important law
- fixed-shape/cardinality wrappers for the small forms that later milestones
  need to compose honestly

Implementation guidance:

- implement proof-set composition before collection helpers so the collection
  wrappers can reuse the same proof story rather than inventing local marker
  conventions
- ship the smallest structurally honest built-ins first:
  - canonical-order-bearing collection
  - uniqueness-bearing collection
  - `NonEmpty`
  - exactly-one
  - `Pair`
  - `DisjointPair`
- treat disjointness and normalization as first-class design targets even if a
  subset lands as explicit `Debt`; do not ship placeholder wrappers that claim
  to be generic enough for them without actually carrying the fact
- keep fixed-shape helpers in `collections/*` and structural-fact markers in
  `proof/structural_facts.rs`; do not collapse them into one mixed helper file

This phase is complete only when later progression surfaces can consume proven
collections and fixed-shape forms directly rather than falling back to raw
collections plus comments.

### Phase 3: Zero-Cost Construction And Access Boundaries

Harden the core artifact surface so it can be used safely without accidental
raw escape hatches, while still staying lightweight enough for broad adoption.

Must ship:

- constructor/access patterns that preserve proof honesty
- ownership and borrowing surfaces that do not force artificial clones or
  hidden allocations
- explicit crate-internal/public boundary guidance for the core substrate
- a core topology that later sealing and witness milestones can extend without
  replacing
- no ambient metadata channels for carrying proof or assumption state

Implementation guidance:

- freeze the initial public surface through `facade.rs` during this phase
  instead of exposing deep modules directly and cleaning it up later
- define which constructors stay crate-private now, even if full sealed minting
  waits for Milestone 2; later privacy hardening should refine the boundary, not
  discover it
- add the ownership and borrowing paths that representative later milestones
  will rely on:
  - owned artifact construction
  - borrowed read access
  - destructuring or extraction paths that preserve explicit phase/proof shape
- if any helper requires clone-heavy ergonomics to feel usable, treat that as a
  design defect in this phase rather than pushing it into Phase 4 certification

This phase is complete only when the core artifact model is stable enough that
Milestone 2 can add sealing and witnesses on top of it rather than redesigning
it.

### Phase 4: Compile-Fail And Codegen Certification

Prove that the core substrate is mechanically honest.

Must ship:

- compile-fail coverage for the most important illegal progression shapes
- compile-fail coverage for invalid fixed-shape or proven-collection misuse
- exact codegen/overhead checks on representative wrapper patterns where
  feasible
- explicit complexity/codegen honesty notes where the milestone still carries
  debt
- a closure record of what later milestones may assume about the artifact core

Implementation guidance:

- certify one representative lane per core family rather than one generic smoke
  test for the whole crate:
  - plain phase/proof artifact
  - artifact with assumption carrier
  - proven-collection wrapper
  - fixed-shape wrapper
- the compile-fail bundle should map back to the contract sections above so
  later milestones can tell exactly which boundary is already proven
- the codegen report should compare against one or more intentionally plain
  handwritten wrapper baselines, not just inspect the shared crate in isolation
- write the closure record in a way Milestone 2 can consume directly: what is
  frozen, what remains debt, and what later milestones are forbidden to
  reinterpret

This phase is complete only when the milestone has machine-checkable evidence
that it is zero-cost in the ways the roadmap claims, not merely plausible by
inspection.

## Must Ship

- one canonical core artifact wrapper family for phase/proof/assumption-carrying
  values
- one canonical proof marker and proof-set composition story for the static
  core
- one canonical way to express proof-bearing structural facts and proven
  collections
- one canonical way to express fixed-shape/cardinality-bearing forms where
  cardinality is part of correctness
- compile-fail coverage proving obvious weaker-form misuse is uncallable
- explicit codegen/overhead evidence that the core shapes do not introduce
  hidden dynamic machinery
- a module topology that later milestones can extend without replacing the
  substrate
- milestone-local implementation notes that map directly onto the crate-level
  certification bar in `worth-proof/test-requirements.md`

## Must Preserve

- zero-cost hot-path posture after monomorphization
- no mandatory heap allocation
- no mandatory dynamic dispatch
- no mandatory runtime proof lookup
- no generic metadata bag standing in for typed proof or assumption state
- no collapse of fixed-shape law into raw collections
- no collapse of structural facts into comments or side-channel booleans
- no drift of `worth-proof` into diagnostics, provenance, digests, or runtime
  semantics
- clean extension room for sealing, witnesses, staleness law, transition
  algebra, and composition-family work in later milestones

## Acceptance Evidence

Milestone 1 is complete only when `worth-proof` satisfies the named milestone
suite:

- `Core Artifact And Proof Substrate Test`

Required machine-checkable outputs:

- `type_shape_report`
- `compile_fail_bundle`
- `proof_shape_digest`
- `basis_digest`
- `codegen_honesty_report`
- `debt_inventory`

Milestone-specific proof obligations:

- later-phase operations are not callable on earlier-phase artifacts
- fixed-shape forms reject obviously wrong cardinality at the type/API level
- proven-collection-only APIs do not accept raw collections directly
- structural facts travel with the type rather than through ambient side state
- proof-set composition does not force hidden dynamic proof lookup or boxed
  runtime storage
- representative wrapper patterns do not introduce mandatory allocation or
  virtual dispatch
- later milestones can layer sealing and witness law onto the same core
  artifact model without replacement
- the suite includes compile-fail tests for illegal progression and misuse of
  fixed-shape and proven-collection forms
- the suite includes hostile type-shape tests proving structural facts travel
  with the type rather than through ambient side state
- the suite includes representative codegen/overhead checks showing:
  - no hidden dynamic proof map
  - no hidden virtual dispatch
  - no mandatory allocation introduced by the core wrapper patterns
- the suite includes representative ergonomics proof that later milestones can
  layer sealing and witness law onto the same core artifact model without
  replacement
- the suite publishes an explicit debt list for any proof-bearing collection or
  fixed-shape helper that is intentionally deferred

Milestone 1 is not closed by "the wrappers compiled" or "the generics look
zero-cost by inspection" arguments.

## Architectural Notes

- Milestone 1 is intentionally austere. It should solve the core artifact law,
  not try to deliver a full public DSL for every later progression pattern.
- The artifact core must be generic enough to host later law, but not so
  generic that it introduces erased runtime machinery.
- Proven structural facts belong here only when later phases genuinely need to
  consume them without re-proof. This milestone must not turn into a bag of
  ornamental wrappers.
- Fixed-shape helpers belong here only where cardinality changes correctness or
  cost honesty. "Wrapper because maybe useful" is out of scope.

## Implementation Topology

This milestone should create a decomposed core topology rather than one large
`artifact.rs` or `proof.rs` file that later milestones will have to unwind.

Preferred initial shape:

```text
crates/worth-proof/src/
  lib.rs
  facade.rs
  artifact/
    mod.rs
    carrier.rs
    access.rs
    constructors.rs
  phase/
    mod.rs
    markers.rs
  proof/
    mod.rs
    markers.rs
    sets.rs
    structural_facts.rs
  assumption/
    mod.rs
    basis.rs
    freshness.rs
  collections/
    mod.rs
    non_empty.rs
    exactly_one.rs
    pair.rs
    disjoint_pair.rs
```

This is not a forced final topology, but the ownership boundaries are
intentional:

- `artifact/*`
  - core artifact shape, access law, and construction posture
- `phase/*`
  - phase marker vocabulary
- `proof/*`
  - proof markers, proof-set composition, structural fact markers
- `assumption/*`
  - basis and future freshness vocabulary
- `collections/*`
  - fixed-shape helpers in Milestone 1 and later proven-collection helpers
- `lib.rs`
  - crate bootstrap only
- `facade.rs`
  - the only public crate entry surface

The milestone should avoid:

- one giant catch-all `carrier.rs` that owns phases, proofs, collections, and
  helpers together
- one catch-all `helpers.rs`
- public exposure of deep internal module structure before the facade is chosen

## Sequencing Notes

- This milestone belongs first because every other roadmap item assumes there is
  already one zero-cost core artifact model worth extending.
- Sealed minting belongs after this milestone because sealing weak wrappers does
  not help if the wrappers themselves are still the wrong shape.
- Staleness/trust-boundary law belongs after this milestone because those later
  transitions depend on a stable phase/proof/assumption carrier.
- Transition algebra belongs after this milestone because transitions need a
  trustworthy input/output artifact grammar first.
- Same-family symbolic composition remains later because it should reuse the
  fixed-shape, collection, and proof-bearing core rather than force Milestone 1
  to overfit to one downstream pressure case.

## Closure Record

Milestone 1 is closed.

### Frozen Surfaces

The following surfaces are now the admitted Milestone 1 core and later
milestones must extend them rather than replace them:

- `Artifact<P, T, S = NoProofs, A = NoAssumptionBasis>`
- `ArtifactView`
- `ArtifactParts`
- `Proof<P>`
- `NoProofs`
- `ProofSetCons`
- `AssumptionBasis<B>`
- `NoAssumptionBasis`
- structural fact markers:
  - `CanonicalOrder`
  - `Uniqueness`
  - `Disjointness`
  - `Normalization`
- fixed-shape and proven-collection forms:
  - `ExactlyOne`
  - `NonEmpty`
  - `Pair`
  - `DisjointPair`
  - `CanonicalVec`
  - `UniqueVec`
- the single public crate boundary through `facade.rs`

Normative consequence:

- Milestone 2 may harden minting and witness authority on top of these shapes
  but must not invent a second core artifact carrier model
- Milestone 3 may add freshness and downgrade law on top of the admitted basis
  carrier but must not replace the basis-carrying story
- later milestones may add richer proof-bearing forms, but they must preserve
  the same static-first, no-runtime-proof-map posture already frozen here

### Machine-Checkable Evidence

Milestone 1 closure is backed by the named suite:

- `Core Artifact And Proof Substrate Test`

Implemented evidence outputs:

- `type_shape_report`
- `compile_fail_bundle`
- `proof_shape_digest`
- `basis_digest`
- `codegen_honesty_report`
- `debt_inventory`

The current certification tree proves:

- phase-mismatched artifact use is uncallable
- raw collections do not satisfy proven-collection-only APIs
- raw wrong-cardinality forms do not satisfy fixed-shape APIs
- stronger proof-bearing constructors are not public
- compile-fail family selection fails closed
- representative artifact, basis-bearing, view, parts, proven-collection, and
  fixed-shape lanes remain size/alignment/drop-honest for the admitted scope

### Explicit Debt

Milestone 1 intentionally leaves the following debt visible:

- no handwritten MIR/ASM baseline diff yet; current codegen certification is
  representative size/alignment/drop honesty rather than full machine-code
  equivalence
- `assumption/freshness.rs` remains deferred until Milestone 3 makes freshness,
  re-admission, and downgrade law load-bearing
- `Proof<P>` is not yet sealed against external construction; Milestone 2 owns
  sealed minting and witness authority hardening

These are explicit debt items, not hidden omissions.

### What Later Milestones May Assume

Later milestones may assume:

- the public core carrier family exists and is stable enough to build on
- explicit owned extraction preserves proof shape rather than silently
  collapsing to payload-only forms
- the compile-fail harness is fail-closed for family selection
- representative proof-shape and basis digests are derived from actual
  representative types rather than handwritten labels
- the support tree is already decomposed by proof responsibility and may grow by
  responsibility rather than by convenience bucket

### What Later Milestones Must Not Reinterpret

Later milestones must not:

- reintroduce raw payload or raw collection convenience paths on surfaces where
  proof-bearing or fixed-shape law is the actual contract
- replace typed basis carriage with one generic metadata bag
- introduce runtime-string phase/proof identity into static hot-path
  progression
- treat Phase 1 codegen evidence as proof of full handwritten machine-code
  parity
- claim Milestone 1 sealed minting already exists

### Closure Verification

Closure verification at the time of closeout:

- `cargo fmt --all`
- `cargo test -p worth-proof`
- hostile implementation QA loop completed with no meaningful findings
- hostile `qa-tests` pass completed with no meaningful findings
