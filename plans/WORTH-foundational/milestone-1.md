# Milestone 1: Aspec-Native Canonical Value And Aspect State Substrate

## Goal

Establish the shared canonical value language and aspect-state vocabulary that
every later `worth-foundational` surface depends on.

This milestone creates the first real capability boundary for
`worth-foundational`: crates may keep their own optimized internal
representations, but when they exchange aspect-native truth at a boundary, they
must be able to speak one shared value, aspect-state, patch, identity, and
locator language without semantic reinterpretation.

## Governing Document Summaries

### `MENTALITY.md`

Protects hard-problem-first engineering and mechanical enforcement over
convention. The shaping constraint for this milestone is that value and aspect
truth cannot begin as a loose compatibility bag; the canonical vocabulary must
exist before downstream diagnostics, digests, receipts, or migrations depend on
it.

### `arch_laws.md`

Protects contractual facades, proof-bearing phase boundaries, one canonical
artifact, authority over derivation, and type-level semantic distinctions. The
shaping constraint is that `worth-foundational` must standardize boundary
meaning without becoming a runtime or collapsing identities, handles, basis ids,
authoritative values, derived artifacts, and locators into generic containers.

### `worth-proof`

Protects compile-time proof-bearing progression, witnesses, typed transition
outcomes, fixed-shape proof collections, and phase markers. The shaping
constraint is that Milestone 1 should depend on `worth-proof` for
proof-bearing progression surfaces such as contract validation, evolution
classification, compatibility lowering, authoritative-state admission, and
digest-preparation readiness, while keeping raw foundational vocabulary plain
and lightweight.

### `composition_laws.md`

Protects files and functions as responsibility-shaped semantic units. The
shaping constraint is that this milestone must map to predictable modules such
as values, aspects, identities, locators, compatibility, and canonicalization
support rather than a convenience `common` or `types` bucket.

### `domain_structure_laws.md`

Protects filesystem structure as responsibility topology rather than storage.
The shaping constraint is that authoritative aspect truth, compatibility
bridges, identity vocabulary, locator vocabulary, and derived boundary helpers
must occupy separate structural homes whose names reject unrelated future code.

### `perf_laws.md`

Protects semantic-delta-bounded execution, cost-honest boundaries, explicit
layout vocabulary, and no repeated rediscovery of facts. The shaping constraint
is that canonical value materialization must be a boundary operation, not a
forced universal hot-path representation or hidden JSON reconstruction step.

### `worth_foundational_vision.md`

Protects the thesis that WORTH needs one shared semantic and boundary-contract
language without forcing one runtime representation. The shaping constraint is
that Aspec-native values, aspect keys, aspect state, aspect patches, identity
categories, locators, and compatibility debt are the first substrate because
all later diagnostics, digests, reports, receipts, and profiles depend on them.

### `worth_foundational_roadmap.md`

Protects the sequencing rule: standardize shared meaning once and preserve
local representation freedom always. The shaping constraint is that Milestone 1
must establish value and aspect-state primitives before canonical digests,
profiles, materialization contracts, diagnostics, lineage, or cross-crate
migrations can honestly close.

### `worth-foundational` Test Requirements

`test-requirements.md` protects the proof bar for the new implementation plan:
build `worth-foundational` as a complete shared semantic crate before broad
adopting-crate refactors. The shaping constraint is that Milestone 1 must
certify value, contract, mask, patch, state, locator, compatibility, and
digest-preparation semantics in isolation rather than depending on incremental
crate migrations to reveal mistakes.

### `worktree-4` Relational Aspect-Native Rewrite Gate

Protects the move from JSON-shaped authoritative payload truth to native
aspect-state truth in `worth-relational`. The shaping constraint is that
Milestone 1 must adopt the exact relational aspect-value families already
defined there as the starting canonical vocabulary, because that branch is the
current database of the native aspect truth model.

## Why This Milestone Exists

`worth-foundational` cannot begin with diagnostics, receipts, provenance,
profiles, or digest tooling, because those surfaces all need to carry values,
point at aspect/field loci, and distinguish authoritative state from derived
descriptions.

The first milestone therefore defines the substrate every later milestone
consumes:

- boundary-safe canonical scalar values
- boundary-safe canonical composite values where an aspect is naturally a
  schema-declared structure rather than a scalar
- aspect keys
- aspect contracts that declare value shape, admissible masks, absence/null
  law, equivalence basis, and evolution posture
- canonical aspect-state maps
- authoritative record aspect-state wrappers
- authoritative aspect patches
- field masks and aspect masks for selection, update, projection, and
  diagnostics without treating masks as values
