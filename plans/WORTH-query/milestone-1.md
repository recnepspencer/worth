# Milestone 1 Engineering Spec: Canonical Query Artifact And Result Shape Authority

> **Status:** Completed.
>
> **Roadmap parent:** [worth_query_roadmap.md](./WORTH_query_roadmap.md)
>
> **Vision parent:** [worth_query_vision.md](./WORTH_query_vision.md)
>
> **Shipped closeout:** [milestone-1-closeout.md](./milestone-1-closeout.md)
>
> **Test requirements:** [test-requirements.md](./test-requirements.md)
>
> **Primary architectural driver:** lock one canonical typed query artifact and one canonical typed result-shape artifact before validation, planning, live promotion, saved queries, or store-backed execution are allowed to multiply representations
>
> **Companion docs:**
> - [MENTALITY.md](../../docs/coding-guidelines/MENTALITY.md)
> - [arch_laws.md](../../docs/coding-guidelines/arch_laws.md)
> - [perf_laws.md](../../docs/coding-guidelines/perf_laws.md)
> - [domain_laws.md](../../docs/coding-guidelines/domain_structure_laws.md)
> - [worth_query_vision.md](./WORTH_query_vision.md)
> - [worth_query_roadmap.md](./WORTH_query_roadmap.md)
> - [test-requirements.md](./test-requirements.md)
> - [milestone-1.md](../worth-store/milestone-1.md)
> - [milestone-1.md](../WORTH-runtime-bridge/milestone-1.md)
> - [milestone-2.md](../WORTH-runtime-bridge/milestone-2.md)

## Goal

Make query intent a first-class typed artifact with one canonical identity,
one canonical result-shape meaning, and one framework-owned normalization path
before any later milestone is allowed to validate, plan, persist, bind, or
subscribe queries.

## Why This Milestone Exists

`worth-query` only works if later milestones inherit one artifact authority
instead of rebuilding query meaning in each layer:

- schema validation must validate one canonical query, not one builder path
- planning must lower one canonical query, not host-local helper state
- live promotion must preserve one canonical query identity, not infer meaning
  from delivery wiring
- saved queries and templates must serialize one canonical query artifact, not
  whichever builder happened to create them
- type-bound execution must bind onto query-owned descriptors, not invent a
  second route-local query model

Milestone 1 is therefore not "make a nice builder." It is "freeze what a query
is, what a result shape is, what construction proves, and what must never live
outside those artifacts."

## Governing Summaries

- `MENTALITY.md`: solve the hard architectural problem first, keep one
  canonical authority artifact, and refuse convenience abstractions that create
  hidden debt.
- `arch_laws.md`: Laws 7, 26, 28, 29, 30, 40, and especially 41 govern this
  milestone. Query artifacts must be self-describing, equivalence must be
  explicit, shared lifecycle may be abstracted but cost/failure/correctness
  boundaries may not be erased, and proof-bearing phase types must distinguish
  raw authored intent from canonicalized intent.
- `perf_laws.md`: canonicalization must preserve semantic delta, avoid repeated
  rediscovery, and expose counters at the normalization boundary so later
  performance claims are attached to named work rather than guesses.
- `domain_laws.md`: `worth-query` must start with one public facade and
  domain-aligned subdomains. Query authoring, normalization, result shaping,
  identity, and diagnostics are separate responsibilities and may not be hidden
  in one catch-all module.
- `worth_query_vision.md`: typed composable expressions, aspect-aware
  projection, typed result shapes, bounded traversal, view-shape-aware future
  growth, and eventual type-bound execution all depend on a stable canonical
  artifact model now.
- `worth_query_roadmap.md`: Milestone 1 is the authority milestone. It must
  make structurally equal queries normalize identically, prevent alternate
  query ASTs, and provide the proof surface that later milestones consume.
- `test-requirements.md`: the `Canonical Query Normalization Parity Test` is
  the release proof for this milestone. Equivalent construction paths must
  converge to the same `query_digest`, `result_shape_digest`,
  `canonicalization_report`, and `counter_snapshot`.

## Adversarial Constraint

Milestone 1 must survive the following hostile condition:

> Different builders, helper combinators, future scope/template expansion
> surfaces, and type-bound host binding descriptors all express the same query
> intent, but only one canonical query artifact and one canonical result-shape
> artifact are allowed to exist. No later phase is allowed to recover missing
> meaning from ambient host context, insertion order, helper nesting, or
> post-fetch reshaping.

