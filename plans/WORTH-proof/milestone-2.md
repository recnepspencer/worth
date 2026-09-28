# Milestone 2 Engineering Spec: Sealed Minting And Witness Authority

> **Status:** Closed
>
> **Closeout:** [milestone-2-closeout.md](./milestone-2-closeout.md)
>
> **Roadmap parent:** [worth_proof_roadmap.md](./WORTH_proof_roadmap.md)
>
> **Vision parent:** [worth_proof_vision.md](./WORTH_proof_vision.md)
>
> **Test requirements:** [test-requirements.md](./test-requirements.md)
>
> **Adjacent milestone closeout:** [milestone-1.md](./milestone-1.md)
>
> **Impacted later milestones:**
> - `Milestone 3` (`Assumption, Freshness, And Staleness Law`)
> - `Milestone 4` (`Transition And Outcome Algebra`)
> - `Milestone 5` (`Lowering And Execution Readiness`)
> - `Milestone 6` (`Static Fork And Join Progression`)
>
> **Primary architectural driver:** make stronger proof-bearing forms
> mechanically unWORTHable now that Milestone 1 has frozen the carrier family

## Goal

Seal proof minting and authority-bearing progression so that only trusted
proving boundaries can construct stronger proof-bearing forms, while preserving
the zero-cost static carrier grammar established in Milestone 1.

## Why This Milestone Exists

Milestone 1 deliberately froze the core carrier family before full sealing.
That was the right order, but it left one explicit debt:

- `Proof<P>` is still WORTHable by ordinary callers

As long as that remains true, the substrate still relies partly on caller
discipline rather than fully on compiler-enforced progression law.

That weakness is deeper than one public constructor:

- WORTHd proof markers can be threaded into later carrier forms by local
  convenience if later milestones do not harden the minting story now
- authority-bearing APIs will drift into ad hoc crate-local witness patterns if
  the shared substrate does not define one canonical witness language
- symbolic -> resolved -> lowered -> admitted progression will keep using
  bespoke sealing and privacy patterns in each domain crate
- later stale/re-admission and transition milestones will be building on a
  substrate that still cannot tell the difference between "proof exists" and
  "proof type was constructed"

Milestone 2 therefore exists to solve the next hard structural problem:

- who may mint stronger forms
- how that authority is represented
- how external callers are prevented from forging proof-bearing progression
- how domain crates get one canonical sealing/witness pattern instead of local
  reinvention

## Hard Part

The hard part is not making constructors private.

The hard part is preserving all of these at once:

- zero-cost proof-bearing forms
- one public facade
- domain-controlled proving functions
- unWORTHable stronger types
- witness-based authority that compiles away
- honest symbolic/resolved/admitted progression boundaries
- no fallback to ambient runtime permission bags or stringly registries

The design fails if:

- proofs are merely "harder to WORTH" but still WORTHable by ordinary callers
- sealing depends on runtime registries or hidden global state
- witness authority is represented by runtime booleans, dynamic permission
  checks, or generic bags when a static witness should suffice
- domains are forced into one giant magic macro surface to get sealing
- recipe progression remains structurally WORTHable by convenience constructors
- full sealing requires replacing Milestone 1 carrier shapes instead of
  extending them

## Explicit Assumptions

- Milestone 1 core carrier law and closure record remain authoritative.
- `worth-proof` still owns progression law only; it does not own diagnostics,
  lineage, provenance, execution receipts, or runtime authorization services.
- domain crates remain the semantic authorities for what constitutes proof,
  resolution, eligibility, or execution legality.
- Milestone 2 may introduce crate-private constructors, sealed traits, witness
  tokens, and canonical minting boundaries, but must not introduce a dynamic
  runtime proof registry.
- full basis drift, freshness, re-admission, and downgrade law remain
  Milestone 3 concerns, though Milestone 2 must not block them.

## Governing Summaries

- `MENTALITY.md`
  The most important thing it protects is solving the hard enforcement boundary
  before more features pile on top. Milestone 2 therefore hardens minting and
  witness authority now rather than letting later milestones build on
  convention-dependent proof forms.
- `arch_laws.md`
  The most important thing it protects here is that proof-bearing types must
  encode what has actually been established, and construction must be
  compiler-enforced. Laws 9, 11, 24, 37, 39, 40, and 41 shape this milestone
  most strongly.
- `perf_laws.md`
  The most important thing it protects is that sealing and authority do not
  smuggle runtime lookup, coordination, or hidden allocation into hot paths.
  Witnesses must compile away.
- `domain_laws.md`
  The most important thing it protects is responsibility clarity. Sealing,
  witness authority, recipe-stage admission, and compile-fail boundary support
  must be decomposed cleanly rather than collapsed into one vague privacy file.
