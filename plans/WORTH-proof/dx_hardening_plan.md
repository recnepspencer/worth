# worth-proof DX Hardening Plan

> **Status:** Planned engineering spec
>
> **Roadmap parent:** [worth_proof_roadmap.md](/C:/Users/shepworth/Documents/programming/WORTH/plans/worth-proof/worth_proof_roadmap.md)
>
> **Vision parent:** [worth_proof_vision.md](/C:/Users/shepworth/Documents/programming/WORTH/plans/worth-proof/worth_proof_vision.md)
>
> **Test requirements:** [test-requirements.md](/C:/Users/shepworth/Documents/programming/WORTH/plans/worth-proof/test-requirements.md)
>
> **Adjacent milestone:** [milestone-7.md](/C:/Users/shepworth/Documents/programming/WORTH/plans/worth-proof/milestone-7.md)
>
> **Primary architectural driver:** make the shipped `worth-proof` surface read
> like a proof language instead of only like a stabilized substrate, without
> weakening any progression-law boundary already closed by Milestones 1 through
> 6

## Goal

Harden the public `worth-proof` API surface so common WORTH authoring becomes
materially easier, clearer, and more compressible for humans and AIs without
changing the already-closed proof-bearing semantics underneath it.

This work is not about inventing:

- a second proof engine
- a runtime workflow runtime
- a magical one-call execution abstraction
- a generic graph framework
- or a builder layer that shrinks semantic truth

It is about making the existing proof-bearing lifecycle model pleasant enough
that real WORTH crates reach for it first instead of falling back to bespoke
local progression wrappers out of authoring fatigue.

The governing standard is:

> The DX layer is a declaration compiler. It compresses repetition, but it
> never shrinks semantic truth.

## Why This Milestone Exists

Milestones 1 through 6 did the right thing first:

- phase and proof carriers are canonical
- proof minting and witness authority are sealed
- freshness, downgrade, and trust-boundary law are explicit
- transitions and failure topology are typed
- lowered, ready, and executed forms are distinct
- fixed-arity composition and deterministic family lowering are canonical

That was the correct first job.

But the current public surface still looks too much like a rigorously closed
substrate and not enough like an application-facing proof language.

Today the common lane still forces developers to repeat or over-think:

- explicit transition type names even when only one canonical next step exists
- context-wrapper construction for routine straight-line progression
- verbose checked-readiness aliases before the intended semantic path is clear
- raw `Pair`, `JoinInputs2`, and family-lifecycle assembly even for common
  canonical shapes
- repeated authority and capability posture in domain-local flows

That means the crate is semantically strong but ergonomically weak.

The practical success condition is not just shorter examples.

It is that a normal WORTH engineer or AI can open a domain module and
understand:

- the obvious entrypoint
- the canonical straight-line progression
- where the dangerous boundaries still stay explicit
- where to drop to the lower-level substrate when the weird case actually earns
  it

without needing to start from raw transition nouns, support tests, or milestone
closeouts unless the module is intentionally working at the raw law boundary.

## Hard Part

The hard part is not inventing nice names.

The hard part is compressing syntax without compressing any of these real law
surfaces:

- sealed proof minting
- witness-authorized progression
- stale vs rebind vs authority-revalidation distinction
- trust-boundary crossing
- symbolic vs authoritative family identity
- checked failure topology
- canonical ordering responsibility
- fixed-arity cost honesty

If the new surface is pleasant only because it:

- hides trust-boundary transitions
- turns proofs and witnesses into the same mental model
- flattens `Denied`, `Deferred`, `Stale`, `RebindRequired`, and `Failed`
- treats lowered, ready, and executed forms as one convenience abstraction
- makes weird family-composition cases fall straight through to the raw lane
- or encourages domain code to rebuild stronger forms manually

then it has failed.

## Governing Summaries

