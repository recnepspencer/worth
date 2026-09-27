# Milestone 2: Canonical Digest And Canonicalization Substrate

## Goal

Define the shared canonicalization and digest-basis substrate that makes
foundational boundary meaning reproducible, comparable, replayable, and
certifiable across crates.

Milestone 2 turns Milestone 1's stable nouns into stable evidence. Values,
aspect contracts, masks, authoritative state, patches, identities, locators, and
compatibility-lowered inputs must be able to produce versioned, ordered,
equivalence-aware, mismatch-explainable canonical bases without depending on
producer-private state, transport encoding, or one runtime representation.

This milestone does not own final cryptographic digest algorithms, receipt
semantics, diagnostics ontology, lineage/provenance ontology, profile
semantics, branch/merge/commit semantics, or adopting-crate migrations. It owns
the canonical evidence grammar those later surfaces must consume.

## Governing Document Summaries

### `MENTALITY.md`

Protects adversarial-constraint-first engineering, mechanical enforcement, and
architecture tests over MVP plausibility. The shaping constraint for Milestone 2
is that digest and canonicalization law must be proven before later reports,
profiles, diagnostics, receipts, or migrations rely on it.

### `arch_laws.md`

Protects proof-bearing phase boundaries, explicit equivalence contracts,
canonical artifacts, authority/derivation separation, and self-describing
boundary envelopes. The shaping constraint is that digest-readiness must be a
typed progression over canonical basis, not a boolean or ad hoc hash helper.

### `composition_laws.md`

Protects responsibility-shaped files, named semantic steps, and test topology
that identifies the responsibility that failed. The shaping constraint is that
canonical basis construction, equivalence basis, mismatch basis, algorithm
registry, and export fixtures must not collapse into a generic
`canonicalization` dump.

### `domain_structure_laws.md`

Protects structure as responsibility topology with source truth, projections,
boundary crossings, and test support separated by meaning. The shaping
constraint is that digest basis, canonical export, mismatch evidence, and
readiness artifacts must occupy distinct homes whose names reject unrelated
future profile, diagnostic, receipt, or migration code.

### `perf_laws.md`

Protects cost-honest boundaries, no repeated rediscovery of proven facts,
explicit equivalence basis, and materialization costs that are visible in API
shape. The shaping constraint is that canonicalization must be an explicit
boundary operation with counters/basis evidence where useful, not a hidden
getter or automatic hot-path reconstruction.

### `worth_foundational_vision.md`

Protects the thesis that `worth-foundational` standardizes shared semantic
language without forcing one runtime representation. The shaping constraint is
that digest/canonicalization helpers must make boundary artifacts stable and
auditable while preserving crate-local layout freedom.

### `worth_foundational_roadmap.md`

Protects the sequence: Milestone 1 establishes aspect-native substrate,
Milestone 2 establishes canonical digest/canonicalization substrate, and later
milestones layer profiles, artifact categories, branch/merge/commit evidence,
diagnostics, provenance, performance vocabulary, and migrations on top. The
shaping constraint is that Milestone 2 must unblock all later digest-basis
participation without stealing their domain-specific ontology.

### `test-requirements.md`

Protects standalone proof before adopting runtime migration. The shaping
constraint is that Milestone 2 must certify canonical bases through hostile
local producers, blind consumers, golden artifacts, compile-fail category
boundaries, property seeds, production-test readiness evidence, and
worth-harness expansion points rather than waiting for real runtimes to reveal
canonicalization mistakes.

### `milestone-1-closeout.md`

Protects what is already complete: canonical values, aspect contracts, masks,
authoritative state, patches, identities, locators, compatibility lowering, and
initial digest-preparation readiness. The shaping constraint is that Milestone 2
should build on those certified surfaces and must not revisit their basic value
or aspect-state law unless it exposes an explicitly named Milestone 1 defect.

## Why This Milestone Exists

Milestone 1 made canonical meaning constructible. Milestone 2 makes canonical
meaning reproducible.

Later milestones cannot honestly define profiles, reports, receipts,
diagnostics, provenance, branch/merge/commit evidence, performance reports, or
cross-crate migration parity if each surface invents its own digest basis,
ordering law, mismatch vocabulary, export fixture shape, or equivalence claim.

Milestone 2 exists to give every later milestone a paved road:

- define the semantic surface
- define its canonical basis entries
- define its equivalence basis
- define how mismatch is localized
- define its readiness artifact
- lock golden fixtures and hostile construction-path parity

The milestone is successful when adding a later foundational surface requires
declaring new basis entries and tests, not rediscovering canonicalization law.

## Adversarial Constraint

Two WORTH crates with different internal layouts, insertion orders,
compatibility pressures, display labels, builder strategies, and transport
encodings must be able to materialize the same foundational meaning into the
same canonical basis, while semantically different meanings never collapse just
because their storage shape, debug string, JSON encoding, enum order, or local
construction path happens to look similar.

This milestone fails if:

- a digest can be assembled from insertion-order-dependent data
- a hash value becomes the semantic authority instead of a compression of the
  canonical basis
- equivalence, reuse, suppression, parity, or certification sameness can be
  claimed without a declared basis
- mismatches require producer-private state to explain
- canonical export fixtures compare debug output, transport JSON, or display
  strings instead of semantic basis entries
- canonicalization erases meaningful distinctions between adjacent categories
- the same semantics produce different bases because two crates store data
  differently
- final digest algorithms, receipts, diagnostics, profiles, or migrations get
  smuggled into this milestone because the basis substrate is too vague

## Boundary Thesis

Milestone 2 owns canonical digest-preparation and canonicalization law.

It owns:

- canonical basis entry vocabulary
- canonical basis sequences and bundles
- canonicalization rule versioning
- basis-domain typing
- equivalence basis participation
- mismatch-basis primitives
- proof-bearing readiness artifacts
- canonical export/golden fixture shapes
- hostile independent-construction parity
- digest algorithm slots and algorithm metadata sufficient to keep hash
  derivation downstream of basis semantics
- production-test readiness evidence for canonicalization surfaces

It does not own:

- final cryptographic digest policy for all WORTH artifacts
- report, summary, artifact, or receipt taxonomy
- profile vocabulary
- diagnostics and explanation ontology
- lineage and provenance ontology
- branch, merge, and commit vocabulary
- performance/layout vocabulary beyond cost-honest canonicalization boundaries
- real adopting-crate migrations
- a universal serializer, storage engine, executor, or runtime container

