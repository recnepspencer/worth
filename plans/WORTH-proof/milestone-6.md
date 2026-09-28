# Milestone 6 Engineering Spec: Static Fork And Join Progression

> **Status:** Closed
>
> **Roadmap parent:** [plans/worth-proof/worth_proof_roadmap.md](./WORTH_proof_roadmap.md)
>
> **Vision parent:** [plans/worth-proof/worth_proof_vision.md](./WORTH_proof_vision.md)
>
> **Test requirements:** [plans/worth-proof/test-requirements.md](./test-requirements.md)
>
> **Adjacent milestone:** [plans/worth-proof/milestone-5.md](./milestone-5.md)
>
> **Adjacent milestone closeout:** [plans/worth-proof/milestone-5-closeout.md](./milestone-5-closeout.md)
>
> **This milestone closeout:** [plans/worth-proof/milestone-6-closeout.md](./milestone-6-closeout.md)
>
> **Impacted later milestones:**
> - `Milestone 7` (`Certification And Cross-Crate Migration Closure`)
>
> **Primary architectural driver:** make fixed-arity multi-artifact progression and deterministic same-family composition canonical now that Milestone 5 has made lowered and execution-ready boundaries explicit

## Goal

Define the canonical static fork/join substrate for `worth-proof` so WORTH can
split one proof-bearing artifact into several artifacts, join several
proof-bearing inputs into one result, and lower one same-commit symbolic family
into one deterministic family program without drifting into a dynamic artifact
graph runtime or caller-owned ordering folklore.

## Why This Milestone Exists

Milestone 4 made transition and branching outcome law canonical.
Milestone 5 made lowered, ready, executed, and runtime-readmission boundaries
canonical.

That still leaves the next structural gap:

- WORTH repeatedly needs small fixed-arity multi-artifact progression
- many of those flows are not "a vector of things" but:
  - one authoritative artifact splitting into two or three proof-bearing outputs
  - two or three proof-bearing inputs joining into one lowered or admitted form
  - one same-commit family mixing symbolic siblings, existing targets,
    continuity-preserving rewrites, supersession, and retirement
- the hard cases are not just cardinality; they also require:
  - compile-time honest fixed shape
  - proof-preserving multi-input ordering
  - disjointness-sensitive composition
  - deterministic family lowering before one authority boundary closes

Without Milestone 6:

- domain crates will keep inventing bespoke split wrappers, join wrappers,
  sibling-resolution maps, and family-lowering packets
- same-family symbolic references will keep risking accidental treatment as
  stable identities
- multi-input lowering pressure will try to smuggle broad erased containers into
  the hot path
- Milestone 7 would have to certify migrations against several local
  composition dialects instead of one canonical substrate

Milestone 6 therefore exists to solve the next hard shared-law problem:

- how one proof-bearing thing forks into several proof-bearing outputs
- how several proof-bearing inputs join into one result without hiding
  cardinality or cost
- how same-family symbolic composition remains explicit and deterministic
- how family-local symbols become resolved authoritative family members only
  through one explicit family-resolution boundary

## Hard Part

The hard part is not adding a generic `Graph` or `Batch` abstraction.

The hard part is preserving all of these at once:

- fixed-arity honesty instead of broad erased collections
- proof-preserving composition across several inputs without weakening any prior
  progression law
- cardinality visibility at the API boundary
- deterministic same-family lowering when siblings can create, rewrite,
  supersede, retire, or target already-existing authoritative values
- symbolic family references that are useful during composition but impossible
  to confuse with stable authoritative identity
- zero-cost representative lanes that do not allocate or dispatch broadly just
  because more than one artifact is involved

The design fails if:

- `worth-proof` standardizes multi-artifact work around erased `Vec`-like
  containers by default
- fixed-arity joins conceal cost topology behind a generic batch surface
- composition-family lowering depends on caller-owned ordering folklore after
  entering lowered proof-bearing form
- symbolic sibling handles can cross into authority-identity APIs without
  explicit family resolution
- same-family composition pressure forces `worth-proof` into owning a generic
  runtime graph engine

## Explicit Assumptions

- Milestone 1 carrier law, Milestone 2 sealed minting, Milestone 3 freshness
  law, Milestone 4 transition law, and Milestone 5 lowered/ready/executed law
  remain authoritative.