- typed identity, handle, key, and basis-id categories
- structural locators for values, aspects, fields, and boundary artifacts
- compatibility bridges that explicitly lower legacy JSON-originated inputs
  into canonical aspect-native meaning

This belongs first in the roadmap because a digest substrate cannot be honest
until there is a canonical value basis to digest, and cross-crate migrations
cannot be honest until there is a shared aspect-state language to migrate to.

## Adversarial Constraint

Several WORTH crates must be able to independently construct semantically
identical aspect-native values, aspect states, patches, and locators from
different local layouts or transitional compatibility inputs, then exchange
those boundary forms such that ordering, equality, patch application, and
digest-preparation meaning are identical without requiring producer-private
state, JSON object ordering, or crate-local aspect folklore.

This milestone fails if:

- JSON-shaped payloads remain the canonical long-term meaning of boundary
  values
- two crates can construct the same aspect state with different ordering or
  equality semantics
- aspect patches rely on object-merge folklore instead of explicit set/clear
  law
- schema-declared struct values are forced through opaque JSON or string/blob
  hacks
- masks are confused with values instead of being explicit selector/update
  contracts over canonical locators
- absence, null, clear, and default semantics are left to crate-local folklore
- aspect schema evolution is deferred until after digests or migrations depend
  on unstable shape assumptions
- equality, reuse, and suppression claims can be made without an explicit
  aspect equivalence basis
- identity, handle, key, and basis-id categories collapse because their
  underlying representation is identical
- locators become string paths whose interpretation depends on producer-private
  conventions
- the shared value vocabulary forces one hot-path runtime value bag on crates
  that need different internal layouts

## Relational Aspect Value Source Of Truth

Milestone 1 must carry forward the exact value families already defined in
`worktree-4` under
`crates/worth-relational/src/payloads/data/aspect_values.rs`.

The initial foundational vocabulary must preserve these exact aspect-value
forms and supporting canonical wrappers:

```rust
pub struct ContentRefId(pub u64);

pub struct CanonicalF32(pub u32);

pub struct CanonicalF64(pub u64);

pub struct CanonicalDecimal(pub String);

pub struct CanonicalBigInt(pub String);

pub struct CanonicalRational {
    pub numerator: CanonicalBigInt,
    pub denominator: CanonicalBigInt,
}

pub struct CanonicalDate {
    pub days_from_unix_epoch: i32,
}

pub struct CanonicalTime {
    pub nanos_since_midnight: u64,
}

pub struct CanonicalTimestamp {
    pub micros_since_unix_epoch: i64,
}

pub struct CanonicalTimestampTz {
    pub utc_micros_since_unix_epoch: i64,
    pub offset_minutes: i32,
}

pub enum AspectValue {
    Null,
    Bool(bool),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    Float32(CanonicalF32),
    Float64(CanonicalF64),
    Decimal(CanonicalDecimal),
    BigInt(CanonicalBigInt),
    Rational(CanonicalRational),
    String(InternedString),
    Bytes(ContentRefId),
    Uuid([u8; 16]),
    Date(CanonicalDate),
    Time(CanonicalTime),
    Timestamp(CanonicalTimestamp),
    TimestampTz(CanonicalTimestampTz),
    EntityRef(EntityId),
    ContentRef(ContentRefId),
}
```

This block is intentionally treated as a vocabulary lock, not illustrative
pseudocode. Naming may move into `worth-foundational` namespaces during
implementation, but the semantic families must not be narrowed, renamed into
JSON-era concepts, or replaced by `serde_json::Value`.

## Aspect Completeness Requirements

An aspect is complete only when the crate can answer more than "what value is
stored under this key?" It must also answer:

- what authority declared the aspect
- what shape the aspect admits
- whether the value is scalar, structured, collection-like, opaque, or
  reference-bearing
- which masks are legal over that shape
- what absence, null, clear, and default mean
- what equality, comparison, ordering, and equivalence basis apply
- whether the aspect can be indexed, projected, patched, merged, digested, or
  represented only opaquely
- how the aspect declaration can evolve without changing the meaning of
  already-committed boundary artifacts

The platform-grade trap is treating `AspectValue` as the whole aspect model.
`AspectValue` is the carrier. `AspectContract` is the law that makes the carrier
interpretable.

Milestone 1 must therefore define the substrate for:

- scalar aspect contracts
- schema-declared struct aspect contracts
- canonical struct fields and field paths
- field masks and aspect masks
- patch admissibility over whole aspects and fields
- absence/null/default/clear law
- declaration revision and evolution posture
- equality and equivalence basis declarations
- opaque/content-bearing contracts that keep large or recursive data behind
  handles rather than reintroducing document-shaped authority

