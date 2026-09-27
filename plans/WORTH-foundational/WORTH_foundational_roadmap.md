# WORTH Foundational Future Roadmap

## Purpose

This document defines the future work for `worth-foundational`.

It is a future-only roadmap. It does not assume the foundational crate already
exists, and it does not treat shared truth vocabulary as a grab-bag of helpers.
It exists to sequence the work required to make WORTH speak one shared
aspect-native, proof-composable, digest-honest, profile-honest, and
representation-honest language across crates.

The operating rule for this roadmap is:

`standardize shared meaning once, preserve local representation freedom always`

That rule governs every milestone:

1. canonical cross-crate meaning must be encoded in shared types and contracts
   rather than crate-local folklore
2. aspect-native value vocabulary must replace JSON-default thinking at
   canonical boundaries
3. proof-bearing artifacts from `worth-proof` must compose with shared
   descriptive surfaces without pulling those surfaces into the proof kernel
4. diagnostics, lineage, provenance, digests, and receipts must be rich enough
   to survive cross-crate boundaries without producer-side reinterpretation
5. identities, handles, basis ids, equivalence claims, and locators must be
   explicit shared laws rather than crate-local folklore
6. profiles must control richness, posture, and support semantics explicitly
7. profiles must be able to remove optional descriptive richness centrally
   without changing authoritative outcomes or requiring leaf-call-site policy
   branching
8. execution objective, observation activation, diagnostic richness, retention,
   support, certification, and durability must remain distinct meanings rather
   than collapsing into one environment label
9. no milestone may force one hot-path memory layout or one runtime
   representation across crates that legitimately need AoS, SoA, AoSoA, or
   custom topology
10. lowered plans may need to execute at runtime, but shared crates must expose
    only the proof/composition and descriptive boundary language, not a generic
    executor. The explicitly scoped pure expression kernel in
    [Query 9.17.6.1](../WORTH-query/milestone-9.17.6.1.md) additionally owns portable
    grammar, typing, canonical programs, deterministic operators and bounded
    stateless interpretation. It owns no runtime registry, clock, scheduler,
    currentness, resource reservation, cache lifecycle, or operational authority
11. shared boundary vocabulary must not collapse authoritative truth, derived
   artifacts, and descriptive/forensic surfaces into one generic artifact
   model
12. same-family symbolic composition, lifecycle outcomes, and resolution maps
    should have one descriptive language across crates rather than local
    receipt folklore
13. branching, merging, and commits must have one shared authority-transition
    language rather than being hidden inside generic artifact materialization
    or crate-local transaction folklore

## Adversarial Constraint

`worth-foundational` must survive the following hostile condition:

> Several WORTH crates with different authority boundaries, proof-bearing
> lifecycles, memory layouts, and support surfaces must exchange aspect-native
> values, patches, diagnostics, lineage/provenance artifacts, branch/merge/commit
> evidence, profile-bearing reports, and certified receipts such that the same
> semantic thing has one canonical meaning everywhere, while each crate remains
> free to keep its own cost-honest internal representation and materialization
> policy.

If any supported usage:

- treats JSON-shaped payloads as the canonical long-term meaning of boundary
  values
- forces `worth-proof` to own descriptive ontology instead of progression law
- standardizes one in-memory representation where only shared meaning should be
  shared
- allows two crates to mean the same thing with different artifact categories,
  profile semantics, or digest bases
- makes canonical boundary interpretation depend on producer-side hidden state
- weakens digest, provenance, or receipt honesty in exchange for convenience

then `worth-foundational` has failed.

## Roadmap Rules

Rules for every remaining foundational item:

- each milestone must establish a real shared semantic capability boundary, not
  just move utility code into a new crate
- each milestone must preserve the split:
  `worth-proof` owns progression law, `worth-foundational` owns shared truth
  vocabulary, and domain crates own domain semantics plus hot-path
  representation
- no milestone is complete until the shared types are strong enough that two
  crates could exchange the surface without semantic reinterpretation
- no milestone may hide cost boundaries, failure topologies, or correctness
  boundaries behind a convenience abstraction
- every milestone must declare what may remain crate-local on purpose
- every milestone must define machine-checkable acceptance evidence through
  canonicalization tests, parity tests, compile-fail tests, digest equivalence,
  or hostile materialization cases
- if a capability depends on future `worth-proof` surfaces or future domain
  migrations, that dependency must remain explicit debt rather than implied
  completion
`worth-foundational` now has a dedicated
[`test-requirements.md`](test-requirements.md). That document is authoritative
for the whole-crate proof bar, especially under the implementation strategy
where foundational is built completely before major adopting-crate refactors.
Milestone acceptance evidence remains milestone-local, but it must satisfy the
crate-level test requirements before closure.

## Operating Modes

The roadmap preserves these foundational operating modes explicitly:

- `Canonical boundary mode`: values and artifacts are materialized into shared
  foundational shapes for exchange, persistence, replay, or support