The semantic authority is the canonical basis. A digest is a derived
compression of that basis.

## QA Tightening Notes

This spec must not close as an abstract "make hashes stable" milestone.
Milestone 2 is only useful if an implementer can point at concrete types,
builder constraints, fixture files, hostile producer shapes, and compile-fail
boundaries that prove canonical meaning is practical.

The naive traps to avoid are:

- treating a canonical basis entry as a loosely typed key/value pair
- using `Debug`, display text, serde order, or transport JSON as a shortcut for
  exportable canonical evidence
- allowing a digest algorithm slot to accept raw bytes and thereby bypass basis
  law
- comparing basis sequences without naming the equivalence basis that makes
  the comparison meaningful
- building fixture snapshots that prove formatting stability instead of
  semantic stability
- letting local adversarial producers become an unowned mini-runtime or
  harness dialect inside `worth-foundational`
- hiding canonicalization work behind cheap-looking accessors with no visible
  cost boundary

Milestone 2 must therefore make the following implicit assumptions explicit:

- canonicalization is a boundary operation, not a passive getter
- every basis entry has a typed domain, typed locus, typed entry kind, and typed
  value carrier
- rule version participates in every sequence, bundle, export fixture, and
  digest derivation
- exact comparison, compatibility-lowered comparison, projection-equivalent
  comparison, and digest-equivalent comparison are different claims
- unsupported comparison is a first-class outcome, not a panic, boolean false,
  or omitted report
- algorithm identity is metadata over a basis-derived compression path, not
  semantic authority

## WORTH-Proof Dependency Boundary

Milestone 2 must use `worth-proof` for proof-bearing progression surfaces.
This is not optional and should not be reopened during implementation.

`worth-proof` is mandatory for:

- carrying phase truth for canonical basis readiness, export readiness,
  comparison readiness, digest derivation readiness, and production-test
  readiness
- carrying proof facts such as canonical order, uniqueness, normalization,
  domain coherence, rule-version binding, and digest-derivation readiness
- checked transitions where canonicalization, comparison, export, or digest
  derivation can be denied, deferred, stale, rebind-required, or failed
- readiness gates that decide whether a stronger canonicalization artifact may
  be constructed
- assumption-basis and freshness wrappers when a basis artifact crosses an
  export, fixture, replay, transport, or runtime-adoption trust boundary
- authority or capability witnesses for trusted progression lanes such as
  canonical basis construction, export publication, digest derivation, boundary
  readmission, and production-test readiness certification

`worth-proof` is forbidden for:

- plain `CanonicalBasisEntry`, `CanonicalBasisValue`, `CanonicalBasisLocus`,
  `CanonicalBasisDomain`, `CanonicalBasisEntryKind`, and rule-version data
- plain equivalence-basis and mismatch-basis vocabulary
- algorithm id, digest metadata, fixture manifest, producer-shape metadata, and
  canonicalization-cost vocabulary
- replacing diagnostics, provenance, receipts, profiles, branch/merge/commit
  evidence, or later milestone ontology
- building a runtime engine, serializer, storage model, dynamic proof registry,
  or workflow harness inside `worth-foundational`

The operating rule is:

`worth-foundational` defines the nouns; `worth-proof` proves which noun has
progressed into which stronger state, under which proof set and basis.

Concrete mandatory proof phases:

- `RawCanonicalBasisInput`: local foundational data has not yet proven
  canonical order or domain coherence.
- `CanonicalBasisReady`: one domain sequence is ordered, versioned,
  domain-coherent, and safe as digest-basis input.
- `CanonicalBundleReady`: multiple ready sequences form a version-coherent
  bundle.
- `CanonicalComparisonReady`: two ready sequences or bundles have an explicit
  equivalence basis and may be compared.
- `CanonicalExportReady`: a bundle has a manifest, producer-shape metadata,
  cost counters, and golden/export fixture metadata.
- `CanonicalDigestDerivationReady`: a ready sequence or bundle has an admitted
  digest algorithm slot and input-shape proof.
- `CanonicalDigestDerived`: a digest value has been derived from canonical
  basis evidence and carries algorithm/rule-version/input-shape metadata.
- `CanonicalProductionTestReady`: Milestone 2 certified surfaces, hostile
  producer evidence, blind-consumer evidence, compile-fail evidence, golden
  fixtures, cost evidence, assumptions, non-assumptions, and residual debt have
  been recorded.

Concrete mandatory proof facts:

- `CanonicalOrder`
- `Uniqueness`
- `Normalization`
- `CanonicalDomainCoherence`
- `CanonicalRuleVersionBound`
- `CanonicalEquivalenceBasisDeclared`
- `CanonicalMismatchLociBound`
- `CanonicalExportManifestBound`
- `CanonicalDigestInputShapeBound`
- `CanonicalizationCostObserved`
- `CanonicalProductionReadinessCertified`

Use built-in `worth-proof` facts such as `CanonicalOrder`, `Uniqueness`, and
`Normalization` where they fit exactly. Add Milestone 2 proof-marker types in
`worth-foundational` for domain-specific facts such as rule-version binding,
export-manifest binding, digest input-shape binding, and production readiness.

## Practical Type Targets

The implementation may choose better names, but it must contain these
responsibilities somewhere. If a responsibility has no concrete type, artifact,
or sealed construction path, the milestone has not closed honestly.

