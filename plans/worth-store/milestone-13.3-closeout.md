# Milestone 13.3 Closeout: Subscription Support Accuracy Taxonomy And Certification

## Status

Closed for Milestone 13.3 scope.

Milestone 13.3 closes the subscription-support cleanup arc by making support
trust explicit, role-scoped, evidence-bound, and consumable by generic/domain
certification without reinterpreting raw operational rows. It does not claim
physical database readiness, durable certification-run persistence, or future
extension-family support registration.

Parent spec:
[milestone-13.3.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-13.3.md)

## What Shipped

- Dedicated trust subdomain under
  [crates/worth-store/src/subscription_support/trust](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/subscription_support/trust).
- Split trust vocabulary for class, strength, provenance, use boundary,
  downgrade reason, failure kind, and recovery posture.
- Proof-widening trust pipeline from raw trust request through admission,
  translation, drift, equivalence, operational classification, certification
  coverage, and certified trust classification.
- Concrete receipt families for resume classification, operational verdict,
  family role, basis, cursor/checkpoint, compatibility, portability,
  maintenance, retention, and import admission evidence.
- Sealed exact, degraded, rebuild-derived, rejected, operational, equivalence,
  and certified trust witnesses.
- Explicit equivalence contracts for rebuild, replication, migration, and
  import lanes.
- Drift reports with deterministic primary cause and suppressed causes.
- Certification coverage plans, rows, matrices, batch scopes, counter
  snapshots, evidence bundles, and coverage witnesses.
- Generic certification report and domain certification bundle that consume
  certified support trust instead of raw operational reports.
- Roadmap handoff report that closes semantic support trust while keeping
  Roadmap 2 physical database readiness explicit.
- Named closeout suite:
  `Subscription-Support Accuracy And Certification Test`.
- Production-facing
  `SubscriptionSupportAccuracyCertificationRunner` that emits the named suite,
  performance closeout, access/debt closeout, persistence posture, and a run
  digest.

## Machine-Checkable Closeout Surface

The official closeout proof surface is
`SubscriptionSupportAccuracyCertificationRun`, emitted by
`SubscriptionSupportAccuracyCertificationRunner::production().certify(...)`.

The run contains:

- `SubscriptionSupportAccuracyCertificationSuite`
- `SubscriptionSupportAccuracyPerformanceCloseout`
- `SubscriptionSupportAccuracyAccessCloseout`
- `SubscriptionSupportAccuracyPersistencePosture`
- `run_digest`

The suite emits the required machine-checkable outputs:

- `artifact_digest`
- `subscription_support_digest`
- `diagnostics_digest`
- `counter_snapshot_digest`
- `certification_summary_digest`

## Trace Matrix

