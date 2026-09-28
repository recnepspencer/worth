# Milestone 9.3 Engineering Spec: Subscription Family Diagnostics, Bridge Parity, And Runtime Certification

> **Status:** Completed.
>
> **Roadmap parent:** [worth_query_roadmap.md](./worth_query_roadmap.md)
>
> **Vision parent:** [worth_query_vision.md](./worth_query_vision.md)
>
> **Prior milestone:** [milestone-9.2.md](./milestone-9.2.md)
>
> **Prior closeout:** [milestone-9.2-closeout.md](./milestone-9.2-closeout.md)
>
> **Next milestone:** [milestone-9.3.1.md](./milestone-9.3.1.md) will own
> cross-runtime causal diagnostics and Query inspection before the public
> runtime API stabilization gate.
>
> **Test requirements:** [test-requirements.md](./test-requirements.md)
>
> **Primary architectural driver:** make automatic query subscription-family
> selection explainable as one query-owned, bridge-honest, signal-admitted
> proof chain whose support claims, diagnostics, and certification coverage stay
> mechanically aligned.
>
> **Companion docs:**
> - [MENTALITY.md](../../docs/coding-guidelines/MENTALITY.md)
> - [arch_laws.md](../../docs/coding-guidelines/arch_laws.md)
> - [perf_laws.md](../../docs/coding-guidelines/perf_laws.md)
> - [domain_laws.md](../../docs/coding-guidelines/domain_structure_laws.md)
> - [worth_query_vision.md](./worth_query_vision.md)
> - [worth_query_roadmap.md](./worth_query_roadmap.md)
> - [test-requirements.md](./test-requirements.md)
> - [milestone-9-closeout.md](./milestone-9-closeout.md)
> - [milestone-9.1.md](./milestone-9.1.md)
> - [milestone-9.1-closeout.md](./milestone-9.1-closeout.md)
> - [milestone-9.2.md](./milestone-9.2.md)
> - [milestone-9.2-closeout.md](./milestone-9.2-closeout.md)

## Goal

Make Query's automatic subscription-family selection, runtime-backed lifecycle
support reporting, bridge parity explanation, and certification closure
mechanically explainable. Every admitted automatic subscription family must be
diagnosable as one canonical query-owned declaration and one canonical
bridge-facing lowering, with runtime-backed lifecycle evidence and support
metadata that agree on what is truly supported.

## Why This Milestone Exists

Milestones 9.1 and 9.2 already made subscription declaration and active
lifecycle real. They did not yet close the honesty boundary around automatic
subscription selection and support claims.

Without Milestone 9.3, `worth-query` could still fail in subtle but dangerous
ways:

- automatic subscription-family selection could choose a family that works in
  runtime code but cannot be explained through canonical declaration and bridge
  lowering artifacts
- support reporting could claim a query family is runtime-backed even when the
  certified lifecycle matrix lacks hostile proof for that family
- diagnostics could stop at "subscription admitted" rather than exposing which
  declaration family, bridge family, signal strategy, basis posture, lifecycle
  class, or denial boundary actually decided the outcome
- lifecycle certification could prove one generic active lane story while
  hiding family-specific differences between detail, inspector, collection,
  grouped, and bounded-materialization subscriptions
- offline analysis could require re-querying the runtime rather than consuming
  one self-describing subscription bundle

Milestone 9.3 therefore owns the explanation boundary that 9.1 and 9.2
intentionally left narrow. It answers:

- how Query reports whether a query family is subscription-capable and what
  runtime-backed surface is actually supported
- how automatic family selection, declaration, bridge lowering, signal
  strategy, active lifecycle, continuation, preview, and closeout evidence are
  gathered into one canonical diagnostic bundle
- how bridge parity is expressed as an explicit proof artifact rather than a
  narrative claim
- how certification rows prove every admitted automatic family is covered by
  diagnostics, support reporting, and hostile rejection paths
- how diagnostics distinguish declaration-family drift from lifecycle-instance
  churn

It intentionally stops before Milestone 10 store-backed execution parity and
Milestone 11 durable replay/restart closure. Runtime-backed explanation and
certification can close now; persisted subscription replay cannot.

## Governing Summaries

- `MENTALITY.md`: protects adversarial constraint first and enforcement over
  convenience. For 9.3, the hard problem is not "show some debugging info"; it
  is proving that automatic subscription behavior is not inventing hidden
  semantics above bridge and signal.
- `arch_laws.md`: Laws 7, 8, 10, 21, 27, 30, 32, 33, 35, 40, and 41 dominate
  this milestone. Diagnostics and certification must be self-describing
  envelopes built from one canonical proof chain, not duplicate authorities or
  post-hoc reconstruction.
- `perf_laws.md`: support reporting, bridge parity explanation, and diagnostic
  bundle assembly must expose exact counters for family coverage, bundle width,
  parity comparison breadth, and denial stages. Offline explanation cannot hide
  broad rescans or repeated rediscovery behind "debug-only" surfaces.
- `domain_laws.md`: family support reporting, diagnostic evidence, bridge parity
  explanation, bundle assembly, support matrices, and certification harnesses
  are separate responsibilities. They must not collapse into one catch-all
  `subscription_diagnostics.rs` bag.
- `worth_query_vision.md`: Query promises one typed read model promotable to
  live subscriptions with query-shaped maintenance. Milestone 9.3 closes the
  explainability and trust surface for that promise.
- `worth_query_roadmap.md`: Milestone 9.3 must prove automatic subscription
  family selection is diagnosable, bridge-honest, and runtime-certified before
  store-backed milestones build on top of it.
- `test-requirements.md`: the missing 9.3 named certification suite must become
  the closeout bar, and it must require family-aware diagnostics, bridge parity
  proof, support reporting parity, hostile rows, and canonical verification
  output.