```rust
pub struct CanonicalizationRuleVersion(/* version identity */);

pub enum CanonicalBasisDomain {
    Value,
    AspectContract,
    AspectMask,
    AuthoritativeState,
    AuthoritativePatch,
    Identity,
    Locator,
    CompatibilityLowering,
    Future(&'static str),
}

pub struct CanonicalBasisEntry {
    domain: CanonicalBasisDomain,
    locus: CanonicalBasisLocus,
    kind: CanonicalBasisEntryKind,
    value: CanonicalBasisValue,
}

pub struct CanonicalBasisEntryId<D> {
    domain: PhantomData<D>,
    ordinal: u32,
}

pub enum CanonicalBasisValue {
    Null,
    Bool(bool),
    SignedInteger { width: IntegerWidth, value: i64 },
    UnsignedInteger { width: IntegerWidth, value: u64 },
    FloatBits { width: FloatWidth, bits: u64 },
    ExactText(CanonicalText),
    BytesDigest(ContentDigestId),
    UuidBytes([u8; 16]),
    Temporal(CanonicalTemporalBasis),
    Reference(CanonicalReferenceBasis),
    NestedSequence(CanonicalBasisSequenceId),
}

pub struct CanonicalBasisSequence {
    version: CanonicalizationRuleVersion,
    domain: CanonicalBasisDomain,
    entries: CanonicalBasisEntries,
    cost: CanonicalizationCost,
}

pub struct CanonicalBasisBundle {
    version: CanonicalizationRuleVersion,
    sequences: CanonicalBasisSequences,
}

pub enum CanonicalEquivalenceBasis {
    ExactCanonicalBasis,
    DeclaredAspectEquivalence,
    CompatibilityLoweredNativeEquivalence,
    ProjectionEquivalent,
    DigestEquivalent,
}

pub struct CanonicalMismatchBasis {
    left: CanonicalBasisLocus,
    right: CanonicalBasisLocus,
    kind: CanonicalMismatchKind,
}

pub enum CanonicalMismatchKind {
    MissingEntry,
    AdditionalEntry,
    EntryKindMismatch,
    ValueMismatch,
    OrderingMismatch,
    EquivalenceBasisMismatch,
    VersionMismatch,
    UnsupportedComparison,
}

pub struct CanonicalDigestAlgorithmId(/* identity only */);

pub struct CanonicalDigestAlgorithmSlot {
    id: CanonicalDigestAlgorithmId,
    input_domain: CanonicalBasisDomain,
    rule_version: CanonicalizationRuleVersion,
    input_shape: CanonicalDigestInputShape,
}

pub struct CanonicalizationCost {
    entry_count: u32,
    ordering_comparisons: u32,
    nested_sequence_count: u32,
    compatibility_lowering_count: u32,
}

pub enum CanonicalDigestInputShape {
    SingleSequence,
    DomainBundle,
    ExportBundle,
}

pub struct CanonicalDomainCoherence;
pub struct CanonicalRuleVersionBound;
pub struct CanonicalEquivalenceBasisDeclared;
pub struct CanonicalMismatchLociBound;
pub struct CanonicalExportManifestBound;
pub struct CanonicalDigestInputShapeBound;
pub struct CanonicalizationCostObserved;
pub struct CanonicalProductionReadinessCertified;
pub struct CanonicalizationProofAuthority;

pub type CanonicalBasisReadyArtifact =
    worth_proof::Artifact<
        CanonicalBasisReady,
        CanonicalBasisSequence,
        worth_proof::ProofSetCons<
            worth_proof::Proof<worth_proof::CanonicalOrder, CanonicalizationProofAuthority>,
            worth_proof::ProofSetCons<
                worth_proof::Proof<CanonicalDomainCoherence, CanonicalizationProofAuthority>,
                worth_proof::ProofSetCons<
                    worth_proof::Proof<CanonicalRuleVersionBound, CanonicalizationProofAuthority>,
                    worth_proof::Proof<
                        CanonicalizationCostObserved,
                        CanonicalizationProofAuthority,
                    >,
                >,
            >,
        >,
        worth_proof::FreshnessScopedBasis<
            worth_proof::CurrentValidity,
            worth_proof::AssumptionBasis<CanonicalizationRuleVersion>,
        >,
    >;

pub type CanonicalBundleReadyArtifact =
    worth_proof::Artifact<
        CanonicalBundleReady,
        CanonicalBasisBundle,
        worth_proof::ProofSetCons<
            worth_proof::Proof<CanonicalRuleVersionBound, CanonicalizationProofAuthority>,
            worth_proof::Proof<CanonicalDomainCoherence, CanonicalizationProofAuthority>,
        >,
        worth_proof::FreshnessScopedBasis<
            worth_proof::CurrentValidity,
            worth_proof::AssumptionBasis<CanonicalizationRuleVersion>,
        >,
    >;

pub type CanonicalComparisonReadyArtifact =
    worth_proof::Artifact<
        CanonicalComparisonReady,
        CanonicalComparisonInput,
        worth_proof::ProofSetCons<
            worth_proof::Proof<CanonicalEquivalenceBasisDeclared, CanonicalizationProofAuthority>,
            worth_proof::Proof<CanonicalMismatchLociBound, CanonicalizationProofAuthority>,
        >,
        worth_proof::NoAssumptionBasis,
    >;

pub type CanonicalExportReadyArtifact =
    worth_proof::Artifact<
        CanonicalExportReady,
        CanonicalExportBundle,
        worth_proof::ProofSetCons<
            worth_proof::Proof<CanonicalExportManifestBound, CanonicalizationProofAuthority>,
            worth_proof::Proof<CanonicalizationCostObserved, CanonicalizationProofAuthority>,
        >,
        worth_proof::FreshnessScopedBasis<
            worth_proof::CurrentValidity,
            worth_proof::AssumptionBasis<CanonicalizationRuleVersion>,
        >,
    >;

pub type CanonicalDigestDerivationReadyArtifact =
    worth_proof::Artifact<
        CanonicalDigestDerivationReady,
        CanonicalDigestDerivationInput,
        worth_proof::ProofSetCons<
            worth_proof::Proof<CanonicalDigestInputShapeBound, CanonicalizationProofAuthority>,
            worth_proof::Proof<CanonicalRuleVersionBound, CanonicalizationProofAuthority>,
        >,
        worth_proof::FreshnessScopedBasis<
            worth_proof::CurrentValidity,
            worth_proof::AssumptionBasis<CanonicalDigestAlgorithmSlot>,
        >,
    >;

pub type CanonicalProductionTestReadyArtifact =
    worth_proof::Artifact<
        CanonicalProductionTestReady,
        CanonicalProductionReadinessReport,
        worth_proof::Proof<
            CanonicalProductionReadinessCertified,
            CanonicalizationProofAuthority,
        >,
        worth_proof::NoAssumptionBasis,
    >;
```

These sketches are intentionally basis-centered. No type should imply that the
hash algorithm is the source of truth. No type should allow arbitrary byte blobs
to claim digest equivalence without canonical basis evidence.

`CanonicalizationProofAuthority` must be a sealed milestone-owned authority
that implements `worth_proof::AuthorityProves<...>` only for the proof facts
that its construction path can actually establish. Implementations must not use
an unscoped/default proof authority, and mixed-authority proof sets must not be
admitted into readiness artifacts.

The sketches also imply concrete implementation obligations:

- `CanonicalBasisEntryKind` must be a closed milestone-owned vocabulary for the
  Milestone 1 surfaces, with future extension slots that cannot pretend to be
  final later-milestone ontology.
