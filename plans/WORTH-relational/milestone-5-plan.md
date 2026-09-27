# Milestone 5 Plan: Honest Schema Continuity, Transition Truth, and Classified Reconciliation

## Summary

Milestone 5 will make schema evolution an authoritative runtime capability rather than a builder-time assumption or host-side coordination pattern. The runtime will explicitly understand what kind of continuity is honest at each schema boundary, classify that boundary across multiple semantic layers, and publish canonical transition artifacts that CDC, replay, recovery, and reconciliation consume directly.

This revision makes the following hard commitments:

- schema is stratified and formally scoped
- generic compatibility is allowed only as a derived summary, never as an authority-bearing decision concept
- continuation authority lives in continuation outcome classes, not in broad compatible/incompatible labels
- subscriber contract is a distinct layer from publication contract
- historical interpretation is explicitly defined and guarded
- reconciliation ordering semantics are explicit
- descriptor semantics, versioning, and invalidation are first-class authority
- locality and asymptotic cost requirements are design laws, not just observability goals
- audit recomputation is an explicit typed mode, not an implementation fallback

The governing invariant is:

Already-published history must never be silently reinterpreted by later schema evolution. Any new interpretation of prior facts must appear as new authoritative history through an explicit reconciliation or interpretation-publishing commit; prior canonical artifacts remain immutable.

## Core Semantic Model

### 1. Schema strata

Every schema declaration and every schema diff atom must declare which strata it affects. Unstratified schema change is not a valid runtime concept.

Required strata:

- `StructuralShape`
  - entity kinds
  - relation kinds
  - field presence
  - relation endpoint structure
  - optional vs required shape
  - declared authoritative derived-field existence
- `ValueDomain`
  - field types
  - enum domains
  - units
  - coordinate frames
  - tolerances
  - precision contracts
  - default/null semantics
- `EntityIdentitySemantics`
  - schema meaning that affects entity identity continuity
  - identity-bearing field interpretation
- `CorrespondenceSemantics`
  - schema meaning that affects cross-branch or cross-version matching
- `LineageSemantics`
  - schema meaning that affects successor/predecessor interpretation and historical identity resolution
- `BehavioralSemantics`
  - legality contracts
  - semantic invariants
  - authoritative computed meaning where it affects truth interpretation
- `PublicationContract`
  - patch/CDC observable surface
  - projection-visible observable surface
  - authoritative publication encoding choices
- `SubscriberContract`
  - subscriber-declared consumable schema surface
  - continuation/upgrade expectations
  - contract capabilities and accepted boundary classes

Rule:

- `PublicationContract` describes what the runtime emits.
- `SubscriberContract` describes what a subscriber has declared it can consume.
- These are related but not identical, and the plan must not collapse them into one layer.

### 2. Historical interpretation

The runtime must carry an explicit `HistoricalInterpretationSensitivity` classification. Historical interpretation means any change that could alter the meaning of already-published facts when observed later.

It includes:

- value meaning of prior facts
- legality meaning of prior facts
- identity/correspondence meaning of prior facts
- publication/projection meaning of already-published facts
- derived semantic interpretation of prior facts

It does not mean:

- non-authoritative formatting differences
- internal storage layout differences
- diagnostics-only presentation changes

Rule:

- schema bridges may govern future continuation across a boundary
- they may not silently mutate the meaning of prior committed artifacts
- if the system needs to publish a new interpretation layer over prior facts, that must be an explicit new authoritative artifact with its own schema lineage and diagnostics

### 3. Compatibility layers

Generic compatibility is permitted only as a derived summary field such as `CompatibilityObservation`. It must never drive runtime behavior.

Runtime authority must instead use these layers:

- `ReconciliationCompatibility`
  - can these schema states converge into one authoritative schema result?
- `ContinuationCompatibility`
  - can a current subscriber contract continue across this boundary?
- `Bridgeability`
  - if continuation is allowed, what kind of bridge is honest?
- `HistoricalInterpretationSensitivity`
  - does this boundary risk changing the meaning of already-published truth?

The runtime must never contain control flow of the form `if compatible { continue }`.

### 4. Continuation outcome classes

Automatic continuation is in scope only for transitions that classify into a non-rejected continuation outcome and admit a canonical continuation descriptor under the declared subscriber contract.

Required continuation outcomes:

- `ContinueUnchanged`
  - no contract-relevant change for the subscriber