- `worth-proof` still owns progression law only; it does not become a graph
  executor, receipt owner, symbolic storage system, or domain-specific merge
  engine.
- domain crates remain the semantic authorities for what a fork means, what a
  join means, what family continuity means, and what execution or publication
  semantics apply after the family is lowered.
- Milestone 6 may define fixed-arity composition carriers, family-local symbol
  stages, deterministic family-lowering surfaces, and checked multi-artifact
  progression contracts, but it must not smuggle in a dynamic orchestration
  runtime.
- Milestone 7 remains responsible for broad cross-crate migration closure, even
  though Milestone 6 must already expose a migration-worthy substrate shape.

## Governing Summaries

- `MENTALITY.md`
  The most important thing it protects here is solving the real adversarial
  composition problem before migration pressure hardens local dialects further.
  Milestone 6 therefore standardizes multi-artifact and same-family law now
  instead of certifying it later as an accidental byproduct.
- `arch_laws.md`
  The most important thing it protects is that proof-bearing progression,
  explicit equivalence/identity boundaries, lowered execution plans, and phase
  ordering remain encoded mechanically. Laws 24, 26, 27, 29, 30, 37, 40, and
  41 shape this milestone most strongly.
- `perf_laws.md`
  The most important thing it protects is cost honesty. Fixed-arity APIs must
  reveal cardinality and avoid broad container or graph-runtime drift, and
  expensive structural facts like disjointness or canonical family order must
  be carried forward instead of rediscovered repeatedly.
- `domain_laws.md`
  The most important thing it protects is responsibility clarity and future
  proofing. Fork/join carriers, family-symbol stages, deterministic lowering,
  and hostile certification need separate homes rather than one broad
  composition bucket.
- `worth_proof_vision.md`
  The most important thing it protects is `worth-proof` as a static progression
  substrate that owns fork/join law and deterministic composition-family law,
  but not a generic runtime graph engine or domain family semantics.
- `worth_proof_roadmap.md`
  The most important thing it protects is sequence integrity. Milestone 6 is
  the first milestone that widens progression into many-input and many-output
  law, and Milestone 7 depends on that law being canonical first.
- `worth-proof` test requirements
  The most important thing it protects is that multi-artifact and
  composition-family pressure must close through the named `Static Fork/Join
  And Composition Family Test`, including hostile symbolic/authoritative
  confusion lanes and cost-honesty proof.
- `milestone-5.md`
  The most important thing it protects is the canonical lowered-versus-ready
  boundary. Milestone 6 must compose on top of that boundary rather than
  weakening it whenever several artifacts participate.
- `milestone-5-closeout.md`
  The most important thing it protects is what later milestones may now safely
  assume: canonical lowered, ready, executed, and runtime-readmitted forms
  already exist, and execution-facing APIs already reject merely lowered forms.

## Adversarial Constraint

The milestone must survive the following hostile condition:

> Several WORTH subsystems with small fixed-arity forks, small fixed-arity
> joins, disjointness-sensitive composition, and same-commit symbolic sibling
> interaction must be able to express multi-artifact progression so that
> cardinality remains explicit, symbolic family references never masquerade as
> authoritative identity, deterministic family lowering does not depend on
> caller folklore, and representative static lanes remain allocation-free and
> virtual-dispatch-free.

The design fails if:

- a caller can satisfy a fixed-arity join API through a broad erased container
- symbolic family handles can be reused as if they were stable authority ids
- family-lowering order depends on arbitrary caller iteration order after
  lowered proof-bearing form exists
- representative fork/join helpers allocate or dispatch broadly when the shape
  is statically known
- Milestone 7 would need to reinterpret multi-artifact or family-lowering law
  before certifying real migrations

## Product Decision Lock

- fixed-arity multi-artifact progression is first-class and must stay
  cardinality-honest
- same-family composition-local symbols are temporary proof-bearing references,
  not authoritative identities
- deterministic same-family lowering is part of the proof substrate whenever
  sibling interaction would otherwise depend on caller folklore
- broad erased collections are not the normative API surface for representative
  fixed-arity composition
- static dispatch remains the normative posture for representative fork/join
  and family-lowering lanes
- `worth-proof` may standardize composition structure, but not domain family
  semantics or graph execution
- the public facade remains the only public entry surface

