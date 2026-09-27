# Milestone 3 Engineering Spec: Assumption, Freshness, And Staleness Law

> **Status:** Closed
>
> **Closeout:** [milestone-3-closeout.md](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-proof/milestone-3-closeout.md)
>
> **Roadmap parent:** [worth_proof_roadmap.md](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-proof/worth_proof_roadmap.md)
>
> **Vision parent:** [worth_proof_vision.md](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-proof/worth_proof_vision.md)
>
> **Test requirements:** [test-requirements.md](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-proof/test-requirements.md)
>
> **Adjacent milestone closeout:** [milestone-2-closeout.md](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-proof/milestone-2-closeout.md)
>
> **Impacted later milestones:**
> - `Milestone 4` (`Transition And Outcome Algebra`)
> - `Milestone 5` (`Lowering And Execution Readiness`)
> - `Milestone 6` (`Static Fork And Join Progression`)
> - `Milestone 7` (`Certification And Cross-Crate Migration Closure`)
>
> **Primary architectural driver:** make proof validity basis-scoped and trust-shift-honest now that Milestone 2 has sealed construction and authority-bearing progression

## Goal

Make scoped validity first-class in `worth-proof` so proofs, witnesses, and
staged forms can be carried honestly across basis drift, authority drift, and
trust-boundary crossings without ambient conventions, fake global validity, or
silent proof reuse.

## Why This Milestone Exists

Milestone 1 froze the zero-cost carrier family.
Milestone 2 hardened who may mint stronger forms.

That leaves the next structural weakness:

- stronger proof-bearing forms can still look globally valid once they exist,
  even when their validity actually depends on a branch epoch, schema version,
  policy basis, authority digest, replay context, or trust boundary

Without Milestone 3:

- domain crates will keep inventing bespoke stale markers, rebind flags, and
  restore/readmission folklore around the shared substrate
- values that crossed serialization, restore, bridge transport, or authority
  loss will keep being treated as if the original proof still held
- later transition algebra will have no canonical way to distinguish "denied",
  "stale but readable", "rebind required", and "authority must revalidate"
- lowering and execution-readiness milestones will be forced to build on proof
  forms that are mechanically sealed but not mechanically freshness-honest

Milestone 3 therefore exists to solve the next hard progression problem:

- what basis a proof is valid under
- how basis drift changes what may still be consumed
- how strong forms degrade when their basis or trust boundary changes
- how explicit re-admission regains stronger proof-bearing status
- how stale/readable/rebind-required/authority-required states remain typed
  instead of ambient

## Hard Part

The hard part is not attaching a basis value to a type.

The hard part is preserving all of these at once:

- the zero-cost artifact grammar from Milestone 1
- the sealed minting and witness authority posture from Milestone 2
- explicit basis-scoped validity rather than fake global truth
- explicit downgrade and re-admission boundaries rather than "probably still
  okay" adapter behavior
- readable stale states without accidentally letting them flow through strong
  APIs
- trust-boundary honesty without a runtime registry or generic metadata blob

The design fails if:

- assumption scope is carried only as optional metadata with no effect on
  admissibility
- stale or rebind-required forms are still consumable by stronger APIs through
  convenience accessors
- restore, serialization, or transport can preserve strong proof status
  ambiently
- basis drift is represented only as stringly diagnostics rather than typed
  progression state
- later milestones would need to replace the Milestone 1 or Milestone 2
  surfaces to make freshness honest

## Explicit Assumptions

- Milestone 1 core carrier law and Milestone 2 sealing/witness closure remain
  authoritative.
- `worth-proof` still owns progression law only; it does not become the owner
  of diagnostics, lineage, provenance, storage, or runtime authorization.
- domain crates remain the semantic authorities for what a schema basis, branch
  epoch, policy digest, authority digest, or replay context means.
- Milestone 3 may introduce freshness classes, downgrade carriers,
  re-admission markers, and basis-sensitive transitions, but must not introduce
  a dynamic runtime proof registry.
- full transition algebra, lowering readiness, and executor admission remain
  later milestones, though Milestone 3 must leave them with an honest stale and
  readmission substrate.

## Governing Summaries

- `MENTALITY.md`
  The most important thing it protects is solving the adversarial constraint
  before later transition and execution milestones pile on. Milestone 3
  therefore hardens basis-scoped validity and trust-boundary honesty now
  instead of letting stale-proof folklore spread first.