- `ContinueWithTransparentBridge`
  - automatic continuation
  - old subscriber interpretation remains semantically correct even if boundary metadata is ignored
- `ContinueWithVisibleBridge`
  - automatic continuation
  - boundary crossing is surfaced explicitly
  - old subscriber interpretation remains correct, but the boundary is materially observable
- `ContinueWithContractUpgrade`
  - runtime may upgrade only if the subscriber contract explicitly declared support for that upgrade class
- `RequireRenegotiation`
  - valid schema transition, but subscriber must re-handshake to continue honestly
- `Rejected`
  - no honest continuation path exists

Rule for `ContinueWithVisibleBridge`:

- if a subscriber ignores the surfaced boundary metadata and its semantic interpretation would become wrong, this class is invalid
- such a case must instead be `ContinueWithContractUpgrade` or `RequireRenegotiation`

### 5. Reconciliation policies

Replace vague ergonomic policy names with preservation-based authority names.

Required policies:

- `RejectLossyNarrowing`
- `PreserveInformation`
- `PreserveTargetContract`
- `PreserveSourceContract`
- `PermitLossyNarrowingWithAnnotation`
- `RequireExplicitProjection`

Rules:

- default narrowing policy is `RejectLossyNarrowing`
- policies must state what is preserved, not which side wins in informal terms
- any lossy resolution must be explicitly annotated in canonical artifacts and diagnostics

### 6. Reconciliation ordering semantics

This milestone must choose a deterministic reconciliation contract rather than merely acknowledging order dependence.

Chosen contract for the plan:

- reconciliation classification is performed on a canonicalized ordered pair of schema inputs
- canonicalization must normalize the pair before classification using an authoritative ordering rule
- the ordering rule becomes part of the reconciliation artifact semantics
- if truly directional reconciliation is required for a case, direction must be explicit runtime input and recorded in the artifact
- `A -> B` and `B -> A` may not silently diverge under the same canonicalized reconciliation mode

Reconciliation must produce:

- explicit resulting schema identity
- explicit schema lineage parentage
- explicit canonical ordering mode used
- explicit note of whether the result is symmetric or direction-sensitive

### 7. Descriptor authority

Descriptors are not convenience metadata. They become part of the authoritative truth story for schema continuity.

Required descriptor families:

- `SchemaBoundaryFingerprint`
- `SchemaBridgeDescriptor`
- `SchemaContinuationDescriptor`
- `SchemaReconciliationDescriptor`
- `SchemaInterpretationDescriptor` if reinterpretation is ever explicitly published

Descriptor rules:

- descriptors must have semantic version identity
- canonicalization version is explicit
- replay compatibility expectations across descriptor evolution are explicit
- normalization rule changes are treated as semantic descriptor-version changes, not invisible tooling changes
- descriptor invalidation rules must be explicit:
  - if descriptor construction logic changes semantically, old descriptors remain authoritative for old history
  - new logic must version forward rather than silently reinterpret old descriptors

### 8. Performance intent laws

These must be built into the early phases, not deferred to counters alone.

Required asymptotic intent:

- transition classification cost scales with changed boundary width plus reused fingerprint checks, not total registry size
- bridge descriptor construction cost scales with changed surface, not total schema size
- subscriber continuation evaluation scales with descriptor lookup plus contract match, not schema reclassification
- replay verification scales with persisted boundary digest/summary depth, not full historical reclassification in the normal path
- recovery verification scales with persisted boundary summaries and descriptor chain validation, not full re-diffing except in audit modes
- reconciliation breadth should preserve unchanged subtrees and classify only affected subgraphs when locality is available

## Sequential Phases

### Phase 1. Lock the formal model and certification contract

Deliverables:

- update [plans/worth-relational/test-requirements.md](C:\Users\Esther\Documents\Programming\WORTH_workspace\WORTH\plans\worth-relational\test-requirements.md) with:
  - `Schema evolution CDC contract test`
  - `Schema reconciliation classification test`
- update milestone planning docs with:
  - schema strata model
  - historical interpretation definition
  - continuation outcome classes
  - reconciliation ordering contract
  - descriptor semantic versioning contract
  - asymptotic intent statements

Required adversarial constraints to write explicitly:

- already-published history cannot be silently reinterpreted
- continuation decisions must come from persisted boundary descriptors, not raw rediscovery
- reconciliation must produce deterministic resulting schema identity and lineage
- long-lived subscribers must not accumulate unbounded continuation cost