Concretely, the design must remain correct when all of the following are true:

- the same detail or collection query is authored through direct construction,
  fluent builder composition, and future reusable helper layers
- host frameworks attach binding metadata or caller-local labels in different
  orders across runs
- result-shape construction uses different helper composition paths
- later milestones add validation, planning, live promotion, and persistence,
  but Milestone 1 must not fake those proofs early
- future scope/template/saved-query surfaces must lower into this milestone's
  artifact model rather than replacing it

If any supported path:

- creates a second authoritative query AST
- stores query meaning in builder-private state that disappears after
  construction
- allows result shape to drift from query identity through post-hoc wrapper
  logic
- treats canonicalization as a best-effort formatting pass rather than a
  digest-bearing semantic normalization step

then Milestone 1 has failed.

## Product Decision Lock

The following decisions are explicit and not open questions for this
milestone:

- `worth-query` will be a dedicated crate with one public facade
- queries are Rust types and canonical artifacts, not strings, not opaque DSL
  blobs, and not host-owned adapter structs
- result shapes are first-class typed artifacts, not post-fetch resource
  wrappers
- canonicalization is a proving phase with typed outputs, not a convenience
  helper
- Milestone 1 proves canonicalization only; it does not claim schema legality,
  execution eligibility, or runtime/store parity yet
- type-bound execution is represented only as future-facing binding metadata
  attached to canonical query artifacts, not as a second execution surface

Normative consequence:

- any implementation path that allows consumers to bypass the facade and build
  alternate authoritative query trees is out of spec
- any implementation path that turns result shaping into arbitrary dynamic maps
  is out of spec
- any implementation path that collapses query identity, execution basis, and
  host binding context into one blob is out of spec

Bypass clarification:

- forbidden bypass means creating alternate authoritative canonical artifacts or
  alternate raw authored artifact families outside sanctioned modules
- allowed ergonomic surfaces include macros, codegen, and helper builders that
  lower exclusively into admitted raw authored forms and sanctioned
  canonicalization entry points

## Scope

### In Scope

- one `worth-query` crate boundary and root `facade.rs`
- typed query expression families for detail reads, collection reads,
  aspect-aware projection, bounded relation traversal, and typed result shapes
- one canonicalization pipeline that normalizes admitted authored query forms
  into canonical query and result-shape artifacts
- canonical query identity, canonical result-shape identity, and explicit
  equivalence contracts
- query-owned binding descriptor shapes for future type-bound execution
- normalization diagnostics, digests, and counters
- certification proving equivalent construction-path parity

### Explicitly Out Of Scope

- schema-aware legality proof
- predicate legality and workflow-aware predicates
- execution planning and snapshot-backed execution
- live promotion and signal integration
- store-backed persistence, saved-query durability, and import/export
- historical, diff, lineage, correspondence, policy, or tenant semantics

Milestone 1 must leave room for those later milestones without pretending to
ship them now.

## Canonical Artifact Model

### DRY Envelope Rule

Milestone 1 must satisfy Architectural Law 7 without collapsing distinct
semantic artifacts into one vague bag.

The DRY rule is:

- the categories of boundary information are shared
- the semantic artifact types are not shared unless they truly have the same
  meaning

That means `worth-query` should reuse one envelope pattern for boundary output,
but not collapse query identity, result-shape identity, canonicalization proof,
and future execution basis into one undifferentiated struct.

Representative milestone-owned envelope categories:

- primary artifact identity
- structured warnings
- canonicalization trace
- integrity markers and digests
- performance counters

Representative milestone-owned semantic artifacts:

- `RawAuthoredQuery`
- `CanonicalQueryArtifact`
- `CanonicalResultShapeArtifact`
- `CanonicalQueryBundle`
- `QueryBindingDescriptor`
- `CanonicalizationReport`

If two values differ in meaning, they must be different types even when their
backing representation is similar. That is Law 40 and Law 41 territory, not a
stylistic preference.

### Canonical Artifact Families In Milestone 1

Milestone 1 must establish exactly these authoritative artifact families:

1. Raw authored query intent
2. Canonical query artifact
3. Raw authored result-shape intent
4. Canonical result-shape artifact
5. Canonical query bundle tying the query artifact to the result-shape artifact
   and admitted binding descriptors

Everything else in Milestone 1 is derived.

The canonical query artifact owns:

- query family identity
- root entity identity
- projection intent
- traversal intent
- collection/detail mode
- canonical clause ordering
- canonical digest basis

The canonical result-shape artifact owns:

- result-shape family identity
- field ordering
- delivery-shape meaning that later milestones must preserve
- canonical digest basis independent of builder path

Milestone 1 does not admit authored derived-field semantics. If future
artifact space is reserved for derived fields, that space must remain
non-authorable, non-semantic, and excluded from canonical digests in this
milestone.

The canonical query bundle owns:

- `query_digest`
- `result_shape_digest`
- compatibility relation between query projection and result shape
- canonicalization report
- counter snapshot

Host labels, UI names, route strings, and consumer-local helper metadata are
never authoritative query meaning.

### Projection-Result Shape Compatibility Law

Milestone 1 must define compatibility between canonical query projection and
canonical result shape as a normative rule, not as a future interpretation.

Allowed:

- result shape may reorder projected material
- result shape may omit projected material
- result shape may rename delivered fields where aliasing is explicitly
  represented in the canonical result-shape artifact
- result shape may map one projected source field to one delivered field

Forbidden:

- result shape may not reference source material absent from the canonical
  query projection
- result shape may not invent new semantic fields through post-fetch wrapper
  logic
- result shape may not erase collection/detail family mismatch by implicit
  coercion
- result shape may not rely on host-local adapter state to resolve field
  meaning

Derivable only under admitted rule:

- if Milestone 1 carries any future-facing derived-field placeholder, it must
  be non-authorable and non-semantic in this milestone

Omission behavior:

- omission is allowed and does not change query digest
- omission participates in result-shape digest because delivered shape changed

Alias behavior:

- aliases that affect delivered field identity participate in
  `CanonicalResultShapeDigest`
- cosmetic aliases that do not affect delivered field identity are excluded and
  may only survive as diagnostics

Family mismatch behavior:

- detail-query to collection-shape mismatch is a typed compatibility failure
- collection-query to detail-shape mismatch is a typed compatibility failure

Compatibility failure taxonomy:

- `UnprojectedShapeField`
- `QueryShapeFamilyMismatch`
- `AmbiguousShapeAliasIdentity`
- `UnsupportedDerivedFieldPlaceholder`

Compatibility is established at canonical bundle construction and may not be
re-decided later by validation, planning, or host delivery code.

### Binding Descriptor Identity Policy

Binding descriptors are a future-facing preparation surface, so Milestone 1
must classify them explicitly.

Identity-bearing binding descriptors:

- bindable root identity that changes the query's addressed subject
- bindable slot identity when slot structure changes canonical query meaning
- bindable collection/detail selector if it changes query family identity

Non-identity-bearing binding metadata:

- route names
- controller/component names
- UI labels
- debugging labels
- host attachment handles

Forbidden metadata:

- hidden filters
- hidden branch/basis selectors
- hidden policy context
- hidden result-shape mutations
- host-specific execution hints that alter query meaning

Rules:

- only identity-bearing binding descriptors may enter canonical digest basis
- non-identity-bearing metadata must be excluded from canonical digests and may
  survive only as diagnostics
- forbidden metadata must be rejected before bundle construction
- binding descriptors must never become a smuggling lane for future
  execution-time semantics

### Admitted Authored Form Boundary

Milestone 1 must define what can be authored, what can exist as builder sugar,
and what must fail before canonicalization.

Admitted authorable primitives:

- detail-query root selection
- collection-query root selection
- explicit aspect projection entries
- bounded relation traversal entries
- result-shape field declarations over admitted projected material
- identity-bearing binding descriptor declarations

Admitted authorable combinations:

- detail query plus projection
- collection query plus projection
- detail query plus bounded traversal
- collection query plus bounded traversal
- either query family plus compatible typed result shape
- either query family plus identity-bearing binding descriptors

Non-admitted constructs that must typed-fail before canonicalization:

- unbounded traversal
- dynamic map result shapes
- host-owned filter payloads
- hidden basis selection
- hidden policy or tenant selectors
- partially specified result-shape family without field identity

Non-admitted constructs that may exist only as internal builder sugar:

- helper-local ordering conveniences
- macro/codegen expansion helpers
- fluent composition state used only to assemble admitted raw authored forms

Rules:

- builder sugar may exist only if it lowers exclusively into admitted raw
  authored forms
- partially authorable intermediate states may not survive into
  `RawAuthoredQuery` or `RawAuthoredResultShape`