- `worth_proof_vision.md`
  The most important thing it protects is the identity of `worth-proof` as a
  static progression-law substrate: sealed proof minting, witness-based
  authority, symbolic-to-resolved progression, and deterministic composition
  boundaries must remain compile-time-first and domain-owned in meaning.
- `worth_proof_roadmap.md`
  The most important thing it protects is sequencing. Milestone 2 exists
  immediately after Milestone 1 because sealing weak wrappers later would let
  WORTHd progression patterns spread first.
- `worth-proof` test requirements
  The most important thing it protects is that sealing and witness authority
  must be proven by hostile compile-fail and WORTH-attempt suites, not by
  "private enough" inspection.
- `milestone-1.md`
  The most important thing it protects is the inherited closure boundary:
  Milestone 2 must harden the admitted carrier family from Milestone 1 rather
  than inventing a second core artifact story.

## Adversarial Constraint

The milestone must survive the following hostile condition:

> Several WORTH subsystems with different proving functions, authority models,
> and symbolic/resolved progression paths must be able to mint stronger
> proof-bearing forms only through trusted proving boundaries, such that
> ordinary callers cannot WORTH those forms, witness-bearing authority compiles
> away on hot paths, and later phases still consume the exact Milestone 1
> carrier family rather than a second wrapper system.

The design fails if:

- external callers can still synthesize `Proof<P>` or equivalent stronger
  proof-bearing forms directly
- domains have to pass untyped booleans, strings, or runtime objects to stand
  in for static authority witnesses
- recipe-stage progression can still skip unresolved -> resolved -> lowered ->
  admitted by convenience construction
- privacy hardening forces callers to abandon the Milestone 1 facade and reach
  into deep modules
- witness carriage changes the runtime representation or allocation behavior of
  hot-path forms materially relative to Milestone 1
- sealing patterns differ so much between artifact, collection, and recipe
  forms that domain crates still need bespoke local sealing scaffolding

## Product Decision Lock

- the Milestone 1 carrier family remains the one canonical progression carrier
- stronger proof-bearing forms must be minted only by trusted proving
  boundaries
- witness-based authority is the canonical static expression for binary,
  authority-granted preconditions that are statically representable
- ordinary callers may observe stronger forms through read-only surfaces, but
  may not construct them directly
- recipe-stage progression that claims stronger resolved/lowered/admitted status
  must be sealable through the same substrate rather than bespoke local
  constructor folklore
- the public facade remains the only public entry surface for the crate
- witness and sealing machinery must remain zero-cost after monomorphization
- the crate does not introduce runtime proof registries, runtime permission
  bags, or dynamic policy evaluators to solve sealing

Normative consequence:

- any implementation that leaves stronger proof-bearing forms externally
  constructible is out of spec
- any implementation that requires runtime permission lookup where a static
  witness would suffice is out of spec
- any implementation that hardens minting by introducing a second parallel
  carrier family is out of spec

## Required Contracts

### Sealed Minting Rule

Every stronger proof-bearing form admitted by this milestone must be mintable
only through trusted crate-defined proving boundaries.

Required vocabulary:

- sealed-construction boundary
- minting witness or token surface
- crate-private constructor posture
- trusted proving module posture

Rules:

- public callers may not synthesize stronger forms through struct literals,
  `Default`, public `new`, or convenience conversion APIs
- the proving boundary may remain domain-defined later, but the substrate must
  provide one canonical sealing pattern now
- sealing must apply to the representative core proof-bearing surfaces admitted
  by Milestone 1, not only to one demonstration type

### Witness Authority Rule

Authority- or capability-gated progression that is statically representable
must use witness-bearing types rather than ambient runtime discipline.

Required vocabulary:

- zero-sized authority witness
- zero-sized capability witness
- witness-bearing constructor or transition input
- denial posture for absent witness paths

Rules:

- witness-bearing APIs must expose their authority requirement in the type
  signature
- witness carriage must not add hidden runtime lookup or allocation
- witness semantics must remain distinct from descriptive provenance or
  diagnostics, which belong outside this crate

### Witness Issuance Rule

Witnesses themselves must be unWORTHable by ordinary callers.

Required vocabulary:

- trusted witness issuer boundary
- unWORTHable witness construction posture
- facade-visible witness consumption surface
- crate-private witness minting surface

Rules:

- a zero-sized witness with an externally callable constructor does not satisfy
  this milestone
- ordinary callers may present a witness to a gated API only if a trusted
  proving boundary issued it
- witness issuance must preserve the same facade discipline as proof minting;
  the user of a witness may not need deep internal access to the crate
- witness issuance may remain representative in Milestone 2, but whatever ships
  must already be mechanically unWORTHable

### Recipe Admission Rule

Symbolic, resolved, lowered, and admitted recipe-stage forms must be hardenable
through the same substrate rather than left to bespoke local sealing patterns.