## WORTH-Proof Dependency Boundary

Milestone 1 should use `worth-proof` for proof-bearing progression, not for
plain vocabulary.

Use `worth-proof` for:

- contract validation progression from raw `AspectValue` into an admitted
  aspect value
- aspect evolution classification before old/new contracts are compared,
  patched, lowered, or prepared for digesting
- compatibility lowering outcomes where JSON-originated input is admitted,
  denied, deferred, stale, rebind-required, or failed
- authoritative-state admission from contract-validated entries
- digest-preparation readiness once ordering and equality have been proven
- compile-fail tests that prove raw artifacts cannot satisfy validated APIs

Do not use `worth-proof` for:

- `AspectValue`
- `AspectKey`
- scalar wrappers
- content/reference ids
- locator data
- plain mask data
- identity newtypes

Those remain plain `worth-foundational` boundary vocabulary. The proof layer
wraps or progresses them only when an API needs to know what has already been
proven.

## Practical Type Targets

The implementation may rename these types if the local module design finds
better names, but the spec expects concrete equivalents. If an implementation
does not contain these responsibilities somewhere, it has not implemented the
milestone honestly.

```rust
pub enum AspectValue { /* relational value families from above */ }

pub struct AspectKey(/* canonical interned symbol or equivalent */);

pub struct AspectContract {
    key: AspectKey,
    identity: AspectIdentity,
    shape: AspectShape,
    masks: AspectMaskContract,
    absence: AbsenceLaw,
    equivalence: AspectEquivalenceBasis,
    evolution: AspectEvolutionPolicy,
}

pub enum AspectShape {
    Scalar(ScalarAspectType),
    Struct(StructAspectShape),
    Opaque(OpaqueAspectType),
    Reference(ReferenceAspectType),
    Content(ContentAspectType),
}

pub struct StructAspectShape {
    fields: CanonicalFieldDeclarations,
}

pub struct AspectMask<Mode> {
    paths: CanonicalMaskPaths,
    mode: PhantomData<Mode>,
}

pub enum ProjectionMask {}
pub enum MutationMask {}
pub enum DiagnosticMask {}

pub struct ContractValidatedAspectValue {
    key: AspectKey,
    value: AspectValue,
    contract_revision: AspectContractRevision,
}

pub type ContractValidatedAspectArtifact =
    worth_proof::Artifact<ContractValidated, ContractValidatedAspectValue, AspectContractProof>;

pub type DigestPreparationReadyAspectState =
    worth_proof::Artifact<DigestPreparationReady, AuthoritativeRecordAspectState, CanonicalOrderingProof>;

pub struct AuthoritativeRecordAspectState {
    aspects: CanonicalAspectStateMap,
}

pub struct AuthoritativeRecordAspectPatch {
    whole_aspect_changes: CanonicalWholeAspectChanges,
    field_changes: CanonicalFieldPatchSet,
}
```

The important point is not that the code must copy this sketch exactly. The
important point is that shape, masks, absence law, equivalence basis, evolution
policy, and contract validation are first-class values, not comments.
Proof-bearing variants of these values should use `worth-proof` artifacts,
witnesses, and outcomes where those primitives directly fit the progression
being modeled.

## Compile-Time Enforcement Targets

The following rules should be enforced as high as possible in the enforcement
hierarchy. If the compiler cannot enforce a rule directly, the spec expects a
focused compile-fail test, property test, or facade test proving the weaker
mechanical enforcement.

| Rule | Preferred enforcement |
| --- | --- |
| A raw value cannot be inserted into authoritative aspect state without contract validation. | Private `AuthoritativeRecordAspectState` fields plus a constructor that accepts only `ContractValidatedAspectValue` or equivalent proof-bearing input. |
| Mutation masks cannot be passed where projection or diagnostic masks are expected. | Phantom-typed `AspectMask<MutationMask>`, `AspectMask<ProjectionMask>`, and `AspectMask<DiagnosticMask>` or equivalent distinct types. |
| Field-level patches cannot target scalar, opaque, reference-only, or content-only aspects. | Patch constructors consume a struct-contract witness such as `StructAspectWitness`. |
| Absence, null, default, and clear cannot collapse accidentally. | Distinct enums/witnesses such as `AspectPresence`, `DefaultApplication`, and `ClearIntent`, with no boolean substitute. |
| Aspect identity, handles, basis ids, content refs, and entity refs cannot be interchanged. | Phantom-tagged newtypes or distinct wrapper structs with no shared public constructor. |
| Schema-declared structs cannot be authored as arbitrary JSON documents. | No `serde_json::Value` constructor on authoritative value/state APIs; compatibility lowering lives behind an explicitly named bridge. |
| Evolution classification must happen before old/new contracts are compared, patched, or digested. | Old/new contract operations consume an `AspectEvolutionVerdict` carried in a `worth-proof` artifact or equivalent proof-bearing wrapper. |
| Digest-preparation input cannot be assembled from insertion-order-dependent maps. | Canonical iterators and private map serialization helpers produce a `DigestPreparationReady` proof-bearing artifact; hostile ordering tests lock behavior. |
| Compatibility lowering cannot flatten denial/deferred/stale/rebind/failure into a boolean. | Lowering APIs return `worth_proof::TransitionOutcome` or an equivalent category-preserving proof outcome. |