- `Crate-local optimized mode`: a crate keeps its own AoS, SoA, AoSoA, packed,
  sparse, or custom representation while conforming to foundational meaning at
  the boundary
- `Compatibility bridge mode`: transitional JSON-shaped or legacy local
  surfaces may remain temporarily, but they must be named debt and lowered into
  canonical foundational meaning explicitly
- `Proof-composed mode`: proof-bearing artifacts from `worth-proof` attach
  foundational profiles, diagnostics, provenance, and receipts without forcing
  runtime proof registries or descriptive baggage into the proof kernel

## Obligation Surface Convention

Every milestone in this roadmap must be read as four separate obligation
surfaces, even when a section heading still says `Must Ship` for readability:

- `surface primitives`: the concrete shared types, markers, builders, or
  materialization contracts that must exist
- `semantic guarantees`: the meaning those primitives must preserve across
  crate boundaries
- `representation boundaries`: what must remain crate-local and must not be
  flattened into the shared crate
- `proof obligations`: the parity checks, hostile cases, and machine-checkable
  evidence required before the milestone is honest

## Standardization Boundary Stance

`worth-foundational` is allowed to standardize:

- boundary-facing vocabulary
- canonical value forms
- aspect-state and aspect-patch meaning
- diagnostics and explanation ontology
- lineage, provenance, and receipt categories
- profile semantics
- digest-basis and canonicalization rules
- materialized artifact contracts
- performance and layout vocabulary

`worth-foundational` is not allowed to standardize:

- one runtime container for all values
- one internal diagnostics store
- one lineage graph representation
- one hot-path artifact layout
- one memory topology for systems with different workload shapes
- one universal representation that hides O(1) versus O(n) behavior

## Early Cross-Feature Proof Gates

The following gates must be preserved from the first milestone onward:

- the canonical aspect-native value language must be precise about width,
  precision, temporal basis, and reference kind
- boundary-facing meaning must not depend on serde-object ordering accidents or
  crate-local merge folklore
- compatibility JSON must remain explicit transition debt, not invisible truth
- shared profiles must be structurally typed rather than string labels floating
  through crates
- proof-bearing artifacts must be able to attach foundational surfaces without
  forcing descriptive concerns into `worth-proof`
- any shared surface that looks cheap must have an honest materialization
  boundary rather than concealed hot-path reconstruction
- reduced-richness profiles must be able to suppress optional history, replay,
  lineage, provenance, or forensic materialization at named seams without
  changing authoritative outcomes
- throughput objective and on-demand observation activation must remain
  orthogonal to richness, retention, authority, security, and durability
- shared identity, equivalence, outcome, and locator surfaces must remain
  explicit enough that comparison, suppression, certification, and debugging do
  not devolve into local folklore
- same-family symbolic resolution and lifecycle evidence must remain canonical
  enough that one coherent family visibility boundary can be described the same
  way across crates
- any shared boundary category must preserve whether it is authoritative,
  derived, planned, descriptive, or receipt-bearing rather than merging those
  meanings for convenience

## Milestone 1: Aspec-Native Canonical Value And Aspect State Substrate

Detailed spec: [`milestone-1.md`](milestone-1.md)

The detailed spec is authoritative for Milestone 1 closure. In particular, it
expands this roadmap summary by treating aspect contracts, schema-declared
struct aspect values, masks, absence/null/default/clear law, evolution posture,
and equivalence basis as part of the substrate rather than later add-ons. It
also defines linear phase gates and compile-time enforcement targets so the
milestone cannot close as an abstract vocabulary exercise. Milestone 1 should
use `worth-proof` for proof-bearing progression surfaces while keeping raw
foundational vocabulary lightweight.

### Goal

Establish the canonical value language and aspect-state vocabulary that every
later foundational surface depends on.

### Must Ship

- a shared aspect-native scalar vocabulary with explicit width, precision,
  temporal basis, and reference kind
- schema-declared struct aspect value support without treating arbitrary
  recursive JSON documents as ordinary authority
- aspect contracts that declare value shape, admissible masks, patch law,
  absence/null/default semantics, equivalence basis, and evolution posture
- proof-bearing validation, evolution, admission, compatibility-lowering, and
  digest-preparation readiness surfaces built on `worth-proof`
- typed identity/key/handle/basis-id vocabulary for shared boundary surfaces
- canonical structural aspect vocabulary for:
  - `AspectKey`
  - canonical aspect-state maps
  - authoritative aspect-state wrappers
  - authoritative aspect patches with explicit `set` and `clear`
- aspect masks and field masks for projection, mutation, diagnostics, and
  report/support selection
- canonical field/path/locator vocabulary for aspect-native and
  boundary-facing surfaces
- explicit patch semantics where `set` dominates overlapping `clear`
- explicit field-level patch semantics for schema-declared struct aspects
- canonical ordering and equality rules for aspect-state and aspect-patch
  materialization
- explicit compatibility-bridge surfaces for transitional JSON-originated
  inputs where needed

### Semantic Guarantees