- `arch_laws.md`
  The most important thing it protects here is that types must encode what has
  been proven and when that proof is no longer consumable. Laws 24, 30, 37,
  40, and 41 shape this milestone most strongly.
- `perf_laws.md`
  The most important thing it protects is that freshness law must carry proof
  forward or explicitly revalidate it without hidden lookup, hidden allocation,
  or repeated rediscovery on hot paths.
- `domain_laws.md`
  The most important thing it protects is responsibility clarity. Basis
  carriage, downgrade law, re-admission law, and hostile certification must be
  decomposed cleanly rather than collapsed into one vague stale-state bucket.
- `worth_proof_vision.md`
  The most important thing it protects is the identity of `worth-proof` as a
  static progression-law substrate that can represent assumption-scoped
  validity, stale-readable forms, trust-boundary re-admission, and downgrade
  paths without becoming a runtime registry or provenance engine.
- `worth_proof_roadmap.md`
  The most important thing it protects is sequencing. Milestone 3 is the first
  milestone that makes sealed proof-bearing forms freshness-honest, and later
  transition, lowering, and migration milestones depend on that honesty.
- `worth-proof` test requirements
  The most important thing it protects is that stale, rebind-required, and
  re-admitted behavior must be certified with basis-drift and trust-shift
  hostile lanes rather than closed by comments or one-off API examples.
- `milestone-1.md`
  The most important thing it protects is the canonical carrier family.
  Milestone 3 must extend the admitted artifact/proof/assumption grammar rather
  than introducing a second basis- or freshness-carrying wrapper story.
- `milestone-2.md`
  The most important thing it protects is the sealed authority boundary.
  Milestone 3 must preserve Milestone 2's unWORTHable stronger forms while
  making their validity conditional and degradeable when their basis changes.
- `milestone-2-closeout.md`
  The most important thing it protects is the set of assumptions later
  milestones may now safely build on: sealed stronger forms, witness-based
  authority, recipe-stage sealing, and the fact that no parallel carrier family
  should be introduced to express freshness.

## Adversarial Constraint

The milestone must survive the following hostile condition:

> Several WORTH subsystems with branch-sensitive proof validity, restore and
> transport boundaries, authority rebasing, and replay-sensitive workflows must
> be able to carry previously admitted proof-bearing forms such that callers
> can still read what remains valid, cannot silently consume stale forms as
> fully admitted forms, and can regain strong status only through explicit
> rebind or re-admission progression.

The design fails if:

- a proof-bearing form valid under basis `B1` can still flow through APIs that
  require basis `B2` without explicit downgrade or revalidation
- a restored or transported value can regain strong proof-bearing status
  without an explicit re-admission step
- stale-readable forms expose the same API surface as fully valid forms
- rebind-required and authority-revalidation-required collapse into one generic
  optional error field
- the implementation needs a runtime registry or ambient side table to decide
  whether a form is stale on statically knowable paths
- Milestone 4 or Milestone 5 would have to reinterpret stale or re-admitted
  forms because Milestone 3 left the downgrade shape ambiguous

## Product Decision Lock

- proof validity is basis-scoped, not globally timeless
- stale, rebind-required, authority-revalidation-required, and re-admitted are
  first-class progression states, not commentary about ordinary admitted forms
- trust-boundary crossings are progression events; strong proof-bearing status
  may not survive them ambiently
- downgrade and invalidation are as structurally important as proof minting and
  must be represented by explicit types or transitions
- the Milestone 1 carrier family and Milestone 2 sealing/witness surfaces
  remain canonical; freshness law extends them rather than replacing them
- the public facade remains the only public entry surface for the crate
- basis carriage, freshness classes, and re-admission surfaces must remain
  zero-cost after monomorphization for the admitted representative scope
- the crate does not introduce runtime proof registries, generic metadata bags,
  or provenance-owned validity engines to solve freshness

Normative consequence:

- any implementation that treats basis-scoped proofs as globally valid is out
  of spec
- any implementation that lets a trust-boundary crossing preserve strong proof
  status implicitly is out of spec
- any implementation that models downgrade only as diagnostics text or optional
  booleans rather than typed progression is out of spec

## Required Contracts

### Assumption-Scoped Validity Rule