This table is intentionally part of the spec, not implementation advice. It is
the anti-naive-trap checklist for Milestone 1.

## Phases

Phases are mandatory and linear. Each phase must leave behind proof-bearing
types, tests, or facade boundaries that the next phase consumes. A later phase
must not defensively rediscover facts that an earlier phase was required to
prove.

Phase progression gates:

| Phase | Gate before next phase |
| --- | --- |
| Phase 1 | The crate boundary and facade exist, and internal module homes are named before any semantic type lands. |
| Phase 2 | Primitive canonical values exist and explicitly reject generic document-shaped authority before contracts introduce structured values. |
| Phase 3 | Aspect contracts prove shape, mask admissibility, absence law, evolution posture, and equivalence basis before authoritative state can exist. |
| Phase 4 | Authoritative state accepts only contract-admissible values before patches can mutate that state. |
| Phase 5 | Patch law is canonical and mask-aware before locators are used to explain or point at patch loci. |
| Phase 6 | Identity and locator categories are typed before compatibility bridges can expose producer-originated boundary explanations. |
| Phase 7 | Compatibility lowering proves native-equivalent canonical meaning before digest-preparation parity is certified. |
| Phase 8 | Canonical ordering/equality/digest-preparation proof exists before Milestone 2 begins digest algorithms. |

### Phase 1: Create The Foundational Crate Boundary

Purpose:

Establish `worth-foundational` as a real workspace crate with a narrow facade
and responsibility-shaped internal modules before adding any value or aspect
semantics.

Must ship:

- workspace registration for `crates/worth-foundational`
- workspace dependency on `worth-proof` for proof-bearing progression surfaces
- a public facade that exports only milestone-owned vocabulary
- internal module boundaries for:
  - canonical values
  - aspect state and patches
  - identity categories
  - locators
  - compatibility bridges
  - canonical ordering/equality support
- crate-level docs stating that the crate standardizes boundary meaning, not
  hot-path runtime storage

Must preserve:

- `worth-proof` remains the owner of progression law
- `worth-foundational` does not duplicate `worth-proof` artifact, witness, or
  transition-outcome machinery
- domain crates remain owners of truth mutation, storage layout, and execution
  behavior
- no module named as a generic helper bucket

Acceptance evidence:

- the crate compiles as a workspace member
- the crate compiles with `worth-proof` as the proof-bearing progression
  dependency for Milestone 1 proof surfaces
- public exports come through a facade rather than deep module paths
- structure review can predict where each Milestone 1 concept belongs

### Phase 2: Define Canonical Aspect Values

Purpose:

Move the value families from the relational aspect-native rewrite source into
the shared foundational vocabulary without narrowing the database-like scalar
surface. This phase defines primitive carriers and canonical wrappers only.
Schema-declared struct values are admitted in Phase 3 after the aspect-contract
law exists.

Must ship:

- the exact value families listed in `Relational Aspect Value Source Of Truth`
- canonical wrappers for float bit carriers, decimal, big-int, rational, date,
  time, timestamp, timestamp-with-offset, content references, entity
  references, UUID bytes, strings, and byte/content references
- sealed or constructor-disciplined canonical wrappers where representation
  normalization matters
- equality and ordering rules sufficient for deterministic aspect-state maps
  and digest-preparation inputs
- explicit marker that struct/document-shaped values are not admitted by the
  primitive value carrier until Phase 3 contract law names their shape
- explicit documentation that arbitrary recursive document/JSON trees are not
  ordinary authoritative aspect truth

Must preserve:

- width-specific integer identity
- exact numeric escalation paths
- temporal precision and timezone-offset semantics
- handle-based treatment of large bytes, large text, and content-bearing values
- crate-local freedom to store optimized local values and materialize these
  forms only at boundaries