Required vocabulary:

- unresolved recipe or symbolic stage
- resolved stage
- lowered stage
- admitted stage
- stage-specific minting boundaries

Rules:

- later stages may not be constructible directly by ordinary callers
- stage progression must remain explicit rather than implied by comments or
  module naming
- Milestone 2 may ship representative recipe-stage substrate surfaces without
  shipping the full later transition algebra

### Compile-Time Boundary Rule

The highest-risk WORTHd-construction and witness-misuse boundaries in this
milestone must be compiler-enforced rather than doc-only.

Required compile-time posture:

- WORTHd stronger proof-bearing construction fails to compile
- witness-required APIs reject callers without the witness
- recipe stages reject direct skip-construction by ordinary callers
- the facade does not leak internal minting or witness machinery unintentionally

### Performance-Shaping Rule

Sealing and witness authority must preserve the zero-cost posture of the
Milestone 1 carrier family.

Required performance-shaping surfaces:

- witness-bearing proof forms
- sealed stronger proof forms
- representative recipe-stage forms

Rules:

- sealed minting must not force runtime registries or broad coordination
- witness-bearing paths must not introduce mandatory allocation
- representative sealed/witness forms must remain size/alignment/drop-honest
  for the admitted scope

## Scope

### In Scope

- sealing patterns for Milestone 1 stronger proof-bearing surfaces
- crate-private versus public constructor hardening
- witness/token patterns for proof minting
- zero-sized authority/capability witness surfaces
- representative recipe-stage sealing patterns
- compile-fail support for WORTHd construction and witness misuse
- facade hardening where needed to keep internal minting surfaces private
- module topology for sealing and witness authority infrastructure

### Explicitly Out Of Scope

- full basis drift, freshness, or downgrade law
- full transition trait algebra
- full lowering/execution readiness law
- diagnostics or provenance for authority decisions
- runtime authorization services or policy evaluators
- cross-crate migrations beyond what is required to validate the substrate shape

## Phases

### Phase 1: Sealed Core Proof Surfaces

Harden the Milestone 1 core proof-bearing surfaces so stronger forms are no
longer externally WORTHable.

Must ship:

- sealed construction posture for `Proof<P>` or its replacement surface
- sealed construction posture for stronger proof-bearing collection forms
- explicit crate-internal minting paths for representative stronger forms
- preserved public read-only access for already-admitted observation surfaces

Implementation guidance:

- start with the narrowest representative stronger forms from Milestone 1:
  - `Proof<P>`
  - `CanonicalVec`
  - `UniqueVec`
  - `DisjointPair`
- harden the minting boundary first before adding witness ergonomics
- decide in this phase which constructors remain crate-private, which become
  fully sealed, and which are intentionally left as explicit debt; do not defer
  that posture discovery to later phases
- preserve the Milestone 1 facade while reducing externally callable minting
  paths
- avoid giant "sealing.rs" buckets; split core minting posture by real
  responsibility

This phase is complete only when external callers can no longer mint the core
stronger forms directly.

### Phase 2: Witness Vocabulary And Authority Surfaces

Introduce the canonical witness language for statically representable
authority-gated progression.

Must ship:

- zero-sized authority witness surface
- zero-sized capability witness surface
- witness-bearing progression examples on representative stronger forms
- witness-specific facade exports for the admitted public surface

Implementation guidance:

- keep witness semantics separate from proof markers; witnesses represent who
  may mint or consume, not what has been proven about the payload
- freeze the first witness consumption surface through the facade during this
  phase; do not let witness-bearing APIs leak first through deep modules and
  "clean it up later"
- design the witness vocabulary so later milestones can reuse it for transition
  admission and execution readiness without reinterpretation
- choose one representative trusted witness issuer path and harden it fully;
  avoid shipping several half-different issuer stories in first ship
- prove witness carriage is zero-cost before broadening the surface

This phase is complete only when a representative authority-gated API is
statically witness-bearing rather than ambiently guarded.

### Phase 3: Recipe-Stage Sealing

Apply the shared sealing/witness patterns to representative staged recipe
progression so unresolved/resolved/lowered/admitted progression becomes
unWORTHable too.

Must ship:

- representative recipe-stage type surfaces
- representative stage-specific minting boundaries
- explicit unresolved -> resolved -> lowered -> admitted progression posture
- compile-time rejection of direct stage skipping by ordinary callers

Implementation guidance:

- keep the recipe surface intentionally representative and substrate-level;
  full transition algebra still belongs later
- avoid inventing domain semantics here; encode only progression law and stage
  distinction
- preserve extension room for later lowering/readiness milestones

This phase is complete only when the substrate can honestly express sealed
recipe-stage progression rather than only sealed one-off proof wrappers.