- `MENTALITY.md`
  The main protection here is solving the hostile DX failure first rather than
  merely shortening syntax. The DX plan must start from adversarial misuse and
  long-term WORTH authoring pressure, not from cosmetic builder enthusiasm.
- `arch_laws.md`
  The main protection here is that any pleasant surface must still preserve
  proof-bearing meaning, phase ordering, compile-time enforcement, explicit
  boundary crossings, and facade discipline. Laws 9, 20, 24, 30, 37, 39, 40,
  and 41 shape this work most strongly.
- `perf_laws.md`
  The main protection here is cost honesty. Nice-looking APIs must not hide
  broad work, repeated proof rediscovery, hidden allocation, or a second
  planning engine behind cheap-looking calls.
- `domain_laws.md`
  The main protection here is subsystem clarity. Constructors, progression
  facades, checked helpers, grouped read helpers, and raw escape hatches need
  distinct homes instead of one mega-builder.
- `worth_proof_vision.md`
  The main protection here is ownership: `worth-proof` remains the compile-time
  law layer. DX hardening may improve authoring, but it must not turn the crate
  into a second runtime or a descriptive boundary system.
- `worth_proof_roadmap.md`
  The main protection here is sequencing. DX hardening belongs after the
  substrate is semantically closed enough to wrap honestly, and before broad
  platform teaching or cross-crate adoption tries to standardize on a weak
  public lane.
- `worth-proof` test requirements
  The main protection here is proof quality. Any pleasant lane must eventually
  certify raw-vs-facade equivalence, hostile-boundary honesty, and hot-path
  honesty instead of only shipping nicer docs.
- `milestone-6-closeout.md`
  The main protection here is what the DX layer may assume: fixed-arity
  composition, ready-join posture, symbolic-vs-authoritative family separation,
  and deterministic family lowering are already canonical and must remain so.
- `milestone-7.md`
  The main protection here is that DX work must not compromise the eventual
  migration proof. The pleasant lane must remain a lowering surface over the
  same semantic substrate that real crate migrations will later certify.
- `worth_signal_wasm/api_surface_dx_plan.md`
  The main protection here is the product pattern: one obvious entrypoint,
  semantic finalizers, scoped defaults, one-step-at-a-time degradation, raw
  lane preserved as escape hatch, and line-consumption DX treated as part of
  product quality rather than as optional polish.

## Adversarial Constraint

This milestone must survive the following hostile condition:

> Real WORTH authoring across `relational`, `query`, `signal`, and
> `runtime-bridge` must be able to use a pleasant `worth-proof` lane that is
> shorter, clearer, and more composable than the raw substrate while still
> preserving exact progression law, exact stale and trust-boundary law, exact
> failure topology, and exact hot-path honesty relative to the raw facade it
> lowers into.

The design fails if the pleasant lane can:

- produce a stronger form without the same witness or authority requirements as
  the raw lane
- hide broad or expensive work behind a cheap-looking method chain
- erase stale, rebind, or authority-revalidation distinctions
- collapse symbolic and authoritative family meaning into one fluent API
- force conventional flows into giant declaration objects or giant builder blobs
- make the common CRUD-shaped or plan-shaped lane pretty while weird
  trust-boundary or family-composition cases immediately fall off a cliff into
  raw substrate ceremony
- or imply that the pleasant lane owns semantics the raw facade still owns

## Product Decision Lock

- keep the current raw facade as the semantic authority
- add one obvious pleasant entrypoint per major authoring family rather than
  one giant universal builder
- prefer verb-first progression surfaces for the common lane
- the common recipe lane should start from one obvious constructor such as
  `recipe(...)`
- fixed-shape and family-composition lanes should start from small explicit
  constructors, not opaque resource-style magic
- no `.build()` stage is part of the common lane
- `admit_with(...)` and `ready_with(...)` must remain distinct and clearly
  taught as distinct progression boundaries rather than as interchangeable
  convenience verbs