- schema-declared structs are not implemented as disguised JSON objects or
  opened before the contract phase exists

Acceptance evidence:

- tests prove each value family round-trips through canonical construction
  without losing width, precision, temporal basis, or reference kind
- equality tests distinguish values with equal storage shape but different
  semantic variants
- no ordinary canonical value variant is a JSON object or untyped recursive
  document carrier
- compile-time or facade tests prevent callers from constructing generic
  document-shaped authority through the primitive value carrier

### Phase 3: Define Aspect Contracts, Struct Shapes, And Mask Law

Purpose:

Define the law attached to aspect keys so value carriers, struct values, masks,
patches, projections, indexes, and compatibility lowering remain interpretable
without crate-local folklore.

Must ship:

- `AspectContract` or equivalent declaration type
- proof-bearing contract-validation outputs built on `worth-proof` artifacts or
  equivalent `worth-proof` progression primitives
- stable aspect identity fields that distinguish canonical key, declaration
  authority, revision/basis, and display naming where those distinctions matter
- value-shape declarations for scalar, schema-declared struct, collection-like
  if admitted, opaque, reference-bearing, and content-bearing aspect families
- canonical structured-value carriers admitted only through schema-declared
  struct contracts
- struct field declarations with stable field identity, field type, canonical
  order, required/optional status, absence/null/default law, and evolution
  posture
- mask declarations for:
  - whole-aspect masks
  - field masks
  - nested field masks if nested structs are admitted
  - projection masks
  - mutation masks
  - diagnostic/report masks
- admissibility rules that determine which masks and patch operations are legal
  for each aspect shape
- equivalence-basis declarations for equality, reuse, suppression, parity,
  digest-preparation, and mismatch explanation
- evolution declarations for additive fields, removed fields, renamed fields if
  admitted, type widening, type narrowing, default changes, and incompatible
  revision breaks

Must preserve:

- masks select, update, or explain truth; masks are not truth values
- struct aspects are schema-declared product types, not arbitrary document
  payloads
- absence, null, clear, and default are distinct unless an aspect contract
  explicitly collapses them
- evolution law is explicit before digests and migrations depend on aspect
  shape
- equivalence basis is declared before any caller can claim reuse,
  suppression, or parity

Acceptance evidence:

- contract tests prove scalar and struct aspects expose distinct admissible
  mask sets
- mask admissibility tests reject masks that target nonexistent fields,
  opaque-only values, or fields not admitted for mutation/projection
- absence/null/default/clear tests prove the four states cannot drift into
  each other accidentally
- evolution tests prove additive changes, incompatible changes, and narrowed
  shapes are classified deterministically
- equivalence-basis tests prove equality and digest-preparation do not rely on
  ad hoc comparator behavior
- struct-value tests prove field ordering, missing-field semantics, nulls,
  defaults, and equality do not depend on construction order or serializer
  behavior
- proof-progression tests prove raw contract inputs cannot satisfy
  contract-validated APIs

### Phase 4: Define Aspect Keys, State Maps, And Authoritative Wrappers

Purpose:

Define the canonical aspect-state language consumed by relational, query,
signal, store, diagnostics, digest, and migration surfaces.

Must ship:

- `AspectKey`
- `CanonicalAspectStateMap`
- `AuthoritativeRecordAspectState`
- contract-aware state validation that proves each aspect value is admissible
  under its declared aspect contract
- state admission APIs that consume proof-bearing contract-validated entries
  rather than raw values
- deterministic map serialization/materialization rules that do not depend on
  insertion order or transport object ordering
- read-only accessors that preserve map ordering and do not expose alternate
  mutation paths
- explicit distinction between authoritative record aspect state and derived,
  projected, or diagnostic views of aspect data

Must preserve:

- aspect keys are semantic keys, not raw JSON field names
- canonical ordering is part of boundary meaning
- authority state is separate from downstream projection state
- state wrappers are not interchangeable with patches, reports, or diagnostics
- aspect state cannot claim canonical authority while bypassing its contract

Acceptance evidence:

- independently constructed aspect states with different insertion orders
  materialize to the same canonical order
- serialized map entries remain stable across construction paths
- compile-time or facade tests prevent external callers from bypassing
  authoritative constructors where the API can enforce it
- contract-validation tests reject values that do not match scalar, struct,
  opaque, reference, or content-bearing aspect declarations
- compile-fail tests prove raw values and unvalidated contract outputs cannot
  enter authoritative state

### Phase 5: Define Authoritative Aspect Patches

Purpose:

Encode aspect-state change as explicit set/clear semantics rather than object
merge convention.

Must ship:

- `AuthoritativeRecordAspectPatch`
- explicit `set` and `clear` collections
- explicit field-level patch forms for schema-declared struct aspects where the
  aspect contract admits partial update
- explicit mask-bearing patch forms for selecting which struct fields are set,
  cleared, or left unchanged
- constructor law where `set` entries dominate overlapping `clear` entries
- `apply_to` semantics equivalent to clearing first and applying set entries
  second
- deterministic overlap law for whole-aspect replacement versus field-level
  patching
- rejection law for ambiguous overlapping masks, inadmissible field updates,
  opaque-only values, and schema-incompatible patches
- no-op and empty-patch behavior that is canonical rather than caller-defined
- deterministic ordering for set and clear materialization

Must preserve:

- patch meaning is independent of JSON object merge folklore
- set and clear remain distinct semantic categories
- applying a patch never requires producer-private state
- patch application produces authoritative aspect state, not a diagnostic or
  report artifact
- field-level patches are allowed only under declared struct contracts
- whole-aspect replacement and partial struct update are not silently
  interchangeable

Acceptance evidence:

- hostile overlapping set/clear tests prove set dominance
- empty and no-op patch tests prove stable semantics
- order-insensitivity tests prove patch materialization is canonical
- patch application tests prove clearing precedes setting
- struct patch tests prove field set, field clear, whole-aspect replace, and
  whole-aspect clear remain distinct operations
- overlap tests prove ambiguous whole-aspect and field-mask combinations are
  either canonicalized or rejected by law
- mask-admissibility tests prove patches cannot target fields outside the
  aspect contract

### Phase 6: Define Identity Categories And Locators

Purpose:

Create the shared vocabulary for boundary identities and structural pointers so
later diagnostics, provenance, mismatch reports, receipts, and support bundles
can refer to the same things in the same way.

Must ship:

- typed identity/key/handle/basis-id vocabulary for shared boundary surfaces
- clear distinction between identities, handles, keys, basis ids, epochs,
  digests, content references, entity references, and other representation-like
  ids whose meaning differs
- canonical locator/path vocabulary for:
  - aspect keys
  - aspect values
  - aspect contracts
  - structural fields
  - struct fields and nested field paths where admitted
  - masks
  - boundary artifact fields
  - mismatch loci
  - provenance/source loci
- locator categories that preserve whether the target is authoritative,
  derived, projected, support-only, planned, or receipt-bearing where that
  distinction matters

Must preserve:

- equal representation does not imply equal meaning
- locators are not opaque strings whose meaning depends on producer folklore
- identity vocabulary standardizes boundary semantics, not allocator strategy
  or handle lifetime mechanics
- locators must be usable by diagnostics later without importing diagnostic
  ontology into this milestone

Acceptance evidence:

- type tests or compile-fail tests prove distinct identity categories cannot be
  passed interchangeably through public APIs
- locator construction tests prove equivalent paths canonicalize identically
- hostile tests prove producer-private string formatting is not required to
  interpret a locator
- field-path locator tests prove struct-field diagnostics and patches can point
  at the same locus without stringly path drift

### Phase 7: Define Compatibility Bridge Boundaries

Purpose:

Allow transitional JSON-originated inputs to lower into canonical
aspect-native values explicitly while preventing compatibility code from
becoming the canonical truth model.

Must ship:

- compatibility bridge types or functions for lowering admitted transitional
  JSON-originated inputs into canonical aspect values and aspect state
- compatibility lowering APIs that preserve `worth-proof` transition categories
  such as success, denied, deferred, stale, rebind-required, and failed where
  those categories apply
- compatibility bridge lowering through aspect contracts, including
  schema-declared struct contracts, masks, absence/null/default law, and
  rejection rules
- explicit debt markers on compatibility bridge surfaces
- rejection surfaces for JSON shapes that cannot honestly lower into the
  canonical value vocabulary
- tests proving native construction and compatibility lowering produce the same
  canonical state for semantically identical inputs
- documentation that compatibility bridges are boundary shims, not ordinary
  authoritative storage law

Must preserve:

- `serde_json::Value` is not the canonical value type
- compatibility lowering happens at named boundaries only
- compatibility bridges may not leak into hot-path storage or mutation
  authority as co-equal truth
- opaque JSON content, if admitted later, must be represented as an explicit
  content/opaque value family rather than ordinary object truth

Acceptance evidence:

- bridge parity tests compare native and compatibility-originated aspect states
- rejection tests cover ambiguous numeric width, unordered object semantics,
  unsupported recursive document truth, missing required struct fields,
  ambiguous null/default/absence semantics, and incompatible reference shapes
