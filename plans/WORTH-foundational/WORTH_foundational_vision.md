# WORTH Foundational Vision

## Thesis

`worth-proof` defines how truth-bearing artifacts progress.
`worth-foundational` defines the shared language those artifacts speak.

WORTH already converges on the same deep structural categories across crates:

- aspects and aspect patches
- identities, keys, handles, and basis ids
- diagnostics and explanations
- decisions and structured outcomes
- lineage and provenance
- digests and canonicalization
- equivalence and reuse contracts
- reports, summaries, receipts, and artifacts
- support and certification artifacts
- composition-family resolution and lifecycle artifacts
- field, path, and locator references
- profiles, policies, and support matrices
- performance contracts and layout choices
- boundary-facing canonical value encodings

Today those surfaces exist, but each crate often speaks its own local dialect.
The result is not merely duplicate code. It is duplicate ontology:

- two crates mean almost the same thing with different types
- one crate materializes a report as a summary while another calls the same
  shape an artifact
- one subsystem carries diagnostics richness through a profile while another
  encodes similar policy in a different envelope
- canonical values, aspect state, and boundary digests are repeated with small
  semantic drift

`worth-foundational` exists to stop that drift.

It is not a runtime.
It is not a storage engine.
It is not a planner.
It is not a proof kernel.

It is the shared semantic and boundary-contract crate for WORTH.

## What This Crate Is For

`worth-foundational` exists for every WORTH subsystem that needs a canonical,
cross-crate answer to questions like:

- what is a boundary-safe typed value?
- what is an aspect key, aspect state, and aspect patch?
- what is a diagnostic, explanation, or denial artifact?
- what is a structured decision, outcome, or mismatch surface?
- what is provenance versus lineage?
- what is an identity versus a handle versus a basis id?
- what is the equivalence basis for reuse, suppression, parity, or
  certification sameness?
- what is a report versus a summary versus an artifact versus a receipt?
- what is a lowered plan boundary artifact versus an execution receipt?
- what is the canonical descriptive language for a same-family symbolic
  composition program, its resolution map, and its lifecycle outcomes?
- what is the canonical locator/path language for pointing at aspects, fields,
  diagnostics, provenance sources, or mismatches?
- what profile controls richness, retention, or support posture?
- what digest basis makes a boundary reproducible and auditable?
- how do we describe layout and performance choices without forcing one memory
  representation?
- how do we centrally remove optional history, replay, lineage, provenance, or
  forensic richness from hot paths without refactoring leaf call sites?

It is meant to support:

- `worth-relational` truth-state semantics, lineage, schema, durability-facing
  artifacts, and aspect-native mutation/publication boundaries
- `worth-query` query artifacts, support profiles, delivery contracts, result
  shapes, diagnostics, and policy-aware summaries
- `worth-signal` diagnostics tiers, runtime policies, trace/provenance
  materialization, replay views, and aspect-aware incremental boundaries
- `worth-store` compatibility evidence, durable receipts, diagnostics digests,
  lineage support artifacts, and maintenance/admission reports
- future Aspec-native WORTH surfaces that must no longer depend on JSON-shaped
  compatibility payloads as the default boundary language

The technical thesis is the same across all of them:

- shared semantics should be defined once
- boundary vocabulary should be canonical
- internal layout may differ, but materialized meaning must not drift
- profile-driven richness should be standardized
- profile-driven descriptive elision should be centrally enforceable
- canonical values should be aspect-native and digest-honest
- shared boundary categories must not blur authoritative truth, derived
  artifacts, and descriptive/forensic surfaces
- shared sameness, identity, outcome, and locator laws should be explicit
  enough to support reuse, parity, certification, and debugging without
  crate-local folklore

## Mission

`worth-foundational` exists to make WORTH speak one shared language without
forcing every crate into one runtime representation.

It must answer these questions as native crate responsibilities:

- What is the canonical WORTH value vocabulary for aspect-native data?
- How are aspect keys, authoritative aspect state, and aspect patches encoded
  so that crates can exchange them without semantic drift?
- What information must a diagnostic, explanation, lineage record, provenance
  record, or execution receipt carry to be self-describing?