- unsupported authored forms must fail before canonicalization, not be repaired
  by it

### Canonicalization Rule

Milestone 1 must not use "canonical" as a vibe word. It must define canonical
bases precisely enough that two independent implementations could produce the
same artifacts.

For every canonical artifact in this milestone, the spec must define:

- exact ordered input set
- ordering key
- deduplication rule
- digest basis
- identity-bearing fields
- explanatory-only fields

Canonicality must cover at least:

- authored query clause ordering
- aspect projection ordering
- traversal clause ordering
- result-shape field ordering
- binding descriptor ordering
- canonical query bundle digest inputs

If any artifact can vary because a host used insertion order, helper nesting
order, or map iteration order, the design is out of spec.

### Canonical Digest Basis Rules

Milestone 1 must define canonical digest basis per artifact family, not just at
the bundle level.

#### Canonical Query Artifact

Included in `CanonicalQueryDigest`:

- query family
- root entity identity
- collection/detail mode
- canonicalized projection entries
- canonicalized traversal clauses
- identity-bearing binding descriptors

Excluded from `CanonicalQueryDigest`:

- authored construction order before canonicalization
- helper provenance
- builder/fluent call history
- host labels, UI labels, debug names, and route strings
- non-identity-bearing binding metadata
- diagnostics richness

Normalized ordering key:

- query family first
- root entity identity second
- projection entries by canonical aspect identity then canonical field identity
- traversal clauses by canonical path identity
- identity-bearing binding descriptors by canonical binding slot identity

Equivalence relation:

- two authored queries are equivalent if they reduce to the same query family,
  root entity identity, collection/detail mode, projection set, traversal set,
  and identity-bearing binding descriptor set

Deduplication precedence:

- exact semantic duplicates collapse to one canonical entry
- if duplicate authored entries differ only in excluded metadata, one canonical
  entry survives and excluded metadata becomes diagnostic-only residue
- if duplicate authored entries disagree on identity-bearing meaning, the query
  is rejected as ambiguous rather than repaired

Conflict behavior:

- identity-bearing conflicts are typed failures
- excluded-metadata conflicts may produce warnings but may not affect digest

#### Canonical Result-Shape Artifact

Included in `CanonicalResultShapeDigest`:

- result-shape family
- canonical field set
- canonical field aliases where alias changes delivered field identity
- canonical field derivation source kind where admitted
- collection/detail family alignment markers

Excluded from `CanonicalResultShapeDigest`:

- authored helper order before canonicalization
- cosmetic labels that do not affect delivered field identity
- debug display names
- diagnostics richness

Normalized ordering key:

- result-shape family first
- canonical field identity second
- alias identity third where alias is delivery-bearing

Equivalence relation:

- two authored result shapes are equivalent if they declare the same
  result-shape family and the same canonical delivered field identities in the
  same family context

Deduplication precedence:

- duplicate fields with identical delivered field identity collapse
- duplicate fields with conflicting delivered field identity reject
- cosmetic metadata disagreement is warning-only if the field identity is
  otherwise the same

Conflict behavior:

- field-identity conflicts are typed failures
- cosmetic disagreements may warn but may not change digest

#### Canonical Query Bundle

Included in bundle identity:

- `query_digest`
- `result_shape_digest`
- compatibility decision outcome
- canonical identity-bearing metadata freeze point

Excluded from bundle identity:

- warning text
- helper provenance
- human-readable canonicalization narratives
- counter formatting

Bundle invariants:

- query/result-shape compatibility is established exactly once
- all identity-bearing metadata is frozen at bundle creation
- all excluded metadata is non-authoritative
- counters correspond to this canonicalization run only

#### Identity And Equivalence Law

Milestone 1 must treat digest and equivalence this way:

- identical digest implies semantic equivalence within Milestone 1's admitted
  artifact space
- semantic equivalence does not require byte-identical authored form
- different digests for semantically equivalent admitted authored forms are a
  defect
- the same digest for semantically distinct admitted forms is a defect

#### Determinism Law

Canonicalization must be deterministic across:

- process restarts
- repeated runs
- host iteration-order variation
- replay scenarios
- compiler-irrelevant ordering differences in authored helper composition

### Proof-Carrying Artifact Pipeline

Milestone 1 must explicitly satisfy Architectural Laws 30 and 41.

The query pipeline in this milestone is not:

- builder in
- normalized struct out