- public facade tests prove compatibility debt is opt-in and visibly named
- transition-outcome tests prove compatibility lowering does not collapse
  denial, deferred, stale, rebind-required, or failed cases into a boolean

### Phase 8: Certify Canonical Ordering, Equality, And Digest-Preparation Basis

Purpose:

Prove that Milestone 1 surfaces are stable enough for Milestone 2 digest and
canonicalization work to build on.

Must ship:

- canonical ordering rules for aspect states, aspect contracts, masks, field
  paths, and patches
- equality rules for values, aspect state, patch state, identities, and
  locators
- digest-preparation basis builders or test-only fixtures that demonstrate
  stable semantic ordering without claiming to close the full digest milestone
- proof-bearing digest-preparation readiness artifacts for surfaces whose
  ordering/equality law has been certified
- hostile construction-path tests across native, reordered, and compatibility
  inputs
- named residual debt for digest functionality intentionally deferred to
  Milestone 2

Must preserve:

- Milestone 1 does not claim to own final digest algorithms
- digest preparation must be stable enough that Milestone 2 can add digest
  helpers without revisiting value or patch semantics
- semantic equality must not erase meaningful variant distinctions
- transport encoding remains downstream of canonical semantic form

Acceptance evidence:

- cross-construction parity tests show semantically identical aspect states
  produce identical digest-preparation sequences
- hostile insertion-order tests prove map ordering cannot affect materialized
  meaning
- value-distinction tests prove equal-looking representations in different
  variants remain semantically distinct
- struct and mask parity tests prove independent construction paths produce the
  same digest-preparation basis
- a Milestone 2 readiness note identifies which digest APIs remain future work
- compile-fail tests prove non-ready aspect state cannot be passed to
  digest-preparation APIs that require readiness proof

## Must Ship

- `worth-foundational` crate boundary and facade
- canonical Aspec-native `AspectValue` vocabulary preserving the exact
  relational value families named above
- canonical wrappers for numeric, temporal, string, UUID, reference, and content
  value families
- schema-declared struct aspect value support with canonical field identity,
  ordering, absence/null/default law, and evolution posture
- aspect contract declarations covering shape, admissible masks, patch law,
  equivalence basis, and evolution rules
- `worth-proof` artifacts, witnesses, and transition outcomes for
  proof-bearing Milestone 1 progression surfaces
- `AspectKey`
- `CanonicalAspectStateMap`
- `AuthoritativeRecordAspectState`
- `AuthoritativeRecordAspectPatch`
- aspect masks and field masks for projection, mutation, diagnostics, and
  report/support selection
- field-level patch semantics for schema-declared struct aspects where the
  aspect contract admits partial update
- typed shared boundary identity, key, handle, and basis-id categories
- canonical locator/path vocabulary
- explicit compatibility bridge boundaries for transitional JSON-originated
  inputs
- canonical ordering, equality, and digest-preparation basis tests sufficient
  to unblock the digest milestone

## Must Preserve

- shared meaning over shared representation
- domain-crate freedom to keep cost-honest local storage and materialize
  foundational values only at explicit boundaries
- `worth-proof` ownership of proof progression law
- plain foundational vocabulary remains plain; `worth-proof` wraps or
  progresses it only at proof-bearing API boundaries
- domain-crate ownership of mutation, storage, runtime behavior, and execution
  orchestration
- aspect-native meaning as the canonical long-term truth vocabulary
- explicit compatibility debt for JSON-originated surfaces
- distinction between authoritative truth, derived views, projected views,
  support descriptions, planned artifacts, and receipts
- distinction between absence, null, default, and clear semantics
- distinction between whole-aspect replacement, whole-aspect clearing,
  field-level struct patching, and mask-based selection
- explicit aspect evolution posture before digest, profile, diagnostic, or
  migration work depends on the shape
- no hidden dependence on map insertion order, serializer ordering, or
  crate-local aspect folklore
- no universal runtime value bag forced into hot paths

## Acceptance Evidence

- crate compiles as a workspace member with facade-controlled public exports
- canonical value tests preserve width, precision, temporal basis, reference
  kind, and semantic variant distinctions
- aspect-contract tests prove scalar, struct, opaque, reference-bearing, and
  content-bearing shapes expose the correct admissible masks and patch forms
- `worth-proof` compile-fail tests prove raw values, unresolved evolution, and
  non-ready state cannot satisfy proof-bearing APIs
- struct-value tests prove field order, missing fields, nulls, defaults, and
  equality remain canonical across construction paths
- aspect-state ordering tests prove construction and insertion order cannot
  change canonical materialization