- `CanonicalBasisValue` must encode the value family that determines meaning;
  it may not fall back to `String`, `serde_json::Value`, or arbitrary bytes for
  ordinary semantic entries.
- `CanonicalBasisEntryId<D>` or an equivalent domain-typed handle must exist
  anywhere an API refers back to an entry from mismatch, export, or digest-slot
  code.
- `CanonicalizationCost` must be attached to sequence construction results so
  hostile tests can prove canonicalization breadth and nested-sequence work are
  visible at the boundary.
- `CanonicalDigestInputShape` must distinguish single-sequence, domain-bundle,
  and export-bundle digest inputs so broad export compression cannot masquerade
  as a cheap single-surface digest.
- every `Artifact<...>` alias in the sketch is mandatory in spirit: an
  implementation may rename payload types, but it must preserve the same
  `worth-proof` phase, proof-set, and assumption-basis roles.
- sequence and bundle readiness must carry current-validity assumption bases
  tied to `CanonicalizationRuleVersion`; crossing export/replay/transport
  boundaries must explicitly downgrade through `worth-proof` boundary bridging
  and require readmission before a caller treats the artifact as current again.
- comparison readiness must consume ready basis artifacts and an explicit
  equivalence basis before returning structured comparison outcomes.
- digest derivation readiness must consume basis/export-ready artifacts and an
  admitted `CanonicalDigestAlgorithmSlot`; it may not accept plain sequence
  payloads or raw bytes.
- production-test readiness must be a proof-bearing artifact, not only a
  markdown closeout, so adopting crates can require it in APIs or harness
  adapters without reading prose.

## Compile-Time Enforcement Targets

| Rule | Preferred enforcement |
| --- | --- |
| Raw unordered maps cannot satisfy digest-basis APIs. | Basis builders consume canonical iterators, Milestone 1 readiness artifacts, or sealed ordered wrappers. |
| A digest algorithm cannot accept category-erased blobs as semantic input. | Algorithm slots consume `CanonicalBasisReady` or `CanonicalExportReady` artifacts, not `Vec<u8>` or strings. |
| Equivalence claims cannot be made without an equivalence basis. | Comparison APIs require `CanonicalEquivalenceBasis` or a proof-bearing equivalence artifact. |
| Mismatch explanation cannot depend on producer-private paths. | Mismatch APIs return canonical loci and entry kinds, not raw debug strings. |
| Canonical export fixtures cannot be constructed from debug output. | Export constructors consume basis sequences/bundles, not `Debug`, display strings, or transport JSON. |
| Canonicalization rule version cannot be omitted. | Basis sequence and bundle constructors require a `CanonicalizationRuleVersion`. |
| Value, contract, state, patch, identity, locator, and compatibility bases cannot be substituted silently. | Domain-typed basis wrappers or phantom-typed domain markers where APIs need one domain. |
| Final receipt/report/profile/diagnostic types cannot be smuggled into Milestone 2. | Milestone 2 exposes extension slots and future domains, not final later-milestone category constructors. |
| Mismatch references cannot point at untyped integer positions. | Mismatch APIs use domain-typed entry ids or canonical loci, never raw indexes alone. |
| Export-bundle digest slots cannot be used where a single-surface digest slot is required. | Digest slots carry `CanonicalDigestInputShape` or distinct phantom-typed input shapes. |
| Canonicalization cost cannot be invisible. | Sequence construction returns or exposes `CanonicalizationCost`; tests assert entry counts, nested-sequence counts, and compatibility-lowering counts. |
| Compatibility-origin basis cannot claim native construction provenance. | Compatibility basis builders carry an origin marker that canonicalizes to native meaning only after successful lowering, while export metadata preserves producer shape. |
| Unsupported comparison cannot be represented as ordinary inequality. | Comparison APIs return a structured comparison outcome with an unsupported variant. |
| Canonical basis readiness cannot be WORTHd by constructing the payload shape. | APIs that require readiness consume `CanonicalBasisReadyArtifact`, not `CanonicalBasisSequence`. |
| Export readiness cannot be WORTHd by bundling ready sequences manually. | Export publication consumes `CanonicalExportReadyArtifact` carrying manifest and cost proofs. |
| Digest derivation cannot skip algorithm-slot admission. | Digest derivation consumes `CanonicalDigestDerivationReadyArtifact`, not a digest slot plus bytes. |
| Boundary-restored fixtures cannot be treated as current without readmission. | Export/replay restoration returns boundary-bridged or stale/rebind-required `worth-proof` basis states until readmitted by an authority witness. |
| Production-test readiness cannot be claimed by a prose closeout alone. | Production-shaped adoption APIs and harness adapters consume `CanonicalProductionTestReadyArtifact` or an equivalent proof-bearing readiness artifact. |
| A witness cannot substitute for a carried proof. | Authority/capability witnesses authorize transitions; resulting artifacts must carry the proof facts they establish. |

## Phases

Phases are mandatory and linear. Each phase must leave behind proof-bearing
types, tests, or facade boundaries that the next phase consumes. Later phases
must not defensively rediscover facts that earlier phases are required to prove.

Phase progression gates:

| Phase | Gate before next phase |
| --- | --- |
| Phase 1 | Canonical basis domains, version identity, entry grammar, proof-marker facts, and `CanonicalBasisReadyArtifact` exist before any surface-specific basis builder lands. |
| Phase 2 | Milestone 1 surfaces produce `CanonicalBasisReadyArtifact` values before equivalence or mismatch can compare them. |
| Phase 3 | `CanonicalComparisonReadyArtifact`, equivalence basis, and mismatch-basis primitives exist before export fixtures or digest slots can claim parity. |
| Phase 4 | `CanonicalExportReadyArtifact`, manifest proof, and boundary-bridged export/readmission posture exist before algorithm slots compress basis evidence. |
| Phase 5 | `CanonicalDigestDerivationReadyArtifact` is explicitly downstream of basis/export readiness and algorithm-slot admission before production-test readiness can close. |
| Phase 6 | `CanonicalProductionTestReadyArtifact`, hostile producer parity, compile-fail boundaries, and readiness evidence exist before Milestone 3 profile work begins. |

### Phase 1: Define Canonical Basis Grammar

Purpose:

Create the generic canonical evidence grammar that every Milestone 2 and later
surface will use.

Must ship:

- `CanonicalizationRuleVersion`
- `CanonicalBasisDomain`
- `CanonicalBasisLocus`
- `CanonicalBasisEntry`
- `CanonicalBasisEntryKind`
- `CanonicalBasisValue`
- `CanonicalBasisSequence`
- `CanonicalBasisBundle`
- deterministic entry ordering law
- sealed constructors or builders that reject duplicate, unordered, or
  domain-incoherent basis input where the API can know it
- `CanonicalBasisReadyArtifact` built on `worth-proof::Artifact`
- domain-typed entry references for mismatch, export, and digest-slot code
- concrete basis value carriers for booleans, integer widths, float bits,
  exact text, content digests, UUID bytes, temporal values, references, and
  nested sequences
- canonicalization cost accounting for entry count, ordering work, nested
  sequence count, and compatibility-lowering count
- Milestone 2 proof markers for domain coherence, rule-version binding, and
  observed canonicalization cost

Must preserve:

- canonical basis is semantic evidence, not a transport encoding
- every sequence and bundle carries a rule version
- domains are visible and typed enough to prevent category-erased comparison
- future domains can be reserved without implementing future milestone
  ontology early
- basis entry values preserve semantic family instead of reducing ordinary
  entries to strings or arbitrary bytes
- cost accounting observes canonicalization work without changing canonical
  meaning
- raw `CanonicalBasisSequence` is not enough for downstream APIs that require
  canonical basis readiness
- witnesses authorize basis construction but do not themselves become the
  canonical-order or domain-coherence proof

Acceptance evidence:

- basis-entry ordering tests across hostile insertion orders
- duplicate and domain-incoherence rejection tests
- compile-fail tests proving raw unordered maps or raw byte blobs cannot satisfy
  basis-ready APIs
- compile-fail tests proving raw indexes cannot satisfy mismatch APIs requiring
  domain-typed entry references
- cost tests proving entry count and nested-sequence count are stable and
  visible for the same semantic input across producer shapes
- compile-fail tests proving raw `CanonicalBasisSequence` cannot satisfy APIs
  requiring `CanonicalBasisReadyArtifact`
- proof-carriage tests proving ready artifacts carry canonical-order,
  domain-coherence, rule-version, and cost-observed proof facts
- golden tests for the basis grammar itself

### Phase 2: Build Basis Sequences For Milestone 1 Surfaces

Purpose:

Promote Milestone 1 digest-preparation readiness into the shared canonical
basis grammar without changing Milestone 1 semantics.

Must ship basis builders for:

- canonical values
- aspect contracts
- struct fields and declared field order
- masks and mask modes
- authoritative aspect state
- authoritative aspect patches
- identities and basis ids
- locators
- compatibility-lowered values and state
- producer-shape metadata that distinguishes native ordered authority,
  reordered local authority, compatibility-originated input, and
  reduced/export-only materialization without letting producer shape alter
  canonical meaning
- blind-consumer inspection helpers that read only canonical basis sequences,
  not source Milestone 1 objects
- basis-builder APIs that return `worth-proof::TransitionOutcome` or a
  narrower checked outcome whenever construction can be denied, deferred,
  stale, rebind-required, or failed
- authority/capability witnesses for trusted builder lanes where raw
  compatibility or reordered producer input is being admitted into ready basis

Must preserve:

- no revisiting Milestone 1 value or aspect-state law unless a defect is named
- native and compatibility-originated construction paths produce the same basis
  when they mean the same thing
- semantically distinct variants with equal storage shape remain distinct
- basis builders consume readiness artifacts or canonical Milestone 1 types,
  not raw unvalidated values where validation was previously required
- producer-shape metadata explains test provenance and export pressure; it is
  not part of semantic equality unless the comparison basis explicitly asks for
  construction-origin comparison
- basis-builder non-success categories remain typed; ambiguous compatibility
  lowering or unsupported Milestone 1 surface preparation may not collapse into
  `Result<T, String>` or boolean failure

Acceptance evidence:

- cross-construction parity tests for each Milestone 1 surface
- compatibility-originated versus native basis parity tests
- hostile tests for storage-equal but meaning-distinct variants
- compile-fail tests proving raw unvalidated Milestone 1 inputs cannot satisfy
  basis builders that require proof-bearing readiness
- blind-consumer tests proving values, contracts, masks, patches, state,
  identities, locators, and compatibility-lowered bases can be interpreted from
  basis entries alone
- producer-shape tests proving reordered authority and compatibility-originated
  producers converge semantically while export metadata still records how the
  fixture was produced
- checked-outcome tests proving denied, deferred, stale, rebind-required, and
  failed builder paths remain distinct where the builder exposes those lanes
- compile-fail tests proving unvalidated Milestone 1 payloads cannot be wrapped
  in a ready basis artifact by constructing the payload shape directly

### Phase 3: Define Equivalence Basis And Mismatch Basis

Purpose:

Make sameness and difference explainable before any digest, export, support, or
later diagnostic surface claims parity.

Must ship:

- `CanonicalEquivalenceBasis`
- equivalence-scope vocabulary for exact, compatibility-lowered,
  projection-equivalent, digest-equivalent, and declared-aspect equivalence
- `CanonicalMismatchBasis`
- `CanonicalMismatchKind`
- canonical mismatch loci that reference basis entries, fields, masks,
  contracts, values, identities, locators, and compatibility origins without
  producer-private paths
- comparison APIs that return structured match/mismatch outcomes rather than
  booleans where explanation matters
- a `CanonicalComparisonOutcome` or equivalent with distinct `Equivalent`,
  `Mismatched`, and `Unsupported` outcomes
- mismatch evidence that carries left/right rule versions, domains, loci, entry
  kinds, and equivalence basis
- `CanonicalComparisonReadyArtifact` that consumes ready basis artifacts plus
  an explicit equivalence basis before comparison executes
- checked comparison readiness gates that preserve denied, deferred,
  stale/rebind-required, unsupported, and failed comparison posture

Must preserve:

- a digest match is not automatically semantic equivalence unless the basis says
  so
- mismatch explanation remains primitive and canonical, not a full diagnostics
  ontology
- equivalence claims do not erase meaningful variant distinctions
- unsupported comparisons fail closed with a structured unsupported mismatch
  kind
- comparison APIs never silently coerce rule versions, domains, or producer
  shapes to make a comparison succeed
- raw ready sequences cannot be compared by semantic APIs until comparison
  readiness has bound the equivalence basis and mismatch loci

Acceptance evidence:

- equivalence-basis tests for exact, declared-aspect, and
  compatibility-lowered equivalence