Required machine-checkable outputs:

- `schema_transition_digest`
- `schema_boundary_cdc_digest`
- `schema_reconciliation_digest`
- `subscriber_contract_matrix`
- `transition_decision_digest`
- `schema_lineage_digest`
- `descriptor_version_digest`

Exit condition:

- milestone semantics are locked in docs before code changes begin

### Phase 2. Build the schema strata and transition type system

Deliverables:

- add types for:
  - `SchemaStratum`
  - `SchemaElementRef`
  - `SchemaDiffAtom`
  - `HistoricalInterpretationSensitivity`
  - `ProposedSchemaTransition`
  - `ValidatedSchemaTransition`
  - `LoweredSchemaTransitionPlan`
  - `SchemaTransitionArtifact`
  - `SchemaTransitionSummary`
- make every diff atom carry:
  - affected stratum/strata
  - affected element
  - publication impact
  - subscriber impact
  - historical interpretation sensitivity
- establish four explicit barriers:
  - `ConstructionBarrier`
  - `ValidationBarrier`
  - `LoweringBarrier`
  - `ExecutionBarrier`

Compile-time enforcement targets:

- direct schema registry swap cannot produce an execution-ready transition
- unstratified change sets cannot be lowered
- narrowing without declared policy cannot be lowered
- subscriber continuation code cannot consume raw schema diffs directly

Honesty rule:

- the type system should prevent malformed requests where possible
- dynamic truths such as whether a canonical bridge exists or whether a subscriber contract remains semantically unambiguous are validation/lowering concerns, not overpromised compile-time guarantees

Exit condition:

- the runtime has a phase-typed schema transition model with explicit barrier ownership

### Phase 3. Implement transition-local classification with locality guarantees

Deliverables:

- add classification families:
  - `SchemaReconciliationClassification`
  - `SchemaContinuationClassification`
  - `SchemaBridgeabilityClassification`
  - `CompatibilityObservation` as derived metadata only
- define precise membership criteria, examples, and non-examples for each change class
- add stable sub-schema fingerprints/hashes

Fingerprint rules:

- fingerprints derive only from canonical authoritative schema surfaces within relevant strata
- they must be invariant under non-authoritative formatting or declaration-order noise
- they must not include non-authoritative metadata

Classification rules to pin down:

- adding an optional field with explicit default/null semantics:
  - structural/value-domain additive
  - may be `ContinueUnchanged` or `ContinueWithTransparentBridge` depending on subscriber contract
- widening enum domain:
  - value-domain change
  - continuation may be visible bridge, upgrade, or renegotiation
- optional to required:
  - narrowing
  - never transparent
- unit/tolerance/precision reinterpretation:
  - value-domain + behavioral + historical-interpretation-sensitive
  - never transparent
- one field split into two:
  - structural + behavioral + publication-sensitive
  - not transparent; projected bridge only if explicitly classified honest

Performance constraints enforced in this phase:

- full schema diff/classification happens only at transition time
- classification result is converted into compact descriptors
- no later path is allowed to rediscover classification from raw schema state in normal operation

Exit condition:

- classification is rule-based, locality-aware, and deterministic

### Phase 4. Add continuation, reconciliation, and interpretation descriptors

Deliverables:

- add descriptor families:
  - `SchemaBoundaryFingerprint`
  - `SchemaBridgeDescriptor`
  - `SchemaContinuationDescriptor`
  - `SchemaReconciliationDescriptor`
  - reserve `SchemaInterpretationDescriptor` for explicit reinterpretation-publishing commits
- define descriptor semantic versioning:
  - `descriptor_semantics_version`
  - `canonicalization_version`
  - compatibility policy for replay/recovery across descriptor versions
- define descriptor invalidation semantics:
  - logic fixes that change semantics require a new descriptor semantics version
  - old descriptors remain authoritative for old history
  - replay compares against the semantics version recorded at commit time

Bridge-chain normalization rules:

- multiple successive continuation-compatible boundaries must compose into a normalized descriptor
- checkpoints store normalized continuation proof, not an unbounded raw boundary list
- define a max raw chain depth before mandatory normalization
- normalization must preserve semantic meaning and determinism

Exit condition:

- transition-time work has been amortized into canonical reusable descriptors
- descriptor semantics are themselves authoritative and versioned

### Phase 5. Integrate schema transition and reconciliation into the commit pipeline