- trust-boundary crossing must remain explicit in code
- basis replacement during readmission or rebinding must remain explicit in code
- symbolic and authoritative family identity must remain explicit in code
- stale, rebind, authority-revalidation, denied, deferred, and failed topology
  must remain explicit in code
- the pleasant lane must degrade one step at a time for weird cases rather than
  forcing an immediate fall to raw transition plumbing
- grouped read-path DX is in scope; declaration DX alone is incomplete
- the raw facade remains the escape hatch and semantic anchor

Normative consequence:

- any DX surface that changes progression semantics relative to the raw lane is
  out of spec
- any DX surface that hides adversarial boundary crossings behind vague names is
  out of spec
- any DX surface that makes the common lane pretty but the weird lane miserable
  is out of spec
- any DX surface that compresses type-level truth into builder folklore is out
  of spec

## Architectural Model

### Ownership split

This milestone freezes the intended ownership boundary:

1. **raw proof substrate**
   - remains the semantic authority at the crate boundary
   - continues to own proof-bearing progression law, witnesses, stale and
     trust-boundary semantics, transitions, fixed-arity composition, and
     deterministic family lowering
2. **DX facade lane**
   - owns pleasant authoring, canonical helper naming, scoped progression
     defaults, grouped read helpers, and one-step-at-a-time degradation
   - lowers into the raw proof substrate
   - does not define independent proof semantics
3. **domain-local adapters**
   - remain responsible for domain marker names, domain basis values, domain
     denial payloads, and domain semantic wrappers
   - do not redefine shared progression law

The DX layer is therefore not:

- a second proof runtime
- a second transition engine
- a generic builder blob detached from substrate truth
- a substitute for `worth-foundational`

### History and lineage authority do not move through DX

`worth-proof` may express generic progression, freshness, inversion, and
authority-composition laws. It does not own canonical commit history, entity
lineage, branch heads, parent selection, ancestry, or publication order. Those
remain with Relational. A Query-local chain removed during Runtime Phase 8 must
not be recreated here or routed into `worth-foundational` as canonical history.

`worth-foundational` may describe an already-established portable fact that
means the same thing across runtimes. Runtime Bridge may map or transport that
description. Neither layer may turn description into current legality, choose a
parent or head, or mint lineage authority. A pleasant proof API must preserve
this split rather than hiding an authority transfer behind fluent DX.

### Implementation topology

The default implementation posture should be additive surfaces over the
existing raw types, not a competing wrapper universe.

Preferred direction:

- small explicit constructor helpers
- extension traits over canonical raw forms where that keeps ownership clear
- a `dx`, `prelude`, or equally explicit facade module that gathers the blessed
  pleasant lane

Avoid by default:

- parallel semantic wrapper hierarchies that duplicate the raw type graph
- a second source of truth for progression state
- DX-only state carriers whose semantics diverge from the raw substrate

Normative consequence:

- the raw substrate remains the single semantic source of truth
- the pleasant lane should lower by calling into existing substrate semantics
  rather than by reimplementing them
- any parallel wrapper form must justify why extension traits and helper
  constructors were insufficient

### AI consumption model

This milestone explicitly treats the DX surface and its docs as AI-consumed
training and retrieval material, not only as human-facing polish.

The pleasant lane must therefore optimize for:

- strict naming regularity
- regular signature shapes across analogous verbs
- obvious degradation patterns
- one obvious blessed path for the common lane
- high predictability under analogy-driven code generation

Rules:

- common progression verbs should follow a strict `verb_with(...)` grammar
  wherever a witness, authority, basis, or capability remains explicit
- checked progression verbs should follow a strict `try_verb(...)` grammar
  wherever checked topology is preserved
- zero-argument semantic finalizers such as `.execute()` may remain plain verbs
  only when they do not hide any additional authority or capability boundary
- similar-looking pleasant-lane methods must not differ silently in whether
  they require witness-bearing context, checked topology, or raw escape posture