- How does WORTH declare equivalence, reuse, suppression, parity, and mismatch
  bases without each crate inventing its own sameness folklore?
- How are identity, key, handle, and basis-id categories kept distinct even
  when their underlying representation is identical?
- What decision/outcome vocabulary is shared across crates without flattening
  real correctness differences into one fake universal enum?
- How do support and certification artifacts become first-class boundary
  surfaces instead of bespoke exports and test leftovers?
- How does a boundary artifact point at the exact aspect, field, row, source,
  or mismatch locus it is talking about without stringly drift?
- What profile system controls operational, development, forensic, compact,
  standard, extended, support, or certification richness?
- How are reports, summaries, artifacts, and receipts distinguished so the same
  nouns mean the same thing across crates?
- How are canonical digests built so replay, parity, and cross-crate
  certification remain mechanically stable?
- How do we standardize performance and layout vocabulary without standardizing
  away AoS, SoA, AoSoA, or custom internal data layouts?
- How does WORTH replace JSON-default payload thinking with Aspec-native value
  semantics while still allowing compatibility surfaces where transition debt
  remains?

## Architectural Model

### Runtime stack

| Layer | Responsibility | Owns |
| --- | --- | --- |
| `worth-proof` | Proof-bearing progression law | phases, proof composition, witnesses, transitions, staleness law |
| `worth-foundational` | Shared truth vocabulary | aspects, values, diagnostics, lineage, provenance, digests, profiles, boundary contracts, performance vocabulary |
| Domain crates | Domain semantics and execution | truth semantics, planning, storage layout, runtime behavior, effect execution |

### Ownership boundary

`worth-foundational` owns:

- Aspec-native scalar and structural value vocabulary
- aspect-key, aspect-state, and aspect-patch vocabulary
- typed identity/key/handle/basis-id vocabulary
- canonical digest and canonicalization helpers
- equivalence/reuse vocabulary and basis contracts
- proof-adjacent attachments that must compose with `worth-proof` artifacts,
  including shared report, receipt, provenance, and profile surfaces
- diagnostics, explanation, denial, and support-report vocabulary
- structured decision/outcome vocabulary
- lineage and provenance record vocabulary
- artifact/report/summary/receipt/envelope category boundaries
- support and certification artifact vocabulary
- boundary vocabulary for plan-shaped artifacts and execution-shaped receipts
  where WORTH crates need one shared descriptive language
- field/path/locator vocabulary for aspect-native and boundary-facing surfaces
- composition-family resolution maps, lifecycle outcomes, and family receipts
- typed authoritative/derived/projected/support-only boundary categories
- profile vocabulary for richness, posture, and support classes
- performance-contract and layout-choice vocabulary
- materialization contracts for converting crate-local optimized structures into
  canonical boundary shapes

`worth-foundational` does not own:

- proof-bearing progression law
- runtime scheduling or reactive invalidation
- truth mutation semantics
- storage layout decisions
- hot-path memory topology
- execution orchestration
- generic plan execution runtimes
- dynamic plugin graphs
- domain-specific business meaning above shared WORTH vocabulary

Structural rule:

`worth-foundational` standardizes meaning at boundaries, not representation in
hot paths.

## Adversarial Constraint

`worth-foundational` must survive the following hostile condition:

> Several WORTH crates with different memory layouts, execution models, and
> authority boundaries must be able to exchange aspect-native values,
> diagnostics, lineage/provenance artifacts, support profiles, and certified
> boundary digests such that the same semantic thing has one meaning everywhere,
> while each crate retains the freedom to implement its own cost-honest
> internal representation.

If `worth-foundational`:

- forces a universal runtime value bag into hot paths
- standardizes one memory layout for systems that need AoS, SoA, AoSoA, or
  custom storage
- treats JSON-shaped payloads as the primary canonical value language
- collapses distinct artifact categories into one convenience type
- collapses authoritative truth and derived/descriptive surfaces into one
  boundary vocabulary
- standardizes representation where only meaning should be shared
- weakens digest honesty or profile honesty in exchange for convenience

then it has failed.

## Why This Crate Is Different