- semantically identical aspect-native values and aspect states must mean the
  same thing regardless of which crate materialized them
- aspect patches must preserve explicit set-versus-clear semantics rather than
  relying on object-merge convention
- compatibility bridges must lower into the same canonical meaning as native
  aspect construction
- proof-bearing progression must use `worth-proof` rather than a local
  duplicate proof substrate
- masks must select, update, or explain truth without becoming truth values
- absence, null, default, and clear must remain distinct unless an aspect
  contract explicitly collapses them
- aspect evolution must be classified before digest, diagnostics, or migration
  surfaces depend on a shape
- equal representation must not collapse identity, handle, and basis-id meaning
- canonical locators must point at the same structural kinds the same way
  across crates

### Representation Boundaries

- crates may retain local payload/value layouts and materialize canonical
  foundational forms only at explicit boundaries
- the shared value language must not require one universal runtime value bag
- compatibility JSON adapters are boundary shims only, not canonical internal
  storage law
- identity and locator vocabulary standardize meaning, not one allocator,
  one handle lifetime model, or one traversal engine

### Must Preserve

- aspect-native meaning as the canonical long-term truth vocabulary
- no collapse back to untyped JSON-object semantics
- no hidden semantic dependence on map iteration accidents or serializer quirks
- no forced runtime bag-of-values abstraction in hot paths
- freedom for crates to keep crate-local optimized internal representations and
  materialize the canonical forms only at boundaries

### Acceptance Evidence

- canonical ordering tests for aspect-state and patch materialization
- equality and digest-preparation parity tests across independently constructed
  but semantically identical aspect states
- hostile patch tests covering overlapping `set`/`clear`, empty patches,
  no-op patches, and canonical merge behavior
- compatibility-bridge tests proving transitional JSON-shaped inputs lower into
  the same canonical aspect-native meaning as native construction

## Milestone 2: Canonical Digest And Canonicalization Substrate

Detailed spec: [`milestone-2.md`](milestone-2.md)

The detailed spec is authoritative for Milestone 2 closure. In particular, it
expands this roadmap summary by treating canonical basis entries, basis domains,
rule versioning, equivalence basis, mismatch basis, export/golden fixtures,
digest algorithm slots, and production-test readiness as distinct
responsibilities. Milestone 2 makes canonical basis the semantic authority and
keeps digest values as derived compression of that basis.

### Goal

Define the shared digest-basis and canonicalization toolkit so boundary
artifacts can be reproduced, compared, certified, and replayed across crates.

### Must Ship

- canonical digest-basis helpers for aspect-native values, patches, profiles,
  and boundary artifacts
- explicit equivalence-basis and mismatch-explanation vocabulary for reuse,
  suppression, parity, and certification sameness claims
- shared canonicalization contracts for stable ordering and serialization-ready
  normalization
- explicit distinction between canonical semantic form and transport encoding
- digest-basis extension points for later diagnostics, provenance, receipts,
  and support artifacts
- compile-time or strongly typed guardrails preventing semantically unstable
  digest assembly paths where the crate can enforce them

### Semantic Guarantees

- one semantic thing must imply one canonical digest basis across crates
- digest sameness claims must rest on explicit equivalence bases rather than
  incidental field order or transport encoding quirks
- canonicalization must normalize meaning, not erase meaningful distinctions
- reuse and suppression claims must be explainable in terms of declared basis,
  not comparator folklore

### Representation Boundaries

- crates may assemble digest inputs from local structures so long as the
  resulting basis is canonical
- the digest toolkit must not require all shared surfaces to share one storage
  topology before they can be hashed or compared
- transport encoding remains downstream of canonical semantic form
- equivalence vocabulary must not imply one comparator engine or one reuse
  runtime

### Must Preserve

- one semantic thing must hash from one canonical basis across crates
- no digest meaning may depend on crate-local incidental layout
- no use of transport-specific object ordering as an authority claim
- no flattening of profile, provenance, or artifact categories into one generic
  "hashable blob" surface

### Acceptance Evidence

- cross-construction digest parity tests
- canonicalization round-trip tests for stable semantic identity
- hostile ordering tests proving insertion order, temporary builder order, or
  intermediate container layout cannot change canonical digest output
- replay-basis tests proving identical semantic inputs produce identical digest
  bases across independent construction paths

## Milestone 3: Profile And Policy Vocabulary

Detailed spec: [`milestone-3.md`](milestone-3.md)
Closeout: [`milestone-3-closeout.md`](milestone-3-closeout.md)

The detailed spec is authoritative for Milestone 3 closure. In particular, it
expands this roadmap summary by treating profile families, profile composition,
target-aware attachment, canonical profile identity, central materialization
planning, certification posture, and production-test readiness as distinct
responsibilities. Milestone 3 makes profile meaning canonical while preserving
crate-local policy execution.

### Goal

Define the shared profile system for richness, posture, support, compatibility,
and delivery semantics so crates stop carrying these ideas through unrelated
local dialects.

### Must Ship

