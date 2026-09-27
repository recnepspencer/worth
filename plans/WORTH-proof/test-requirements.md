# WORTH Proof Test Requirements

## Scope

This document defines the certification-grade proof-substrate test requirements
for `worth-proof`.

It governs milestone closeout for:

- Milestone 1 through Milestone 7
- cross-milestone substrate gates for compile-time enforcement
- zero-cost and codegen-honesty proof lanes
- cross-crate migration closure lanes

Unlike application-facing crates, `worth-proof` is not proving one domain
feature. It is proving the law that several domain crates will rely on. That
means its tests must certify enforcement, honesty, and non-regression at the
substrate level rather than only validating one local API story.

## Purpose

`worth-proof` cannot be considered shipped merely because:

- a few typestate examples compile
- one compile-fail case rejects misuse
- a wrapper carries some markers
- a unit test shows a proof marker round-tripped
- code "looks zero-cost"

The crate is making stronger claims:

- illegal progression is structurally uncallable
- compile-time proof-bearing forms replace repeated crate-local staging law
- proof-set composition remains static and does not degrade into runtime lookup
- assumption-scoped validity survives trust shifts honestly
- lowering and execution readiness remain distinct proof-bearing boundaries
- composition-local symbols never masquerade as authoritative identity
- fixed-shape and proof-carrying collection forms eliminate re-proof without
  hiding cost topology
- the machine code remains materially comparable to bespoke handwritten domain
  code on the hot path

Those are adversarial claims. They need certification suites, not just happy
path examples.

## Global Adversarial Constraint

The `worth-proof` certification suite must prove the following:

> Under alternate construction paths, WORTHd-constructor attempts,
> assumption-basis drift, trust-boundary re-admission pressure, invalid
> phase ordering, proof-set widening pressure, fixed-shape misuse,
> symbolic-versus-resolved confusion, same-family composition pressure, and
> hot-path codegen scrutiny, the same canonical progression law must reject
> illegal forms, preserve legal proof-bearing meaning, and remain materially
> zero-cost relative to equivalent bespoke domain code.

If a proof surface works only:

- for one happy-path constructor
- without hostile caller misuse
- without replay, restore, or trust-boundary shifts
- without mixed proof-set pressure
- without fixed-shape/cardinality misuse
- without zero-cost scrutiny

then it is not certified.

## Meta-Rules

These are certification tests. They must:

- emit machine-checkable artifacts, not "the compile error looked right"
- compare canonical digests or canonical reports across independently produced
  equivalent lanes where equivalence is claimed
- prove typed or compile-time rejection for illegal progression
- prove trust-boundary and stale-basis denial before semantic drift occurs
- prove exact codegen/cost posture wherever the roadmap names zero-cost claims
- prove that public facade exposure stays synchronized with intended boundary
  law
- prove cross-crate migration parity whenever the roadmap claims migration
  readiness

These requirements are mandatory, not advisory.

### Test-Code Quality Rule

All `worth-proof` test code is production code.

That includes:

- compile-fail fixtures
- parity suites
- codegen-honesty harnesses
- proof-shape helpers
- migration harnesses
- support fixtures
- digest/assertion utilities

No suite is considered honest if:

- the runtime-facing proof is strong but the test code is noisy, duplicated, or
  blurry
- helpers hide the semantic edge they are supposed to certify
- the harness makes failure diagnosis harder than direct test code would

### Harness Architecture Rule

`worth-proof` test support is itself a subsystem and must be organized by real
responsibility rather than convenience categories.

At minimum, the harness must distinguish when they materially differ:

- fixture/setup construction
- proof-shape or digest extraction
- compile-fail matrix support
- codegen-honesty or counter capture
- migrated-domain parity adapters
- domain-specific pressure builders for `relational`, `query`, `signal`, or
  `store`

The harness must not collapse these into one flat bucket such as:

- `helpers`
- `common`
- `support`
- `utils`
- `assertions`

unless the file is so narrow that the category name still corresponds to one
 actual responsibility.

### File And Directory Discipline Rule

The default workspace structural rules apply to the proof test program:

- no test or support file over `400` lines without an explicit written
  exemption