- mismatch tests for missing entries, additional entries, value mismatch,
  entry-kind mismatch, ordering mismatch, version mismatch, and unsupported
  comparison
- compile-fail or facade tests proving comparison APIs cannot be called without
  an explicit equivalence basis where the API can enforce it
- blind-consumer tests proving mismatch output can be interpreted without
  producer-private state
- unsupported-comparison tests proving cross-version, cross-domain, and
  unsupported projection comparisons do not collapse into ordinary inequality
- mismatch-locality tests proving each mismatch points at the smallest
  canonical locus available rather than only reporting whole-bundle failure
- compile-fail tests proving semantic comparison APIs reject raw ready sequences
  that have not progressed through `CanonicalComparisonReadyArtifact`
- proof-carriage tests proving comparison-ready artifacts carry equivalence
  basis and mismatch-loci proof facts

### Phase 4: Define Canonical Export And Golden Fixture Bundles

Purpose:

Create stable exportable evidence for tests, support bundles, and future
worth-harness replay without making transport encoding the authority.

Must ship:

- `CanonicalExportReady` proof phase
- export-ready basis bundles
- golden fixture bundle shape for canonical basis sequences
- stable semantic fixture comparison helpers
- fixture metadata for rule version, basis domain, producer shape, and
  equivalence basis
- rejection of debug-output, display-string, and transport-JSON fixture sources
  as canonical authority
- fixture debt markers for surfaces intentionally deferred to later milestones
- fixture manifests that list every sequence by domain, rule version, producer
  shape, equivalence basis, expected entry count, and expected cost counters
- semantic fixture comparison that can report the first canonical mismatch
  locus instead of snapshot-only diff noise
- `CanonicalExportReadyArtifact` carrying export-manifest and
  cost-observed proof facts
- boundary-bridged export restoration surfaces that downgrade basis freshness
  after fixture/replay/transport crossing and require authority readmission
  before current-validity claims resume

Must preserve:

- fixtures compare semantic basis entries, not incidental formatting
- export shape is a test/support boundary, not a universal runtime serializer
- transport encoding remains downstream from canonical basis
- golden fixtures can be used by `worth-harness` later without changing their
  proof grammar
- fixture files are stable evidence, but the manifest and basis entries are the
  authority; pretty formatting is not
- exported or restored fixture artifacts are not silently current after a trust
  boundary crossing

Acceptance evidence:

- golden fixture tests for every Milestone 1 basis domain
- fixture round-trip tests that preserve canonical basis meaning
- hostile formatting tests proving debug/display/JSON ordering cannot alter
  fixture comparison
- worth-harness expansion seed inventory for parity and replay suites
- manifest completeness tests proving every fixture names domain, rule version,
  producer shape, equivalence basis, entry count, and cost counters
- first-mismatch tests proving fixture comparisons return canonical mismatch
  evidence rather than only a file diff
- compile-fail tests proving a restored boundary-bridged export cannot satisfy
  current-validity APIs before readmission
- readmission tests proving an authority witness is required to restore current
  export readiness after a trust boundary

### Phase 5: Define Digest Algorithm Slots And Derived Digest Values

Purpose:

Allow canonical basis evidence to be compressed into digest values without
letting hash algorithms become the semantic authority.

Must ship:

- `CanonicalDigestAlgorithmId`
- `CanonicalDigestAlgorithmSlot`
- derived `CanonicalDigestValue` or equivalent digest carrier
- algorithm metadata that names input domain, canonicalization rule version,
  and digest output shape
- digest derivation APIs that consume `CanonicalBasisReady` or
  `CanonicalExportReady` artifacts
- explicit test-only deterministic digest support if needed for fixtures
- debt markers for production cryptographic policy if the final algorithm
  policy remains later work
- digest input-shape typing for single sequences, domain bundles, and export
  bundles
- digest metadata that records algorithm id, rule version, input domain,
  input-shape, entry count, and basis bundle id or sequence id
- negative APIs or compile-fail fixtures proving digest values cannot be used
  as a replacement for equivalence basis evidence
- `CanonicalDigestDerivationReadyArtifact` that consumes ready basis/export
  artifacts plus an admitted algorithm slot before deriving digest values
- algorithm-slot admission readiness that can deny unsupported domain/shape/
  version combinations through `worth-proof` checked outcomes

Must preserve:

- digest values are derived from canonical basis only
- algorithm identity is not semantic category identity
- algorithm slots do not accept raw bytes, debug strings, or transport JSON as
  semantic input
- final receipt, provenance, branch/merge/commit, and diagnostic digest
  semantics remain later milestone work
- digest equality alone does not authorize reuse, suppression, parity, or
  certification claims without a declared equivalence basis
- plain digest slots and plain basis payloads are not enough to derive a
  semantic digest; derivation readiness is the proof-bearing gate

Acceptance evidence:

- digest derivation tests from canonical basis artifacts
- compile-fail tests proving raw bytes/category-erased blobs cannot satisfy
  digest derivation APIs
- algorithm-version tests proving rule-version changes are visible in digest
  metadata
- collision-shaped hostile tests proving equal display strings or storage bytes
  in different domains remain distinct digest inputs
- input-shape tests proving export-bundle digest slots cannot be substituted
  for single-sequence digest slots
- digest-versus-equivalence tests proving matching digest values still require
  explicit equivalence basis before semantic sameness is claimed
- compile-fail tests proving digest derivation APIs reject plain basis payloads
  and plain digest slots that have not progressed into
  `CanonicalDigestDerivationReadyArtifact`
- checked algorithm-admission tests proving unsupported rule version, domain,
  or input-shape combinations deny without becoming generic failure

### Phase 6: Certify Production-Test Readiness

Purpose:

Close the milestone with adversarial local proof that the canonicalization
substrate is ready for production-shaped testing, without pretending real
runtime adoption has already been proven.

Must ship:

- Milestone 2 production-test readiness artifact
- certified-surface inventory for basis grammar, Milestone 1 basis builders,
  equivalence basis, mismatch basis, export bundles, and digest algorithm slots
- synthetic runtime pressure inventory covering ordered authority producers,
  reordered compatibility producers, support/export consumers, and
  category-adjacent hostile producers
- compile-fail boundary inventory
- canonical golden artifact inventory
- property seed inventory for ordering, category adjacency, compatibility
  lowering, equivalence, mismatch, and digest slot hostility
- worth-harness expansion points for future run matrices and replay/export
  suites