Every stronger proof-bearing form admitted by this milestone must state the
basis under which its stronger guarantees remain consumable.

Required vocabulary:

- assumption-scoped proof-bearing form
- basis digest or basis carrier posture
- basis-preserving read posture
- basis-sensitive strong-consumption posture

Rules:

- basis carriage must travel with the proof-bearing form or its immediate
  wrapper, not through ambient side state
- basis-sensitive APIs must declare whether they require same-basis, any-basis
  readable access, or explicit rebind/re-admission
- changing the basis must change admissibility meaning, not only diagnostics

### Freshness And Staleness Rule

Freshness classes must distinguish what remains readable from what remains
consumable as strong proof-bearing truth.

Required vocabulary:

- fresh or currently valid posture
- stale-readable posture
- rebind-required posture
- authority-revalidation-required posture
- invalidated or downgraded posture

Rules:

- stale-readable forms may not type-check as fully valid forms
- rebind-required and authority-revalidation-required may not collapse into one
  generic stale bucket when their remediation path differs
- freshness state must be represented structurally, not by ad hoc comments or
  side booleans

### Trust-Boundary Re-Admission Rule

Crossing serialization, restore, transport, or other trust boundaries must
explicitly degrade or suspend stronger proof-bearing status until re-admission
occurs.

Required vocabulary:

- boundary-bridged form
- re-admission surface
- pre-readmission weakened posture
- re-admitted strong posture

Rules:

- values crossing trust boundaries may remain readable, but not silently strong
- re-admission must be an explicit progression event, not an adapter detail
- the substrate must support representative restore/transport pressure without
  assuming one storage or transport mechanism

### Downgrade And Invalidation Rule

Losing a proof is a progression event and must be expressible as such.

Required vocabulary:

- downgrade transition
- invalidation transition
- basis-expiry posture
- authority-loss posture

Rules:

- downgrade must preserve whatever weaker truth remains readable without
  pretending the stronger proof still holds
- invalidation triggers may include basis drift, authority drift, restore, or
  policy drift, but the substrate must not encode domain semantics for those
  triggers
- later milestones must be able to consume downgrade outcomes directly instead
  of re-deriving them procedurally

### Compile-Time Boundary Rule

The highest-risk stale misuse and trust-shift misuse boundaries in this
milestone must be compiler-enforced rather than doc-only.

Required compile-time posture:

- stale or rebind-required forms reject strong-consumption APIs
- unresolved forms do not type-check as resolved-only APIs where resolution
  affects validity
- pre-readmission trust-shifted forms reject strong-consumption APIs
- the facade does not leak internal freshness or re-admission machinery

### Performance-Shaping Rule

Freshness law must preserve the zero-cost posture of the Milestone 1 and
Milestone 2 carrier family while preventing repeated rediscovery of already
known validity facts.

Required performance-shaping surfaces:

- basis-bearing stronger forms
- stale-readable or downgraded forms
- representative re-admission forms

Rules:

- freshness classification must not require runtime registries on statically
  knowable paths
- re-admission and downgrade surfaces must not introduce mandatory allocation
- representative freshness and re-admission forms must remain
  size/alignment/drop-honest for the admitted scope

## Scope

### In Scope

- assumption-scoped proof vocabulary
- freshness and staleness marker or wrapper model
- rebind-required and authority-revalidation-required state surfaces
- downgrade and invalidation progression helpers
- trust-boundary admission and re-admission helpers
- representative unresolved-versus-resolved distinction helpers where
  resolution changes what remains valid
- compile-fail support for stale misuse and trust-shift misuse
- facade hardening needed to keep freshness and readmission machinery private
- module topology for basis, freshness, downgrade, and readmission
- milestone-local certification notes that map directly onto the crate-level
  `Assumption, Freshness, Re-Admission, And Downgrade Test`

### Explicitly Out Of Scope

- full transition trait algebra or rich denial taxonomy beyond the freshness
  surfaces required here
- full lowering/execution-readiness law
- diagnostics, provenance, lineage, or storage schemas for stale/readmission
  events
- runtime authorization services or dynamic policy evaluators
- domain-specific meaning of schema versions, branch epochs, authority digests,
  or replay contexts
- cross-crate migration closure beyond what is needed to prove the substrate
  shape

## Phases

### Phase 1: Assumption And Freshness Vocabulary