- typed profile vocabulary for diagnostics richness, support posture,
  compatibility posture, admission/readiness posture, delivery/retention
  posture, and other truly shared profile families
- shared profile composition and attachment contracts for boundary artifacts
- canonical profile digests or profile-basis participation where later
  certification and support artifacts need reproducible profile identity
- explicit distinction between profile semantics and domain policy execution
- explicit central elision controls for optional descriptive surfaces such as
  history, replay, lineage, provenance, and forensic diagnostics
- room for proof-bearing artifacts to carry foundational profiles without
  importing descriptive policy execution into `worth-proof`

### Semantic Guarantees

- profile identity must mean the same richness/posture/support contract across
  crates
- profile changes may alter descriptive breadth, retention, or support posture
  but must not silently redefine authoritative domain truth
- reduced-richness profiles must preserve authoritative outcomes while removing
  only the optional descriptive surfaces they explicitly govern
- profile semantics must remain explicit enough for digesting, comparison, and
  support interpretation

### Representation Boundaries

- domain crates keep ownership of actual policy execution and runtime behavior
- profiles standardize shared meaning, not one engine for applying that meaning
- profile attachment must not require one internal profile store or one global
  registry
- profile elision decisions must attach at named boundary materialization or
  retention seams rather than forcing leaf-call-site branching through domain
  logic

### Must Preserve

- profile semantics must stay typed and explicit rather than string labels
- profile richness must not silently widen hot-path work
- reduced-richness profiles must be able to pull optional descriptive work out
  of the hot path centrally
- profiles must not collapse domain-specific policy meaning into one fake
  universal enum when correctness boundaries differ
- profile attachment must remain boundary-facing; crates retain freedom over how
  they implement the underlying operational mechanics

### Acceptance Evidence

- profile identity and equality tests
- profile composition tests for compatible and incompatible combinations
- hostile tests proving richness/posture changes alter only the allowed
  descriptive surface and do not change authoritative domain outcome semantics
- hostile tests proving a reduced-richness profile can remove optional history,
  replay, lineage, provenance, or forensic materialization without requiring
  broad leaf-call-site policy rewrites
- proof-composition tests demonstrating a proof-bearing artifact can attach
  foundational profiles without needing runtime proof registries

## Milestone 4: Boundary Artifact Taxonomy And Materialization Contracts

Detailed spec: [`milestone-4.md`](milestone-4.md)
Closeout: [`milestone-4-closeout.md`](milestone-4-closeout.md)

The detailed spec is authoritative for Milestone 4 closure. In particular, it
expands this roadmap summary by treating category vocabulary, role/authority
law, materialization seams, canonical basis participation, planned-work and
same-family descriptive extension room, and production-test readiness as
distinct responsibilities. Milestone 4 makes boundary artifact meaning
canonical while preserving crate-local materialization mechanics.

### Goal

Standardize the shared artifact categories and the contracts for materializing
crate-local optimized state into canonical boundary-facing forms.

### Must Ship

- explicit shared categories for:
  - `Summary`
  - `Report`
  - `Artifact`
  - `Receipt`
- explicit authoritative/derived/projected/support-only boundary distinctions
  where crates need them
- explicit room for plan-shaped artifacts that describe intended runtime work
  without implying one shared runtime execution representation
- explicit room for same-family composition artifacts, resolution maps, and
  lifecycle outcomes where crates need one shared descriptive language
- explicit non-ownership of branch, merge, and commit authority-transition law,
  which is split into Milestone 5
- typed materialization contracts that let crates expose canonical boundary
  forms without standardizing their internal storage
- attachment points for profiles, digest bases, diagnostics, provenance, and
  performance accounting
- explicit category boundaries so the same noun means the same thing in every
  crate
- guidance-bearing APIs or traits that make boundary crossings mechanically
  visible instead of looking like cheap getters
- explicit materialization and retention seams where reduced-richness profiles
  may suppress optional descriptive surfaces centrally

### Semantic Guarantees

- `Summary`, `Report`, `Artifact`, and `Receipt` must remain distinct semantic
  categories across the workspace
- authoritative, derived, projected, support-only, planned, and receipt-bearing
  surfaces must remain distinguishable where those differences matter
- plan-shaped artifacts must describe intended work without being confused for
  authoritative truth or for receipts of completed work
- same-family composition artifacts must describe symbolic resolution and
  lifecycle meaning without being confused for authority execution engines
- branch, merge, and commit evidence must not be treated as just another
  materialized artifact category
- descriptive or support-facing categories must not silently masquerade as
  canonical authority surfaces
- profile-driven descriptive elision must happen at materialization/retention
  seams rather than by changing authoritative domain logic

### Representation Boundaries

- foundational category contracts describe boundary meaning, not one storage
  representation
- crates may materialize the same category from different internal layouts
- shared traits or builders must not pressure crates to expose deep internals
  just to satisfy category shape
- explicit materialization categories must not force one always-on rich view of
  authoritative state
- reduced-richness behavior must be enforceable at boundary seams, not by
  rethreading local branch logic through leaf producers