- runtime adoption assumptions and non-assumptions
- residual debt inventory
- explicit readiness pass/fail checklist mapping every phase gate to concrete
  evidence files
- fixture manifest inventory mapping golden artifacts to their owning tests and
  later `worth-harness` expansion point
- named non-assumptions for final crypto policy, later profile/receipt/
  diagnostic/branch domains, and real runtime lowering
- `CanonicalProductionTestReadyArtifact` carrying
  `CanonicalProductionReadinessCertified`
- readiness certification authority witness controlled by the milestone
  closeout/certification boundary

Must preserve:

- production-shaped tests may assume only what the readiness artifact names
- adopting crates still own proof that their real runtime lowering is correct
- local doubles do not grow into a generic runtime, scheduler, serializer, or
  storage engine
- Milestone 3 may consume profile-ready basis slots without reworking
  Milestone 2 canonicalization law
- no readiness artifact may imply production runtime safety beyond local
  foundational category law
- a markdown closeout may describe readiness, but only the proof-bearing
  readiness artifact can satisfy APIs that require certified Milestone 2
  readiness

Acceptance evidence:

- readiness artifact tests or golden snapshot
- no milestone-owned test file or fixture directory becomes a responsibility
  dump
- topology check proving basis, equivalence, mismatch, export, digest slots,
  and readiness tests live in responsibility-owned homes
- full crate tests and compile-fail tests pass with the Milestone 2 surfaces
  enabled
- readiness checklist tests proving every certified surface has hostile
  producer evidence, blind-consumer evidence, compile-fail boundaries, and
  golden fixtures where required
- residual-debt tests or manifest checks proving final cryptographic policy and
  later milestone domains are named debt rather than implied completion
- compile-fail tests proving production-shaped adoption or harness APIs reject
  closeout prose, plain manifests, and uncertified reports where
  `CanonicalProductionTestReadyArtifact` is required
- authority-witness tests proving readiness certification cannot be minted from
  ordinary caller code

## Must Ship

- canonical basis entry grammar with versioned rule identity
- typed basis domains and canonical basis loci
- typed basis entry references for mismatch, export, and digest-slot APIs
- concrete canonical basis value carriers that preserve value family rather
  than reducing semantic entries to strings or bytes
- canonical basis sequences and bundles
- canonicalization cost accounting on sequence construction
- proof-bearing canonical basis readiness and export readiness artifacts
- proof-bearing comparison readiness, digest-derivation readiness, and
  production-test readiness artifacts
- Milestone 2 proof markers for domain coherence, rule-version binding,
  equivalence-basis declaration, mismatch-loci binding, export-manifest binding,
  digest input-shape binding, cost observation, and production readiness
- basis builders for every completed Milestone 1 surface
- equivalence-basis vocabulary and comparison APIs
- mismatch-basis primitives with canonical loci and mismatch kinds
- canonical export/golden fixture bundle shape
- digest algorithm slots and derived digest value carriers that consume basis
  artifacts rather than raw blobs
- digest input-shape metadata for single-sequence, domain-bundle, and
  export-bundle compression
- hostile producer/consumer test fixtures that simulate runtime diversity
- production-test readiness artifact for Milestone 2

## Must Preserve

- canonical basis, not hash output, is the semantic authority
- no canonicalization meaning depends on insertion order, builder order,
  transport encoding, display labels, debug output, or crate-local layout
- equivalence, reuse, suppression, parity, and certification claims require an
  explicit basis
- mismatch explanation is self-describing enough for blind consumers
- unsupported comparisons remain explicit structured outcomes
- `worth-proof` owns proof progression law; `worth-foundational` owns the
  shared canonical evidence language
- every stronger readiness state is represented as a `worth-proof` artifact
  with explicit phase, proof set, and assumption basis
- domain crates keep ownership of storage layout, runtime execution, durability,
  scheduling, and migration behavior
- later milestone surfaces get extension points, not premature final ontology
- local adversarial doubles stay small semantic fixtures, not fake adopting
  runtimes
- canonicalization cost is visible at the boundary but cannot change canonical
  meaning
- digest equality cannot substitute for declared equivalence basis

## Acceptance Evidence

- cross-construction canonical-basis parity tests for every Milestone 1 surface
- property tests or hostile seed suites proving ordering independence across
  insertion order, builder order, serializer order, local map layout, and
  compatibility-origin ordering
- golden canonical basis fixtures for values, contracts, masks, state, patches,
  identities, locators, compatibility lowering, equivalence basis, mismatch
  basis, export bundles, and digest slots
- compile-fail tests proving raw unordered maps, raw bytes, debug strings,
  category-erased blobs, and unvalidated values cannot satisfy proof-bearing
  canonicalization APIs
- compile-fail tests proving raw indexes cannot satisfy domain-typed mismatch
  references and export-bundle digest slots cannot satisfy single-sequence
  digest APIs
- compile-fail tests proving raw basis sequences, plain bundles, plain digest
  slots, boundary-bridged fixtures, prose closeouts, and ordinary reports
  cannot satisfy APIs requiring their corresponding proof-bearing readiness
  artifacts
- mismatch tests proving blind consumers can interpret differences without
  producer-private state
- unsupported-comparison tests proving incompatible versions, domains, or
  equivalence scopes fail closed with structured evidence
- digest derivation tests proving digest values are derived from basis artifacts
  and carry algorithm/rule-version metadata
- cost-accounting tests proving entry counts, nested-sequence counts, and
  compatibility-lowering counts are stable and visible across producer shapes
- fixture-manifest tests proving every golden fixture names rule version,
  domain, producer shape, equivalence basis, entry count, cost counters, and
  owning test
- production-test readiness artifact naming certified surfaces, synthetic
  runtime pressures, compile-fail boundaries, golden artifacts, property seeds,
  worth-harness expansion points, runtime assumptions, non-assumptions, and
  residual debt
- proof-carriage tests proving each readiness artifact carries the specific
  proof facts named in the `WORTH-Proof Dependency Boundary`
- topology review showing canonical basis, equivalence, mismatch, export,
  digest-slot, and readiness tests are responsibility-shaped rather than a flat
  dump

## Architectural Notes

The implementation should preserve distinct responsibility homes. A likely
shape is:

```text
crates/worth-foundational/src/
  canonicalization/
    basis/
    domains/
    readiness/
    equivalence/
    mismatch/
    export/
    digest_slots/
    production_readiness/
```

Names may adapt during implementation, but the structure must keep these
responsibilities separable:

- basis grammar owns entry, sequence, bundle, ordering, and version law
- domain builders own conversion from completed foundational surfaces into
  canonical basis
- equivalence owns sameness claims and comparison scope
- mismatch owns difference localization and primitive mismatch kinds
- export owns golden/support/replay fixture shape
- digest slots own derived compression metadata and algorithm identity
- production readiness owns handoff evidence, not semantic construction
- cost accounting owns canonicalization boundary work counters, not
  performance/layout vocabulary for the runtime
- hostile producers own tiny local construction diversity only; reusable
  run-matrix or replay grammar belongs in `worth-harness`

Public exports should remain facade-controlled. Internal modules may reorganize,
but external callers should depend on durable canonicalization capabilities, not
deep topology.

## Test Topology

Expected test homes:

```text
crates/worth-foundational/tests/certification/canonicalization/
  basis/
  domains/
  equivalence/
  mismatch/
  export/
  digest_slots/
  cost_accounting/
  hostile_producers/
  production_readiness/

crates/worth-foundational/tests/ui/canonicalization/
  basis/
  equivalence/
  digest_slots/
  export/
  mismatch/
```

Test support must stay narrow. A fixture that only constructs a value basis
belongs with value-basis tests. A fixture that simulates a hostile producer for
multiple domains should either be a small local semantic fixture or become a
`worth-harness` expansion point. It must not become an unowned generic harness
inside `worth-foundational`.

## Production-Test Readiness Requirements

Milestone 2 must close with a concrete production-test readiness artifact.

It must name:

- certified canonicalization surfaces
- hostile producer and blind consumer shapes tested locally
- compile-fail boundaries for raw/unordered/category-erased inputs
- golden canonical basis fixtures
- property seeds for ordering, equivalence, mismatch, compatibility, and digest
  slot hostility
- worth-harness expansion points for replay/export and parity run matrices
- assumptions adopting runtimes may make about canonical basis law
- non-assumptions adopting runtimes must prove for themselves
- residual debt for final cryptographic policy, later milestone domains, or
  migration-specific parity
- phase-gate evidence files proving Phases 1 through 6 closed in order
- fixture manifest ownership, including which test owns each golden artifact
  and which future `worth-harness` lane may reuse it
- cost-counter evidence for every certified canonicalization boundary

Production-shaped tests may assume:

- Milestone 1 surfaces can be converted into canonical basis through the
  certified Milestone 2 APIs
- canonical basis entries are ordered, versioned, and domain-typed
- equality, mismatch, and digest-slot operations are basis-driven
- listed canonicalization cost counters are visible and stable for the
  certified surfaces
- Milestone 2 readiness states named in this spec are carried as `worth-proof`
  artifacts with explicit proof facts and basis posture

Production-shaped tests may not assume:

- real adopting-runtime lowering is correct
- final receipt, profile, diagnostic, provenance, branch/merge/commit, or
  performance ontology exists
- digest algorithm policy is final unless the readiness artifact explicitly
  says so
- local synthetic producers prove real runtime invariants
- a digest value alone is sufficient evidence for semantic equivalence,
  suppression, reuse, parity, or certification
- fixture formatting stability is proof of semantic stability unless the
  manifest and basis-entry comparison also pass
- a boundary-bridged or restored artifact is current until `worth-proof`
  readmission or rebinding has happened
- an authority/capability witness is itself proof that canonicalization,
  comparison, export, digest derivation, or production readiness occurred

## Sequencing Notes

Milestone 2 belongs immediately after Milestone 1 because every later
foundational surface needs canonical basis participation before it can claim
digest parity, replay stability, support comparability, or migration readiness.

- Milestone 3 profiles need canonical basis slots before profile identity and
  profile-driven elision can be digestible.
- Milestone 4 artifact/materialization categories need basis and mismatch law
  before reports, summaries, artifacts, and receipts can compare honestly.
- Milestone 5 branch/merge/commit vocabulary needs basis domains and mismatch
  loci before commit evidence can be replayed and compared.
- Milestone 6 diagnostics need mismatch primitives and locators before
  explanations can point at canonical difference.
- Milestone 7 lineage/provenance/receipts need digest-basis and export bundles
  before provenance can be self-describing and certifiable.
- Milestone 8 performance/layout vocabulary needs cost-visible
  materialization boundaries before performance reports can attach to canonical
  artifacts honestly.
- Milestone 11 migrations need canonical basis parity before crate-local
  dialects can converge with confidence.

## Explicit Non-Goals

- final cryptographic digest policy for every WORTH artifact
- final report, summary, artifact, or receipt taxonomy
- profile vocabulary or profile-elision policy
- diagnostics and explanation ontology
- lineage and provenance ontology
- branch, merge, and commit vocabulary
- performance and layout vocabulary beyond canonicalization boundary honesty
- adopting-crate migrations or real runtime lowering proof
- generic serializer/deserializer runtime
- generic executor, scheduler, storage engine, or workflow harness
- replacing `worth-proof` artifact, witness, phase, or transition machinery

## Self-Check

- Does the milestone solve a real structural problem or just package work
  cosmetically? Yes. It creates the canonical evidence substrate every later
  shared surface needs for replay, comparison, certification, and migration.
- Is the adversarial constraint precise and load-bearing? Yes. It attacks
  construction order, local layout, transport encoding, debug/display strings,
  category collapse, unsupported-comparison collapse, hidden cost, and
  hash-as-authority shortcuts.
- Does the milestone preserve crate authority boundaries? Yes.
  `worth-foundational` owns canonical basis language only; domain crates keep
  runtime, storage, execution, durability, and migration authority.
- Does the milestone define proof obligations, not just implementation tasks?
  Yes. Closure requires compile-fail boundaries, property/hostile tests,
  golden fixtures, fixture manifests, mismatch evidence, unsupported
  comparison outcomes, digest-slot derivation, cost-accounting evidence, and
  `worth-proof` readiness artifacts for canonical basis, comparison, export,
  digest derivation, and production-test readiness.
- Could a competent engineer map this spec into honest types, modules, and
  tests? Yes. The phases map directly to basis grammar, domain builders,
  equivalence, mismatch, export fixtures, digest slots, cost accounting,
  hostile producers, and readiness evidence.
- Does the milestone belong in this roadmap sequence, or is it out of order?
  Yes. It must come after Milestone 1's semantic substrate and before profiles,
  artifacts, branch/merge/commit evidence, diagnostics, provenance, performance
  vocabulary, and migrations consume canonical evidence.