- no test/support directory over `10` direct files without an explicit written
  exemption

If a certification family grows beyond those limits, the harness must be
reorganized by proof responsibility before milestone closure.

### Proof-Readability Rule

Certification tests must read like proof, not ceremony.

Forbidden proof-surface smells:

- giant inline fixture construction repeated across suites
- ad hoc tuple unpacking that hides what fact is being certified
- one giant test function proving several distinct law surfaces at once
- helpers that return opaque "success bundles" without exposing the certified
  categories directly
- parity checks that compare full blobs when the real law surface is narrower

When a test becomes hard to read, the default response is decomposition by
responsibility, not more comments.

### Global Certification Shape

Every named suite must define at least these lanes unless it explicitly states a
stronger reason not to:

- `control_lane` - canonical admitted baseline
- `hostile_lane` - adversarial misuse or boundary pressure
- `parity_lane`, `replay_lane`, or `codegen_lane` - an independently produced
  equivalent lane, trust-shift lane, or machine-code comparison lane

If the suite is about compile-time denial, the hostile lane may terminate in
compile failure, but it still needs a successful or equivalent comparison basis
for the admitted posture it is protecting.

### Mandatory Assertion Classes

Every named suite must include all applicable assertion classes:

- equality assertions for semantically equivalent admitted lanes
- inequality assertions for intentionally different basis, phase, or lifecycle
  lanes
- compile-fail or typed-failure assertions for rejected lanes
- zero-or-absence assertions for forbidden residue:
  - hidden dynamic lookup
  - hidden virtual dispatch
  - hidden allocation
  - illegal boundary widening

### Canonical Proof Certification Bundle

At minimum, certification bundles should emit the canonical fields applicable to
the suite scope:

- `type_shape_report`
- `compile_fail_bundle`
- `proof_shape_digest`
- `basis_digest`
- `transition_digest`
- `composition_digest`
- `failure_digest`
- `counter_snapshot`
- `codegen_honesty_report`
- `migration_parity_report`
- `residual_debt_report`

Not every suite uses every field, but every suite should emit a stable,
scope-appropriate bundle rather than free-form notes.

### Mutation-Sensitivity Rule

Every named suite must include at least one perturbation from each applicable
class:

- a perturbation that changes API path, facade path, or basis path without
  changing canonical proof meaning
- a perturbation that changes canonical proof meaning and must therefore change
  at least one declared digest or report field
- a perturbation that must fail explicitly before semantic drift occurs

### Anti-Fake-Test Rule

The following do not count as certification:

- asserting only that a wrapper compiles
- asserting only that a compile-fail fixture exists
- comparing a value only to itself from the same run
- checking only that a type carries a marker without proving the marker matters
- validating only a happy-path lane without a hostile lane
- claiming zero-cost from code inspection alone
- using one compile-fail fixture as proof that the full boundary is hardened
- burying several distinct law surfaces in one oversized test body
- hiding domain pressure inside fake-generic support helpers
- proving parity only through one helper whose own correctness is uncertified

### Named Harness Responsibility Rule

Once `worth-proof` has enough suites to justify support modules, the support
tree must be named by real proof responsibility.

Healthy examples:

- `tests/support/compile_fail/...`
- `tests/support/proof_shapes/...`
- `tests/support/codegen/...`
- `tests/support/migration/relational/...`
- `tests/support/migration/query/...`

Unhealthy examples unless narrowly scoped:

- `tests/support/helpers.rs`
- `tests/support/common.rs`
- `tests/support/assertions.rs`
- `tests/support/builders.rs`

The structure must teach the proof program, not merely store it.

## Milestone-To-Suite Map

| Milestone | Required Named Suite(s) |
| --- | --- |
| M1 | Core Artifact And Proof Substrate Test |
| M2 | Sealed Minting And Witness Authority Test |
| M3 | Assumption, Freshness, Re-Admission, And Downgrade Test |
| M4 | Transition And Outcome Algebra Test |
| M5 | Lowering And Execution Readiness Boundary Test |
| M6 | Static Fork/Join And Composition Family Test |
| M7 | Cross-Crate Migration And Hot-Path Honesty Test |

Each milestone is not closeable until its required named suite passes.