- `milestone-9.1.md` and `milestone-9.1-closeout.md`: declaration, bridge
  lowering, admission diagnostics, support profiles, and activation input are
  already authoritative. 9.3 may explain and certify them, but may not invent a
  second declaration authority.
- `milestone-9.2.md` and `milestone-9.2-closeout.md`: runtime-backed lifecycle,
  sharing, continuation, preview isolation, closeout, performance receipts, and
  shipped lifecycle certification already exist. 9.3 must consume those
  artifacts as evidence, not recreate lifecycle meaning from scratch.

## Adversarial Constraint

Milestone 9.3 must survive the following hostile condition:

> For every admitted automatic query subscription family, Query must emit one
> diagnostic and certification bundle that lets an offline observer reconstruct
> the exact family selection, declaration, bridge lowering, signal strategy,
> runtime-backed lifecycle support, continuation/preview posture, and denial or
> success path without access to hidden runtime state, while ensuring that no
> supported family claim can exist unless there is family-specific runtime
> certification and hostile rejection coverage proving the same semantics.

Concretely, the design must remain correct when all of the following are true:

- two subscriptions share the same active lifecycle shape but differ in
  declaration family, view-shape, or bridge slice meaning
- a grouped or inspector family lowers onto a bridge family shared with another
  query-side family, but diagnostics must still preserve query-family
  distinctness
- runtime support metadata is asked for before activation, after activation,
  after continuation, and after preview discard or promotion
- an admitted family has both a control-lane certification row and one or more
  hostile rows that fail at different boundaries
- diagnostics richness changes, but the underlying subscription meaning and
  support claim must not change
- an offline tool must explain why a request was unsupported, bridge-denied,
  lifecycle-denied, preview-denied, or uncertified without re-running Query

If any supported path:

- claims runtime-backed support without a family-specific certified row
- emits a diagnostic bundle that cannot be mapped back to the canonical query,
  declaration, bridge, signal, lifecycle, and support digests
- collapses distinct query subscription families into one generic bridge/lane
  explanation
- requires mutable runtime access or hidden host observer state to interpret a
  diagnostic bundle
- treats support metadata as advisory text rather than mechanically derived
  evidence
- lets diagnostics or certification mutate subscription meaning instead of
  reporting it
- ships a supported family with no hostile rejection coverage

then Milestone 9.3 has failed.

## Product Decision Lock

- `worth-query` owns query-family support reporting, subscription diagnostic
  evidence, bridge parity explanation artifacts, canonical subscription bundle
  assembly, runtime-backed certification orchestration, and the milestone 9.3
  family-coverage matrix.
- `worth-query` does not own bridge subscription protocol semantics, signal
  observation semantics, relational identity truth, or durable replay
  semantics. It may only report and certify how admitted query-owned artifacts
  lower into those authorities.
- `worth-runtime-bridge` remains authoritative for bridge declaration family,
  basis request posture, preview residue classes, and bridge-facing lifecycle
  semantics beneath admitted query lowering.
- `worth-signal` remains authoritative for admitted observation strategy,
  invalidation, scheduling, and maintenance execution beneath the active query
  lane.
- `worth-relational` remains authoritative for query meaning, policy/tenant
  masking, branch and snapshot basis identity, identity evolution,
  correspondence, and preview promotion authority.
- Milestone 9.3 diagnostics and certification must consume already-authoritative
  artifacts:
  - `QuerySubscriptionFamilySelection`
  - `QuerySubscriptionDeclarationArtifact`
  - `BridgeSubscriptionLoweringPlan`
  - `QuerySubscriptionAdmissionArtifact`
  - `SubscriptionActivationInput`
  - `ActiveSubscriptionLaneAdmission`
  - `ActiveSubscriptionLaneHandle`
  - `SubscriptionConsumerAttachment`
  - `QueryDeliveryWindow`
  - `QueryDeliveryBatch`
  - `SubscriptionContinuationEvidence`
  - `PreviewSubscriptionIsolationArtifact`
  - `SubscriptionLifecycleCloseout`
  - `SubscriptionLifecycleCertificationBundle`
- Support reporting is not a second lifecycle authority:
  - support metadata says what surfaces are admitted, supported, denied, or
    deferred
  - lifecycle certification says what runtime-backed proof was actually closed
  - bridge parity explanation says how the admitted query artifacts lower into
    bridge-facing semantics
  - canonical diagnostic bundles tie those surfaces together without letting
    one rewrite the others
- Automatic subscription diagnostics must remain family-aware:
  - detail, inspector, collection, grouped, and bounded-materialization
    families remain distinct query-side families even when they share lower
    bridge or signal strategies
  - declaration-family changes must remain distinct from lifecycle-instance
    changes such as lane handle, attachment, delivery sequence, continuation
    epoch, or preview closeout
- 9.3 may widen support and certification visibility, but it may not add hidden
  fallback families, generic subscription shortcuts, durable restart claims, or
  store-backed parity claims.

## Typed Phase Progression Lock

Milestone 9.3 must define one proof-bearing explanation chain. Diagnostics and
certification must not be loose string formatting around existing artifacts.

Required phase progression:

- `QuerySubscriptionSupportSubject`
  - sealed phase-typed support subject identifying whether support is being
    reported for declaration, admitted activation, active lifecycle,
    continuation, preview closeout, or explicit deferred/durable scope
- `QuerySubscriptionSupportReport`
  - query-family support posture for one canonical query-owned subscription
    family, including admitted, denied, deferred, and uncertified surfaces for
    one explicit `QuerySubscriptionSupportSubject`
