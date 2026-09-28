# Milestone 2 Engineering Spec: Schema-Aware Validation, Predicate Legality, And Projection Semantics

> **Status:** Completed.
>
> **Roadmap parent:** [worth_query_roadmap.md](./WORTH_query_roadmap.md)
>
> **Vision parent:** [worth_query_vision.md](./WORTH_query_vision.md)
>
> **Prior milestone:** [milestone-1.md](./milestone-1.md)
>
> **Prior closeout:** [milestone-1-closeout.md](./milestone-1-closeout.md)
>
> **Closeout:** [milestone-2-closeout.md](./milestone-2-closeout.md)
>
> **Test requirements:** [test-requirements.md](./test-requirements.md)
>
> **Primary architectural driver:** make query legality a proof-bearing phase so canonical query meaning cannot drift into planning through schema mismatch, unsupported predicates, illegal traversal, or silent widening
>
> **Companion docs:**
> - [MENTALITY.md](../../docs/coding-guidelines/MENTALITY.md)
> - [arch_laws.md](../../docs/coding-guidelines/arch_laws.md)
> - [perf_laws.md](../../docs/coding-guidelines/perf_laws.md)
> - [domain_laws.md](../../docs/coding-guidelines/domain_structure_laws.md)
> - [worth_query_vision.md](./WORTH_query_vision.md)
> - [worth_query_roadmap.md](./WORTH_query_roadmap.md)
> - [test-requirements.md](./test-requirements.md)
> - [milestone-1.md](./milestone-1.md)
> - [milestone-1-closeout.md](./milestone-1-closeout.md)
> - [milestone-2.md](../WORTH-runtime-bridge/milestone-2.md)
> - [milestone-2-closeout.md](../WORTH-runtime-bridge/milestone-2-closeout.md)

## Goal

Make schema-aware legality a first-class proof phase so canonical queries,
predicates, projections, traversals, and typed result-shape bindings become
explicitly legal or explicitly rejected before planning or execution can begin.

## Why This Milestone Exists

Milestone 1 froze what a query is. Milestone 2 freezes whether that query is
allowed to exist as an executable read against a particular schema authority.

`worth-query` only becomes trustworthy if later milestones inherit validated
query meaning instead of re-deciding legality in planning, execution, live
promotion, or host adapters:

- planning must lower one legal query artifact, not rediscover whether a field
  exists or whether a predicate type-checks
- execution must consume already-narrowed aspect scope, not silently widen from
  invalid projection or unsupported predicate pressure
- live promotion must preserve the same legality basis as one-shot reads rather
  than accept query shapes that only happened to work on a runtime path
- future scopes, templates, saved queries, and tenant/schema variants must bind
  onto a validation phase that is already explicit and proof-carrying

Milestone 2 is therefore not "add filters." It is "make legality itself a
typed artifact boundary so every later phase can trust what has already been
proven and reject what has not."

## Governing Summaries

- `MENTALITY.md`: the hard problem is not syntax richness but fail-closed
  legality under schema pressure. The spec must solve that structural problem
  first and enforce it mechanically.
- `arch_laws.md`: Laws 3, 17, 24, 26, 27, 30, 40, and especially 41 dominate
  this milestone. Schema authority, applicability resolution, rejection before
  construction, explicit equivalence, and proof-bearing phase transitions all
  require validation to be a real type boundary rather than helper code.
- `perf_laws.md`: legality, narrowing, and policy/topology decisions belong
  before execution. Validation must preserve semantic delta, make widening
  denials observable, and expose exact counters at the validation boundary.
- `domain_laws.md`: query validation must decompose by responsibility. Predicate
  legality, projection legality, traversal legality, schema views, and
  diagnostics are separate subdomains, not one validation filing cabinet.
- `worth_query_vision.md`: schema-aware validation at construction time,
  workflow-aware predicates, structured content queries, aspect projection, and
  bounded traversal are all product theses, not optional polish.
- `worth_query_roadmap.md`: Milestone 2 exists to keep illegal, over-broad, or
  schema-dishonest queries from reaching planning. It must ship before
  planning, and it is not blocked on `worth-store`.
- `test-requirements.md`: the `Schema-Aware Rejection And Projection Legality
  Test` is the closeout proof. Legal queries must lower deterministically after
  validation, illegal queries must fail early and typed, and no silent
  whole-entity widening may occur.
- `milestone-1.md`: Milestone 1 froze canonical query and result-shape
  authority. Milestone 2 must consume those proof-bearing artifacts rather than
  re-accepting weaker authored forms.
- `milestone-1-closeout.md`: the current crate already has canonical query and
  result-shape artifacts, compile-fail boundaries, deterministic certification,
  and no alternate authority path. Milestone 2 must extend that proof chain,
  not puncture it.