- every pleasant-lane method doc should include a short
  `Equivalent raw surface` note
- every pleasant-lane workflow doc should include the representative raw-lane
  equivalent and the point where the degradation ladder intentionally drops
  lower

Success condition:

- an AI should be able to stay inside the blessed pleasant lane for the
  overwhelming majority of ordinary WORTH authoring without needing to study
  raw transition-type names first

### Authoring model

The intended ergonomic direction is one obvious entrypoint plus semantic
progression verbs.

Representative direction:

```rust
let executed = recipe("payload")
    .resolve_with(resolution_authority, 8_u8)
    .lower_with(lowering_capability)
    .ready_with(readiness_authority, "runtime admission")
    .execute();
```

Important constraints:

- the entrypoint must remain explicit and grepable
- each verb must map to one already-closed semantic transition boundary
- the chain must not mint stronger forms through convenience alone
- checked, trust-boundary, and family-composition variants must remain in the
  same grammar one step longer before requiring raw substrate escape
- the result of the chain is already the usable proof-bearing form
- the common lane does not rely on hidden global state or magical ambient
  authority

### Checked outcome model

The pleasant checked lane must not collapse the already-closed Milestone 4
topology into `Result`-shaped success or failure folklore.

Preferred direction:

- expose a narrow pleasant-lane outcome view such as `ProofOutcome<T>` or an
  equally explicit alternative
- keep it as a view or facade over `TransitionOutcome`, not as a replacement
  semantic engine
- preserve exact `Denied`, `Deferred`, `Stale`, `RebindRequired`, `Failed`,
  and success topology
- add narrow inspectors that make ordinary branching easier without flattening
  meaning

Representative direction:

```rust
let outcome = recipe("payload")
    .try_resolve(resolution_gate)
    .try_lower(lowering_readiness);

match outcome.kind() {
    ProofOutcomeKind::Success => { /* ... */ }
    ProofOutcomeKind::Stale => { /* ... */ }
    ProofOutcomeKind::RebindRequired => { /* ... */ }
    ProofOutcomeKind::Denied => { /* ... */ }
    ProofOutcomeKind::Deferred => { /* ... */ }
    ProofOutcomeKind::Failed => { /* ... */ }
}
```

Normative consequence:

- no pleasant checked API may return plain `Result` if doing so would collapse
  substrate topology
- the common branch questions must become easier, but the full semantic
  topology must remain recoverable

### Degradation ladder

The DX lane must degrade one step at a time:

1. canonical happy path
2. checked progression path
3. explicit trust-boundary or family-boundary path
4. raw substrate escape hatch

Not:

1. pleasant happy path
2. immediate fall to raw transition plumbing

Representative direction:

```rust
let executed = recipe("payload")
    .resolve_with(resolution_authority, 8_u8)
    .lower_with(lowering_capability)
    .ready_with(readiness_authority, "runtime admission")
    .execute();
```

then:

```rust
let outcome = recipe("payload")
    .try_resolve(resolution_gate)
    .try_lower(lowering_readiness)
    .try_ready(readiness_gate);
```

then:

```rust
let resumed = lowered
    .bridge_trust_boundary()
    .readmit_with(readmission_authority, 19_u16)
    .ready_with(readiness_authority, "runtime admission");
```

then finally raw explicit surfaces when the domain really needs them.

### Scoped-default model

The pleasant lane should support explicit repeated progression posture without
repeating the same witnesses and capability lanes at every local step.

Representative direction:

```rust
let flow = proof_flow()
    .resolution_authority(resolution_authority)
    .lowering_capability(lowering_capability)
    .readiness_authority(readiness_authority);

let executed = flow
    .recipe("payload")
    .resolve(8_u8)
    .lower()
    .ready("runtime admission")
    .execute();
```

Rules:

- shared defaults must be explicit and typed
- local overrides must remain possible
- the DX layer must make it visible which progression lanes were inherited and
  which were overridden