Normative consequence:

- any implementation that hides fixed cardinality behind broad collection APIs
  by default is out of spec
- any implementation that lets symbolic family references enter authority-id
  APIs without explicit resolution is out of spec
- any implementation that leaves deterministic same-family ordering to caller
  convention after lowered family form exists is out of spec
- any implementation that turns the milestone into a dynamic artifact graph
  runtime is out of spec

## Required Contracts

### Fixed-Arity Fork Rule

One proof-bearing input must be able to fork into several proof-bearing outputs
through an API that makes output cardinality explicit.

Required vocabulary:

- fixed-arity fork carrier or outcome
- fork transition or progression contract
- explicit output positions
- proof-preserving read-only access posture

Rules:

- representative fork surfaces must expose output cardinality in the type shape
- fork progression must preserve prior proof-bearing meaning rather than
  widening into raw payload collections
- fixed-arity fork APIs must not smuggle broad container semantics into small
  static lanes

### Fixed-Arity Join Rule

Several proof-bearing inputs must be able to join into one result through an
API that keeps input cardinality, input proof shape, and required disjointness
or canonicality explicit.

Required vocabulary:

- fixed-arity join carrier or contract
- explicit input positions
- disjointness-sensitive or canonical-order-sensitive join posture where
  applicable
- joined result posture

Rules:

- representative join surfaces must make the number of inputs explicit
- if disjointness or canonical order is required, that fact must remain in the
  type-level contract or proof-bearing input shape
- representative joins must reject wrong-cardinality or weak-shape inputs
  structurally

### Multi-Artifact Composition Ordering Rule

Composed multi-input progression must preserve proof order, non-success
short-circuiting, and readiness boundaries inherited from earlier milestones.

Required vocabulary:

- sequential multi-input composition posture
- early-terminal non-success posture
- lowered-versus-ready-sensitive composition posture where applicable

Rules:

- later composition steps must not run after an earlier non-success posture
- a multi-artifact helper must not let one input skip the proof-bearing stages
  still required of its siblings
- Milestone 5 lowered/ready boundaries remain authoritative even when several
  artifacts participate

### Composition-Family Symbol Rule

Same-family symbolic references must remain composition-local and must not be
confused with authoritative identity.

Required vocabulary:

- composition-local symbolic family reference
- resolved authoritative family member reference
- family-resolution transition
- family-local declaration posture

Rules:

- symbolic family references must be distinct types from authoritative family
  identity
- symbolic references may participate in family-local planning and lowering,
  but cannot cross authority-id APIs unchanged
- family resolution must be explicit and typed

### Deterministic Family Lowering Rule

When same-family siblings interact destructively or continuity-transformingly,
the family must lower into one deterministic proof-bearing family program before
authority closes.

Required vocabulary:

- family declaration posture
- lowered family program
- deterministic family order or equivalent canonical family sequencing surface
- same-family lifecycle categories for representative create, rewrite,
  supersede, and retire pressure

Rules:

- the lowered family form must be deterministic for semantically equivalent
  family inputs
- once lowered, family ordering must no longer depend on caller iteration
  folklore
- representative family-lowering surfaces must remain progression law only, not
  runtime graph execution

### Performance-Shaping Rule

Fork/join and family-lowering law must remain zero-cost honest for
representative static lanes.

Required performance-shaping surfaces:

- representative fixed-arity fork lane
- representative fixed-arity join lane
- representative deterministic family-lowering lane

Rules:

- representative fixed-arity lanes must not require mandatory allocation or
  virtual dispatch
- representative APIs must reveal cardinality and cost topology honestly
- expensive structural facts like disjointness or canonical family order must
  be carried forward rather than rediscovered later

## Scope

### In Scope

- explicit fixed-arity fork carriers or helpers
- explicit fixed-arity join carriers or helpers
- multi-input proof-preserving composition helpers
- family-local symbolic reference stages
- explicit family-resolution surfaces from symbolic to authoritative members
- deterministic lowered family program surfaces
- compile-fail support for wrong-cardinality, wrong-stage, and
  symbolic-versus-authoritative misuse
- facade hardening needed to keep internal family-lowering machinery private
- milestone-local certification notes that map directly onto the crate-level
  `Static Fork/Join And Composition Family Test`

### Explicitly Out Of Scope