- `worth-runtime-bridge` Milestone 2 spec/closeout: the strongest adjacent
  lesson is that precision becomes trustworthy only when the middle phase owns a
  canonical vocabulary, typed fallback policy, proof-carrying packets, and
  machine-checkable replay/certification artifacts. Query validation needs the
  same discipline for legality.

## Adversarial Constraint

Milestone 2 must survive the following hostile condition:

> Canonical queries built through different admitted construction paths are
> validated against evolving schema authority with rich aspect projection,
> typed predicates, bounded traversal, workflow predicates, structured-content
> clauses, and typed result-shape bindings. The same legal query intent must
> validate into the same proof-bearing legal artifact every time, and every
> illegal, unsupported, or over-broad form must fail explicitly before
> planning, with zero silent widening, zero host-local repair, and zero
> execution-path-dependent legality.

Concretely, the design must remain correct when all of the following are true:

- the same canonical query is presented with equivalent construction-path
  variation and diagnostics richness variation
- schema authority exposes aspects, fields, relations, and structured-content
  capabilities with explicit compatibility rules
- predicates target scalar fields, workflow-derived fields, and
  structured-content surfaces with different legality rules
- traversal requests are legal only over certain relation kinds or depth bounds
- result shapes request bindings that are structurally plausible but illegal for
  the projected or ordered query surface
- callers attempt unsupported or non-admitted shapes that a naive system would
  silently widen into whole-entity reads or post-filtered execution

If any supported path:

- lets planning rediscover whether a field, relation, or predicate is legal
- widens an invalid projection into a broader read rather than rejecting it
- accepts workflow or structured-content predicates through host-specific
  escape hatches rather than canonical validation
- changes legality outcome depending on builder path, diagnostics tier, or
  future execution path
- allows typed result shapes to reference material that was not legally
  projected

then Milestone 2 has failed.

## Product Decision Lock

The following decisions are explicit and not open questions for this
milestone:

- Milestone 2 introduces a distinct validated-query proof phase; canonical
  query artifacts from Milestone 1 are necessary but not sufficient for
  planning
- schema legality is resolved through query-owned validation over authoritative
  schema views supplied by `worth-relational`, not by ad hoc host callbacks
- validation may use compile-time proof where available, but the milestone bar
  is construction-time proof for every admitted query family
- unsupported or illegal query constructs must typed-fail before planning; they
  may not degrade into widening, post-filtering, or "best effort" execution
- workflow-aware predicates and structured-content queries are part of the
  validation surface in this milestone only to the extent that legality is made
  explicit; execution semantics remain later milestones
- result-shape legality remains structural and query-owned; host serializers may
  not reinterpret illegal bindings into valid delivery contracts

Normative consequence:

- any implementation path that lets the planner be the first phase to discover
  an unknown aspect, illegal field predicate, or illegal traversal is out of
  spec
- any implementation path that treats invalid projection or predicate requests
  as "read more and filter later" is out of spec
- any implementation path that permits workflow or structured-content legality
  through host-owned helper closures is out of spec
- any implementation path that produces a validated type without sealed proof
  construction is out of spec

## Compile-Time Enforcement Policy

Milestone 2 must explicitly classify which guarantees are enforced at type
construction time, which must be uncompilable, and which may remain
construction-time rejection.

`Unrepresentable` in public types:

- publicly constructible validated artifacts with missing schema basis
- publicly constructible validated artifacts missing validated predicate,
  projection, traversal, ordering, or result-shape proof surfaces
- public query families that encode unsupported validation states as ordinary
  user-buildable variants

`Uncompilable` through visibility and compile-fail enforcement:

- external construction of `ValidatedQueryArtifact`,
  `ValidatedResultShapeArtifact`, or `ValidatedQueryBundle` without the proving
  path
- public weaker-to-stronger conversions that bypass validation
- public access to internal schema-view machinery that would let callers
  fabricate validation proof or reinterpret schema authority outside the facade
- public planning entry points that still accept weaker canonical artifacts once
  the validated proof path exists

`Construction-time rejection`:

- unknown aspect, field, relation, workflow, or structured-content capabilities
- incompatible predicate family / field-kind combinations
- illegal ordering, traversal, or result-shape bindings
- schema basis incompatibility or unsupported compatibility class

Rules:

- the strongest boundary available must be used
- sealed constructors and `pub(crate)` boundaries are mandatory for validated
  proof types
- compile-fail harness coverage is required for the public proof-boundary rules
- runtime validation is allowed only for legality that genuinely depends on
  schema authority unavailable at Rust type-check time

## Scope

### In Scope

- validation of Milestone 1 canonical query and result-shape artifacts against
  authoritative schema views
- typed predicate legality over admitted scalar field, enum, membership, range,
  and pattern predicate families
- workflow-aware predicate legality where the schema explicitly exposes
  workflow-queryable fields or capabilities
- structured-content projection and predicate legality where the schema
  explicitly exposes queryable content structure