Deliverables:

- add explicit commit phases:
  - schema transition validation
  - stratum-aware classification
  - descriptor construction
  - authoritative apply
  - artifact assembly
- extend [CommitResult](C:\Users\Esther\Documents\Programming\WORTH_workspace\WORTH\crates\worth-relational\src\transactions\data\outcomes.rs) and summary families with:
  - schema transition summary
  - continuation summary
  - reconciliation summary
  - descriptor version summary
  - historical interpretation sensitivity summary
- extend [CanonicalCommitEnvelope](C:\Users\Esther\Documents\Programming\WORTH_workspace\WORTH\crates\worth-relational\src\replay\data\mod.rs) with:
  - source schema identity
  - target schema identity
  - schema transition artifact
  - continuation descriptor
  - reconciliation descriptor
  - resulting schema identity
  - schema lineage parentage
  - descriptor semantics version

Add commit failure classes for:

- unstratified schema diff
- unsupported bridge descriptor
- historical reinterpretation violation
- narrowing without policy
- type-incompatible conflict
- structural-incompatible conflict
- directionality mismatch under canonical reconciliation mode
- descriptor-version incompatibility during lowering

Important interpretation rule:

- if reconciliation or explicit interpretation work produces a new way to understand prior facts, the runtime must publish that as new authoritative history
- it must not mutate old artifacts or rewrite old envelopes

Exit condition:

- schema continuity becomes part of the canonical commit story end-to-end

### Phase 6. Evolve the subscriber contract state machine

Deliverables:

- extend [SubscriberCheckpoint](C:\Users\Esther\Documents\Programming\WORTH_workspace\WORTH\crates\worth-relational\src\publication\cdc\data\subscriber_checkpoint.rs) with:
  - subscriber contract identity
  - normalized continuation proof
  - post-boundary contract identity
  - descriptor semantics version
- extend `SubscriberResumeRequest` so subscribers declare:
  - consumable schema surface
  - accepted continuation classes
  - upgrade support classes
- extend [SubscriberStreamBatch](C:\Users\Esther\Documents\Programming\WORTH_workspace\WORTH\crates\worth-relational\src\publication\cdc\data\subscriber_stream_batch.rs) with:
  - crossed boundary fingerprints
  - continuation outcome
  - applied continuation descriptor summary
  - upgrade metadata when applicable

Rules:

- `ContinueWithContractUpgrade` is allowed only if subscriber declared support
- `RequireRenegotiation` is a hard typed outcome, not an implicit pause
- visible bridge must preserve correctness even if boundary metadata is ignored
- subscribers may cross multiple compatible boundaries via normalized descriptors
- subscriber resume evaluation must be descriptor lookup plus contract match, never whole-schema rediscovery

Optional sanity rule to include in implementation:

- define a maximum tolerated normalized descriptor complexity before the runtime forces renegotiation instead of infinite continuation growth

Exit condition:

- the runtime can honestly say what continuation is safe for each subscriber contract

### Phase 7. Make replay and durability descriptor-first

Deliverables:

- extend durability compatibility/reporting with:
  - transition artifact mismatch
  - continuation descriptor mismatch
  - reconciliation descriptor mismatch
  - descriptor semantics version mismatch
  - schema lineage mismatch
- extend replay mismatch classes with:
  - schema transition drift
  - continuation descriptor drift
  - reconciliation drift
  - descriptor version drift
  - schema lineage drift

Add explicit verification modes:

- `NormalRecoveryVerification`
- `AuditRecoveryVerification`
- `CorruptionDiagnosisReplay`

Rules:

- normal recovery uses persisted descriptor digests and summaries
- audit modes may recompute classification/descriptor construction from first principles
- corruption diagnosis may perform deep recomputation and artifact reconstruction
- these modes must be explicit planner/runtime inputs, not ad hoc debugging branches

Verification order:

1. descriptor identity/digest parity
2. summary parity
3. deep artifact parity only in audit or failure modes

Exit condition:

- normal replay/recovery are fast, descriptor-first, and semantically honest
- expensive recomputation is structurally isolated to explicit audit paths

### Phase 8. Diagnostics, traceability, and complexity contracts

Deliverables:

- add diagnostics artifacts for:
  - schema classification
  - continuation classification
  - bridge descriptor construction
  - contract upgrade decision
  - renegotiation decision
  - reconciliation result
  - interpretation sensitivity
  - descriptor version selection