### Phase 4: Hostile Certification And Closure

Prove that sealed minting and witness authority are mechanically honest and
inherit Milestone 1 without regression.

Must ship:

- hostile compile-fail coverage for WORTHd minting attempts
- hostile compile-fail coverage for witness misuse
- representative sealed/witness type-shape and codegen-honesty evidence
- closure record of what later milestones may now assume about minting and
  authority

Implementation guidance:

- the certification suite should map back to the contract sections above rather
  than one omnibus privacy test
- certify one representative lane each for:
  - sealed proof minting
  - sealed proven-collection minting
  - witness-bearing authority path
  - recipe-stage skip denial
- write the closure record so Milestone 3 and Milestone 4 can reuse it
  directly

This phase is complete only when the milestone has machine-checkable evidence
that WORTHd minting is uncallable and witness-bearing authority remains
zero-cost for the admitted scope.

## Must Ship

- one canonical sealing story for stronger proof-bearing forms
- one canonical witness vocabulary for static authority-gated progression
- one representative sealed recipe-stage progression story
- compile-fail coverage proving WORTHd minting and witness misuse are
  uncallable
- explicit zero-cost/codegen evidence for representative sealed and
  witness-bearing forms
- milestone-local implementation notes that map directly onto the crate-level
  certification bar in `worth-proof/test-requirements.md`

## Must Preserve

- the Milestone 1 carrier family and facade boundary
- zero-cost hot-path posture after monomorphization
- no mandatory heap allocation
- no mandatory dynamic dispatch
- no runtime permission bag or proof registry
- no drift of `worth-proof` into policy engines, diagnostics, provenance, or
  execution semantics
- clean extension room for Milestone 3 freshness law and Milestone 4 transition
  algebra

## Acceptance Evidence

Milestone 2 is complete only when `worth-proof` satisfies the named milestone
suite:

- `Sealed Minting And Witness Authority Test`

Required machine-checkable outputs:

- `compile_fail_bundle`
- `proof_shape_digest`
- `failure_digest`
- `codegen_honesty_report`
- `residual_debt_report`

Milestone-specific proof obligations:

- ordinary callers cannot mint stronger proof-bearing forms directly
- witness-required APIs are uncallable without the witness
- ordinary callers cannot mint the witness itself directly
- witness-bearing authority remains statically visible in the signature
- recipe-stage progression cannot skip unresolved/resolved/lowered/admitted
  boundaries by convenience construction
- representative witness-bearing and sealed forms do not introduce hidden
  dynamic lookup, hidden virtual dispatch, or mandatory allocation
- the facade does not leak internal minting machinery
- the suite includes hostile compile-fail coverage for WORTHd proof minting,
  WORTHd witness minting, witness misuse, and stage-skipping attempts
- the suite publishes explicit residual debt for any intentionally deferred
  sealing or witness surfaces

Milestone 2 is not closed by "constructors became private" or "the witnesses
are zero-sized by inspection" arguments.

## Architectural Notes

- Milestone 2 is about mechanical authority of construction, not about full
  transition algebra yet.
- Witnesses and proofs are distinct concepts. A witness says who may mint or
  consume. A proof says what has been established.
- This milestone should prefer small explicit representative recipe-stage
  surfaces over one giant generic recipe framework.
- If sealing a surface requires a second parallel carrier family, the design is
  wrong and must be revised.

## Implementation Topology

This milestone should extend the Milestone 1 topology rather than replace it.

Preferred additions:

```text
crates/worth-proof/src/
  facade.rs
  artifact/
    minting.rs
  proof/
    minting.rs
    witnesses.rs
  recipe/
    mod.rs
    stages.rs
    minting.rs
```

This is not a forced final topology, but the ownership boundaries are
intentional:

- `artifact/minting.rs`
  - artifact-level trusted minting posture
- `proof/minting.rs`
  - proof-specific sealing and constructor visibility
- `proof/witnesses.rs`
  - authority/capability witness vocabulary
- `recipe/*`
  - representative staged recipe progression and sealing

The milestone should avoid:

- one giant mixed `witnesses_and_minting.rs`
- pushing recipe-stage law into unrelated artifact or proof files
- exposing minting internals publicly instead of through the facade

## Sequencing Notes

- This milestone belongs immediately after Milestone 1 because the carrier
  family is now frozen and the next highest-risk weakness is WORTHd minting.
- Milestone 3 depends on this milestone because stale/downgraded/re-admitted
  forms are only meaningful if stronger forms were mechanically trustworthy to
  begin with.
- Milestone 4 depends on this milestone because typed transitions need sealed
  outputs and witness-bearing admission paths.
- Milestone 5 depends on this milestone because execution-ready forms are
  authority-bearing stronger forms and should inherit one canonical minting
  story rather than invent a special-case path.