It is a proof chain:

- `RawAuthoredQuery`
- `RawAuthoredResultShape`
- `CanonicalQueryArtifact`
- `CanonicalResultShapeArtifact`
- `CanonicalQueryBundle`

These types prove different things:

- `RawAuthoredQuery` proves only that the caller assembled an admitted authoring
  form
- `CanonicalQueryArtifact` proves normalization and canonical identity, but not
  schema legality
- `CanonicalResultShapeArtifact` proves result-shape normalization and stable
  field identity, but not legality against runtime schema
- `CanonicalQueryBundle` proves query/result-shape pairing, canonical digest
  stability, and bundle-level reporting, but not execution eligibility

Rules:

- constructors for proof-bearing types must be sealed to the proving module
- fields that encode proof-bearing transitions must not be publicly writable
- later milestones must consume the strongest available proof type rather than
  re-accepting raw authored inputs
- no type in Milestone 1 may claim validation, planning, or execution proof
  that has not yet occurred
- runtime checks for properties already guaranteed by canonicalization are a
  design failure

Representative progression:

```rust
pub struct RawAuthoredQuery { ... }
pub struct RawAuthoredResultShape { ... }

pub struct CanonicalQueryArtifact {
    digest: CanonicalQueryDigest,
    query_family: QueryFamily,
    projection: CanonicalProjection,
    traversal: CanonicalTraversal,
    clauses: CanonicalClauseSet,
}

pub struct CanonicalResultShapeArtifact {
    digest: CanonicalResultShapeDigest,
    shape_family: ResultShapeFamily,
    fields: CanonicalResultFieldSet,
}

pub struct CanonicalQueryBundle {
    query: CanonicalQueryArtifact,
    result_shape: CanonicalResultShapeArtifact,
    binding_descriptors: CanonicalBindingDescriptorSet,
    report: CanonicalizationReport,
    counters: CanonicalizationCounters,
}
```

## Query Surface And Abstraction Rules

### One Public Surface

Milestone 1 should introduce:

- `crates/worth-query/`
- `crates/worth-query/src/facade.rs`

The facade must expose query-owned concepts only:

- authoring entry points
- canonicalization entry points
- typed query artifacts
- typed result-shape artifacts
- binding descriptors
- canonicalization diagnostics

It must not expose:

- direct relational execution internals
- host-framework route adapters as the primary API
- alternate AST entry points that bypass canonicalization

### Smart Abstraction Boundary

Milestone 1 must respect the abstraction permissions from Laws 28 and 29.

Admitted abstraction:

- shared lifecycle across detail and collection authoring when the lifecycle is
  genuinely the same: author, normalize, bundle, report
- shared envelope categories for canonicalization output
- shared canonical clause containers where semantics and cost remain honest

Forbidden abstraction:

- one mega-query type that hides cost differences between detail, collection,
  traversal, and future aggregation families
- one generic "field-like thing" type that erases projection fields,
  traversal hops, derived fields, and result-shape fields
- one catch-all builder that can silently accumulate semantically distinct
  concepts because they all look like "clauses"
- one digest type that merges query identity, result-shape identity, and future
  execution basis

The rule is simple:

- abstract shared lifecycle
- do not abstract away semantic, cost, or proof boundaries

### Type-Bound Execution Preparation

The vision names generalized route-model binding as an important developer
experience outcome. Milestone 1 does not implement execution binding, but it
must reserve the correct ownership boundary now.

Milestone 1 must therefore introduce query-owned binding descriptors that:

- describe bindable slots or bindable roots in a typed way
- participate in canonical ordering where they affect query identity
- remain separate from host-specific route/context adapters

Milestone 1 must not:

- implement framework-specific route execution
- let host adapters become a second query authority
- allow binding metadata to smuggle in hidden filters, basis selection, or
  result-shape meaning

## Required Internal Subsystems

Milestone 1 should decompose into these internal subdomains:

- `authoring/`
  admitted raw query and result-shape authoring forms
- `canonicalization/`
  normalization, ordering, deduplication, and digest derivation
- `identity/`
  digests, equivalence contracts, and canonical identity types
- `result_shape/`
  typed result-shape families and field canonicalization
- `binding/`
  query-owned bindable descriptor types
- `diagnostics/`
  reports, warnings, and counter snapshots
- `harness/`
  milestone certification adapters and fixtures

This keeps authored intent, canonical proof, and diagnostics separate.

