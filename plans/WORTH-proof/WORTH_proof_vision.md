# WORTH Proof Vision

## Thesis

`worth-proof` exists to make proof-bearing lifecycle progression a reusable,
zero-cost substrate across WORTH.

WORTH already relies on phased progression everywhere:

- raw -> validated -> lowered -> bound
- declaration -> admitted -> certified
- preview -> authority-ready -> executed
- unresolved -> scoped -> lowered -> replay-safe

Today those laws are repeatedly rediscovered inside domain crates as bespoke
typestate wrappers, sealed constructors, lowering artifacts, validation tokens,
staleness markers, and proof-bearing reports. That repetition is not merely
duplicate code. It is duplicate architectural decision-making around one of the
most important invariants in the codebase:

- what has been proven
- who is allowed to prove it
- what may consume the proof
- when the proof becomes stale
- where lowering must stop and execution may begin

`worth-proof` is the static substrate for encoding those invariants once.

It is not a runtime workflow engine.
It is not a dynamic plugin registry.
It is not an artifact database.
It is not a diagnostics framework.
It is not a lineage store.

It is the compile-time law layer that lets WORTH express proof-bearing
progression mechanically, consistently, and at zero runtime cost.

## What This Crate Is For

`worth-proof` exists for every WORTH subsystem that needs to represent:

- phased artifacts
- proof-bearing transitions
- sealed construction boundaries
- symbolic-to-resolved progression
- unresolved-versus-resolved distinctions
- trust-boundary admission and re-admission
- deterministic composition-local sibling families
- stale or assumption-scoped validity
- lowered-vs-executable plan separation
- typed denial paths
- fork/join progression across multiple artifacts
- authority witnesses and capability witnesses
- proof-carrying collections and fixed-shape inputs

It is meant to support:

- `worth-relational` commit, merge, lineage, schema, and validation phase
  progression
- `worth-query` workflow lowering, live promotion, policy-aware narrowing,
  bridge lowering, and branch/history proof surfaces
- `worth-signal` planner, reuse, resource policy lowering, runtime proof
  admission, and replay-safe artifact progression
- `worth-store` compatibility admission, retention planning, support artifact
  proof chains, and maintenance execution readiness

The technical thesis is the same across all of them:

- valid progression should be encoded in types
- proof minting should be sealed
- stale-proof boundaries should be explicit
- execution should consume lowered proof-bearing forms rather than rediscover
  their legality
- domain crates should own domain semantics, not generic progression boilerplate
- runtime-executed plans must still be representable as proof-bearing
  progression without forcing a generic execution runtime into the crate
- unresolved, symbolic, derived, and authoritative forms should be distinct
  where confusing them would create correctness or staleness hazards
- trust-boundary crossings should be explicit progression events, not ambient
  assumptions
- composition-local symbolic siblings should lower into one deterministic family
  program before authority executes or publishes anything

## Mission

`worth-proof` exists to let WORTH express the strongest possible progression
laws without paying for them on the hot path.

It must answer these questions as native crate responsibilities:

- How does a type state exactly what has been proven about a value?
- How does a transition consume one proof-bearing form and produce a stronger
  one without letting callers WORTH the output?
- How does a subsystem represent proofs that are only valid under a specific
  schema version, branch epoch, policy basis, or authority digest?
- How does a crate distinguish symbolic references from resolved bindings, or
  unresolved recipes from admitted executable recipes, without ambient
  conventions?
- How does a crate re-admit a value after serialization, restore, boundary
  transport, or other trust-boundary crossing without pretending the original
  proof still holds unchanged?
- How does a crate express one same-commit family where symbolic siblings,
  existing authoritative targets, identity-preserving rewrites, supersession,
  retirement, and follow-up mutation interact deterministically before one
  coherent visibility boundary closes?
- How does a crate model denial, explicit rebind required, stale-but-readable,
  and authority-validation-required outcomes without collapsing them into
  untyped booleans?
- How does lowering remain a pure proof-bearing phase boundary rather than an
  ad hoc convention?
- How do multi-artifact transitions preserve proof honesty across fork and join
  operations?