### Must Preserve

- no one-category envelope that collapses distinct boundary semantics
- no concealment of expensive materialization behind cheap-looking accessors
- no requirement that disabling optional descriptive richness be implemented by
  touching hundreds of leaf call sites
- no requirement that all crates store summaries, reports, artifacts, or
  receipts in the same topology
- no smuggling of branch, merge, or commit authority transitions through generic
  `Artifact` or `Receipt` shapes
- no pressure toward a generic execution runtime just because several crates
  lower plans into runtime-applied work
- no weakening of facade boundaries by exposing deep internal storage just to
  satisfy a shared trait

### Acceptance Evidence

- artifact-category distinction tests
- materialization honesty tests showing boundary crossing is explicit
- hostile tests proving crate-local optimized structures can materialize shared
  categories without semantic drift
- hostile tests proving a reduced-richness profile can suppress optional
  history-, replay-, lineage-, provenance-, or forensic-bearing materialization
  at named seams while leaving authoritative outputs unchanged
- compile-fail or trait-bound tests preventing obviously invalid category
  substitutions where the API can enforce them

## Milestone 5: Branching, Merging, And Commit Vocabulary

Detailed spec: [`milestone-5.md`](milestone-5.md)
Closeout: [`milestone-5-closeout.md`](milestone-5-closeout.md)

### Goal

Define the shared language for branch-local intent, merge resolution, and
authority-bearing commit evidence so crates can describe state transitions
without inventing incompatible transaction, version, or receipt folklore.

### Must Ship

- typed branch identity, branch lineage, and branch visibility vocabulary for
  boundary-facing artifacts
- typed commit identity, commit basis, parent-basis, and committed-delta
  vocabulary
- typed merge identity, merge input, merge basis, conflict, resolution, and
  merged-output vocabulary
- explicit distinction between:
  - branch-local planned intent
  - branch-local staged state
  - merge candidates
  - merge verdicts
  - authority-bearing commits
  - commit receipts
- compatibility surfaces for crates that currently model transactions,
  versions, branches, or commits through local dialects
- digest-basis participation for branch, merge, and commit evidence
- locator support for branch loci, merge-conflict loci, committed-delta loci,
  and commit-receipt loci
- profile attachment points for support, certification, forensic richness, and
  reduced-richness commit reporting

### Semantic Guarantees

- branch-local state must not be confused with committed authoritative state
- merge resolution must preserve conflict, denial, advisory, and accepted
  outcomes as structured categories rather than booleans
- commit receipts must attest to completed authority transitions, not merely a
  plan or candidate merge
- branch lineage, commit parentage, and merge basis must be self-describing
  enough for consumers without producer-private state
- digest and equivalence claims over commits must name their basis explicitly
- reduced-richness profiles may remove optional forensic branch/merge detail
  but must not change the authoritative commit outcome

### Representation Boundaries

- `worth-foundational` standardizes branch/merge/commit boundary meaning, not
  one transaction engine, VCS model, storage journal, or concurrency-control
  runtime
- domain crates keep ownership of mutation execution, conflict policy,
  storage layout, and commit durability mechanics
- branch and merge evidence may be materialized from different internal
  topologies so long as the boundary vocabulary remains canonical
- commit vocabulary must compose with `worth-proof` artifacts without moving
  proof progression law or execution authority into `worth-foundational`

### Must Preserve

- no collapse of branch-local candidate state into committed authoritative
  truth
- no collapse of merge verdicts into generic success/failure
- no receipt claiming an authority transition happened before commit authority
  has actually completed
- no assumption that every crate uses the same branch graph, journal, or
  storage topology
- no hidden producer-private interpretation required to understand commit
  parentage, merge basis, conflict loci, or committed deltas

### Acceptance Evidence

- branch/commit category-separation tests
- merge-verdict topology tests proving conflict, denial, advisory, and accepted
  outcomes remain distinct
- digest-basis parity tests for semantically identical commit evidence produced
  through independent construction paths
- locator tests for branch loci, merge-conflict loci, committed-delta loci, and
  commit-receipt loci
- hostile tests proving branch-local candidate state cannot satisfy APIs that
  require committed authority evidence
- hostile profile tests proving reduced-richness branch/merge reporting does
  not change authoritative commit outcomes

### Query Milestone 9.17.1 Vocabulary Extension

Status: Closed on 2026-08-28. The canonical cross-runtime descriptive grammar is
[`branch-references.md`](../../crates/worth-foundational/docs/branching-merging-and-commit-vocabulary/branch-references.md).
It grants no component operating authority and defines no compatibility
conversion from the earlier epoch/equivalence vocabulary.

[Query Milestone 9.17.1](../WORTH-query/milestone-9.17.1.md) extends this closed
vocabulary milestone with the exact mutable-reference grammar required by both
Relational and Signal. The extension separates immutable target basis from
mutable branch identity and adds generic descriptive target, reference
generation, exact observation, fork, comparison, movement, and mismatch
values.