### Domain-Standards Compliance

Milestone 1 must follow the workspace domain standards from day one.

This means:

- organize by query subdomain, not file type
- default to more decomposition when responsibilities may grow apart later
- treat folders as architectural boundaries
- expose one root `facade.rs` only
- avoid catch-all files like `query.rs`, `helpers.rs`, `utils.rs`, or one giant
  `builder.rs` that mixes authoring, normalization, digesting, and diagnostics

### Expected Subdomain Layout

Milestone 1 should begin with a layout shaped like this:

```text
crates/worth-query/src/
  facade.rs
  lib.rs
  authoring/
    mod.rs
    detail.rs
    collection.rs
    projection.rs
    traversal.rs
    result_shape.rs
  canonicalization/
    mod.rs
    query.rs
    result_shape.rs
    ordering.rs
    deduplication.rs
    bundle.rs
  identity/
    mod.rs
    query_digest.rs
    result_shape_digest.rs
    equivalence.rs
  result_shape/
    mod.rs
    fields.rs
    families.rs
    compatibility.rs
  binding/
    mod.rs
    descriptors.rs
    slots.rs
  diagnostics/
    mod.rs
    report.rs
    warnings.rs
    counters.rs
  harness/
    mod.rs
    adapter.rs
    fixtures/
      mod.rs
      query_parity.rs
      result_shape_parity.rs
      binding_descriptor_parity.rs
    profiles.rs
    matrices.rs
```

Rules:

- query canonicalization and result-shape canonicalization are not the same
  responsibility just because both are "normalization"
- digest derivation is not the same responsibility as equivalence policy
- authoring forms are not the same responsibility as canonical proof types
- binding descriptors are not the same responsibility as host integrations

Subdomain responsibility boundaries:

- `authoring/` must not compute digests, infer compatibility, or retain hidden
  authority after raw authored artifact creation
- `canonicalization/` must not accept host adapter semantics or invent missing
  meaning from builder history
- `identity/` must not infer projection/result-shape compatibility
- `result_shape/` must not mutate query meaning or carry host execution hints
- `binding/` must not carry hidden filters, basis selectors, or policy context
- `diagnostics/` must not become semantic authority for query meaning,
  result-shape meaning, or compatibility
- `harness/` must not introduce alternate canonicalization logic separate from
  the production pipeline

## Phases

### Phase 1: Lock Canonical Query And Result-Shape Authority

Phase 1 exists to make query meaning structurally representable before any
later milestone can reinterpret it.

Milestone 1 must first define:

- admitted authored query families for detail, collection, projection, and
  bounded traversal
- admitted authored result-shape families
- canonical query artifact types
- canonical result-shape artifact types
- explicit equivalence contracts and digest bases

This phase leaves the system in a coherent state where:

- there is one authoritative query artifact model
- there is one authoritative result-shape artifact model
- alternate ASTs and builder-private semantic residue are explicitly out of
  spec

### Phase 2: Canonicalization Pipeline, Bundles, And Binding Descriptors

Phase 2 exists to turn the new artifact vocabulary into a real proof-carrying
pipeline.

Milestone 1 must then implement:

- normalization from admitted authored forms into canonical query artifacts
- normalization from authored result-shape forms into canonical result-shape
  artifacts
- canonical query bundles that pair the two with binding descriptors,
  canonicalization reports, and counters
- typed warnings and failure taxonomy for unsupported authored forms,
  canonicalization ambiguity, and digest-basis inconsistencies

This phase leaves the system in a coherent state where:

- the same authored intent lowers to the same canonical bundle every time
- result-shape meaning cannot drift from query meaning through helper layering
- future host binding surfaces have a query-owned place to attach without
  becoming authority

### Phase 3: Certification, Counter Proof, And Boundary Hardening

Phase 3 exists to prove that the artifact model is trustworthy and closed.

Milestone 1 must finally ship:

- milestone-native certification through the `Canonical Query Normalization
  Parity Test`
- hostile parity fixtures covering direct construction, builder composition,
  helper reordering, and admitted binding descriptor variation
- exact counter assertions for canonicalization breadth
- explicit rejection of alternate authoritative query ASTs and silent
  canonicalization fallbacks

This phase leaves the system in a coherent state where:

- canonicalization is certifiable rather than plausible
- later milestones can consume proof-bearing query artifacts instead of raw
  authored inputs
- future convenience APIs must lower into the same artifact model rather than
  inventing a second one