- How do collections carry proven facts such as uniqueness, canonical order,
  disjointness, or acyclicity so later phases stop re-proving them?
- How does a previously proven form downgrade, invalidate, or lose admission
  honestly when its basis expires or its trust boundary changes?
- How do WORTH crates standardize progression mechanics while preserving their
  own cost model, data layout, and semantic meaning?

## Architectural Model

### Runtime stack

| Layer | Responsibility | Owns |
| --- | --- | --- |
| `worth-proof` | Proof-bearing progression law | phases, proof composition, witnesses, transitions, stale/assumption encoding |
| `worth-foundational` | Shared truth vocabulary | aspects, diagnostics, lineage, provenance, digests, reports, performance contracts |
| Domain crates | Domain semantics and execution | truth semantics, storage layout, planning semantics, runtime behavior |

### Ownership boundary

`worth-proof` owns:

- phase-typed artifact progression
- proof markers and proof-set composition
- proof-carrying collection and fixed-shape progression helpers
- sealed proof minting patterns
- transition traits and transition outcome patterns
- assumption-scoped and freshness-scoped proof validity
- trust-boundary admission and re-admission progression markers
- branchable progression outcomes such as admitted, denied, stale, or rebind
  required
- static fork/join progression helpers
- symbolic-to-resolved and unresolved-to-resolved progression markers
- deterministic composition-family progression markers and sibling-family
  helpers
- pure-lowering versus execution-boundary marker types
- executor-readiness proof boundaries for plans that are lowered statically or
  semantically, but only applied at runtime by crate-local executors
- executed-form progression hooks for domains that need the post-execution
  state itself to remain proof-bearing, while leaving receipts and descriptive
  forensic surfaces outside the crate
- authority-versus-derived proof-carrying wrappers where progression law must
  enforce that distinction
- downgrade/invalidation progression helpers where a previously stronger form
  must become stale, unresolved, rebind-required, or otherwise weaker again

`worth-proof` does not own:

- diagnostics schemas or storage
- lineage or provenance record storage
- report, summary, or artifact vocabulary outside progression mechanics
- dynamic plugin registration
- runtime transition graph execution
- generic plan execution or orchestration runtimes
- container layout policy
- domain semantics
- persistence, replay storage, or transport protocols

Structural rule:

`worth-proof` standardizes what it means for progression to be legal. Domain
crates still decide what the progression means. In particular, `worth-proof`
owns authority law and admissibility; `worth-foundational` owns descriptive
provenance, receipts, and boundary explanation of those authority paths.

## Adversarial Constraint

`worth-proof` must survive the following hostile condition:

> Any WORTH subsystem with proof-bearing progression, stale-assumption
> boundaries, mixed denial paths, and high-performance execution requirements
> must be able to encode its valid transitions so that illegal progression is
> unrepresentable, stale proof use is mechanically visible, and the resulting
> machine code introduces no material hot-path overhead relative to bespoke
> handwritten domain code.

If `worth-proof`:

- requires dynamic proof lookup in hot paths
- forces runtime phase tags where the domain already knows the phase statically
- hides cost differences between distinct execution surfaces
- erases failure topology into generic runtime errors
- makes crates pay for lineage, diagnostics, or provenance they do not need on
  a path
- prevents domains from expressing branch-specific or assumption-scoped proof
  invalidation honestly
- lets trust-boundary crossings silently preserve proofs that should have been
  re-admitted
- standardizes storage layout rather than progression law

then it has failed.

## Why This Crate Is Different

These are not optional niceties. They are the properties that make
`worth-proof` strategically different from ordinary typestate helpers:

- zero-cost phase encoding
- proof-set composition rather than one-off marker wrappers
- proof-carrying collections rather than repeated collection re-validation
- canonicalization, uniqueness, disjointness, and similar expensive facts can
  survive as proof-bearing forms instead of being rediscovered repeatedly
- assumption-scoped proof validity
- explicit stale / rebind / authority-required outcomes
- sealed proof minting surfaces
- multi-artifact fork and join progression
- pure lowering as a first-class proof boundary
- execution readiness as an explicit proof-bearing form
- denial and report topology that remains typed rather than string-driven
- witness-based authority and capability encoding
- symbolic-to-resolved recipe progression as a shared law rather than bespoke
  local scaffolding