- defaults must never permit hidden progression if the raw lane would have
  required an explicit witness
- defaults must never rely on ambient global state, thread-local state, or
  hidden task-local state to satisfy witness-bearing progression

### Grouped read model

DX hardening must not stop at authoring compression.

The product is still read-path heavy. Ordinary consumers repeatedly need to
know:

- what stage or wrapper this form is in
- what basis posture currently applies
- whether the strong basis is still available
- what kind of non-success occurred
- what family actions or joined payload lanes exist

The pleasant lane should therefore add grouped read helpers where they can stay
cost-honest and semantically narrow.

Representative direction:

```rust
let stage = executed.stage();
let kind = outcome.kind();
let basis_posture = executed.basis_posture();
let has_strong_basis = executed.has_strong_basis();
```

Normative consequence:

- declaration DX without read-path DX is incomplete
- grouped reads must stay honest about cost and must not materialize richer
  diagnostics or replay artifacts accidentally
- grouped reads must not become a back door for `worth-foundational`-style
  forensic, history, lineage, provenance, or support artifact expansion

### Canonical before / after

Before:

```rust
let resolved = ResolveRecipeTransition.transition(
    unresolved,
    RecipeResolutionContext::new(8_u8, resolution_authority),
);
let lowered = LowerRecipeTransition::new(lowering_capability)
    .transition(resolved.into_value())
    .into_value();
let ready = AdmitExecutionReadyRecipeTransition.transition(
    lowered,
    ExecutionReadinessContext::new("runtime admission", readiness_authority),
);
let executed = ExecuteReadyRecipeTransition.transition(ready.into_value()).into_value();
```

After:

```rust
let executed = recipe("payload")
    .resolve_with(resolution_authority, 8_u8)
    .lower_with(lowering_capability)
    .ready_with(readiness_authority, "runtime admission")
    .execute();
```

The milestone is only successful if the second form lowers to the same raw
proof-bearing truth as the first form while remaining easier to read in normal
WORTH domain code.

## Phases

### Phase 1: Canonical Entrypoints And Prelude Lock

Purpose:

- stop making callers discover the crate through raw module nouns
- define one obvious entrypoint per major authoring family

This phase must ship:

- a deliberate `prelude` or equivalent blessed import lane
- one explicit constructor or alias for common recipe authoring such as
  `recipe(...)`
- one explicit constructor lane for common fixed-shape authoring such as:
  - `pair(...)`
  - `non_empty(...)`
- one explicit constructor lane for family-local identity such as:
  - `sym(...)`
  - `member(...)`
- clear documentation of the pleasant lane versus the raw lane
- proof that the blessed import lane is additive guidance, not a second naming
  universe that competes with the raw facade

Phase 1 gate:

- no later phase begins until a new consumer can discover the intended
  entrypoints without reading milestone docs or raw tests

### Phase 2: Verb-First Canonical Progression

Purpose:

- replace noun-heavy transition composition in the common lane with
  verb-first progression that still lowers into the same semantic substrate

This phase must ship:

- canonical progression verbs for the common recipe lane:
  - `.resolve_with(...)`
  - `.lower_with(...)`
  - `.admit_with(...)`
  - `.ready_with(...)`
  - `.execute()`
- canonical grouped helpers for common straight-line combinations where the raw
  lane already has closed semantics
- narrow grouped inspectors for common proof-local inspection needs such as:
  - `.stage()`
  - `.basis_posture()`
  - `.has_strong_basis()`
- exact lowering equivalence with the existing raw transition surfaces

Phase 2 gate:

- no later phase begins until the pleasant straight-line lane and the raw lane
  can be certified as semantically identical under the current proof substrate

### Phase 3: Checked And Boundary-Explicit Degradation

Purpose:

- keep the weird case inside the same grammar one step longer instead of making
  the user fall straight from pleasant authoring into raw plumbing

This phase must ship:

- checked progression verbs such as:
  - `.try_resolve(...)`
  - `.try_lower(...)`
  - `.try_admit(...)`
  - `.try_ready(...)`
  - `.try_execute()`
- a narrow pleasant checked-outcome view that preserves full substrate topology
  while making ordinary branching easier
- explicit trust-boundary resumption verbs such as:
  - `.bridge_trust_boundary()`
  - `.readmit_with(...)`
  - `.rebind_with(...)`
- explicit downgrade verbs that remain semantically precise:
  - `.downgrade_to_stale...`
  - `.downgrade_to_rebind...`
  - `.downgrade_to_authority_revalidation...`
- a named degradation ladder in docs and tests

Phase 3 gate:

- no later phase begins until hostile checked, stale, rebind, and
  trust-boundary flows can stay inside the pleasant grammar without semantic
  collapse

### Phase 4: Composition And Family Authoring Grammar

Purpose:

- make fixed-arity composition and same-family lifecycle authoring materially
  nicer without inventing a second graph engine

This phase must ship:

- canonical helper constructors for family lifecycle intent such as:
  - `create(...)`
  - `rewrite(...)`
  - `supersede(...)`
  - `retire(...)`
- clear type- and doc-level signaling that these helpers construct family
  lifecycle intent, not already-lowered or authoritative family programs
- a canonical fixed-arity family entry shape such as:
  - `family_pair(...)`
  - or an equally explicit alternative
- a canonical lowering finalizer such as `.lower_by(...)`
- pleasant ready-join helpers that preserve exact ready-lane and non-success
  semantics

Phase 4 gate:

- no later phase begins until family and fixed-arity composition authoring are
  shorter while still preserving symbolic-vs-authoritative law, canonical
  ordering responsibility, and cost honesty

### Phase 5: Scoped Defaults And Grouped Reads

Purpose:

- remove repeated witness and capability posture for common local flows
- make the first read path as pleasant as the first write path

This phase must ship:

- explicit scoped progression-default surfaces
- deterministic inheritance and override semantics for repeated witness or
  capability posture
- visible carriage of inherited progression posture so generated code shows
  what defaults are being relied on
- hostile proof that scoped defaults cannot permit a progression the raw lane
  would have rejected
- grouped read helpers for common inspection needs such as:
  - outcome kind
  - common family or join summaries
- proof that grouped reads do not force hidden rich-path work

Phase 5 gate:

- the pleasant lane is not considered complete until common reads are also
  compressed and cost-honest

### Phase 6: Escape Hatch, Certification, And Teaching Closeout

Purpose:

- close the DX hardening work without trapping the crate in one authoring style

This phase must ship:

- a clean raw-substrate escape hatch that remains first-class and documented
- hostile parity tests between pleasant and raw lanes
- hostile compile-fail coverage proving the pleasant lane does not overclaim
  capability
- codegen or counter honesty proof for representative pleasant-lane hot paths
- representative compile-pass workflow certification for:
  - happy-path recipe progression
  - checked progression
  - trust-boundary readmission
  - fixed-arity ready join
  - deterministic family lowering
- docs that teach the pleasant lane first and the raw lane second
- docs and internal migration notes that teach raw-to-pleasant translation for
  representative common flows

Phase 6 gate:

- the milestone is not closed until the pleasant lane and the raw lane converge
  exactly under hostile certification and the docs teach one obvious default
  path without weakening the escape hatch

## Must Ship

- one blessed import and entrypoint story
- one canonical pleasant recipe progression lane
- one canonical checked and trust-boundary-aware degradation ladder
- one narrow pleasant checked-outcome view that preserves substrate topology
- pleasant fixed-arity and family-composition authoring helpers
- explicit scoped defaults for repeated progression posture
- grouped read-path helpers for common inspection needs
- a documented raw-substrate escape hatch
- hostile parity, compile-fail, and hot-path honesty proof for the pleasant
  lane