## Harness Program Requirements

These rules make the certification program specifiable by `qa-tests` rather
than only by milestone authors.

### 1. Every named suite must declare its support topology

Each implemented suite should declare, either in code comments at the harness
root or in a nearby README-style support note:

- which support modules own setup
- which support modules own proof extraction
- which support modules own parity or digest comparison
- which support modules are domain-specific migration adapters
- which support modules are generic proof mechanics

If that split is not obvious from the tree, the tree is under-designed.

### 2. Generic proof mechanics must be lifted

When several suites repeat the same proof mechanics, those mechanics belong in
generic support.

Examples:

- compile-fail matrix invocation
- proof-shape digest comparison
- basis-drift comparison
- codegen-honesty capture
- fixture-lane parity runners

Leaving clearly generic proof mechanics duplicated across suites is a harness
design failure.

### 3. Domain pressure must not be hidden in fake-generic helpers

When support logic encodes `relational`, `query`, `signal`, or `store`
semantics, it belongs in an explicit domain support home rather than a generic
helper file.

The migration suites must make it obvious which pressure is:

- proof-generic
- crate-specific

### 4. Compile-fail fixtures are part of the architecture

Compile-fail coverage is not garnish. The compile-fail tree must be curated as
a first-class proof matrix with:

- phase misuse lanes
- WORTHd-constructor lanes
- stale or downgraded misuse lanes
- lowered-versus-ready misuse lanes
- symbolic-versus-authoritative misuse lanes
- fixed-shape misuse lanes where compile-time denial is promised

One compile-fail file per family is preferred over one giant omnibus file.

Compile-fail family selection must also fail closed.

- requesting an unknown family must fail the harness immediately
- requesting several families must prove that every requested family is present
- an empty selected compile-fail bundle is a harness failure, not a passing
  no-op

### 5. Codegen-honesty tests need explicit ownership

Any suite that claims zero-cost posture must state:

- who captures the representative lane
- what comparison basis is used
- what hidden-cost classes are forbidden
- where the machine-checkable output is recorded

Zero-cost claims may not be spread as incidental assertions across unrelated
test files.

## Named Suites

### 1. Core Artifact And Proof Substrate Test

Purpose

Prove that the Phase 1 core carrier, proof markers, proof-set vocabulary, and
assumption carrier encode real progression law instead of decorative wrappers.

What to stress

- alternate construction paths for admitted core artifacts
- raw earlier-phase artifact misuse against later-phase-only APIs
- proof-set composition with single and several proof facts
- assumption-bearing and assumption-free forms
- proof-bearing structural fact carriage without runtime lookup
- fixed-shape misuse where raw collections try to stand in for stronger forms
- representative support decomposition for compile-fail, proof-shape, and
  codegen lanes

What to verify

- later-phase operations are uncallable on earlier-phase artifacts
- proof-set composition remains static and does not degrade into runtime proof
  maps
- assumption state remains typed and visible at the carrier boundary
- structurally equivalent admitted forms yield equivalent proof-shape artifacts
- representative wrapper paths avoid hidden allocation or virtual dispatch

Pass condition

The suite must emit:

- `type_shape_report`
- `compile_fail_bundle`
- `proof_shape_digest`
- `basis_digest`
- `codegen_honesty_report`

Equivalent admitted runs must match exactly. Illegal progression must fail at
compile time where the milestone claims it should. The supporting harness must
also make compile-fail ownership, proof-shape extraction, and zero-cost
capture visibly separate responsibilities.

### 2. Sealed Minting And Witness Authority Test

Purpose

Prove that stronger proof-bearing forms and authority-gated surfaces cannot be
WORTHd by ordinary callers or convenience constructors.

What to stress

- direct constructor bypass attempts
- WORTHd witness construction attempts
- facade-only consumers versus internal proving modules
- symbolic, resolved, lowered, and admitted recipe construction pressure
- compile-fail family decomposition rather than one omnibus privacy test file

What to verify

- stronger proof-bearing forms are mintable only through proving boundaries
- witness-bearing APIs compile away but remain unWORTHable externally
- recipe stages cannot skip from weaker to stronger forms by convention
- compile-fail coverage remains synchronized with privacy boundaries