- `QuerySubscriptionDiagnosticTrace`
  - ordered stage-by-stage diagnostic evidence assembled from declaration,
    bridge, admission, lifecycle, continuation, preview, closeout, and support
    artifacts
- `QuerySubscriptionManualBridgeWitness`
  - tangible host-equivalent reconstruction witness describing the exact bridge
    family, bridge slices, basis posture, and admitted signal strategy a careful
    manual host could assemble from canonical query artifacts
- `QuerySubscriptionBridgeParityExplanation`
  - proof-bearing explanation that one query-owned subscription declaration,
    bridge declaration, basis request, admitted signal strategy, and
    `QuerySubscriptionManualBridgeWitness` describe the same long-lived
    semantic request
- `QuerySubscriptionDeniedDiagnosticBundle`
  - canonical offline-readable denied-path bundle tying family-selection,
    declaration, bridge, support, lifecycle-coverage, and failure evidence
    together without admitted lifecycle slots
- `QuerySubscriptionAdmittedDiagnosticBundle`
  - canonical offline-readable admitted-path bundle tying query, family,
    declaration, bridge, signal, support, lifecycle, continuation, preview, and
    failure evidence together
- `QuerySubscriptionRuntimeCertificationScope`
  - scope object freezing which family, support report, bridge parity
    explanation, lifecycle certification bundle, and typed coverage rows belong
    to one certification closure
- `QuerySubscriptionRuntimeCertificationBundle`
  - final 9.3 closure artifact proving one admitted automatic family has
    support/reporting parity, bridge parity, diagnostics sufficiency, and
    hostile certification coverage

Rules:

- no API may construct `QuerySubscriptionSupportReport`,
  `QuerySubscriptionManualBridgeWitness`,
  `QuerySubscriptionBridgeParityExplanation`,
  `QuerySubscriptionDeniedDiagnosticBundle`,
  `QuerySubscriptionAdmittedDiagnosticBundle`, or
  `QuerySubscriptionRuntimeCertificationBundle` from raw strings, host-local
  descriptors, ad hoc JSON, or mutable runtime lookups
- no API may construct a support report without an explicit
  `QuerySubscriptionSupportSubject`
- no API may certify a family unless the support report, bridge parity
  explanation, and runtime-backed lifecycle certification bind to the same
  canonical declaration and query digests
- no API may mark a family as runtime-backed supported when the family lacks at
  least one admitted certification row and one hostile rejection row
- no API may emit an admitted or denied diagnostic bundle that omits source
  digests plus concrete semantic labels for query family, declaration family,
  bridge family, bridge slices, basis posture, signal strategy class, support
  posture, and denial/certification-coverage class
- no API may explain bridge parity through generic "same enough" text; parity
  must bind the exact query family, declaration family, bridge family, bridge
  slices, basis posture, signal strategy digests, and one
  `QuerySubscriptionManualBridgeWitness`
- no API may upgrade durable replay, store-backed parity, or restart claims
  from deferred to supported inside diagnostic assembly

## Compile-Time Enforcement Policy

Milestone 9.3 must classify which diagnostic and certification guarantees
become unrepresentable, uncompilable, or construction-time rejection.

`Unrepresentable` in public types:

- publicly constructible support reports without support posture, source digest,
  support matrix digest, and explicit support subject
- publicly constructible manual bridge witnesses without bridge family, bridge
  slice, basis posture, and signal strategy labels
- publicly constructible bridge parity explanations without query family,
  declaration, bridge declaration, basis, signal strategy digests, and a bound
  manual bridge witness
- publicly constructible admitted or denied diagnostic bundles that omit
  support, bridge parity, failure evidence slots, or concrete semantic labels
- publicly constructible runtime certification bundles that do not bind hostile
  coverage metadata and certification-row coverage digests
- publicly constructible "supported family" markers that are not tied to one
  canonical query family and one certification scope

`Uncompilable` through visibility and compile-fail enforcement:

- external construction of `QuerySubscriptionSupportReport`,
  `QuerySubscriptionSupportSubject`,
  `QuerySubscriptionDiagnosticTrace`,
  `QuerySubscriptionManualBridgeWitness`,
  `QuerySubscriptionBridgeParityExplanation`,
  `QuerySubscriptionRuntimeCertificationScope`,
  `QuerySubscriptionDeniedDiagnosticBundle`,
  `QuerySubscriptionAdmittedDiagnosticBundle`, or
  `QuerySubscriptionRuntimeCertificationBundle`
- public APIs that certify runtime support directly from raw
  `LiveQueryAdmissionArtifact`, raw bridge declaration payloads, raw active lane
  state, or raw delivery batches without the sealed intermediate proof types
- public APIs that expose mutable family-support registries, mutable coverage
  maps, or mutable diagnostic bundle internals
- public APIs that accept booleans such as `supported`, `certified`,
  `bridge_parity`, `offline_safe`, or `host_equivalent` instead of typed
  posture enums
- public APIs that patch support reports or parity explanations after bundle
  assembly
- public APIs that construct admitted-path bundles from denied-path evidence, or
  denied-path bundles from admitted lifecycle bundles with optional holes
- public APIs that merge declaration-family changes into lifecycle-instance
  churn under one generic "subscription changed" surface

`Construction-time rejection`:

- support promotion request that attempts to classify an unsupported or
  uncertified family as `RuntimeBackedCertified`
- support report assembly that overclaims store-backed restart, durable replay,
  or persisted continuation support
- bridge parity explanation assembled from mismatched declaration and bridge
  digests
- manual bridge witness assembled without canonical declaration-family and
  bridge-slice labels
- bridge parity explanation assembled from a lifecycle certification bundle
  whose source digests do not match the declaration family being explained
- admitted bundle assembly without lifecycle certification evidence
- denied bundle assembly that pretends lifecycle evidence exists past the actual
  denial boundary