## Must Preserve

- the raw facade as the semantic authority
- exact proof-bearing progression law from Milestones 1 through 6
- explicit trust-boundary crossing
- explicit stale, rebind, and authority-revalidation distinction
- explicit symbolic-versus-authoritative family identity
- exact checked failure topology
- zero-cost and cost-honest hot-path posture
- the ownership split between `worth-proof`, `worth-foundational`, and domain
  crates
- one strict pleasant-lane naming grammar that remains predictable for AI
  consumption

## Required Named Proof Families

- `The Pleasant And Raw Happy Path Equivalence Test`
- `The Pleasant And Raw Checked Progression Equivalence Test`
- `The Pleasant Checked Outcome Topology Preservation Test`
- `The Pleasant Trust-Boundary Resume Honesty Test`
- `The Pleasant Freshness And Rebind Boundary Test`
- `The Pleasant Fixed-Arity Composition Equivalence Test`
- `The Pleasant Family-Lowering Equivalence Test`
- `The Pleasant Scoped Defaults Inheritance Honesty Test`
- `The Pleasant Grouped Read Cost-Honesty Test`
- `The Pleasant Lane Capability Overclaim Compile-Time Boundary Test`
- `The Pleasant Lane Documentation Default-Path Audit`

## Acceptance Evidence

This milestone is complete only when `worth-proof` can prove:

- the pleasant straight-line lane lowers to the same semantic substrate as the
  raw lane
- the checked pleasant lane preserves the same denial, deferment, stale,
  rebind, and failure topology as the raw lane
- the pleasant checked-outcome view remains a view over substrate topology
  rather than a flattened replacement
- trust-boundary resume helpers preserve explicit weakening and explicit
  readmission semantics
- fixed-arity and family-composition pleasant helpers preserve the same
  symbolic, authoritative, and canonical-order semantics as the raw lane
- scoped defaults reduce repetition without introducing hidden ambient
  progression
- grouped read helpers remain honest about cost and do not materialize richer
  work accidentally
- compile-time boundaries still reject illegal progression and capability
  overclaim from the pleasant lane
- representative workflow docs and representative workflow certification
  compile-pass lanes converge on the same blessed public DX story
- the docs teach one obvious pleasant lane while preserving the raw lane as the
  explicit escape hatch
- representative raw-to-pleasant translation guidance exists for ordinary crate
  adoption and review work

## Architectural Notes

- the DX layer should likely live as a separate facade-oriented subsystem over
  the existing raw facade rather than as a rewrite of the semantic substrate
- helper constructors and semantic verbs should be additive lowering surfaces,
  not replacements for the raw types
- extension traits should be preferred by default where they can express the
  pleasant lane cleanly without inventing a second semantic type hierarchy
- grouped reads should prefer narrow summary views over magical state fusion
- grouped reads should terminate at proof-local summaries and must not become a
  covert descriptive-boundary system
- the docs should teach a migration of thought:
  start from the pleasant lane, stay in it through the common and weird cases
  as long as the semantics remain obvious, and drop lower only when the domain
  truly earns it
- the docs should present the raw lane early as the honest escape hatch, not as
  buried advanced trivia

## Sequencing Notes

This milestone belongs after substrate semantic closeout because:

- the pleasant lane must lower into already-closed proof law rather than
  guessing at still-moving semantics
- trust-boundary, checked-topology, and family-lowering distinctions are now
  concrete enough to wrap honestly instead of hand-waving

This milestone belongs before broad cross-crate teaching and migration because:

- real adoption pressure will choose whatever public surface is easiest to use
- if the pleasant lane does not exist before those migrations, domain crates
  will either keep inventing local sugar or standardize on the noun-heavy raw
  lane

Current judgment:

- treat this as a new product-hardening milestone over an already-serious
  substrate
- keep the raw lane as the semantic anchor
- make the pleasant lane the default recommendation for real WORTH authoring