| Spec Obligation | Production Surface | Proof Surface |
| --- | --- | --- |
| Role-scoped trust taxonomy | `SupportTrustClass`, `SupportTrustStrength`, `SupportTrustProvenance`, `SupportTrustUseBoundary`, `SubscriptionSupportTrustClass` | taxonomy and exact/degraded/rebuild tests in `subscription_support::trust::tests` |
| Operational and certified trust are distinct | `OperationalSupportTrustReport`, `CertifiedSupportTrustReport`, `SupportTrustCertificationStamp` | `uncertified_posture_preserves_operational_boundary`, certification stamp tests |
| Exact trust requires proof-bearing resume and operational inputs | `RawSupportTrustRequest`, `SupportTrustRequestAdmitted`, `SupportTrustTranslatedInputs`, sealed witnesses | Phase 2 pipeline tests and exact witness compile-fail fixtures |
| Degraded, rebuild, stale, policy, unsupported, and role-mismatched support cannot become exact | trust classification reports and failure taxonomy | degraded-as-exact, rebuild, stale, policy, role mismatch, and hostile Phase 7 lanes |
| Rebuild, replication, migration, and import require explicit equivalence | `SupportTrustEquivalenceContract`, lane-specific equivalence witnesses | Phase 3 equivalence tests and Phase 7 transformed-support lanes |
| Drift localizes to typed failures | `SupportTrustDriftReport`, `SupportTrustFailureKind`, `SupportTrustSuppressedCause` | Phase 4 drift tests and multi-drift deterministic lane |
| Required access structures reject instead of scanning globally | `SupportTrustAccessStructurePlan`, `SupportTrustPerformancePlan`, access debt failures | access-structure tests, global-scan rejection tests, `GlobalScanDebtForbidden` lane |
| Lowered performance plans carry density, access path, allocation, and expected counters | `SupportTrustPerformancePlan`, `SupportCertificationBatchScope`, domain batch plan | performance-plan tests, certification/domain counter drift tests |
| Certification coverage proves posture, not artifact existence | `SubscriptionSupportCertificationCoveragePlan`, `SupportCertificationCoverageMatrix`, `SupportCertificationEvidenceBundle` | Phase 5 coverage, gap, row digest, mislabeled, duplicate, and self-comparison tests |
| Generic certification consumes certified support trust | `SupportGenericCertificationReport` | `phase6_generic_certification_consumes_certified_support_trust` |
| Domain certification consumes first-ship support scenarios | `SupportDomainCertificationBundle`, `SupportDomainCertificationRow` | Phase 6 domain scenario tests and Phase 7 domain suite rows |
| Advanced family absence is explicit debt | `SupportDomainCertificationDebtReason`, `SupportDomainCertificationDebtOwner` | domain explicit-debt tests and access closeout counters |
| Roadmap 2 handoff separates semantic trust from physical readiness | `SupportCertificationHandoffReport`, `SupportRoadmapPhysicalReadinessPosture` | handoff tests and runner handoff-binding rejection |
| Named suite rows are artifact-bound | `SubscriptionSupportAccuracyCertificationSuite`, row evidence digests | Phase 7 required-output, missing, duplicate, tampered, and hostile-lane tests |
| Hostile lane evidence comes from production failures | `SubscriptionSupportAccuracyLaneEvidence::typed_rejection_from_failure` | Phase 7 rejection lane tests and private constructor trybuild |
| Pass lane evidence comes from certified reports or zero-debt bundle counters | `certified_pass_from_report`, `certified_counter_pass_from_evidence_bundle` | Phase 7 pass-lane posture and counter-lane tests |
| Production closeout runner cannot be forged externally | `SubscriptionSupportAccuracyCertificationRunner`, `SubscriptionSupportAccuracyCertificationRun` | `subscription_support_accuracy_runner_fields_private.rs`, `subscription_support_accuracy_run_fields_private.rs` |
| Public API exposes the protocol without raw construction bypass | facade exports from `worth_store` and private fields/constructors | phase-boundary compile-fail suite |

## Named Suite Rows

The named suite covers the full Milestone 13.3 acceptance matrix:

- `ExactSupportTrustedControl`
- `DegradedSupportTrusted`
- `RebuildDerivedSupportExactEquivalence`
- `RebuildDerivedSupportDowngraded`
- `ReplicatedSupportIdentityNotEnough`
- `ReplicatedSupportExactEquivalence`
- `MigratedSupportExactEquivalence`
- `ImportedSupportMissingBasisNotResumable`
- `StaleSupportRejected`
- `PolicyRejectedSupport`
- `FamilyRoleMismatchRejected`
- `CompatibilityDriftRejectsExactTrust`
- `OperationalVerdictDriftRejectsExactTrust`
- `PortabilityDriftRejectsExactTrust`
- `CoverageDriftRejectsPlatformTrust`
- `MultiDriftPrecedenceDeterministic`
- `CertificationMatrixComplete`
- `CertificationMissingRowRejected`
- `CertificationDuplicateRowRejected`
- `CertificationMislabeledRowRejected`
- `CertificationSelfComparisonRejected`
- `GenericCertificationIncludesSupportTrust`
- `DomainGeometrySupportTrust`
- `DomainWebDataSupportTrust`
- `DomainAiDegradedSupportTrust`
- `DomainChipRebuildSupportTrust`
- `DomainOfflineOmittedSupportTrust`
- `ForbiddenExactOverclaimZero`
- `GlobalScanDebtForbidden`
- `Roadmap2HandoffPhysicalDebtExplicit`

The suite rejects missing required rows, duplicate rows, tampered artifact-row
evidence, misclassified hostile lane evidence, non-zero exact-overclaim
counters, and non-zero global-scan debt counters.

## Counter Regime