These are not optional add-ons. They are the capabilities that make
`worth-foundational` strategically different from ordinary shared-utils crates:

- Aspec-native canonical value vocabulary
- authoritative aspect-state and aspect-patch language
- explicit report / summary / artifact / receipt distinctions
- shared diagnostics and explanation ontology
- shared lineage and provenance ontology
- canonical digest-basis and serialization honesty
- profile-driven richness and support posture vocabulary
- performance/layout vocabulary without one forced representation
- boundary materialization contracts that preserve crate-local optimization

If these are treated as miscellaneous helpers, WORTH will keep sharing code
without sharing meaning.

## Principles

1. Shared meaning should be standardized once.
2. Representation should stay crate-local unless unification is cost-honest.
3. Aspects are first-class semantic units, not JSON field folklore.
4. Canonical values must be explicit about width, precision, temporal basis,
   and reference identity.
5. Profile-driven richness must be a first-class cross-crate language.
6. Profile-driven descriptive elision must happen at named boundary seams, not
   by scattering policy branches through domain leaf call sites.
7. Diagnostics, lineage, provenance, and receipts are product contracts, not
   debug leftovers.
8. Reports, summaries, artifacts, and receipts must mean distinct things.
9. Boundary digests must be canonical, stable, and mechanically reproducible.
10. Shared vocabulary must compose with `worth-proof`, not duplicate it.
11. Shared vocabulary must preserve cost honesty for crates with different
    layout strategies.
12. Shared vocabulary must preserve the authority/derivation boundary instead
    of flattening canonical truth, lowered intent, support description, and
    execution receipts into one generic artifact story.
13. Shared sameness, identity, outcome, and locator laws must be explicit
    enough that reuse, parity, certification, and debugging do not devolve
    into crate-local folklore.
14. Expensive descriptive materialization must be modeled as an explicit
    boundary category, not disguised as a cheap view on authoritative truth.
15. Disabling optional descriptive richness must never change authoritative
    outcomes or correctness-critical progression law.

## Foundational Decisions

These are locked architectural decisions:

- WORTH is moving toward an Aspec-native default boundary language rather than
  JSON-default semantics
- canonical values must encode widths, temporal semantics, and reference kinds
  explicitly
- aspect state is authoritative as an ordered map from aspect key to typed
  aspect value
- aspect patches are explicit set/clear structures rather than ad hoc object
  merge folklore
- compatibility JSON may exist as explicit debt or transition support, but it
  is not the long-term canonical truth vocabulary
- profiles are first-class shared structures, not string labels floating around
  in crate-local code
- profiles may disable optional descriptive richness, retention, replay, or
  forensic materialization, but they must not disable correctness-critical
  truth or proof law
- equivalence, suppression, parity, and reuse require explicit shared basis
  vocabulary rather than ad hoc comparator folklore
- equal representation does not imply equal meaning; identity, handles, and
  basis ids remain distinct categories even when storage is identical
- internal data layout remains a crate decision; shared layout vocabulary is
  allowed, shared mandatory representation is not
- diagnostics, lineage, provenance, and receipts are shared ontologies, but
  their storage remains crate-local unless a specific cross-crate artifact
  requires canonicalization
- profile-controlled descriptive elision must be expressible centrally through
  shared foundational policy vocabulary and enforced at boundary materialization
  and retention seams rather than through domain leaf-call-site branching
- `worth-proof` remains the home of progression law; `worth-foundational`
  remains the home of shared truth description
- foundational artifact categories describe boundary meaning only; they do not
  elevate derived or descriptive surfaces into authoritative truth

## How This Vision Drives Engineering

This document is intentionally written so a roadmap can be derived from it.

The derivation rule is:

- each capability pillar below implies concrete shared type surfaces that must
  exist
- each technical role implies constraints that implementation must preserve
- each "what this enables" section implies real cross-crate migrations the
  shared vocabulary must unlock
- if several crates materially mean the same thing but encode it differently
  today, it belongs on the roadmap as unification work here
- if a proposed abstraction would standardize hot-path representation rather
  than shared meaning, it must be rejected or pushed down into crate-local code

In other words:

- the vision says what WORTH-wide semantic vocabulary must be shared
- the roadmap says which shared vocabularies and contracts still must be
  engineered
- later migration work says which crate-local dialects should be retired in
  favor of the shared language

## Capability Pillars

### Aspec-Native Value System

#### Canonical scalar vocabulary

Technical role:
WORTH needs one typed value language for aspects and boundary payloads. That
language must be explicit about integer width, unsignedness, floating
representation, decimal/big-int forms, temporal precision, and reference kinds.

Representative forms include shapes equivalent to:

- null
- bool
- `Int8`, `Int16`, `Int32`, `Int64`
- `UInt8`, `UInt16`, `UInt32`, `UInt64`
- canonical float carriers such as `CanonicalF32` and `CanonicalF64`
- canonical exact-value forms such as decimal / big integer / rational
- string and bytes/content references
- UUID
- `CanonicalDate`, `CanonicalTime`, `CanonicalTimestamp`,
  `CanonicalTimestampTz`
- `EntityRef` and `ContentRef`

What this enables:

- crates stop inventing slightly different payload vocabularies
- cross-crate reports and receipts can carry typed values without JSON drift
- digest and replay parity become more stable because boundary values are
  canonical by construction

#### Aspect-native structural state

Technical role:
The shared language must define what an aspect key is, what authoritative
aspect state is, and what an authoritative aspect patch means.

The canonical shape is:

- `AspectKey`
- `CanonicalAspectStateMap`
- `AuthoritativeRecordAspectState`
- `AuthoritativeRecordAspectPatch`

where patches explicitly separate:

- `set`
- `clear`

and `set` entries dominate overlapping clears.

This should remain explicit even when compatibility bridges still exist.
Compatibility JSON may still appear as transitional debt, but the canonical
meaning of authoritative record payloads should be aspect-native rather than
object-merge folklore.

What this enables:

- `worth-relational` can publish authoritative aspect-native deltas canonically
- `worth-query` can reason about aspect projection and result shaping against
  one shared state language
- `worth-signal` can consume aspect-aware change semantics without crate-local
  translation folklore
- `worth-store` can persist and certify aspect-native artifacts directly

#### Typed identity discipline

Technical role:
WORTH repeatedly carries identities, handles, keys, digests, epochs, and basis
ids whose underlying representations may match while their meanings do not. The
shared crate must standardize the distinction so APIs stop collapsing ontology
into generic ids.

What this enables:

- clearer cross-crate APIs
- fewer bespoke wrapper/newtype patterns
- safer support, lineage, provenance, and receipt artifacts because identity
  kinds remain visible

#### Canonical structural locators

Technical role:
Boundary artifacts need to point at exact aspects, fields, rows, sources, or
comparison loci. The shared crate must define a canonical locator/path language
so diagnostics, provenance, support artifacts, and mismatch reports stop using
incompatible local path folklore.

What this enables:

- diagnostics that point at the same kinds of things the same way across crates
- more precise support and certification artifacts
- easier cross-crate tooling because locators become canonical

### Equivalence And Reuse Vocabulary

#### Explicit equivalence basis

Technical role:
WORTH repeatedly needs to claim that two things are "the same enough" for
reuse, suppression, parity, certification, or mismatch reporting. That is not a
helper concern. It is a semantic contract. The shared crate must define the
vocabulary for equivalence basis, comparison scope, and mismatch explanation.

What this enables:

- `worth-signal` can suppress or reuse with explicit sameness semantics
- `worth-query` can express parity and comparison surfaces without bespoke drift
- `worth-store` can certify durable equivalence and mismatch honestly
- support artifacts can explain why two things were or were not considered the
  same

### Proof-Carrying Boundary Composition

#### Shared attachments for proof-bearing artifacts

Technical role:
`worth-proof` owns progression law, proof witnesses, and transition legality.
But once a proof-bearing artifact crosses a crate boundary, it often needs
shared descriptive surfaces:

- diagnostics
- reports
- receipts
- lineage
- provenance
- profiles
- digest bases

`worth-foundational` exists to define those shared attachments so proof-bearing
artifacts can be described consistently without pulling descriptive ontology
into the proof kernel itself.