Define the substrate vocabulary for basis-scoped validity before any downgrade
or re-admission behavior is layered on top.

Must ship:

- explicit basis-bearing proof vocabulary
- first freshness-class vocabulary for:
  - currently valid
  - stale-readable
  - rebind-required
  - authority-revalidation-required
- representative carrier surfaces that can express those classes without
  replacing the Milestone 1 artifact grammar
- explicit rejection of generic metadata bags as the basis/freshness carrier

Implementation guidance:

- extend `assumption/` first rather than scattering basis and freshness
  mechanics across `artifact/`, `proof/`, and `recipe/`
- keep basis carriage separate from the semantic meaning of the basis; this
  milestone owns the progression law, not the domain's basis ontology
- define the minimum representative freshness model that later transitions and
  execution-readiness work can reuse without reinterpretation
- make "readable but not strongly consumable" explicit early so later phases do
  not accidentally collapse stale-readable into denied

This phase is complete only when the crate can honestly say "this form is valid
under basis B", "this form is stale but readable", and "this form requires
rebind or authority revalidation" without ambient conventions.

### Phase 2: Downgrade, Rebind, And Authority-Revalidation Surfaces

Define the explicit progression events that move stronger proof-bearing forms
into weaker but still meaningful states.

Must ship:

- downgrade transitions for basis drift and authority drift
- explicit rebind-required posture where the payload remains meaningful but the
  prior binding or basis does not
- explicit authority-revalidation-required posture where the consumer still has
  a meaningful artifact but not a currently admissible authority state
- representative unresolved-versus-resolved distinction helpers where
  resolution changes validity meaning

Implementation guidance:

- solve downgrade before re-admission so the weakened form is already honest
  before the crate tries to strengthen it again
- keep downgrade surfaces narrow and progression-shaped; do not let them turn
  into free-form status-report types
- preserve whatever weaker truth remains readable through explicit access
  surfaces rather than destructuring back to raw payloads
- distinguish basis loss from authority loss when their remediation path
  differs; one "stale" bucket is too weak for later milestones

This phase is complete only when losing proof strength is a typed transition
rather than a convention around formerly admitted values.

### Phase 3: Trust-Boundary Re-Admission

Define the canonical trust-shift story so restored or transported values may
regain stronger status only through explicit re-admission.

Must ship:

- representative boundary-bridged posture for values that crossed restore,
  serialization, or transport boundaries
- explicit pre-readmission weakened surface
- explicit re-admission transition back into stronger proof-bearing status
- representative recipe or artifact lane proving that trust-shifted values do
  not retain strong status ambiently

Implementation guidance:

- keep trust-boundary law substrate-level and mechanism-agnostic; this
  milestone should not pick one persistence or transport story
- model re-admission as a real progression event with a distinct input and
  output type
- preserve Milestone 2 sealing; re-admission may restore strong status only
  through trusted progression boundaries, not by exposing ordinary constructors
- if a trust-shifted form stays readable, make that readable posture explicit
  and weaker than the re-admitted result

This phase is complete only when trust-boundary crossings are progression
events, not ambient caveats.

### Phase 4: Hostile Certification And Closure

Prove that basis-scoped validity, downgrade, and re-admission are mechanically
honest and inherit Milestone 1 and Milestone 2 without regression.

Must ship:

- hostile compile-fail coverage for stale misuse, unresolved misuse where
  resolution matters, and pre-readmission misuse
- hostile basis-drift tests covering representative schema/branch/policy or
  authority rebasing pressure
- hostile trust-boundary re-admission tests covering restore or transport
  pressure
- closure record of what later milestones may now assume about downgrade and
  re-admission law

Implementation guidance:

- the certification suite should map back to the contract sections above rather
  than one omnibus stale-state test
- certify one representative lane each for:
  - basis-scoped same-basis validity
  - basis-drift downgrade
  - stale-readable versus strong-consumption denial
  - explicit re-admission after trust shift
- publish explicit residual debt if first ship leaves any freshness class
  representative rather than exhaustive

This phase is complete only when the milestone has machine-checkable evidence
that stale forms are not silently consumable and trust-shifted values cannot
regain strong status ambiently.

## Must Ship

- one canonical assumption-scoped validity story for stronger proof-bearing
  forms
- one canonical freshness vocabulary for stale-readable, rebind-required, and
  authority-revalidation-required states