Pass condition

No caller outside the proving boundary may mint stronger forms or bypass witness
requirements. The suite must emit:

- `compile_fail_bundle`
- `proof_shape_digest`
- `failure_digest`
- `codegen_honesty_report`
- `residual_debt_report`

### 3. Assumption, Freshness, Re-Admission, And Downgrade Test

Purpose

Prove that assumption-scoped validity, trust-boundary re-admission, and proof
downgrade are mechanically honest rather than ambient discipline.

What to stress

- schema-version, branch-epoch, policy-basis, and authority-digest drift
- restore or transport across trust boundaries
- stale, rebind-required, and authority-revalidation-required forms
- downgrade from stronger admitted forms into weaker non-authoritative forms
- explicit separation between basis fixtures and downgrade-proof assertions

What to verify

- stale or rebind-required forms do not flow through strong APIs
- trust-boundary crossings require explicit re-admission
- equivalent same-basis lanes match exactly
- intentionally different basis lanes diverge explicitly
- downgrade paths preserve readable but weaker truth without faking strong
  admissibility

Pass condition

The suite must emit:

- `basis_digest`
- `failure_digest`
- `compile_fail_bundle`
- `transition_digest`

Equivalent basis-preserving runs must match exactly. Basis-shifted or
trust-shifted runs must deny or degrade explicitly.

### 4. Transition And Outcome Algebra Test

Purpose

Prove that typed transitions preserve failure topology, denial richness, and
proof-ordering law without collapsing into one generic `Result` story.

What to stress

- success, denial, defer, stale, and rebind-required outcomes
- pre-construction rejection
- transition composition under proof-widening order pressure
- alternate transition paths that should be semantically equivalent
- outcome-family proof helpers that remain typed and diagnosis-friendly

What to verify

- ordering remains structurally enforced
- denial classes remain distinguishable
- illegal progression rejects before expensive construction
- transition composition does not require erased runtime dispatch on hot paths

Pass condition

The suite must emit:

- `transition_digest`
- `failure_digest`
- `compile_fail_bundle`
- `codegen_honesty_report`

Equivalent admitted transition paths must match exactly. Distinct denial classes
must remain typed and machine-checkable.

### 5. Lowering And Execution Readiness Boundary Test

Purpose

Prove that lowered forms, execution-ready forms, and executed-form proof states
stay distinct where the domain claims they are distinct.

What to stress

- pure lowering versus runtime-gated readiness
- authority- or basis-gated readiness admission
- executed-form hooks where post-execution proof state matters
- attempts to treat lowered forms as executor-ready by convenience
- harness separation between lowering fixtures, readiness admission, and
  forbidden shortcut proof lanes

What to verify

- executors consume only admitted proof-bearing forms
- runtime-admitted facts terminate back into static proof-bearing forms
- lowered and execution-ready forms do not silently collapse
- receipts and descriptive execution artifacts remain outside `worth-proof`

Pass condition

The suite must emit:

- `transition_digest`
- `basis_digest`
- `failure_digest`
- `compile_fail_bundle`

Equivalent lowered/admitted lanes must match exactly. Illegal lowered-to-execute
shortcuts must fail explicitly.

### 6. Static Fork/Join And Composition Family Test

Purpose

Prove that multi-artifact progression and same-family symbolic interaction stay
cost-honest and semantically explicit rather than degrading into generic graph
folklore.

What to stress

- one-to-many and many-to-one progression
- fixed-arity forms such as `NonEmpty`, `ExactlyOne`, `Pair`, and
  `DisjointPair`
- same-family symbolic siblings mixed with existing authoritative targets
- identity-preserving rewrites, supersession, retirement, and follow-up
  mutation within one deterministic family program
- decomposition pressure on family fixtures so sibling topology, authoritative
  targets, and parity proof are not blurred together

What to verify

- cardinality remains explicit and type-enforced
- symbolic family handles cannot flow into authority-identity APIs
- small fixed-arity joins do not smuggle in broad generic containers or hidden
  allocation
- deterministic family lowering remains canonical across equivalent family
  histories

Pass condition

The suite must emit:

- `composition_digest`
- `proof_shape_digest`
- `failure_digest`
- `codegen_honesty_report`
- `compile_fail_bundle`

Equivalent family programs must converge exactly. Symbolic-versus-authoritative
confusion must fail compile-time or typed-admission checks.

### 7. Cross-Crate Migration And Hot-Path Honesty Test

Purpose

Prove that `worth-proof` is genuinely fit to replace bespoke progression
machinery in real WORTH crates without semantic drift or hidden cost cliffs.

What to stress

- one proof-heavy migration lane from `worth-signal`
- one proof-heavy migration lane from `worth-relational`
- one proof-heavy migration lane from `worth-query` or `worth-store`
- hostile stale-proof, WORTHd-minting, and boundary-misuse attempts against the
  migrated surfaces
- representative hot-path codegen or counter scrutiny against bespoke baselines
- honest separation between generic migration proof mechanics and crate-specific
  adapters

What to verify

- migrated surfaces preserve semantics and failure topology
- codegen and allocation posture remain materially honest
- residual bespoke machinery is explicitly inventoried rather than forgotten
- facade guidance for `worth-proof` versus `worth-foundational` usage remains
  machine-checkable and reviewable

Pass condition

The suite must emit:

- `migration_parity_report`
- `failure_digest`
- `compile_fail_bundle`
- `codegen_honesty_report`
- `residual_debt_report`

Equivalent migrated lanes must match exactly. Any remaining non-migrated
progression law must be named as explicit debt.

## Cross-Milestone Gates

### The Compile-Time Boundary Coverage Gate

Purpose

Prove that the crate's most important law surfaces are compiler-enforced rather
than merely documented.

What to stress

- out-of-order phase use
- WORTHd minting attempts
- stale form misuse
- lowered-versus-ready confusion
- symbolic-versus-authoritative family confusion
- fixed-shape misuse

Pass condition

The compile-fail matrix covers every named law surface that the roadmap claims
is compiler-enforced.

### The Zero-Cost Codegen Honesty Gate

Purpose

Prove that zero-cost claims are backed by repeatable machine-checkable evidence.

What to stress

- plain artifact carrier lanes
- assumption-bearing lanes
- proof-set composition lanes
- fixed-shape lanes
- representative migrated hot-path lanes

Pass condition

The crate emits a canonical `codegen_honesty_report` showing no hidden dynamic
lookup, no hidden virtual dispatch, and no mandatory allocation on the named
representative lanes.

### The Trust-Boundary Re-Admission Drift Gate

Purpose

Prove that restore, transport, or other trust shifts cannot silently preserve
strong proof-bearing meaning.

What to stress

- restore before and after basis drift
- same payload across trusted and re-admitted forms
- downgrade followed by explicit re-admission

Pass condition

Equivalent trusted and explicitly re-admitted lanes match where they should.
Implicit proof carryover across trust shifts is impossible or rejected.

### The Composition-Local Symbol Non-Identity Gate

Purpose

Prove that composition-local symbols remain temporary family references rather
than accidental authoritative identities.

What to stress

- sibling creation order perturbations
- mixed new/existing family members
- supersession and retirement pressure
- attempts to use unresolved family symbols in authority-only paths

Pass condition

Equivalent family programs converge exactly, and symbolic handles cannot be
consumed as stable authoritative identities.

## What These Suites Collectively Prove

Together, these suites prove that `worth-proof` is:

- compiler-enforced about progression rather than convention-driven
- stale-honest across basis drift and trust shifts
- explicit about lowered, admitted, and executed-form boundaries
- honest about fixed-shape and same-family progression
- zero-cost enough to serve as real shared substrate rather than decorative
  wrapper stack
- migration-worthy across multiple WORTH crates

## Milestone Closeout Rule

No `worth-proof` milestone should be considered closed until:

- its required named suite passes
- the suite emits machine-checkable output
- hostile lanes are present wherever the milestone claims enforcement
- zero-cost or codegen claims are backed by a named report where applicable
- the implemented test/support tree itself passes a hostile `qa-tests` review
  for adversarial strength, harness honesty, abstraction placement, and
  file/directory discipline

Without that, the substrate may be promising, but it is not yet trust-grade.