What this enables:

- proof-bearing outputs from different crates can expose one shared boundary
  language
- `worth-proof` stays zero-cost and law-focused
- support, replay, and certification surfaces can attach to proof-bearing
  artifacts without each crate inventing a new envelope vocabulary

### Decision And Outcome Vocabulary

#### Structured non-binary outcomes

Technical role:
WORTH often needs more than success or failure. It needs accepted, advisory,
denied, deferred, partial, unsupported, basis-mismatch, and related outcome
families. The shared crate must standardize the vocabulary family without
forcing every crate into one fake universal enum.

What this enables:

- less bespoke result/report/decision scaffolding
- better parity between query, signal, relational, and store support surfaces
- clearer APIs for non-terminal or non-binary outcomes

### Diagnostics And Explanation Vocabulary

#### Shared diagnostic ontology

Technical role:
WORTH needs one shared answer for diagnostic codes, scopes, severity classes,
artifact kinds, and explanation-bearing entries.

What this enables:

- crates can exchange diagnostics without inventing translation layers
- support artifacts and certification bundles can compare like with like
- developers can learn one diagnostic language across WORTH

#### Explanation and denial contracts

Technical role:
A diagnostic is not just an error string. It is a structured explanation of
what happened, what was denied, what remained advisory, and what evidence
supports the decision.

What this enables:

- better parity between query, signal, relational, and store denial surfaces
- richer but standardized support and inspection artifacts
- clearer separation between successful reports and rejected/denied artifacts

### Lineage And Provenance Vocabulary

#### Shared lineage language

Technical role:
WORTH uses lineage in several ways: historical identity evolution, event
records, batch digests, replay linkage, and support artifacts. The shared crate
must define the common nouns even if storage remains crate-local.

What this enables:

- one canonical meaning for lineage artifacts and lineage digest bases
- less drift between durable and in-memory lineage surfaces
- easier certification across relational, query, signal, and store

#### Shared provenance language

Technical role:
WORTH also needs one language for "where did this artifact come from, under
what policy/profile/basis, and through which authority path?"

What this enables:

- common provenance attachments across diagnostics, reports, and receipts
- less local reinvention of source/issuer/profile/basis metadata
- stronger support surfaces and durable audit artifacts

### Boundary Artifact Vocabulary

#### Reports, summaries, artifacts, and receipts

Technical role:
WORTH repeatedly uses these words and needs them to mean distinct things:

- `Summary`: compact derived synopsis
- `Report`: structured boundary explanation of an operation or decision
- `Artifact`: persistent or inspectable shaped output with stable identity
- `Receipt`: proof that an effectful or authority-bearing boundary actually
  happened

Where a crate lowers a plan and later executes it at runtime, the shared
vocabulary must also let the crate describe:

- a plan-shaped artifact that captures what should happen
- the report/receipt surfaces that explain what actually happened

without forcing a generic foundational executor or one shared runtime plan
representation.

What this enables:

- less semantic drift in public surfaces
- easier code review because shared nouns stay stable
- easier migration away from one-off local envelope types

#### Explicit authority and derivation categories

Technical role:
Some boundary surfaces are authoritative truth, others are derived projections,
support-only descriptions, lowered intent, or execution receipts. The shared
crate should name those categories explicitly so crates stop smuggling derived
or forensic surfaces through authority-shaped types.

What this enables:

- safer cross-crate APIs
- clearer support and replay stories
- fewer accidental authority/derivation boundary violations

### Support And Certification Artifact Vocabulary

#### First-class proof-of-truth surfaces

Technical role:
WORTH repeatedly needs support bundles, evidence bundles, certification
summaries, parity artifacts, and residual-debt statements. These should be
first-class boundary categories rather than bespoke exports or test leftovers.

What this enables:

- standardized support and QA surfaces across crates
- less bespoke inspection/export/report code
- stronger audit and certification ergonomics as the platform grows

### Composition-Family Boundary Vocabulary

#### Same-family symbolic resolution and lifecycle surfaces