This is a vocabulary extension, not a reopening that moves runtime authority
into Foundational. `TargetBasis` remains an immutable owner descriptor;
Relational and Signal retain runtime identity, currentness, lifecycle,
retention, admission, and movement authority. Epoch-only Milestone 5 forms
cannot be used as compatibility constructors for an exact operational
reference.

## Milestone 6: Diagnostics And Explanation Ontology

Detailed spec: [`milestone-6.md`](milestone-6.md)
Closeout: [`milestone-6-closeout.md`](milestone-6-closeout.md)

The detailed spec is authoritative for Milestone 6 closure. In particular, it
expands this roadmap summary by treating diagnostic primitives, outcome and
absence topology, materialization and support-report law, canonical
basis/comparison participation, certified bundle law, and production-test
readiness as distinct responsibilities. Milestone 6 makes diagnostics and
explanation meaning canonical while preserving crate-local capture, storage,
and replay mechanics.

### Goal

Establish one shared vocabulary for diagnostics, explanations, denials,
advisories, and support-bearing decision context.

### Must Ship

- typed diagnostic categories including codes, scopes, severities, and
  artifact kinds
- structured decision/outcome vocabulary for accepted, advisory, denied,
  deferred, partial, unsupported, mismatch, and related shared families
- structured explanation and denial vocabulary that supports success,
  advisory, and violation style outcomes where the crate needs them
- shared diagnostic attachments for profile, provenance, and digest basis
- support for proof-bearing outputs to attach diagnostics and explanations
  without moving diagnostic storage into `worth-proof`
- materialized diagnostic/report contracts that can be compared across crates

### Semantic Guarantees

- diagnostics must preserve the distinction between success, advisory, denial,
  and failure-bearing context where the crate needs it
- explanatory surfaces must remain descriptive rather than authoritative
- the same diagnostic code/scope/severity combination must mean one thing
  across crates
- shared outcome families must remain extensible enough that crates do not have
  to flatten real correctness differences into one fake universal enum

### Representation Boundaries

- crates remain free to capture and store diagnostics in AoS, SoA, AoSoA, or
  custom forms
- foundational diagnostics standardize the boundary language, not one runtime
  diagnostics engine
- richness-profile materialization must remain explicit rather than always-on
- outcome vocabulary standardizes shared categories, not one forced domain
  result type

### Must Preserve

- diagnostics remain descriptive ontology, not authority over domain outcome
- binary success/failure surfaces must not erase structured denial context
- diagnostic richness remains controlled by profiles and materialization policy
- crates keep freedom to use AoS, SoA, AoSoA, or custom internal storage for
  diagnostic capture

### Acceptance Evidence

- cross-crate-shaped diagnostic parity tests using shared foundational types
- explanation/denial topology tests proving advisory versus violation remains
  distinguishable
- hostile richness-profile tests proving reduced richness does not change the
  authoritative operation result
- materialization tests from multiple internal storage strategies into one
  canonical diagnostic boundary language

## Milestone 7: Lineage, Provenance, And Receipt Vocabulary

### Goal

Define the shared language for where artifacts came from, under what basis they
were produced, and what effectful or authority-bearing boundary actually
happened.

Detailed spec: [`milestone-7.md`](milestone-7.md)
Closeout: [`milestone-7-closeout.md`](milestone-7-closeout.md)

The detailed spec and closeout record are authoritative for Milestone 7
closure. In particular, they expand this roadmap summary by treating category
and locality primitives, provenance layering, receipt-family strength,
continuity/divergence families, support-truth and degraded recovery,
mixed-family attachment materialization, proof-bearing readiness, and
crate-facing docs closure as distinct responsibilities. Milestone 7 makes
boundary evidence canonical while preserving crate-local storage, replay,
history, and support execution mechanics.

### Must Ship

- shared lineage vocabulary for boundary-facing lineage artifacts and lineage
  digest participation
- shared provenance vocabulary for source basis, authority path, profile basis,
  and supporting context attachments
- shared receipt vocabulary for effectful or authority-bearing boundaries
- shared support/certification artifact vocabulary for evidence bundles,
  certification summaries, parity artifacts, and residual-debt statements
- shared descriptive seams for "what was planned" versus "what was executed"
  where runtime-applied plans need canonical boundary explanation
- shared composition-family vocabulary for symbolic resolution maps, lifecycle
  outcomes, and coherent family-boundary receipts
- explicit distinction between lineage, provenance, and receipt semantics so
  the same underlying fields are not reused to mean different things in
  different crates
- proof-composable attachments so proof-bearing outputs can carry foundational
  provenance and receipt surfaces without moving authority law into the
  descriptive layer

### Semantic Guarantees

- lineage, provenance, and receipt must remain distinct categories with stable
  meanings
- provenance must describe basis and authority path, not replace proof law
- receipts must attest to completed effectful or authority-bearing boundaries,
  not merely intention or planning
- planned-versus-executed descriptive seams must remain explicit
- same-family resolution and lifecycle evidence must remain explicit enough to
  describe mixed symbolic and existing-authority programs coherently