- generic runtime graph execution
- receipt, diagnostics, provenance, or family-forensic artifact ownership
- broad cross-crate migration closure
- domain-specific merge semantics, planner semantics, or publication policy
- broad N-ary dynamic composition APIs as the normative first ship

## Phases

### Phase 1: Fixed-Arity Fork And Join Core

Define the representative fixed-arity carriers and contracts before same-family
symbolic composition is introduced.

Must ship:

- canonical small fixed-arity fork and join surfaces
- explicit input/output cardinality in the public type shape
- representative wrong-cardinality compile-time rejection posture
- representative proof-preserving read-only access posture

Implementation guidance:

- start with the smallest honest fixed-arity shapes WORTH actually needs
- prefer explicit positions over broad generic containers
- freeze the facade for the representative fixed-arity surfaces before moving
  on

### Phase 2: Multi-Artifact Composition Ordering

Extend the fixed-arity core into proof-preserving multi-input composition that
inherits Milestone 4 and 5 transition law honestly.

Must ship:

- representative sequential composition helpers for several inputs
- explicit early-terminal non-success behavior
- representative lowered-versus-ready-sensitive composition lane
- explicit preservation of required disjointness or canonical input facts where
  applicable

Implementation guidance:

- build on the existing transition grammar rather than inventing a parallel
  composition runtime
- prove ordering with narrow representative surfaces first
- do not broaden into generic N-ary orchestration just because the first helper
  works

### Phase 3: Composition-Family Symbol And Deterministic Lowering Law

Add the family-local symbolic stage model and one deterministic lowered family
program story.

Must ship:

- composition-local symbolic family reference types
- explicit authoritative family member reference types
- typed family-resolution boundary
- representative deterministic lowered family program
- representative family lifecycle pressure for create, rewrite, supersede, and
  retire interaction

Implementation guidance:

- make the symbolic-versus-authoritative distinction impossible to miss in the
  public API
- solve deterministic family lowering before any convenience helpers around it
- freeze the boundary so later migration work consumes one canonical family
  substrate

### Phase 4: Certification And Closure Surface

Close the milestone with the named suite, hostile cardinality pressure,
hostile symbolic/authoritative confusion pressure, and explicit residual debt.

Must ship:

- machine-checkable evidence for the `Static Fork/Join And Composition Family
  Test`
- compile-fail bundle for wrong-cardinality, wrong-stage, and family-identity
  misuse
- codegen-honesty report for representative fixed-arity and family-lowering
  lanes
- explicit residual debt report for anything intentionally left to Milestone 7

Implementation guidance:

- certify the smallest honest representative lanes rather than bloating the
  first suite into a fake generic migration harness
- require the evidence bundle to be usable directly by Milestone 7
- record any intentionally unshipped broader N-ary or migration pressure as
  explicit debt instead of implied completeness

## Acceptance Evidence

Milestone 6 is not complete until the named suite required by
[plans/worth-proof/test-requirements.md](./test-requirements.md)
passes with a machine-checkable certification bundle for:

- `transition_digest`
- `composition_digest`
- `proof_shape_digest`
- `failure_digest`
- `compile_fail_bundle`
- `compile_pass_bundle`
- `codegen_honesty_report`
- `residual_debt_report`

At minimum, the hostile closure surface must prove:

- wrong-cardinality forms do not type-check where fixed shape is required
- weak-shape inputs do not satisfy fixed-arity join contracts
- representative multi-input composition preserves non-success short-circuiting
- symbolic family references cannot enter authority-identity APIs unchanged
- deterministic same-family lowering converges for semantically equivalent
  family inputs
- representative fixed-arity and family-lowering lanes remain allocation-free
  and virtual-dispatch-free within the certified scope

## Why This Belongs Here

Milestone 6 belongs immediately after Milestone 5 because multi-artifact
progression can only be standardized honestly once the following are already
canonical:

- sealed proof minting
- freshness and readmission law
- typed transition and non-success topology
- lowered-versus-ready execution boundaries

If Milestone 6 were attempted earlier, it would either:

- reinvent those lower-layer laws locally inside composition helpers, or
- flatten multi-artifact progression into a weaker generic batch story

This milestone exists specifically to prevent that regression and to leave
Milestone 7 with one canonical substrate to certify against real WORTH
migrations.