## Must Ship

- one `worth-query` crate and one public facade
- typed authored query families for:
  - entity/detail reads
  - collection reads
  - aspect projection
  - bounded relation traversal
  - typed result shapes
- `RawAuthoredQuery`, `CanonicalQueryArtifact`, `RawAuthoredResultShape`,
  `CanonicalResultShapeArtifact`, and `CanonicalQueryBundle` or materially
  equivalent proof-bearing types
- explicit canonical ordering, deduplication, and digest rules for query and
  result-shape artifacts
- query-owned binding descriptor types for future type-bound execution
- typed canonicalization diagnostics, warnings, and counters
- harness certification proving canonical parity across admitted construction
  paths

## Must Preserve

- query remains expression authority, not truth authority
- result shapes remain typed and structural rather than degrading into dynamic
  maps or post-fetch wrapper code
- structurally equal authored queries normalize identically
- builder sugar remains a projection onto the canonical artifact model rather
  than a parallel authority
- Milestone 1 proof types must not over-claim legality or execution readiness
- host code must not bypass the facade and invent alternate query ASTs

## Complexity / Proof Obligations

Milestone 1 must name its costs and proofs in terms of:

- clause count
- projection width
- traversal clause count
- result-shape field count
- binding descriptor count

Minimum required counters:

- `raw_clause_count`
- `normalized_clause_count`
- `projection_entry_count`
- `traversal_clause_count`
- `result_shape_field_count`
- `binding_descriptor_count`
- `query_deduplication_count`
- `result_shape_deduplication_count`
- `canonicalization_warning_count`
- `canonicalization_fallback_count`

Rules:

- counters belong to the canonical query bundle
- tests should assert exact values in representative parity scenarios
- any non-zero fallback counter on a supported path requires an explicitly
  admitted fallback class
- canonicalization complexity and digest parity must be stated at the
  canonicalization boundary, not as vague end-to-end claims

Normative representative scenarios:

- direct detail query
- equivalent builder-composed detail query
- collection query with reordered projections
- equivalent result-shape helper composition
- identity-bearing binding descriptor order variation
- duplicate clause deduplication case
- explicit unsupported authored form
- forbidden fallback case

These scenarios must have exact counter assertions in milestone certification.

## Allowed Debt

- ergonomic builder sugar may remain `Debt` if it lowers into the frozen
  canonical artifact model with parity proof
- richer result-shape families may remain `Debt` if shipped families are fully
  canonicalized and typed
- compile-time ergonomics may remain `Debt` where proof-carrying construction
  already exists through sealed constructors
- alternate authoritative query ASTs may not exist as debt
- post-fetch result-shape wrappers standing in for canonical result-shape
  artifacts may not exist as debt

## Failure Class Policy

Milestone 1 must classify canonicalization outcomes into explicit classes.

`Rejection`:

- used for unsupported authored forms
- used for identity-bearing ambiguity
- used for compatibility failures
- used for forbidden metadata or forbidden fallback attempts

`Warning`:

- used only for excluded-metadata disagreement that does not alter canonical
  meaning
- allowed on supported paths only when canonical digests remain unchanged

`Explicit Debt-Class Fallback`:

- Milestone 1 admits no meaning-changing fallback classes
- the only admitted fallback in Milestone 1 is diagnostic-only normalization of
  excluded metadata disagreement into warnings
- any widening, semantic repair, or host-context recovery is forbidden

`Internal Invariant Break`:

- used when sealed proof-carrying construction is bypassed internally
- used when bundle invariants cannot be satisfied after canonical artifact
  construction
- represents a bug, not user-authored rejection

Rules:

- supported paths may end with success or success-plus-warning only
- supported paths may not rely on meaning-changing fallback
- warnings may not conceal identity-bearing conflict
- internal invariant breaks may not be downgraded into warnings or user-facing
  fallback

## No Post-Canonical Semantic Residue Rule

After `CanonicalQueryBundle` construction, no later phase may require access
to:

- raw builder state
- helper nesting history
- fluent construction history
- host adapter state
- authored ordering before canonicalization

to interpret:

- query meaning
- result-shape meaning
- projection/result-shape compatibility
- binding descriptor identity

Excluded metadata may survive for diagnostics only, but it must never become
authority.

## Acceptance Evidence

Milestone 1 is complete only when `worth-query` can prove:

- the `Canonical Query Normalization Parity Test` in
  [test-requirements.md](./test-requirements.md)
  passes with canonical machine-checkable artifacts
- equivalent direct construction and builder/combinator construction produce
  identical `query_digest`
- equivalent result-shape construction paths produce identical
  `result_shape_digest`
- helper ordering and admitted binding descriptor ordering do not change
  canonical meaning
- `canonicalization_report` explicitly explains ordering, deduplication, and
  warning outcomes
- `counter_snapshot` proves normalization breadth and shows zero forbidden
  fallback or widening residue on supported paths

## Architectural Notes

### Law 41 Is The Load-Bearing Rule

The most important hardening rule in this milestone is that the type must
encode what has been proven.

That means:

- authored query types are not canonical query types
- canonical query types are not validated query types
- canonical result-shape types are not execution-ready result contracts
- future milestones must consume the strongest upstream proof type rather than
  re-accepting weaker forms

If an implementation exposes a public constructor for canonical artifacts that
does not pass through the proving function, the spec has been violated even if
the runtime tests still pass.

### DRY Means Reuse The Envelope Categories, Not The Meanings

Milestone 1 should reuse the category pattern for boundary envelopes, but it
must not create one generic "artifact envelope" whose fields mean different
things in different contexts.

Correct DRY:

- shared reporting structure
- shared digest policy helpers
- shared ordering helpers where the sort law is genuinely the same

Incorrect DRY:

- one generic clause bag for projection fields, traversal hops, and result
  fields
- one digest over everything because "it is all query-related"
- one host-binding-aware mega-builder that knows too much about later
  validation or planning

### Canonicalization Must Be Honest, Not Clever

Milestone 1 may use smart abstractions only when they preserve semantic,
correctness, and cost honesty.

The canonicalization layer should be smart in exactly these ways:

- collapse equivalent authored orderings into one canonical ordering
- deduplicate semantically identical clauses where equivalence is explicit
- preserve distinct meanings as distinct types even if the storage layout is
  similar

It must not be smart in these ways:

- infer omitted meaning from host context
- silently widen underspecified authoring forms into broader canonical reads
- merge distinct query families because one generic abstraction looks elegant

## Sequencing Notes

This belongs first because every later milestone depends on one canonical query
artifact and one canonical result-shape artifact rather than host-specific
construction paths.

Milestone 1 must land before:

- schema-aware legality, because legality must validate canonical artifacts
- execution planning, because planning must consume canonical artifacts
- saved-query and template work, because persistence requires one durable query
  identity
- view-shape and delivery-shape work, because those must extend typed
  result-shape authority rather than replace it

## Parallelization Notes

Once canonical query and result-shape identity are frozen:

- Milestone 2 validation work can proceed in parallel with polish on builder
  ergonomics
- early Milestone 3 planning prototypes can begin against the canonical proof
  types
- future host bindings can prototype against query-owned binding descriptors
  without becoming authority

## Explicit Failure Taxonomy For Milestone 1

Milestone 1 must ship typed failures for at least:

- unsupported authored query family
- unsupported authored result-shape family
- canonicalization ambiguity
- invalid canonical ordering basis
- digest-basis inconsistency
- duplicate binding descriptor conflict
- non-canonical helper residue detected during bundle assembly
- canonical artifact compatibility failure

These are query failures, not raw string bubbles.

## Anti-Patterns Explicitly Rejected

- alternate authoritative query ASTs hidden in builder internals
- dynamic map result shapes presented as typed query output
- host-specific route or framework adapters acting as query authority
- one mega-builder that erases detail/collection/traversal/result-shape
  distinctions
- public construction of proof-bearing canonical types without passing through
  the proving phase
- canonicalization that depends on insertion order or map iteration order
- silent widening or silent semantic repair during canonicalization

## Closeout Standard

Milestone 1 is complete only when all of the following are true:

- a dedicated `worth-query` crate and facade exist
- one canonical query artifact model exists
- one canonical result-shape artifact model exists
- proof-bearing canonicalization types distinguish raw authored forms from
  canonicalized forms
- direct construction, builder composition, and admitted helper layering all
  normalize to the same canonical bundle for the same declared intent
- binding descriptors remain query-owned metadata rather than a second query
  authority
- certification proves parity with canonical machine-checkable artifacts and
  exact counter evidence

If code lands but query meaning still depends on helper-local state, host
context, dynamic result wrappers, or non-canonical ordering, Milestone 1 is not
complete.