- bounded traversal legality over declared relation kinds and depth rules
- ordering legality over query-visible fields
- result-shape binding legality against validated projection and ordering basis
- typed validation diagnostics, failure taxonomy, counters, and certification
- proof-bearing validated query and validated result-shape artifacts for later
  planning

### Explicitly Out Of Scope

- execution planning
- snapshot-backed execution
- live promotion
- diff, historical, lineage, correspondence, policy, or tenant semantics
- saved-query persistence
- durable store-backed portability
- full query-time derived-field semantics or rollup execution semantics

Milestone 2 may reserve future-facing compatibility space for those later
milestones, but it must not fake their proofs early.

## Validation Architecture

### Validation Boundary Rule

Milestone 2 must introduce validation as a real proof-bearing boundary between
canonical meaning and executable meaning.

The governing rule is:

- Milestone 1 freezes what the query means
- Milestone 2 proves whether that query is legal against a declared schema
  authority and query-context legality surface
- Milestone 3 and later consume only the validated proof types

Validation must not be:

- a convenience method attached to planning
- a best-effort warning pass
- a set of loose booleans hanging off canonical artifacts
- a host-owned adapter callback that can reinterpret legality differently per
  consumer

Validation must produce types that encode:

- which schema authority basis was used
- which canonical query and result-shape artifacts were validated
- which clauses were admitted, normalized, or rejected
- that no forbidden widening or unsupported fallback was used

### Schema Authority Rule

Validation must consume authoritative schema semantics from `worth-relational`
through a query-owned schema view boundary. The validator may not infer schema
legality from host naming conventions, serialization shapes, or execution-path
availability.

The schema view boundary must own at least:

- canonical aspect identities
- canonical field identities and types
- relation kinds, directionality, and traversal legality metadata
- structured-content queryability declarations where admitted
- workflow-queryable capability declarations where admitted
- ordering/queryability metadata for fields used in predicates or ordering

The query crate owns validation over that schema view. It does not own schema
meaning itself.

Schema basis semantics must be explicit in this milestone, not deferred:

- validated artifacts are exact-basis-bound in Milestone 2
- `schema_basis` participates in validated identity as an exact canonical schema
  digest or materially equivalent exact authority marker
- future compatibility-class reuse may be added later, but it must be an
  explicit additional proof type rather than an interpretation change to
  `Validated*` identity
- a query validated against schema basis `A` may not be treated as validated
  against schema basis `B` without revalidation, even if a future schema
  evolution policy later decides the change was compatible

This rule exists so planning, caching, saved-query work, and later schema
evolution milestones inherit honest exact-basis validation rather than a vague
"probably still valid" contract.

### Predicate Legality Rule

Predicate legality must be structural, typed, and fail-closed.

Milestone 2 must admit a closed predicate family set and define legality for
each family against schema field kind and query context. Representative admitted
families include:

- equality and inequality over compatible scalar fields
- range and comparison over ordered scalar fields
- membership over fields with admitted set semantics
- pattern/text matching where schema marks the field queryable by that family
- null/presence checks where schema marks the field nullable or optional

Rules:

- predicate family legality depends on canonical field identity and schema field
  kind, not on host-side coercion
- logically equivalent predicate ordering must normalize deterministically
- unsupported predicate families must fail before validated artifact
  construction
- a predicate that would require whole-entity read, host-side script execution,
  or post-filtering to remain meaningful is non-admitted in Milestone 2

### Validated Predicate Equivalence And Conflict Law

Predicate validation must define not only whether each predicate is legal, but
how legal predicate sets normalize into one validated meaning.

Milestone 2 must define, for admitted predicate families:

- canonical ordering basis
- duplicate-collapse rules
- contradiction classification rules
- range normalization rules where range predicates are admitted
- commutativity basis for logically equivalent conjunction/disjunction surfaces
- conflict behavior when two legal predicates are jointly inconsistent

Rules:

- duplicate semantically identical predicates collapse to one canonical
  validated predicate entry
- excluded explanatory metadata disagreement may survive as diagnostics only
- predicates that are individually legal but jointly contradictory must produce
  a typed contradiction-class rejection or a typed unsatisfiable-query
  classification, but may not survive as a normal validated query artifact
- logically equivalent predicate sets that differ only in authored order or
  grouping within admitted equivalence rules must produce identical validated
  identity
- logically distinct predicate sets must not collapse just because they target
  the same field family

Representative normalization edges that must be specified during implementation:

- duplicate equality predicates on the same field/value
- contradictory equality predicates on the same field
- overlapping versus contradictory range predicates
- commutative membership predicates with different authored ordering
- workflow/content predicates that are semantically equal but differently
  authored through helper composition

### Projection Legality Rule

Projection legality must prove that every requested projected surface is both
declared by schema authority and queryable for the current admitted query
family.

Rules:

- unknown aspects fail explicitly
- unknown fields fail explicitly
- aspect projection may not widen to whole-entity fetch as a repair strategy
- query family restrictions on projection remain structural
- validated projection is part of the validated proof type and may not be
  re-expanded later without leaving the proof chain

### Traversal Legality Rule

Traversal legality must remain bounded and schema-declared.

Rules:

- traversal may only target declared relation kinds
- relation direction and depth legality must be resolved during validation
- unsupported or unbounded traversal requests must fail before planning
- traversal legality must not be discovered through live graph walking or
  execution-time probing
- canonical traversal meaning from Milestone 1 remains authoritative; Milestone
  2 only proves legality, it does not reinterpret traversal intent

### Structured Content Legality Rule

Structured-content legality must be explicit and schema-owned. Milestone 2 may
admit structured-content query forms only where schema authority marks a content
aspect or content block family as queryable.

Admitted Milestone 2 structured-content legality surfaces include:

- projection of declared content block families
- predicates over declared structured-content block properties
- presence/existence checks for declared content block families

Forbidden Milestone 2 structured-content shortcuts:

- treating arbitrary rich text as queryable because it is serializable
- host-side parsing or scanning of opaque content blobs as validation
- projecting an entire content blob because a substructure query was illegal
- accepting content predicates whose semantics depend on non-schema parser
  plugins or framework-local content conventions

If structured content is not queryable according to schema authority, the query
must fail typed and early.

### Workflow Predicate Legality Rule

Workflow-aware predicates are admitted only where the schema authority and query
context explicitly expose workflow-queryable semantics.

Milestone 2 must not permit workflow predicates as vague convenience sugar over
host-local state. They must be represented canonically and validated against a
declared workflow-queryable surface.

Rules:

- workflow predicates must lower into canonical predicate identities, not stay
  as host closures or string labels
- legality must establish that the requested workflow concept is queryable for
  the target entity/aspect context
- workflow context shape mismatches must fail explicitly
- workflow-aware predicates remain part of query legality only; execution,
  policy interaction, and live semantics remain later milestones

### Result-Shape Binding Legality Rule

Milestone 2 must prove that typed result-shape bindings remain legal against the
validated query surface.

This extends the Milestone 1 compatibility law from "references projected
material" to "references legally projected and order-compatible material."

Rules:

- result-shape fields may reference only validated projected material
- result-shape fields may not rely on non-admitted derived-field semantics in
  this milestone
- ordering-dependent delivery bindings must target fields whose ordering basis
  is itself legal
- result-shape legality failures must remain structural and query-owned, not
  serializer-owned

Representative failure classes:

- illegal result-shape binding target
- result-shape field references non-queryable structured-content surface
- result-shape field requires non-admitted derived or aggregate semantics

### Ordering Legality Rule

Ordering legality must be resolved during validation.

Rules:

- only schema-declared orderable fields may participate in ordering
- ordering over fields absent from validated projection may be admitted only if
  the query artifact explicitly preserves that field as ordering-only authority
- ordering may not smuggle whole-entity reads or hidden fields into planning
- unsupported ordering families must fail explicitly before validated artifact
  construction
- canonical ordering basis must become part of the validated proof type so the
  planner does not rediscover it

### Ordering-Only Authority Law

If Milestone 2 admits ordering over fields absent from the delivered projection,
that authority must be represented as a first-class validated artifact surface
rather than hidden query metadata.

Rules:

- ordering-only fields live in a distinct validated ordering proof surface, not
  inside projected delivery fields and not as host-local planner hints
- ordering-only authority participates in validated query identity because it
  changes legal execution and later result semantics
- ordering-only authority does not participate in validated result-shape
  identity unless it is also projected or otherwise delivery-bearing
- later phases may consume ordering-only authority for planning and stable
  delivery semantics, but may not expose it as projected data unless the query
  also legally projected it
- result-shape legality must explicitly prove that an ordering-only field is not
  being smuggled into delivery through aliases, hidden serializer behavior, or
  cursor metadata that changes semantic output

### Failure Class Policy

Milestone 2 must classify validation outcomes into explicit classes.

`Rejection`:

- unknown aspect, field, relation, or workflow predicate capability
- incompatible predicate family for field kind
- illegal structured-content projection or predicate
- illegal traversal edge or depth
- illegal ordering target
- illegal result-shape binding
- schema/query-family mismatch
- forbidden widening attempt

`Warning`:

- Milestone 2 admits warning-only outcomes only for excluded explanatory
  metadata disagreement that does not alter legality or narrowing
- no legality-bearing clause may degrade into warning

`Explicit Debt-Class Fallback`:

- Milestone 2 admits no meaning-changing fallback classes
- no whole-entity widening, host-side filtering, or hidden projection expansion
  may ship as debt

`Internal Invariant Break`:

- validation bundle or schema-view proof surfaces drift internally
- validated artifacts are found inconsistent after sealed construction
- a later phase attempts to manufacture validated types without the proving
  path