- diagnostic bundle assembly that omits hostile coverage rows
- certification scope that mixes family rows from different canonical query or
  declaration digests
- certification scope that includes only happy-path rows
- certification scope that lacks required basis, policy, view-shape, and
  lifecycle-class variation for the family under closure
- support-report or bundle assembly that requires registry-wide scanning beyond
  declared coverage width

## Phases

### Phase 1: Family Support Matrix And Capability Reporting

Define one query-owned support surface for automatic subscription families.

Must ship:

- `QuerySubscriptionSupportSubject`
- `QuerySubscriptionSupportClass`
- `QuerySubscriptionSupportPosture`
- `SupportResolutionPosture`
- `QuerySubscriptionSupportReport`
- `QuerySubscriptionSupportMatrix`
- `SupportLookupReceipt`
- `SubscriptionFamilyCapabilityDigest`
- typed support subjects for:
  - declaration support
  - activation support
  - active lifecycle support
  - continuation support
  - preview closeout support
  - deferred durable/store-backed support
- family-aware support rows for detail, inspector, collection, grouped, and
  bounded-materialization subscriptions

Proof obligations:

- support cannot be reported without an explicit phase-typed support subject
- support reports remain query-family-specific rather than bridge-family-only
- runtime-backed support cannot be reported for families lacking admitted
  runtime proof
- deferred store-backed and durable capabilities remain explicit deferred
  surfaces
- support reporting returns `SupportLookupReceipt` and makes any linear-scan
  posture explicit debt or denial
- exact counters for support report request count, supported family count,
  denied family count, deferred family count, uncertified family denial count,
  and support matrix emission count

### Phase 2: Diagnostic Trace And Offline Bundle Assembly

Assemble one canonical diagnostic surface for admitted and denied subscription
paths.

Must ship:

- `QuerySubscriptionDiagnosticStageTrace`
- `QuerySubscriptionDiagnosticTrace`
- `BundleAssemblyPosture`
- `DiagnosticAssemblyReceipt`
- `QuerySubscriptionDeniedDiagnosticBundle`
- `QuerySubscriptionAdmittedDiagnosticBundle`
- bundle evidence for family selection, declaration, bridge lowering,
  admission, lifecycle, continuation, preview, closeout, support, and failure
  surfaces
- offline bundle serialization contract and digest vocabulary
- minimum offline-readable semantic labels for:
  - query family
  - declaration family
  - bridge family
  - bridge slices
  - basis posture
  - signal strategy class
  - support posture
  - denial or certification coverage class

Proof obligations:

- admitted bundles contain enough evidence for offline explanation without
  re-running Query
- denied bundles localize the failing stage mechanically
- admitted and denied bundles are distinct proof types, not one optional-hole
  struct
- bundle assembly returns `DiagnosticAssemblyReceipt` and records whether any
  semantic labels were carried forward or re-derived
- declaration-family drift remains distinct from lifecycle-instance churn
- diagnostic richness may widen retained detail but may not change support or
  semantic digests
- exact counters for diagnostic trace emission, admitted bundle emission,
  denied bundle emission, omitted-stage denial, and bundle width

### Phase 3: Bridge Parity Explanation And Lowering Honesty

Explain automatic family selection as one bridge-honest lowering chain.

Must ship:

- `QuerySubscriptionManualBridgeWitness`
- `BridgeWitnessAssemblyPosture`
- `BridgeParityReceipt`
- `QuerySubscriptionBridgeParityExplanation`
- `QuerySubscriptionBridgeParityClass`
- `QuerySubscriptionBridgeParityFailure`
- `QuerySubscriptionBridgeParityComparison`
- parity explanation evidence binding declaration family, bridge family, bridge
  slices, basis request posture, signal strategy, and manual-host witness
- bridge-parity comparison report for admitted and rejected paths

Proof obligations:

- every admitted automatic family has one bridge-facing parity explanation
- every bridge-facing parity explanation carries a tangible manual-host witness
  reconstructable from canonical query-owned artifacts
- parity returns `BridgeParityReceipt` and may not semantically rediscover
  bridge meaning after witness construction
- grouped and inspector families remain query-side distinct even if they share
  bridge family classes
- bridge parity explanation fails typed on mismatched declaration, bridge,
  basis, or signal digests
- exact counters for bridge parity comparison count, bridge parity admitted
  count, bridge parity denial count, and family-distinction preservation count

### Phase 4: Runtime Certification Scope And Family Coverage Closure

Close the family-aware certification surface instead of certifying one generic
subscription lane story.

Must ship:

- `QuerySubscriptionRuntimeCertificationScope`
- `QuerySubscriptionRuntimeCertificationBundle`
- `QuerySubscriptionFamilyCoverageMatrix`
- `CoverageResolutionPosture`
- `CertificationCoverageReceipt`
- `CertifiedFamilyCoverageHandle`
- typed family coverage sets for:
  - basis variation
  - policy variation
  - tenant variation
  - relationship-proof variation
  - view-shape variation
  - lifecycle-class variation
- certification coverage records for declaration, lifecycle, sharing,
  continuation, preview, and bridge parity
- hostile-row coverage metadata
- family-coverage digest proving which admitted families are closed

Proof obligations:

- every supported runtime-backed family has at least one admitted row and one
  hostile row
- every supported runtime-backed family also covers required basis, policy,
  tenant, proof, view-shape, and lifecycle-class variation rows
- runtime family certification consumes `CertifiedFamilyCoverageHandle` or
  explicit matrix-scan debt/denial posture rather than raw row iteration
- lifecycle certification stays source-aligned with support reports and bridge
  parity explanations
- unsupported or uncertified families deny before they appear in a supported
  runtime certification bundle