- aspect-patch hostile tests prove set dominance, clear-before-set application,
  no-op behavior, empty-patch behavior, and deterministic materialization
- field-mask and struct-patch hostile tests prove partial updates cannot smuggle
  JSON merge semantics back into the substrate
- absence/null/default/clear tests prove those states remain distinct unless a
  contract explicitly collapses them
- evolution tests prove additive, narrowing, widening, removal, rename if
  admitted, and incompatible-revision cases classify deterministically
- equivalence-basis tests prove equality, reuse, suppression, parity, and
  digest-preparation claims are contract-backed rather than comparator folklore
- identity category tests or compile-fail tests prevent interchangeable use of
  semantically distinct ids with equal representation
- locator tests prove canonical interpretation without producer-private string
  conventions
- compatibility bridge tests prove admitted JSON-originated inputs lower into
  the same canonical meaning as native construction
- compatibility rejection tests prove unsupported or ambiguous JSON-originated
  shapes fail closed
- digest-preparation parity tests prove Milestone 2 can build on stable
  semantic ordering without revisiting Milestone 1 value law

## Architectural Notes

The implementation should map to responsibility-shaped modules. A likely shape
is:

```text
crates/worth-foundational/src/
  lib.rs
  facade.rs
  values/
  aspects/
  identity/
  locators/
  compatibility/
  canonical/
```

The exact module names may adapt to implementation discoveries, but the
responsibility boundaries may not collapse into generic helpers. If a type
answers a value question, an aspect-state question, an identity question, a
locator question, or a compatibility question, its file path should make that
role obvious without grep.

`worth-foundational` must expose materialized boundary forms. It must not
require `worth-relational`, `worth-signal`, `worth-query`, or `worth-store` to
adopt one internal value map, storage topology, or diagnostics layout. The
shared type is the exchange language, not the mandate for every hot path.

## Sequencing Notes

This milestone is first because later roadmap milestones are structurally
downstream:

- Milestone 2 needs stable values, aspect states, patches, identities, and
  locators before digest basis can be canonical.
- Milestone 3 profiles need value and locator attachments before richness and
  posture can be digestible and explainable.
- Milestone 4 materialization contracts need authoritative versus derived
  aspect-state vocabulary before report/artifact/receipt categories can remain
  honest.
- Milestone 5 branch/merge/commit vocabulary needs aspect state, patches, ids,
  locators, and digest-preparation bases before authority-transition evidence
  can be self-describing.
- Milestone 6 diagnostics need locators and values before explanations can
  point at concrete boundary meaning.
- Milestone 7 lineage/provenance/receipts need identity and basis-id categories
  before provenance can be self-describing.
- Milestone 11 migrations need this substrate before crate-local dialects can
  converge.

## Explicit Non-Goals

- final canonical digest algorithms
- profile vocabulary
- report, summary, artifact, or receipt taxonomy
- diagnostics and explanation ontology
- lineage and provenance ontology
- performance and layout vocabulary beyond preserving representation freedom
- cross-crate migration of relational, query, signal, or store surfaces
- proof progression law or proof-kernel responsibilities
- reimplementing `worth-proof` artifact, witness, transition-outcome, or
  phase-progression machinery inside `worth-foundational`
- a generic runtime value executor, storage engine, planner, or mutation
  language
- a general recursive document database hidden behind `AspectValue`
- a universal schema engine or validator runtime beyond the aspect-contract
  substrate needed for canonical boundary meaning

## Self-Check

- Does the milestone solve a real structural problem or just package work
  cosmetically? Yes. It creates the first shared semantic substrate required
  before any later foundational boundary artifact can be honest.
- Is the adversarial constraint precise and load-bearing? Yes. It forbids JSON
  default semantics, producer-private interpretation, ordering drift, patch
  folklore, scalar-only aspect modeling, mask/value collapse, identity
  collapse, and forced hot-path representation.
- Does the milestone preserve crate authority boundaries? Yes.
  `worth-foundational` owns boundary vocabulary only; domain crates keep
  runtime, storage, mutation, and execution authority.
- Does the milestone define proof obligations, not just implementation tasks?
  Yes. Closure requires ordering, equality, patch, locator, compatibility,
  identity, and digest-preparation evidence.
- Could a competent engineer map this spec into honest types, modules, and
  tests? Yes. The phases map directly to crate boundary, values, aspect
  contracts, struct shapes, masks, aspect state, patches, identities, locators,
  compatibility bridges, and certification tests.
- Does the milestone belong in this roadmap sequence, or is it out of order?
  Yes. It is the substrate for every later foundational milestone.