- support and certification artifacts must remain derived proof-of-truth
  surfaces rather than being confused for authoritative state

### Representation Boundaries

- crates retain their own internal lineage/provenance storage and event shapes
  until canonical materialization boundaries
- foundational receipt vocabulary does not imply one executor or one execution
  record layout
- proof-bearing artifacts may attach these descriptive surfaces without moving
  descriptive ownership into `worth-proof`
- support/certification vocabulary standardizes boundary meaning, not one QA
  harness or one persistence model

### Must Preserve

- lineage must not degrade into generic event logging theater
- provenance must remain self-describing enough for consumers that do not know
  producer internals
- receipts must testify to actual effectful or authority-bearing boundaries,
  not merely intent to act
- no single field or envelope may collapse derivational origin, authority
  attestation, and replay linkage into one ambiguous meaning

### Acceptance Evidence

- lineage/provenance/receipt category-separation tests
- digest-basis parity tests for shared lineage/provenance attachments
- hostile attachment tests proving proof-bearing artifacts can carry
  provenance/receipt surfaces without mutating proof semantics
- self-description tests proving a consumer can interpret the boundary artifact
  without producer-private state

## Milestone 8: Performance And Layout Vocabulary

### Goal

Create one shared language for layout intent, access-pattern claims, and
performance-contract surfaces without imposing one memory representation across
crates.

### Must Ship

- shared layout vocabulary for categories such as AoS, SoA, AoSoA, sparse,
  packed, and custom
- shared access-pattern or performance-posture vocabulary that lets a crate
  describe why a representation exists
- attachment points for structural counters, complexity-contract names, and
  performance-facing report surfaces where foundational artifacts need them
- materialization and representation-boundary guidance so shared semantics never
  accidentally standardize hot-path storage
- explicit distinction between performance vocabulary and a shared performance
  runtime container library

### Semantic Guarantees

- layout and performance terms must carry one shared meaning across crates
- performance-facing boundary artifacts must describe cost posture honestly
- shared layout vocabulary must never imply cost equivalence between distinct
  representations

### Representation Boundaries

- foundational performance vocabulary must not become a universal storage or
  execution abstraction
- each crate remains free to keep its own topology, access patterns, and
  hot-path data layout
- shared counters and performance-facing reports attach at boundaries, not by
  forcing one internal measurement runtime

### Must Preserve

- no universal container that pretends AoS and SoA are the same thing
- no abstraction that hides cost topology or failure topology
- no forcing of `worth-relational`, `worth-signal`, `worth-query`, or
  `worth-store` into one diagnostics or lineage representation
- performance claims remain valid only at the boundary they name

### Acceptance Evidence

- layout vocabulary tests
- hostile representation-boundary tests proving shared meaning does not imply
  shared storage
- materialization tests from distinct layout strategies into shared boundary
  artifacts
- complexity-attachment tests proving performance accounting can ride shared
  boundary artifacts without dictating the underlying runtime structure
- adversarial profile-elision tests proving a reduced-richness profile can pull
  optional history, replay, lineage, provenance, or forensic surfaces out of
  the certified hot path without changing authoritative outputs

## Milestone 9: Scoped Merge Selection And Cherry-Pick Vocabulary

Detailed spec: [`milestone-9.md`](milestone-9.md)

### Goal

Extend the Milestone 5 branch/merge/commit transition language with shared
scoped merge and cherry-pick vocabulary before adopting runtimes implement
selected-node or selected-aspect merge execution.

### Must Ship

- full-branch, selected-node, and selected-aspect merge scope request
  vocabulary
- admitted scope evidence, selected no-op evidence, and compact
  skipped-out-of-scope evidence
- typed scope denial and scope-unavailable posture for unknown, ambiguous,
  deleted, unsupported, or non-adoptable selected loci
- scope breadth counters for selected nodes, selected aspects, skipped
  candidates, no-op selections, and conflict checks
- merge candidate/verdict integration without turning `worth-foundational`
  into a merge executor
- canonical basis and locator participation for scoped merge evidence
- compile-fail and hostile producer-diversity certification

### Semantic Guarantees

- cherry-pick means selected merge scope, not consumer-side filtering of a
  broader merge result
- full-branch merge remains a named scope rather than absence of scope meaning
- selected-node and selected-aspect scopes are not substitutable
- skipped and no-op selected work remain visible enough for blind consumers to
  explain what happened
- unsupported scoped merge families deny or become unavailable explicitly

### Representation Boundaries

- adopting crates keep local node ids, journals, aspect sets, and merge
  planners
- `worth-foundational` standardizes boundary meaning only
- runtime execution, source/target storage mutation, and materialization remain
  adopting-crate responsibilities

### Acceptance Evidence

- scoped merge request vocabulary tests
- scoped merge admission evidence tests
- scoped merge denial topology tests
- scoped merge canonical-basis and locator tests
- compile-fail coverage for raw selected loci and category substitution