The production closeout runner enforces the exact first-ship counter regime
instead of accepting merely non-zero or internally consistent counters.

| Surface | Required Counters |
| --- | --- |
| Certification coverage | `row_count = 4`, `index_probes = 4`, `receipt_reuse = 3`, `allocation_count = 1`, `forbidden_exact_overclaim = 0`, `global_scan_debt = 0` |
| Generic certification | `certified_support_report_count = 1`, `generic_row_count = 1`, `index_probes = 1`, `receipt_reuse = 1`, `allocation_count = 1`, `physical_readiness_debt_count = 1` |
| Domain certification | `scenario_row_count = 5`, `certified_semantic_rows = 3`, `explicit_debt_rows = 2`, `index_probes = 5`, `receipt_reuse = 4`, `allocation_count = 1`, `physical_readiness_debt_count = 2` |
| Accuracy suite | `required_row_count = 30`, `certified_row_count = 30`, `forbidden_exact_overclaim = 0`, `global_scan_debt = 0` |

The hostile QA pass added regressions proving that valid bundles with drifted
certification or domain counter regimes are rejected by the runner rather than
blessed by the closeout artifact.

## Compile-Time Enforcement

The phase-boundary trybuild suite proves external callers cannot synthesize the
core proof objects. Milestone 13.3 added or relies on compile-fail coverage for:

- synthetic exact trust witness construction
- degraded or rebuild-derived trust used as exact resume trust
- raw row certification and incomplete bundle construction
- cross-family equivalence witness reuse
- subscription-support accuracy row construction
- subscription-support accuracy lane evidence field construction
- free-form rejection lane construction
- synthetic pass lane construction
- production runner field construction
- final certification run field construction

## Explicit Non-Claims

Milestone 13.3 intentionally does not claim:

- physical database readiness or platform-grade backend boundedness
- durable persistence/reopen of the `SubscriptionSupportAccuracyCertificationRun`
  itself
- final extension-defined support registration
- completed omitted-import support semantics
- Roadmap 2 physical storage substrate closure

The closeout posture is therefore:

- `SubscriptionSupportAccuracyPersistencePosture::InMemoryCertificationOnly`
- `SupportRoadmapPhysicalReadinessPosture::PhysicalDatabaseReadinessDeferredToRoadmap2`
- `SupportDomainCertificationDebtOwner::Roadmap2PhysicalDatabaseFoundation`
  for chip/simulation long-history rebuild physical readiness
- `SupportDomainCertificationDebtOwner::Milestone15ExtensionSupportRegistration`
  for offline/collaborative omitted extension support

These are named future-owned boundaries, not hidden Milestone 13.3 failures.

## Verification

Final verification commands:

```text
cargo fmt --all
cargo test -p worth-store subscription_support::trust::tests::phase7_ -- --nocapture
cargo test -p worth-store --test phase_boundaries_compile_fail -- --nocapture
cargo test -p worth-store
git diff --check -- crates/worth-store/src/lib.rs crates/worth-store/src/subscription_support/mod.rs crates/worth-store/src/subscription_support/trust/certification.rs crates/worth-store/src/subscription_support/trust/domain_certification.rs crates/worth-store/src/subscription_support/trust/mod.rs crates/worth-store/src/subscription_support/trust/named_suite.rs crates/worth-store/tests/ui/subscription_support_accuracy_run_fields_private.rs crates/worth-store/tests/ui/subscription_support_accuracy_run_fields_private.stderr crates/worth-store/tests/ui/subscription_support_accuracy_runner_fields_private.rs crates/worth-store/tests/ui/subscription_support_accuracy_runner_fields_private.stderr
```

Final observed results:

- focused Phase 7 trust suite: 16 tests passed
- phase-boundary compile-fail suite: passed
- full `worth-store` suite: 640 tests passed plus trybuild/doc-tests
- diff hygiene: no whitespace errors; only line-ending warnings from existing
  workspace settings

## Operational Conclusion

Milestone 13.3 is closed for semantic subscription-support trust accuracy and
certification.

The store now has one enforced support trust vocabulary, certified first-ship
family coverage, artifact-bound named suite evidence, exact performance
counters for the closeout regime, and explicit future-owned debt where later
roadmap layers still own physical or extension-family completion.