- exact counters for certified family count, hostile-row coverage count,
  uncovered-family denial count, and certification scope emission count

### Phase 5: Harness, Compile-Fail, And Facade Closure

Close the public proof surface for milestone 9.3.

Must ship:

- milestone 9.3 certification harness rows
- compile-fail boundaries for support-report, parity-explanation, bundle, and
  certification-bundle constructors
- facade exposure for support reporting, diagnostic bundle inspection,
  bridge-parity explanation, and runtime-backed family certification
- trybuild targets for:
  - `subscription_support_report_constructor_private.rs`
  - `subscription_bridge_parity_explanation_constructor_private.rs`
  - `subscription_runtime_certification_scope_constructor_private.rs`
  - `subscription_diagnostic_bundle_constructor_private.rs`
  - `subscription_runtime_certification_bundle_constructor_private.rs`
  - `subscription_support_report_durable_overclaim_forbidden.rs`
  - `subscription_bridge_parity_mismatched_declaration_forbidden.rs`
  - `subscription_bridge_parity_mismatched_signal_strategy_forbidden.rs`
  - `subscription_diagnostic_bundle_missing_hostile_coverage_forbidden.rs`
  - `subscription_runtime_certification_uncertified_family_forbidden.rs`

Proof obligations:

- the named 9.3 certification suite passes with canonical bundles
- required output digests are emitted for admitted and denied automatic family
  paths
- compile-fail boundaries prove external callers cannot mint support, parity,
  bundle, or certification artifacts by hand

## Public Facade And Typestate API Shape

Required facade shape, subject to local naming adjustment:

```rust
pub fn report_query_subscription_support(
    subject: QuerySubscriptionSupportSubject,
    declaration: &QuerySubscriptionDeclarationArtifact,
    admission: &QuerySubscriptionAdmissionArtifact,
) -> (QuerySubscriptionSupportReport, SupportLookupReceipt);

pub fn explain_query_subscription_bridge_parity(
    declaration: &QuerySubscriptionDeclarationArtifact,
    lowering: &BridgeSubscriptionLoweringPlan,
    activation: &SubscriptionActivationInput,
    witness: QuerySubscriptionManualBridgeWitness,
) -> Result<
    (QuerySubscriptionBridgeParityExplanation, BridgeParityReceipt),
    QuerySubscriptionBridgeParityError,
>;

pub fn bundle_admitted_query_subscription_diagnostics(
    support: QuerySubscriptionSupportReport,
    parity: QuerySubscriptionBridgeParityExplanation,
    lifecycle: SubscriptionLifecycleCertificationBundle,
) -> Result<
    (
        QuerySubscriptionAdmittedDiagnosticBundle,
        DiagnosticAssemblyReceipt,
    ),
    QuerySubscriptionDiagnosticBundleError,
>;

pub fn bundle_denied_query_subscription_diagnostics(
    trace: QuerySubscriptionDiagnosticTrace,
    support: QuerySubscriptionSupportReport,
    failure: QuerySubscriptionDiagnosticFailure,
) -> Result<
    (
        QuerySubscriptionDeniedDiagnosticBundle,
        DiagnosticAssemblyReceipt,
    ),
    QuerySubscriptionDiagnosticBundleError,
>;

pub fn certify_query_subscription_runtime_family(
    scope: QuerySubscriptionRuntimeCertificationScope,
) -> Result<
    (
        QuerySubscriptionRuntimeCertificationBundle,
        CertificationCoverageReceipt,
    ),
    QuerySubscriptionRuntimeCertificationError,
>;
```

Rules:

- `report_query_subscription_support` is pure reporting from canonical
  declaration/admission artifacts plus one explicit support subject; it may not
  perform host-side activation or mutate runtime state
- `report_query_subscription_support` must make lookup cost explicit through
  `SupportLookupReceipt`; support posture is not allowed to conceal lookup
  posture
- `explain_query_subscription_bridge_parity` must compare already-authoritative
  declaration, bridge, and signal evidence against one manual bridge witness;
  it may not reinterpret query meaning or recover bridge semantics from strings
- parity explanation must surface `BridgeParityReceipt`; comparison cost is not
  allowed to disappear into the explanation artifact
- admitted and denied bundle builders must be distinct public surfaces; they may
  not collapse into one optional-hole helper
- admitted bundle assembly must consume proof-bearing support, parity, and
  lifecycle artifacts and return one offline-readable bundle; it may not accept
  raw digests or optional "best effort" omissions
- denied bundle assembly must consume a denied diagnostic trace plus support and
  failure evidence and may not pretend admitted lifecycle evidence exists
- both bundle builders must return `DiagnosticAssemblyReceipt`; bundle assembly
  cost is not allowed to disappear into the bundle artifact
- `certify_query_subscription_runtime_family` is the only public path that
  closes 9.3 runtime-backed family certification; it must reject uncovered or
  mismatched family scopes
- runtime family certification must return `CertificationCoverageReceipt` and
  must consume indexed family coverage or explicit debt/denial posture
- diagnostics helpers may wrap these functions, but may not expose weaker
  inputs or alternate construction paths

## Representative Scenario Matrix

Minimum canonical rows:

- `detail-family-support-and-parity`
- `inspector-family-support-and-parity`
- `ordered-collection-family-support-and-parity`
- `grouped-collection-family-support-and-parity`
- `bounded-materialization-family-support-and-parity`
- `detail-family-offline-diagnostic-bundle`
- `grouped-family-hostile-bridge-parity-denial`
- `preview-family-lifecycle-certification-bundle`
- `continuation-family-support-sync`
- `family-coverage-certification-closure`
- `declaration-family-drift-vs-lifecycle-churn-distinctness`
- `basis-policy-viewshape-family-coverage-closure`
- `support-matrix-scale-honesty`