- add structured fields for:
  - schema stratum
  - schema element
  - source/target schema identity
  - continuation outcome
  - bridgeability classification
  - historical interpretation sensitivity
  - resulting schema identity
  - lineage parentage
  - descriptor semantics version
  - policy provenance

Complexity contracts to add now, not later:

- transition classification cost
- bridge descriptor construction cost
- subscriber resume evaluation cost
- reconciliation breadth cost
- normal replay verification cost
- audit replay recomputation cost

Required counters:

- total schema elements inspected
- changed subtrees inspected
- unchanged subtrees reused by fingerprint
- bridge descriptors built
- normalized descriptor compositions performed
- continuation outcomes by class
- renegotiation-required boundaries
- historical-interpretation-sensitive boundaries
- reconciliation outcomes by policy
- descriptor version mismatches encountered

Exit condition:

- milestone 5 is mechanically explainable and cost-honest

### Phase 9. Certification implementation and closeout

Implement and pass the new suites plus extend old ones.

`Schema evolution CDC contract test`

- long-running subscribers crossing harmless additive boundaries
- visible-bridge boundaries where correctness is preserved even if metadata is ignored
- contract-upgrade boundaries for subscribers that declared support
- renegotiation-required boundaries
- incompatible boundary rejection
- normalized multi-boundary continuation
- exact parity across live execution, replay, checkpoint+suffix recovery, and durable rebuild

`Schema reconciliation classification test`

- additive reconciliation
- narrowing default rejection
- preservation-based policy resolution
- type conflict rejection
- structural conflict rejection
- deterministic resulting schema identity and lineage
- canonicalized reconciliation ordering stability

Extend these existing tests with schema-transition-bearing histories:

- `Diff/CDC truth parity test`
- `Hostile commit/replay equivalence test`
- `Durable recovery and schema mismatch test`

Exit condition:

- milestone closeout can truthfully claim that the runtime itself understands what kind of continuity is honest

## Public API / Type Changes

New public families:

- `schema::SchemaStratum`
- `schema::SchemaElementRef`
- `schema::SchemaDiffAtom`
- `schema::HistoricalInterpretationSensitivity`
- `schema::SchemaContinuationClassification`
- `schema::SchemaBridgeabilityClassification`
- `schema::SchemaReconciliationClassification`
- `schema::CompatibilityObservation`
- `schema::SchemaBoundaryFingerprint`
- `schema::SchemaBridgeDescriptor`
- `schema::SchemaContinuationDescriptor`
- `schema::SchemaReconciliationDescriptor`
- `schema::SchemaLineageArtifact`
- `publication::SubscriberContractDeclaration`
- `replay::ReplayVerificationMode`
- `durability::RecoveryVerificationMode`

Evolved public artifacts:

- [CanonicalCommitEnvelope](C:\Users\Esther\Documents\Programming\WORTH_workspace\WORTH\crates\worth-relational\src\replay\data\mod.rs)
- [CommitResult](C:\Users\Esther\Documents\Programming\WORTH_workspace\WORTH\crates\worth-relational\src\transactions\data\outcomes.rs)
- [SubscriberCheckpoint](C:\Users\Esther\Documents\Programming\WORTH_workspace\WORTH\crates\worth-relational\src\publication\cdc\data\subscriber_checkpoint.rs)
- [SubscriberResumeRequest](C:\Users\Esther\Documents\Programming\WORTH_workspace\WORTH\crates\worth-relational\src\publication\cdc\data\subscriber_resume_request.rs)
- [SubscriberStreamBatch](C:\Users\Esther\Documents\Programming\WORTH_workspace\WORTH\crates\worth-relational\src\publication\cdc\data\subscriber_stream_batch.rs)
- recovery mismatch/reporting types
- replay mismatch and verification-mode types

## Assumptions and Defaults

- generic compatibility is retained only as descriptive summary metadata and is never allowed to govern runtime behavior
- subscriber contract is a distinct layer from publication contract
- reconciliation uses canonicalized ordering by default; directionality must be explicit when needed
- descriptor semantics are versioned authority, not tooling detail
- already-published history is immutable; any new interpretation of prior facts must be published as new authoritative history
- normal replay/recovery paths are descriptor-first; deep recomputation is reserved for explicit audit modes
- automatic continuation remains in scope only for non-rejected continuation outcome classes that admit canonical continuation descriptors under the declared subscriber contract