- deterministic same-family symbolic interaction as a shared law rather than
  ad hoc batch folklore

If these are treated as optional extras, WORTH falls back into bespoke
proof-bearing APIs whose semantics are individually honest but collectively
inconsistent.

## Principles

1. Zero runtime cost is non-negotiable.
2. Power must come from compile-time structure, not runtime machinery.
3. Proof-bearing types must encode what has been established, not just what
   payload they contain.
4. Only trusted constructors may mint stronger proof-bearing forms.
5. Staleness, authority gaps, and rebind requirements are first-class
   progression states, not ad hoc side channels.
6. Lowering is a proof-bearing phase boundary and must remain distinct from
   effectful execution.
7. Shared progression law must not erase cost topology, failure topology, or
   correctness topology.
8. Domain crates keep semantic ownership; `worth-proof` owns only progression
   law.
9. Multi-artifact transitions are part of the model, but they must remain
   statically expressible before any dynamic generalization is considered.
10. The crate must help WORTH remove boilerplate without standardizing away real
    differences between domains.
11. Runtime-tagged or erased progression forms are boundary adapters only, not
    the core hot-path substrate.
12. Whenever unresolved, symbolic, derived, authoritative, canonicalized, or
    resolved forms carry different guarantees, they must be distinct types.
13. Crossing an untrusted or differently trusted boundary is a progression
    event. Re-admission is never ambient.
14. Losing a proof is as structurally important as gaining one; downgrade and
    invalidation paths must be representable honestly.
15. Composition-local symbols are temporary proof-bearing references, not
    authoritative identities, and must collapse through one deterministic
    family-lowering boundary.

## Foundational Decisions

These are locked architectural decisions:

- the crate is static-first and compile-time-first
- hot-path APIs must not require dynamic registries
- phases are represented by types, not strings, whenever the phase is known
  statically
- proofs are represented by marker or witness types, not generic runtime proof
  bags, in the core substrate
- proof minting requires sealed construction or equivalent compiler-enforced
  boundaries
- transitions are expressed as typed transformations with typed failure
  topologies
- stale or assumption-scoped validity is modeled explicitly rather than treated
  as undocumented invalidation folklore
- trust-boundary re-admission is modeled explicitly rather than smuggled
  through serialization, restore, or adapter surfaces
- fork and join support is allowed, but only when it remains cost-honest and
  mechanically explicit
- fixed-shape and proof-carrying collection helpers are allowed where they
  eliminate repeated proof work without hiding collection cost topology
- recipe progression from symbolic intent to resolved/executable form is a
  first-class substrate concern
- expensive structural facts such as canonical order, uniqueness, disjointness,
  or normalization may be represented as proof-bearing forms
- same-family symbolic sibling interaction may be represented as a proof-bearing
  deterministic composition program, but generic execution of that family
  remains domain-owned
- diagnostics, lineage, provenance, canonical digest surfaces, and effect log
  schemas live outside this crate
- no crate may be forced into one artifact storage representation to participate
  in the proof substrate
- if a domain can express a progression statically, `worth-proof` must not
  replace it with a weaker runtime form for convenience
- if a boundary adapter requires erased or runtime-tagged progression metadata,
  that adapter must terminate back into a static proof-bearing form before
  hot-path execution resumes

## How This Vision Drives Engineering

This document is intentionally written so the crate roadmap can be derived from
it.

The derivation rule is:

- each capability pillar below implies concrete type surfaces that must exist
- each technical role implies constraints that implementation must preserve
- each "what this enables" section implies real cross-crate migrations the
  substrate must unlock
- if a progression mechanic is repeated across crates today and fits a named
  pillar here, it belongs in the roadmap as extractable infrastructure
- if a proposed abstraction would cross a cost, failure, or correctness
  boundary, it must be rejected even if it looks elegant

In other words:

- the vision says what the proof substrate must be able to express
- the roadmap says what proof-bearing infrastructure still must be engineered
- later migration work says where bespoke crate-local machinery should be
  retired in favor of the shared substrate