Minimum rejection rows:

- `uncertified-family-support-overclaim-forbidden`
- `store-backed-restart-support-overclaim-forbidden`
- `durable-replay-support-overclaim-forbidden`
- `bridge-parity-declaration-source-mismatch`
- `bridge-parity-signal-strategy-source-mismatch`
- `diagnostic-bundle-missing-hostile-row-forbidden`
- `runtime-certification-cross-family-row-mix-forbidden`
- `generic-family-certification-shortcut-forbidden`

Every representative row must identify:

- query digest
- subscription family digest
- declaration digest
- bridge declaration digest
- signal strategy digest
- support report digest
- bridge parity digest
- lifecycle certification digest where admitted
- diagnostic bundle digest
- runtime certification bundle digest where admitted
- failure digest where rejected

## Proposed Module Topology

Milestone 9.3 must extend `crates/worth-query/src/subscription/` through
separate responsibilities rather than one diagnostics bag.

Required or expected subdomains:

- `support.rs`
  - family support classes, postures, reports, and support matrix assembly
- `diagnostic.rs`
  - diagnostic stages, traces, and bundle evidence vocabulary
- `bridge_parity.rs`
  - bridge parity explanation, parity classes, and parity denials
- `runtime_certification.rs`
  - 9.3 certification scope and final runtime family bundle
- `certification.rs`
  - shared certification helpers reused without merging 9.3-specific closure
    into 9.2 lifecycle semantics
- `tests/diagnostics.rs`
  - stage-localization, family distinction, and bundle coverage tests
- `tests/certification.rs`
  - family coverage, support alignment, and hostile closure tests
- `tests/support.rs`
  - support posture and deferred/unsupported matrix tests

Suggested harness layout:

- `crates/worth-query/src/harness/milestone_nine_three_certification/`
  - `mod.rs`
  - `builders.rs`
  - `rows.rs`
  - `tests.rs`

The topology must preserve:

- support reporting independent from bundle serialization
- support subject typing independent from support posture resolution
- bridge parity explanation independent from runtime certification closure
- manual bridge witness construction independent from parity comparison
- family coverage independent from lifecycle artifact construction
- harness row assembly independent from shipped facade types

## Store Dependency

- Runtime-backed support reporting, diagnostic bundling, bridge parity
  explanation, and family certification are not blocked on `worth-store`.
- Store-backed execution parity remains Milestone 10 scope.
- Durable subscription replay, restart-stable continuation, and persisted
  diagnostic replay remain Milestone 11 scope.

## Explicit Assumptions And Deferred Decisions

- 9.3 assumes the 9.1 declaration and 9.2 lifecycle digests remain canonical
  and stable enough to bind into support and diagnostic bundles.
- 9.3 assumes bridge family and slice semantics remain owned by
  `worth-runtime-bridge`; Query only certifies parity against those surfaces.
- Offline-readable bundles may be digest-centric rather than carrying every raw
  field value, as long as they remain sufficient to reconstruct the semantic
  explanation without hidden runtime queries.
- "Sufficient" here means the bundle includes concrete semantic labels for the
  selected query family, declaration family, bridge family, bridge slices,
  basis posture, signal strategy class, support posture, and denial or
  certification coverage class in addition to digests.
- Persisted diagnostic archives, cross-process replay of bundles, and
  store-backed re-certification may remain deferred.
- If later milestones widen supported subscription families, they must also
  widen support matrices, diagnostic rows, and hostile certification coverage
  in lockstep rather than treating 9.3 artifacts as optional metadata.

## Explicit Failure Taxonomy

Milestone 9.3 must name and preserve at least these failure classes:

- unsupported query subscription family
- uncertified runtime-backed family support claim
- durable replay overclaim
- store-backed restart overclaim
- bridge parity declaration mismatch
- bridge parity bridge-family mismatch
- bridge parity basis mismatch
- bridge parity signal-strategy mismatch
- support-report and lifecycle-certification mismatch
- diagnostic bundle missing stage evidence
- diagnostic bundle missing hostile coverage
- runtime certification scope family mixing
- runtime certification uncovered-family denial
- declaration-family drift collapsed into lifecycle-instance churn

## Anti-Patterns Explicitly Rejected

- one generic "subscription supported" flag for every query family
- diagnostic bundles built from free-form strings or host-side JSON assembly
- certification that proves only one active lifecycle happy path and then
  claims all families are supported
- bridge parity reduced to "same bridge family" while ignoring query-family,
  slice, basis, or signal differences
- support metadata that can drift from lifecycle certification rows
- offline explanation that requires calling back into a live runtime to
  interpret the bundle
- durable or store-backed support claims piggybacked onto runtime-backed
  diagnostics as speculative future support

## Sequencing Notes

Milestone 9.3 belongs immediately after Milestone 9.2 because bridge parity and
family-aware diagnostics are only honest once runtime-backed lifecycle,
continuation, preview, and closeout evidence exist.

It belongs before Milestone 10 because store-backed execution parity must build
on a surface where runtime-backed support and automatic family selection are
already explainable and certified.

It belongs before Milestone 11 because durable replay and restart-stable
subscription metadata cannot be certified honestly until runtime-backed family
support closure exists.

## Parallelization Notes

- support matrix and capability-report work can proceed in parallel with
  bridge-parity explanation work
- offline bundle assembly can proceed in parallel with harness-row scaffolding
- final closure should wait until the same family matrix is covered by support
  reporting, diagnostic bundles, bridge parity rows, and hostile certification

## Performance Encoding Lock

Milestone 9.3 must encode diagnostic and certification cost structurally.

Required cost-bearing types:

- `SubscriptionSupportCoverageWidth`
  - counts family rows and capability entries considered during support report
    assembly
- `SupportLookupReceipt`
  - records the exact family-support lookup path, consumed lookup width,
    remaining lookup width, and selected support-resolution posture for one
    `QuerySubscriptionSupportSubject`
- `SubscriptionDiagnosticBundleWidth`
  - counts stage evidence entries, failure evidence entries, and hostile-row
    references embedded in one bundle
- `DiagnosticAssemblyReceipt`
  - records stage-evidence composition count, semantic-label carry-forward
    count, bundle width, and whether any semantic re-derivation occurred during
    bundle assembly
- `SubscriptionBridgeParityWidth`
  - counts compared family, slice, basis, and signal dimensions
- `BridgeParityReceipt`
  - records witness comparison width, parity comparison class, and whether
    parity consumed only pre-lowered artifacts or attempted semantic rebuild
- `SubscriptionCertificationCoverageWidth`
  - counts admitted and hostile row coverage for one family certification scope
- `CertificationCoverageReceipt`
  - records family coverage index lookups, covered-row width, uncovered
    variation width, and the selected coverage-resolution posture
- `CertifiedFamilyCoverageHandle`
  - sealed coverage handle proving family rows were already indexed and grouped
    by family before runtime family certification consumes them

Required posture enums:

- `QuerySubscriptionSupportPosture`
  - `RuntimeBackedCertified`
  - `RuntimeBackedDenied`
  - `RuntimeBackedDeferred`
  - `UncertifiedDenied`
- `SupportResolutionPosture`
  - `IndexedFamilyLookup`
  - `PrecomputedFamilyMatrix`
  - `LinearScanDebtExplicit`
  - `LinearScanDenied`
- `QuerySubscriptionBridgeParityClass`
  - `ExactParity`
  - `FamilyDistinctBridgeShared`
  - `DeniedSourceMismatch`
  - `DeniedUnsupported`
- `BridgeWitnessAssemblyPosture`
  - `PreLoweredWitness`
  - `CanonicalComposition`
  - `SemanticRediscoveryDebtExplicit`
  - `SemanticRediscoveryDenied`
- `QuerySubscriptionDiagnosticBundlePosture`
  - `AdmittedBundle`
  - `DeniedBundle`
  - `CoverageIncompleteDenied`
- `BundleAssemblyPosture`
  - `ComposedFromCanonicalArtifacts`
  - `PartialRediscoveryDebtExplicit`
  - `PartialRediscoveryDenied`
- `CoverageResolutionPosture`
  - `IndexedCoverageSet`
  - `PrecomputedCoverageMatrix`
  - `MatrixScanDebtExplicit`
  - `MatrixScanDenied`

Rules:

- public hot-path-adjacent reporting surfaces may not hide family-coverage,
  bundle-width, or parity-comparison breadth behind opaque helpers
- support reporting must return `SupportLookupReceipt`; lookup cost may not be
  merged invisibly into support posture
- support reporting may index families directly, but any broader scan posture
  must be explicit debt or denial in counters, receipts, and certification rows
- diagnostic bundle assembly must derive stage evidence once and pass it
  forward; later phases may not repeatedly rediscover declaration, bridge, or
  lifecycle facts
- manual bridge witnesses must be pre-lowered or canonically composed from
  authoritative artifacts before parity comparison begins; parity comparison may
  not perform semantic rediscovery of bridge slices, basis, or signal posture
- bundle assembly must return `DiagnosticAssemblyReceipt` and record whether any
  semantic labels were carried forward versus rebuilt
- runtime family certification must consume a `CertifiedFamilyCoverageHandle` or
  explicit `MatrixScanDebtExplicit` posture; raw row iteration is not an
  invisible implementation choice
- certification coverage must be family-scoped and width-bounded; it may not
  silently iterate every historical row in the harness to answer one family
  question
- repeated support, parity, and bundle requests for the same canonical family
  must have an explicit reuse basis through stable family-scoped handles or
  digests; heuristic reuse is forbidden

## Complexity / Proof Obligations

Named contracts:

- `QuerySubscriptionSupportReportContract`
  - bounded by one declaration family, one admission artifact, one family
    support matrix row, one declared `SubscriptionSupportCoverageWidth`, and one
    returned `SupportLookupReceipt`
- `SupportLookupContract`
  - bounded by one support subject, one family capability key, one support
    matrix row or one indexed family lookup, and one declared
    `SupportResolutionPosture`
- `QuerySubscriptionDiagnosticBundleContract`
  - bounded by one support report, one parity explanation, one lifecycle
    certification bundle, one hostile coverage set, and one declared
    `SubscriptionDiagnosticBundleWidth`, with one returned
    `DiagnosticAssemblyReceipt`
- `DiagnosticAssemblyContract`
  - bounded by one stage trace, one support report, one parity explanation,
    one admitted or denied bundle posture, and one declared
    `BundleAssemblyPosture`
- `QuerySubscriptionBridgeParityContract`
  - bounded by one declaration family, one bridge declaration family/slice
    mapping, one basis posture, one signal strategy, one
    `QuerySubscriptionManualBridgeWitness`, and one declared
    `SubscriptionBridgeParityWidth`
- `BridgeWitnessContract`
  - bounded by one canonical declaration/lowering source and one declared
    `BridgeWitnessAssemblyPosture`; witness assembly may compose but may not
    semantically rediscover bridge meaning from strings or live runtime lookups
- `QuerySubscriptionRuntimeCertificationCoverageContract`
  - bounded by one family support report, one parity explanation, one family
    certification scope, one `CertifiedFamilyCoverageHandle`, and one
    hostile/admitted row set counted by `SubscriptionCertificationCoverageWidth`