Rules:

- supported paths may end only in success or success-plus-nonsemantic-warning
- all illegal or unsupported constructs fail before planning
- internal invariant breaks are bugs, not user-authored validation failures

## Proof-Carrying Type Model

Law 41 is load-bearing here.

Milestone 2 must introduce proof-bearing types that make it impossible to
confuse canonical-but-unvalidated query meaning with schema-legal query meaning.

Representative progression:

```rust
pub struct CanonicalQueryArtifact { ... }
pub struct CanonicalResultShapeArtifact { ... }
pub struct CanonicalQueryBundle { ... }

pub struct QuerySchemaView { ... }
pub struct ValidatedPredicateSet { ... }
pub struct ValidatedProjectionSet { ... }
pub struct ValidatedTraversalSet { ... }
pub struct ValidatedOrderingSet { ... }

pub struct ValidatedQueryArtifact {
    canonical_query: CanonicalQueryArtifact,
    schema_basis: SchemaBasisDigest,
    predicates: ValidatedPredicateSet,
    projection: ValidatedProjectionSet,
    traversal: ValidatedTraversalSet,
    ordering: ValidatedOrderingSet,
}

pub struct ValidatedResultShapeArtifact {
    canonical_result_shape: CanonicalResultShapeArtifact,
    schema_basis: SchemaBasisDigest,
}

pub struct ValidatedQueryBundle {
    query: ValidatedQueryArtifact,
    result_shape: ValidatedResultShapeArtifact,
    validation_report: QueryValidationReport,
    counters: QueryValidationCounters,
}
```

Rules:

- validated types must be sealed and privately constructed
- validation functions consume canonical proof types and produce validated proof
  types
- later milestones must consume `Validated*` types rather than re-accepting
  weaker canonical artifacts
- runtime checks for properties already guaranteed by validation are design
  failures unless the boundary is untrusted
- validated types must carry the schema basis used so later schema variation
  milestones have an honest substrate

## Validated Digest Basis Rules

Milestone 2 must define validated identity as explicitly as Milestone 1 defined
canonical identity.

### Validated Query Artifact

Included in validated query identity:

- canonical query digest from Milestone 1
- exact schema basis identity
- normalized validated predicate set
- validated projection legality surface
- validated traversal legality surface
- validated ordering legality surface, including ordering-only authority
- admitted workflow-query legality bindings
- admitted structured-content legality bindings

Excluded from validated query identity:

- authored predicate ordering before validation
- diagnostics richness
- rejection trace richness for successful paths
- host labels, route names, UI labels, and other excluded nonsemantic metadata

Normalized ordering key:

- schema basis first
- canonical query identity second
- validated predicate entries by canonical field identity then family identity
- validated traversal entries by canonical relation path identity
- validated ordering entries by canonical ordering field identity and direction
- workflow/content legality markers by canonical capability identity

Equivalence relation:

- two validated queries are equivalent only if they share the same exact schema
  basis and the same normalized validated meaning across predicates, projection,
  traversal, ordering, workflow legality, and structured-content legality

Conflict behavior:

- contradiction in validated meaning is rejection, not a distinct successful
  identity
- legality-bearing disagreement may not collapse into warning-only residue

### Validated Result-Shape Artifact

Included in validated result-shape identity:

- canonical result-shape digest from Milestone 1
- exact schema basis identity
- normalized validated result-shape binding legality surface
- any ordering-only legality markers that materially affect delivered-shape
  admissibility

Excluded from validated result-shape identity:

- diagnostics richness
- nonsemantic explanatory labels

Equivalence relation:

- two validated result shapes are equivalent only if they share the same exact
  schema basis and the same normalized binding legality meaning

### Validated Query Bundle

Included in validated bundle identity:

- validated query identity
- validated result-shape identity
- validation compatibility relation between them

Excluded from validated bundle identity:

- validation report richness
- counter richness beyond canonical counter snapshot fields required for bundle
  comparison

Rules:

- semantically equivalent validated artifacts must produce identical validated
  identity
- semantically distinct validated artifacts must not collapse just because they
  share the same canonical Milestone 1 digest
- exact schema basis identity is part of validated meaning in Milestone 2

## Query Surface And Abstraction Rules

### One Public Surface

Milestone 2 must continue extending the existing `worth-query` facade rather
than introducing a second validation API family.

The facade should grow query-owned concepts only:

- schema-view-backed validation entry points
- validated query and validated result-shape proof types
- typed predicate expression families
- validation diagnostics, counters, and reports

It must not expose:

- direct relational schema internals as the public query API
- host-framework predicate adapters as the primary legality surface
- execution planning or runtime reads under validation-shaped names

### Smart Abstraction Boundary

Milestone 2 must respect Laws 28 and 29.

Admitted abstraction:

- shared validation lifecycle across predicate, projection, traversal, and
  ordering legality where the lifecycle is genuinely the same: inspect schema,
  prove legality, normalize proof, report counters
- shared envelope categories for validated artifacts and validation reports
- shared comparator/digest helpers where equivalence contracts remain explicit

Forbidden abstraction:

- one mega-validator that erases the distinction between predicate legality,
  traversal legality, structured-content legality, and result-shape legality
- one generic "queryable thing" abstraction that hides cost or failure-mode
  differences between scalar fields, relation edges, workflow fields, and
  structured-content blocks
- one bag-shaped schema adapter that lets host code reinterpret legality by
  convention

The rule is the same as Milestone 1:

- abstract shared lifecycle
- do not abstract away semantic, cost, or proof boundaries

## Required Internal Subsystems

Milestone 2 should decompose into internal subdomains shaped like:

- `schema_view/`
  query-owned schema authority views, basis identity, and queryability metadata
- `validation/predicates/`
  predicate family legality and normalized validated predicate proofs
- `validation/projection/`
  aspect and field projection legality
- `validation/traversal/`
  relation-kind and depth legality
- `validation/ordering/`
  ordering legality and ordering-only proof surfaces
- `validation/result_shape/`
  validated result-shape binding legality
- `validation/workflow/`
  workflow predicate legality
- `validation/structured_content/`
  structured-content legality
- `validation/bundle/`
  validated query/result-shape bundle assembly
- `diagnostics/validation/`
  validation reports, rejection matrices, counters, and failure digests
- `harness/`
  milestone-native certification adapters, fixtures, and rejection matrices

Subdomain rules:

- `schema_view/` must not become query planning or execution authority
- `validation/predicates/` must not inspect live runtime state to rescue illegal
  predicates
- `validation/result_shape/` must not mutate canonical query meaning
- `diagnostics/validation/` must not become the source of legality truth
- `harness/` must not introduce alternate validation logic separate from the
  production pipeline

## Phases

### Phase 1: Freeze Schema-Carrying Validation Inputs

Phase 1 exists to define exactly what validation is allowed to know and what it
is allowed to prove.

Milestone 2 must first define:

- query-owned schema-view artifacts and schema basis identity
- admitted predicate families and their legality domains
- admitted structured-content and workflow-query legality surfaces
- admitted ordering and traversal legality rules
- validated artifact boundaries and sealed constructors

This phase leaves the system in a coherent state where:

- validation has one authoritative schema-view input
- legality is defined as an explicit contract rather than inferred later
- unsupported query families are named and fail-closed instead of drifting

### Phase 2: Validate Canonical Queries Into Proof-Bearing Legal Query Artifacts

Phase 2 exists to make the legality vocabulary operational.

Milestone 2 must then implement:

- validation of canonical projection, predicate, traversal, ordering, and
  result-shape surfaces
- deterministic normalization of validated clause sets
- typed validation reports and rejection bundles
- validated query and validated result-shape artifacts carrying schema basis
  proof
- validated bundle assembly with exact counters and zero forbidden widening

This phase leaves the system in a coherent state where:

- the same canonical query and schema basis validate into the same proof bundle
- illegal queries fail before planning
- later milestones can consume validated proof types instead of re-deriving
  legality

### Phase 3: Certification, Counter Proof, And Boundary Hardening

Phase 3 exists to prove that the validation boundary is closed and trustworthy.

Milestone 2 must finally ship:

- milestone-native certification through the `Schema-Aware Rejection And
  Projection Legality Test`
- hostile parity fixtures covering legal and illegal predicate, projection,
  traversal, workflow, and structured-content scenarios
- exact counter assertions for representative admitted and rejected cases
- compile-time or harness hardening proving that validated types and schema-view
  surfaces cannot be bypassed into planning through weaker forms

This phase leaves the system in a coherent state where:

- validation is certifiable rather than plausible
- planning can treat legality as upstream proof rather than runtime work
- later query surfaces must lower into the same validated artifact model

## Must Ship

- one query-owned schema-view boundary with schema basis identity
- proof-bearing validated query, validated result-shape, and validated bundle
  artifact families or materially equivalent types
- typed predicate family surfaces for admitted scalar, workflow, and
  structured-content legality classes
- structural legality checks for:
  - aspect and field projection
  - predicate family compatibility
  - bounded traversal
  - ordering
  - typed result-shape bindings
  - workflow-aware predicate admission
  - structured-content projection/predicate admission
- typed validation reports, rejection digests, and exact counters
- milestone-native certification proving legal determinism and illegal early
  rejection
- sealed validation construction so planning cannot consume weaker forms by
  accident

## Must Preserve

- canonical query meaning from Milestone 1 remains authoritative
- validation consumes schema authority but does not become schema authority
- planning, execution, and live surfaces remain out of scope and may not leak
  into the validator