- one canonical downgrade and invalidation progression story
- one canonical trust-boundary re-admission story
- representative unresolved-versus-resolved distinction helpers where
  resolution changes validity guarantees
- compile-fail coverage proving stale misuse and pre-readmission misuse are
  uncallable
- milestone-local implementation notes that map directly onto the crate-level
  certification bar in `worth-proof/test-requirements.md`

## Must Preserve

- the Milestone 1 carrier family and the Milestone 2 sealing/witness boundary
- zero-cost hot-path posture after monomorphization
- no mandatory heap allocation
- no mandatory dynamic dispatch
- no fake global validity for basis-scoped proofs
- no hidden stale-proof reuse
- no generic metadata blob standing in for typed freshness law
- no silent preservation of strong proof-bearing status across trust shifts
- no drift of `worth-proof` into diagnostics, provenance, storage, or runtime
  authorization semantics
- clean extension room for Milestone 4 transition algebra and Milestone 5
  lowering/execution readiness law

## Acceptance Evidence

Milestone 3 is complete only when `worth-proof` satisfies the named milestone
suite:

- `Assumption, Freshness, Re-Admission, And Downgrade Test`

Required machine-checkable outputs:

- `basis_digest`
- `failure_digest`
- `compile_fail_bundle`
- `transition_digest`

Milestone-specific proof obligations:

- stale or rebind-required forms are not silently consumable as fully valid
  forms
- assumption-scoped validity travels with the proof-bearing form rather than
  through ambient side state
- equivalent same-basis lanes match exactly
- intentionally different basis lanes diverge explicitly
- trust-boundary crossings require explicit re-admission before strong proof
  status returns
- unresolved forms cannot silently flow into resolved-only APIs where
  resolution affects validity
- downgrade paths preserve whatever weaker readable truth remains without
  pretending strong admissibility
- the suite includes hostile basis-drift, trust-shift, and downgrade lanes
- the suite publishes explicit residual debt for any intentionally deferred
  freshness or readmission family

Milestone 3 is not closed by "basis is present on the type" or "restore goes
through a helper function" arguments.

## Architectural Notes

- Milestone 3 is about validity scope and proof loss, not about the full
  branching outcome algebra yet.
- Downgrade and re-admission are progression events, not descriptive reports.
- This milestone should prefer representative substrate-level freshness law
  over trying to exhaust every possible domain-specific basis class in first
  ship.
- If trust-boundary honesty requires introducing a second carrier family, the
  design is wrong and must be revised.

## Implementation Topology

This milestone should extend the Milestone 1 and Milestone 2 topology rather
than replacing it.

Preferred additions:

```text
crates/worth-proof/src/
  facade.rs
  assumption/
    mod.rs
    basis.rs
    freshness.rs
    downgrade.rs
    readmission.rs
  recipe/
    mod.rs
    stages.rs
```

This is not a forced final topology, but the ownership boundaries are
intentional:

- `assumption/basis.rs`
  - basis-carrying vocabulary and same-basis posture
- `assumption/freshness.rs`
  - freshness classes and readable-vs-strong distinctions
- `assumption/downgrade.rs`
  - downgrade and invalidation progression helpers
- `assumption/readmission.rs`
  - trust-boundary weakening and re-admission progression
- `recipe/*`
  - representative unresolved/resolved and trust-shift-sensitive recipe lanes

The milestone should avoid:

- one giant mixed `freshness_and_readmission.rs`
- pushing trust-boundary law into unrelated artifact or proof files
- treating diagnostics/provenance summaries as if they were freshness law
- exposing downgrade or re-admission internals publicly instead of through the
  facade

## Sequencing Notes

- This milestone belongs immediately after Milestone 2 because sealing stronger
  forms is not enough if those stronger forms still look globally valid once
  their basis changes.
- Milestone 4 depends on this milestone because transition outcomes need honest
  stale, rebind, and re-admission substrate states rather than ambient comments
  about why a transition could not proceed.
- Milestone 5 depends on this milestone because execution-readiness and runtime
  admission are only meaningful if previously proven forms can lose readiness
  honestly when their basis or trust boundary changes.
- Milestone 6 depends on this milestone because composition-family lowering and
  fork/join progression need a canonical story for which sibling proofs remain
  valid across branch or trust shifts.