- `CertificationCoverageLookupContract`
  - bounded by one family coverage key, one indexed coverage handle or one
    explicit matrix-scan debt posture, and one declared
    `CoverageResolutionPosture`
- `SubscriptionSupportScaleSlopeContract`
  - proves family support lookup and bundle/certification assembly scale with
    declared family coverage width rather than unrelated total runtime rows

Required counters:

- `subscription_support_report_request_count`
- `subscription_support_family_index_lookup_count`
- `subscription_supported_family_count`
- `subscription_denied_family_count`
- `subscription_deferred_family_count`
- `subscription_uncertified_family_denial_count`
- `subscription_support_matrix_emission_count`
- `subscription_support_matrix_scan_debt_count`
- `subscription_bridge_parity_comparison_count`
- `subscription_bridge_parity_admitted_count`
- `subscription_bridge_parity_denial_count`
- `subscription_family_distinction_preserved_count`
- `subscription_manual_bridge_witness_build_count`
- `subscription_manual_bridge_witness_rebuild_count`
- `subscription_diagnostic_trace_emission_count`
- `subscription_diagnostic_bundle_emission_count`
- `subscription_denied_bundle_emission_count`
- `subscription_diagnostic_missing_stage_denial_count`
- `subscription_diagnostic_missing_hostile_coverage_denial_count`
- `subscription_diagnostic_bundle_width`
- `subscription_bundle_stage_rederivation_count`
- `subscription_certified_family_count`
- `subscription_hostile_row_coverage_count`
- `subscription_uncovered_family_denial_count`
- `subscription_certification_scope_emission_count`
- `subscription_family_coverage_index_lookup_count`
- `subscription_family_coverage_matrix_scan_debt_count`
- `subscription_support_scale_fixture_row_count`
- `subscription_support_scale_slope_digest_part_count`

Counter rules:

- exact counter assertions are required; elapsed-time thresholds do not satisfy
  9.3
- a supported family row must increment certified-family coverage counters only
  after support, parity, and lifecycle evidence all agree
- grouped/inspector rows that share bridge families must still increment
  family-distinction counters separately
- indexed rows must assert family index lookup counts explicitly and assert
  matrix-scan debt counts stay at zero
- manual bridge witness rows must prove witness rebuild counts stay at zero once
  a pre-lowered witness exists; parity is a comparison boundary, not a
  semantic reconstruction boundary
- bundle rows must prove stage re-derivation count stays at zero under
  `ComposedFromCanonicalArtifacts`
- missing hostile coverage must deny before runtime certification bundle
  emission
- scale-slope rows must prove support and bundle assembly do not grow with
  unrelated fixture row count when the family coverage width is unchanged

## Acceptance Evidence

Milestone 9.3 is complete only when `worth-query` can prove:

- the `Query Subscription Bridge Parity And Diagnostic Sufficiency Test` in
  [test-requirements.md](./test-requirements.md) passes with canonical
  machine-checkable artifacts
- every admitted automatic subscription family can be explained through
  canonical query-owned subscription artifacts and bridge-facing lowering
- support metadata and admitted runtime behavior stay in sync for
  subscription-capable query families
- diagnostics can localize declaration, basis, lifecycle, continuation,
  preview, bridge-parity, and certification-coverage failures mechanically
- diagnostics distinguish declaration-family changes from lifecycle-instance
  churn
- unsupported or uncertified families fail typed and early rather than
  degrading into generic support claims

Required verification output must include:

- `query_digest`
- `subscription_family_digest`
- `subscription_declaration_digest`
- `subscription_equivalence_digest`
- `bridge_declaration_digest`
- `bridge_basis_digest`
- `signal_strategy_digest`
- `support_report_digest`
- `support_matrix_digest`
- `bridge_parity_digest`
- `diagnostic_trace_digest`
- `diagnostic_bundle_digest`
- `lifecycle_certification_digest`
- `runtime_certification_bundle_digest`
- `continuation_digest`
- `preview_isolation_digest`
- `failure_digest`
- `counter_snapshot`
- `subscription_support_scale_slope_digest`
- `compile_fail_boundary_digest`

## Self-Check

This milestone solves a real structural problem rather than packaging work
cosmetically because it closes the missing explanation and trust boundary
between automatic family selection and future store-backed subscription work.

The adversarial constraint is load-bearing because it forbids the easy lie
where automatic subscriptions "work" but cannot be explained through canonical
query, bridge, signal, and lifecycle artifacts.

The milestone preserves crate authority boundaries because `worth-query` owns
support reporting, diagnostic bundles, and runtime family certification, while
bridge, signal, and relational remain authorities for their respective
semantics.

The milestone defines proof obligations rather than implementation chores
because family-aware support, bridge parity, offline bundles, hostile coverage,
compile-fail boundaries, and exact counters are all required closure artifacts.

A competent engineer should be able to map this spec into honest `support`,
`diagnostic`, `bridge_parity`, `runtime_certification`, harness, and facade
subdomains without inventing hidden semantics during implementation.

This milestone belongs at 9.3 because 9.1 and 9.2 already closed declaration
and active lifecycle, while 10 and 11 need an explainable certified
runtime-backed family surface before they can extend it honestly.

## Closeout Standard

Milestone 9.3 is closed only when:

- the named 9.3 certification suite exists and passes
- every admitted automatic subscription family has support reporting, bridge
  parity explanation, admitted certification coverage, and hostile rejection
  coverage
- shipped facade surfaces expose proof-bearing reports and bundles rather than
  weaker debug strings
- compile-fail boundaries prove external callers cannot mint support, parity,
  bundle, or certification artifacts directly
- store-backed and durable subscription claims remain explicit later-milestone
  debt rather than silent implication