- no silent widening from invalid or unsupported forms
- no host-local legality repair or serializer-owned binding repair
- no alternate public validation path outside the query facade
- validated artifacts remain narrower proof types than future planned artifacts

## Complexity / Proof Obligations

Milestone 2 must name costs and proofs in terms of:

- predicate count
- projection width
- traversal clause count
- result-shape field count
- ordering field count
- schema-view field/aspect lookups performed

Minimum required counters:

- `validated_predicate_count`
- `validated_projection_entry_count`
- `validated_traversal_clause_count`
- `validated_result_shape_binding_count`
- `validated_ordering_field_count`
- `schema_lookup_count`
- `validation_rejection_count`
- `projection_widening_denial_count`
- `validation_warning_count`
- `validation_fallback_count`

Rules:

- counters belong to the validated query bundle
- representative certification scenarios must assert exact counts
- any non-zero fallback count on a supported path is forbidden in Milestone 2
- widening denials must be structurally counted instead of disappearing into
  generic rejection counts
- validation complexity claims must be stated at the validation boundary, not
  hidden inside later planning or execution measurements

Normative representative scenarios:

- legal detail query with typed scalar predicates
- equivalent builder-composed legal query with the same schema basis
- illegal unknown-aspect projection
- illegal predicate family against incompatible field kind
- illegal traversal edge or depth
- illegal structured-content predicate
- illegal workflow predicate context/shape
- illegal result-shape binding
- forbidden widening case

### Milestone 2 Validation Certification Matrix

Milestone 2 closeout must include a named validation certification matrix rather
than only loosely related test cases.

The matrix is not optional supporting evidence. It is the closeout surface for
Milestone 2 legality claims. "Covered elsewhere in tests" does not satisfy this
requirement.

Milestone 2 must ship one named certification artifact for this suite, with a
stable machine-checkable aggregate output materially equivalent to:

- `schema_aware_rejection_and_projection_legality_certification_artifact`
- `validation_certification_matrix`
- `bundle_completeness_report`

The artifact must make it possible to determine, offline and without ambient
test interpretation, whether:

- required canonical rows exist
- required rejection rows exist
- each row emitted the required outputs
- each supported lane preserved zero forbidden widening and zero forbidden
  fallback residue
- Milestone 2 is fully complete versus only partially implemented

Minimum required canonical rows:

- `legal-detail-query-parity`
- `equivalent-builder-composed-legal-query`
- `legal-structured-content-query-parity` where structured-content legality is
  admitted
- `legal-workflow-predicate-parity` where workflow legality is admitted

Minimum required rejection rows:

- `unknown-aspect-projection`
- `incompatible-predicate-family`
- `illegal-traversal-edge-or-depth`
- `invalid-result-shape-binding`
- `structured-content-illegality`
- `workflow-context-illegality`
- `forbidden-widening-case`

Each row must emit or reference:

- `query_digest`
- `failure_digest` where rejected
- `validation_rejection_matrix`
- `counter_snapshot`

And each row must prove one of:

- equivalent validated identity
- intentionally distinct validated identity
- typed early rejection before planning

Rules:

- required rows may not be satisfied by nearby or semantically similar rows
- if a required row is not yet implementable because the underlying admitted
  surface does not yet exist, the certification artifact must mark that row as
  unmet rather than silently omitting it
- Milestone 2 may not be declared closed while any required row remains unmet
- partial matrix coverage may be used for implementation progress tracking, but
  not for milestone closeout

## Allowed Debt

- compile-time proof for some schema-derived legality may remain `Debt` where
  construction-time proof is already canonical and sealed
- richer predicate families may remain `Debt` if admitted families are fully
  typed, validated, and parity-proven
- richer structured-content operators may remain `Debt` if admitted structured
  content legality is explicit and fail-closed
- host-side filtering, whole-entity widening, or planner-owned legality may not
  exist as debt

## Acceptance Evidence

Milestone 2 is complete only when `worth-query` can prove:

- the `Schema-Aware Rejection And Projection Legality Test` in
  [test-requirements.md](./test-requirements.md)
  passes with canonical machine-checkable artifacts
- legal queries validate into identical proof-bearing validated artifacts for
  identical canonical meaning and schema basis
- illegal queries fail during validation rather than planning or execution
- `validation_rejection_matrix` localizes unknown aspects, incompatible
  predicates, illegal traversals, invalid result-shape bindings, structured
  content illegality, and workflow predicate illegality
- `counter_snapshot` proves exact validation breadth and shows zero forbidden
  widening or fallback residue on supported paths
- the named Milestone 2 validation certification matrix exists and explicitly
  carries the required parity and rejection rows for milestone closeout

## Architectural Notes

### Law 41 Is Still The Load-Bearing Rule

The most important hardening rule in this milestone is still that the type must
encode what has been proven.

That means:

- canonical query types are not validated query types
- validated query types are not planned query types
- validated result-shape types are not execution-ready delivery contracts
- future milestones must consume the strongest upstream proof type rather than
  re-accept weaker canonical bundles

If an implementation exposes a public constructor for validated artifacts that
does not pass through the proving function, the milestone has been violated
even if feature tests still pass.

### Validation Must Be Honest, Not Helpful

Milestone 2 may use smart abstractions only where they preserve legality,
correctness, and cost honesty.

The validator may be smart in these ways:

- normalize equivalent legal predicate orderings
- reuse shared schema-view lookup logic
- collapse duplicate legal clauses where equivalence is explicit

It must not be smart in these ways:

- infer missing legality from host context
- silently broaden an invalid query into a broader legal one
- treat non-queryable structured content as queryable because a parser happens
  to exist
- convert workflow predicates into host-local filters that bypass canonical
  validation

### DRY Means Shared Validation Lifecycle, Not One Validation Blob

Correct DRY:

- one schema-view envelope pattern
- shared validation report categories
- shared proof-bearing validated bundle categories

Incorrect DRY:

- one generic clause validator that erases field, traversal, workflow, and
  content distinctions
- one fallback bucket for all rejection classes
- one "queryable target" type that merges relation edges, scalar fields,
  structured-content blocks, and workflow capabilities

## Sequencing Notes

This belongs second because planning cannot honestly exist until legality is
explicit and proof-bearing.

Milestone 2 must land before:

- planning, because planners must consume validated artifacts rather than
  rediscover legality
- execution, because execution-path-dependent legality would destroy parity
- scopes/templates/saved queries, because reusable query artifacts need a
  stable legality substrate
- policy and tenant schema work, because those later milestones need an honest
  validated-schema basis to extend rather than replace

## Parallelization Notes

Once the validation proof boundary is frozen:

- Milestone 3 planning can begin against `ValidatedQueryBundle`
- richer builder ergonomics can proceed in parallel as long as they lower into
  the same canonical and validated artifact paths
- early schema-view adapter experimentation can proceed without leaking into the
  public query facade
- future workflow/content surface growth can proceed by extending admitted
  legality families rather than inventing a second validator

## Explicit Failure Taxonomy For Milestone 2

Milestone 2 must ship typed failures for at least:

- unknown aspect projection
- unknown field projection
- incompatible predicate family
- unsupported predicate family
- illegal traversal relation kind
- illegal traversal depth
- unsupported structured-content projection
- unsupported structured-content predicate
- illegal workflow predicate capability or context shape
- illegal ordering field
- illegal result-shape binding
- schema basis incompatibility
- projection widening denied
- validated bundle compatibility failure
- validation artifact invariant break

These are query validation failures, not raw strings or borrowed runtime errors.

## Anti-Patterns Explicitly Rejected

- letting planning rediscover schema legality
- whole-entity widening as a repair strategy for invalid projection
- host-owned closures or strings as workflow predicate authority
- treating arbitrary content blobs as queryable structured content
- serializer-owned fixes for illegal result-shape bindings
- one mega-validator that mixes schema views, predicate legality, traversal
  legality, and diagnostics in one file or type
- public construction of validated proof types without the proving path
- any validation path whose result depends on builder order, diagnostics tier,
  or future execution path

## Self-Check

This milestone solves a real structural problem rather than packaging work
cosmetically because it introduces the first proof-bearing legality boundary
between canonical query meaning and later planning/execution.

The adversarial constraint is load-bearing because it forbids the naive failure
mode of letting invalid or unsupported shapes drift into planning through host
repair, silent widening, or execution-path-dependent legality.

The milestone preserves authority boundaries because `worth-relational` still
owns schema semantics, `worth-query` owns legality proof over a query-owned
schema view, and planning/execution remain later consumers rather than hidden
validation participants.

The milestone defines proof obligations rather than implementation chores
because deterministic validated artifacts, rejection matrices, widening-denial
counters, and early typed failure are required for closeout.

A competent engineer should be able to map this spec into honest schema-view
types, validation subdomains, validated proof types, counters, and
certification harnesses without inventing architecture during implementation.

This milestone belongs second in the roadmap because legality must be frozen
before planning, execution, live promotion, or reusable query composition can
be honest.

## Closeout Standard

Milestone 2 is complete only when all of the following are true:

- a query-owned schema-view boundary exists
- validated query and validated result-shape proof types exist
- canonical query artifacts from Milestone 1 flow into validation and only
  validated artifacts flow out toward planning
- illegal projection, predicate, traversal, workflow, structured-content, and
  result-shape cases fail typed and early
- legal queries validate deterministically for the same schema basis
- no supported path widens or falls back silently
- certification proves legal determinism, illegal early rejection, and exact
  validation counter behavior with canonical machine-checkable artifacts

If code lands but legality still depends on host repair, execution discovery,
whole-entity widening, or non-sealed validated types, Milestone 2 is not
complete.