## Capability Pillars

Each pillar describes both the technical role of the capability and the kinds
of WORTH systems that need it.

### Artifact Progression Architecture

#### Phase-typed artifacts

Technical role:
Artifacts must encode their current progression phase in the type system so
that consumers cannot accidentally apply later-phase operations to earlier-phase
values.

What this enables:

- `worth-relational` can make lowered merge plans, validated schema transition
  bundles, and execution-ready forms mechanically distinct
- `worth-query` can make workflow declarations, lowered plans, live-promoted
  artifacts, and writeback-ready forms structurally ordered
- `worth-signal` can make planner-ready, admission-ready, replay-safe, and
  execution-ready values distinct without runtime-only conventions
- `worth-store` can make compatibility-checked, admitted, and durable-support
  forms explicit

#### Proof-set composition

Technical role:
Artifacts rarely carry one proof. They carry a set of established facts whose
combination determines what later phases may consume. The crate must express
proof widening, proof requirements, and proof-bearing access without pushing
domains into bespoke tuple boilerplate.

What this enables:

- cross-crate standardization of "requires X and Y, produces X + Y + Z"
- removal of repeated local proof-wrapper scaffolding
- honest representation of progression that depends on several independent
  established facts

#### Proof-carrying collections

Technical role:
WORTH repeatedly needs collections whose important properties have already been
proven: canonical order, uniqueness, disjointness, acyclicity, non-emptiness,
fixed cardinality, or resolved join shape. The crate should let domains carry
those facts forward instead of re-proving them at every phase.

What this enables:

- less repeated collection validation logic
- safer fixed-arity and collection-shaped transitions
- stronger cost honesty because later phases can consume proven collection form
  directly

### Construction Authority

#### Sealed proof minting

Technical role:
The type that claims a proof exists must not be WORTHable by ordinary callers.
Only the proving transition may mint the stronger form.

What this enables:

- compile-time enforcement of progression law instead of review-time discipline
- shared privacy-boundary patterns across crates
- less bespoke token and constructor plumbing in individual subsystems

#### Witness-based authority

Technical role:
Some progression depends on authority, capability, or trusted eligibility that
should be representable as zero-sized witnesses rather than rechecked ambiently
everywhere.

What this enables:

- capability-bearing and authority-bearing operations that compile away
- clearer distinction between "caller asked" and "authority proved"
- more local mechanical enforcement around sensitive transitions

#### Symbolic-to-resolved recipes

Technical role:
Many WORTH flows begin as symbolic author intent, validate into a legal recipe,
resolve bindings, lower into canonical plans, and only then become executable.
That progression should be expressible as shared proof law rather than bespoke
local staging.

What this enables:

- cleaner progression from unresolved authoring to admitted execution forms
- less bespoke "resolve later" scaffolding
- stronger separation between symbolic intent and proven executable meaning

#### Deterministic composition families

Technical role:
Some WORTH flows are not isolated recipes but same-family programs where
symbolic siblings, existing authoritative targets, identity-preserving rewires,
supersession, retirement, and follow-up mutation interact before one coherent
authority boundary closes. The substrate should let domains express that family
as one deterministic proof-bearing composition program rather than as scalar
batch folklore.

What this enables:

- same-family symbolic handles that do not masquerade as authoritative identity
- deterministic sibling interaction before publication
- one coherent lowering boundary for created, existing, rewritten, and retired
  family members

### Staleness and Assumption Law

#### Assumption-scoped proof validity

Technical role:
Many proofs are only valid under a specific schema version, branch epoch,
policy digest, runtime basis, or replay context. The crate must let domains
carry that scope honestly.

What this enables:

- stale-proof denial before illegal execution
- first-class expression of "valid under this basis, not valid globally"
- less ad hoc invalidation folklore around schema drift, branch restore, or
  authority rebasing

#### Freshness and rebind states

Technical role:
WORTH often needs more than success or failure. A value may be readable but
stale, lowerable only after rebind, or admissible only after authority
revalidation. These states must be mechanically represented rather than encoded
through comments or booleans.

What this enables:

- honest bridge surfaces between preview and authority
- better expression of history-, branch-, and policy-sensitive workflows
- typed handling of non-terminal progression outcomes

#### Unresolved and resolved distinctions

Technical role:
References, bindings, locators, recipes, and basis attachments often have
meaningfully different guarantees before and after resolution. The substrate
should help domains make that distinction mechanical wherever confusing the two
would create correctness hazards.

What this enables:

- fewer ambient "assume this is resolved already" conventions
- stronger type-level expression of binding and readiness guarantees
- cleaner migration from ad hoc booleans and comments into real proof state

### Transition Model

#### Typed transitions

Technical role:
The crate must provide a canonical way to express transitions that consume one
proof-bearing form and return a stronger form or a typed denial topology.

What this enables:

- standardized progression mechanics across crates
- sharper review surfaces because transition shape becomes predictable
- less reinvention of input/output/report/denial scaffolding

#### Branching outcomes

Technical role:
Not every transition is linear. Some admit, deny, defer, require rebind, or
produce report-shaped partial outcomes. The substrate must support this without
forcing domains into one oversimplified "Result<Next, Error>" story.

What this enables:

- query and signal workflows that preserve rich denial semantics
- branch- and authority-sensitive flows that remain typed
- cleaner distinction between hard failure and semantically denied progression

### Lowering and Execution Boundaries

#### Pure lowering as first-class law

Technical role:
WORTH repeatedly relies on the law that planning/lowering decides what should
happen and execution applies it. `worth-proof` must make that distinction easy
to encode and difficult to blur.

What this enables:

- less plan/execute conflation in hot paths
- stronger executor honesty because execution consumes proof-bearing lowered
  forms rather than rediscovering strategy
- better migration target for the many bespoke lowered-plan surfaces already in
  the repo

#### Execution readiness

Technical role:
A lowered plan and an execution-ready plan are not always the same thing. The
crate must allow domains to prove that a lowered form has crossed whatever
additional basis, authority, or capability boundary is required before
execution. Some of those plans will still execute at runtime rather than at
compile time; `worth-proof` must model that legality boundary without owning a
generic executor.

What this enables:

- exact representation of "planned" vs "safe to apply"
- less bespoke execution token plumbing
- sharper boundaries for effectful paths
- domain-specific runtime executors that consume only proof-bearing admitted
  plans while remaining free to optimize for their own workload shape

### Multi-Artifact Progression

#### Static fork and join

Technical role:
Some WORTH flows split one artifact into several derived forms or combine
several proof-bearing forms into one lowered or certified result. The substrate
must support these patterns without requiring a dynamic artifact graph engine in
the core.

What this enables:

- compile-time honest multi-input and multi-output transitions
- reuse across merge, bridge, certification, and maintenance planning flows
- room for later graph-level abstractions without compromising the zero-cost
  static core

#### Fixed-shape cardinality wrappers

Technical role:
Some important transitions are not "a vector of things" but "exactly one",
"a pair", "a non-empty set", or "a disjoint pair". Where those cardinalities
matter for correctness or cost honesty, the substrate should make them explicit
instead of relying on conventions around generic collections.

What this enables:

- less cardinality-by-comment drift
- clearer small fixed-arity join/fork APIs
- fewer invalid empty/multi-valued states reaching later phases

#### Composition-local symbolic references

Technical role:
Composition-local symbols are temporary references valid only within one family
program until authoritative resolution closes the boundary. The substrate
should help domains keep those references visible and unconfused with stable
identity.

What this enables:

- fewer accidental symbolic-handle-as-identity bugs
- clearer same-commit family reasoning
- cleaner handoff from symbolic family intent into resolved authority-facing
  forms

## What This Crate Must Preserve

- zero-cost execution after monomorphization
- exact cost honesty across distinct storage and execution models
- exact failure topology from the domain surface
- crate-local semantic ownership
- freedom for `worth-signal`, `worth-relational`, `worth-query`, and
  `worth-store` to keep different memory layouts and hot-path structures
- freedom for `worth-foundational` to own diagnostics, lineage, provenance,
  digest, and aspect vocabulary without bloating the core proof substrate