Technical role:
Some boundary artifacts need to describe one same-family composition program:
which symbolic references were declared, how they resolved, which members were
created, rewritten, retired, or superseded, and what one coherent family
visibility boundary meant. The shared crate should standardize that descriptive
language without owning the family execution engine.

What this enables:

- query/relational-style same-family receipts can share one descriptive shape
- support artifacts can explain mixed symbolic and existing-authority family
  behavior consistently
- lifecycle and resolution evidence stop drifting crate by crate

### Profiles And Policy Vocabulary

#### Richness and diagnostics profiles

Technical role:
WORTH already uses operational/development/forensic and similar ideas. These
need a shared profile vocabulary rather than crate-local profile dialects.

The same vocabulary must also be strong enough to remove optional descriptive
surfaces from hot paths centrally. That includes history-, replay-, lineage-,
provenance-, and forensic-richness classes where a workload needs the leanest
possible operational posture.

What this enables:

- one common language for how rich or minimal a surface should be
- shared profile digests where support or certification artifacts need them
- clearer boundary between operational hot paths and forensic materialization
- one cross-crate switch for descriptive elision instead of hundreds of local
  policy branches

#### Support and posture profiles

Technical role:
Many WORTH subsystems describe support posture, capability posture, or admitted
surface posture through profile-like structures. Those should be standardized.

Representative families may include:

- support profiles
- diagnostics richness profiles
- compatibility profiles
- admission or readiness profiles
- certification posture profiles
- delivery or retention profiles

What this enables:

- query support reports, certification support matrices, and store compatibility
  postures can speak one shared language
- capability and readiness communication becomes more reusable
- descriptive richness and retention can be reduced centrally without changing
  correctness-critical domain logic

#### Layout and performance profiles

Technical role:
WORTH needs a shared vocabulary for layout strategy and performance posture
without enforcing one in-memory representation.

Representative categories include:

- AoS
- SoA
- AoSoA
- sparse
- packed
- custom

What this enables:

- crates can state layout intent explicitly
- performance contracts can name the strategy they rely on
- `worth-relational` and `worth-signal` can remain structurally different while
  still belonging to one architectural family

### Digest And Canonicalization Toolkit

#### Shared digest basis helpers

Technical role:
Boundary surfaces need stable, reproducible canonical digests. The crate must
provide shared helpers and vocabulary for digest-basis assembly without forcing
one storage representation.

This includes explicit equivalence-basis vocabulary for any shared surface that
claims reuse, parity, suppression, comparison, or certification sameness.

What this enables:

- replay parity and certification bundles stay comparable across crates
- diagnostics, receipts, lineage, and support artifacts can share digest
  construction patterns
- less bespoke "digest basis" scaffolding
- fewer hidden reuse heuristics that drift because sameness was never modeled

### Materialization Contracts

#### Canonical boundary materialization

Technical role:
Crates need freedom to keep different optimized internal shapes while still
being able to materialize canonical boundary-facing views.

Where materialization is expensive or richness-bearing, the category should be
explicit: hot-path view, boundary materialized form, forensic/support bundle,
or similar shared boundary distinction. The foundational crate should help make
those transitions visible rather than letting them masquerade as cheap getters.

This is also where profile-driven descriptive elision must become real.
Disabling optional history, replay, lineage, provenance, or forensic surfaces
must happen at named materialization and retention seams. It must not require
threading ad hoc policy checks through leaf call sites that produce
authoritative domain truth.

What this enables:

- `worth-relational` can keep truth-optimized internal layouts
- `worth-signal` can keep diagnostics/runtime layouts tuned for its workload
- `worth-store` can keep durable-friendly structures
- all of them can still emit the same canonical boundary shapes when needed
- all of them can also skip optional descriptive surfaces cleanly when the
  active profile demands hot-path austerity

## What This Crate Must Preserve

- freedom for crate-local hot-path storage and layout choices
- honest distinction between shared meaning and local representation
- the separation between progression law and truth vocabulary
- Aspec-native default semantics as the long-term direction
- explicit compatibility debt where JSON-shaped or other transitional surfaces
  still remain
- central profile-driven removal of optional descriptive richness without
  changing authoritative outcomes or polluting leaf call sites