## Milestone 10: Throughput And On-Demand Observation Vocabulary

Detailed spec: [`milestone-10.md`](milestone-10.md)

### Goal

Add shared, orthogonal vocabulary for execution objective and observation
activation, then prove through real `worth-signal` adoption that a throughput
runtime can remove optional observation work from ordinary execution without
changing authoritative outcomes, stable identity, or replay linkage.

### Must Ship

- `LatencyBounded`, `Balanced`, and `Throughput` execution-objective meaning
- `OnDemand` and `Continuous` observation-activation meaning
- actual observation disposition and observation-specific absence vocabulary
- complete profile composition, identity, difference, and progression support
  for the new axes
- optional observation work classes in performance claims
- a compiler-visible Signal runtime-policy pipeline
- `SignalRuntimePolicy::operational()` as the public Throughput + OnDemand
  constructor, with pre-construction hot-path gates and managed observation
  sessions
- exact semantic parity, zero-optional-work, lifecycle, mutation-sensitive,
  and measured performance evidence through Signal's production composition
  root
- a precise Store adoption handoff that preserves every durability invariant

### Semantic Guarantees

- throughput changes cost strategy, never truth, authority, security, or
  durability
- on-demand changes when optional observation work begins, not what correctness
  facts must exist
- richness, activation, retention, support, certification, and execution
  objective remain independent profile meanings
- performed observation evidence requires pre-execution admission and completed
  execution
- missing evidence remains typed and cannot appear as an empty complete report

### Representation Boundaries

- Foundational owns shared meaning and comparison law, not runtime execution,
  telemetry storage, session management, or durability
- Signal owns its policy compiler, observation lifecycle, hot-path gates, and
  local diagnostic/lineage representation
- Store retains complete ownership of WAL, MVCC, acknowledgement, recovery,
  compaction, and persistence policy

### Acceptance Evidence

- canonical profile identity and difference tests for both new axes
- hostile activation/richness/retention composition tests
- optional-work inclusion/exclusion tests
- real Signal no-observer zero-work and explicit-observer performed-evidence
  twins
- operational digest parity across profile, branch, snapshot, restore, serial,
  parallel, and WASM-supported lanes
- scale-sensitive throughput evidence with declared workloads and structural
  counters

## Milestone 11: Cross-Crate Migration And Closure

### Goal

Retire the major crate-local dialects that `worth-foundational` was created to
replace and prove that the shared vocabulary actually converges the stack.

### Must Ship

- migration of the most central aspect/value/digest/profile surfaces in
  `worth-relational`, `worth-query`, `worth-signal`, and `worth-store` onto the
  foundational vocabulary
- migration of the most central identity/equivalence/outcome/locator surfaces
  in `worth-relational`, `worth-query`, `worth-signal`, and `worth-store` onto
  the foundational vocabulary
- migration of the most central same-family composition receipt/lifecycle/
  resolution surfaces in the crates that need them onto the foundational
  vocabulary
- migration of execution-objective, observation-activation, observation
  disposition, and optional-work disclosure where adopting runtimes expose
  those meanings
- explicit debt accounting for any remaining compatibility JSON, local
  diagnostic dialects, local artifact-category drift, or local runtime-profile
  dialects
- cross-crate parity harnesses proving that semantically identical boundary
  artifacts compare, digest, and explain the same way
- facade-safe adoption paths so domain crates consume foundational vocabulary
  without leaking internal reorganizations across the workspace

### Semantic Guarantees

- migrated crates must converge on shared meaning, not merely shared wrapper
  names
- authoritative, derived, planned, descriptive, and receipt-bearing surfaces
  must remain distinguishable after migration
- semantically identical boundary artifacts from different crates must compare
  and explain the same way
- identity, equivalence, outcome, locator, profile, and observation surfaces
  must also converge or remain named debt
- same-family composition receipts and resolution/lifecycle evidence must also
  converge or remain named debt

### Representation Boundaries

- migration must not force any crate to give up a cost-honest internal layout
- adopting the shared vocabulary must not require leaking internal subsystem
  structure across facades
- residual crate-local dialects may remain only as named debt, not as silent
  alternate semantics

### Must Preserve

- migrations must reduce semantic drift rather than merely add wrappers
- no crate may be forced to abandon a cost-honest internal representation just
  to "use the shared crate"
- no milestone closure until the shared vocabulary proves it can replace real
  crate-local dialects under hostile comparison cases
- explicit debt must remain explicit; partial migration is not silent closure

### Acceptance Evidence

- cross-crate migration parity tests
- digest equivalence tests across migrated surfaces
- hostile support-artifact comparison tests across at least two independently
  migrated crates
- profile and observation-disposition parity across at least two independently
  migrated runtimes
- documented debt inventory for any remaining non-foundational dialects that
  still survive by necessity

## Outstanding Future Debt

- shared certification harness substrate once enough foundational vocabulary has
  landed to justify it
- explicit migration closeouts per adopting crate once the first real
  foundational migrations begin
