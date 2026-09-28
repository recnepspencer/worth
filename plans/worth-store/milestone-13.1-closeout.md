# Milestone 13.1 Closeout: Durable Subscription-Support Artifacts And Resume Contracts

## Status

Closed for Milestone 13.1 scope.

Milestone 13.1 establishes durable, family-aware subscription-support artifacts
and typed resume classification inside `worth-store`. It intentionally does not
make the store a subscription manager, delivery runtime, retention participant,
replication participant, or extension-family registry. Those are reserved for
Milestones 13.2, 13.3, 14, and 15.

## What Shipped

- `subscription_support` domain with separated identity, declaration, catalog,
  witnesses, artifacts, records, persistence, classification, handles, restart,
  and evidence responsibilities.
- First-ship support catalog for:
  - `BasisBoundContinuationSupport`
  - `MaterializedNarrowingSupport`
  - `DegradedContinuationSupport`
- Explicit out-of-catalog `ExtensionDefinedSupport` rejection posture.
- Upstream-lowered declaration admission with authority, family, role, scope,
  compatibility binding, and opaque payload digest checks.
- Proof-widening publication pipeline from admitted declaration to durable
  support artifact.
- Store-backed publish/fetch/classify/restart/missing-support/handoff facade
  paths.
- In-memory, local-file, and sqlite persistence for durable support records.
- Direct lookup access structures with typed debt when concrete indexes are
  missing.
- Drift taxonomy for family, compatibility, basis, schema, cursor, checkpoint,
  support digest, placement, and session-memory evidence.
- Exact/degraded/rebuild-required/not-resumable classification surfaces with
  cost evidence.
- Bounded restart-shard reconstruction with zero global scan proof.
- Missing-support recovery posture with `Milestone13_2Required` operational
  debt for rebuild-required cases.
- Runtime handoff posture that preserves durable support meaning while keeping
  delivery-session memory ephemeral.
- Machine-checkable certification bundle with truth, artifact,
  subscription-support, replay, diagnostics, counter snapshot, and counter
  digest evidence.

## Trace Matrix

| Spec Obligation | Production Surface | Proof Surface |
| --- | --- | --- |
| First-ship family catalog | `SubscriptionSupportCatalog`, `SubscriptionSupportFamilyKind`, `SubscriptionSupportFamilyRecord` | catalog unit tests, unsupported family certification lane |
| Upstream declaration envelope | `RawSubscriptionSupportDeclaration`, `AdmittedSubscriptionSupportDeclaration` | declaration rejection lanes and constructor privacy trybuild |
| Canonical support scope | `SubscriptionSupportScope` | non-canonical scope certification lane |
| Deterministic identity and digest | `PublishedSubscriptionSupportArtifact`, `SubscriptionSupportStoredRecordSet` | identity, duplicate retry, reopen, and certification tests |
| Basis/cursor/checkpoint/schema/compatibility witnesses | `SubscriptionSupportPublicationPipeline::prepare_exact` | durable record materialization and drift classification tests |
| Durable publish/fetch | `WorthStore::publish_subscription_support`, `WorthStore::fetch_subscription_support` | persistence, local-file/sqlite reopen, direct lookup counter tests |
| Access structure debt | sqlite support schema/index verification and access debt reports | dropped-index hostile tests and `BackendAccessStructureDebt` lane |
| Resume classification | `WorthStore::classify_subscription_support_resume` | exact, degraded, drift, cross-family, rebuild, session-loss, and certification tests |
| Rebuild-required honesty | `classify_missing_subscription_support` | rebuild-required and rebuild-basis-missing lanes |
| Restart reconstruction | `reconstruct_subscription_support_restart_shard` | sqlite restart lane, bounded shard tests, global-scan zero assertions |
| Runtime handoff | `handoff_subscription_support_runtime` | handoff equivalence lane and distinct runtime-owner hostile test |
| Placement is cost-only | placement drift classification handling | tier recall lane and cost-posture unit test |
| Machine-checkable matrix | `SubscriptionSupportCertificationBundle`, `SubscriptionSupportCertificationMatrix` | 25 required lanes, duplicate/missing/mislabeled lane rejection tests |
| Compile-time boundary enforcement | sealed constructors and public facade signatures | trybuild fixtures for raw declarations, decoded rows, cursor-only evidence, session evidence, private handles, and restart shard proof |

## Certification Matrix

The Milestone 13.1 named suite is represented by
`durable_subscription_support_resume_contract_phase_5b_matrix_is_machine_checkable`.

Required lanes:

- `ExactResumeControl`
- `RestartExactResume`
- `RebuildRequiredMissingSupport`
- `DegradedButRecoverable`
- `NotResumableBasisDrift`
- `NotResumableCursorDrift`
- `SupportDigestDrift`
- `CompatibilityDrift`
- `CursorOnlyExactResumeRejected`
- `CrossFamilyReuseRejected`
- `SessionMemoryLossNonAuthoritative`
- `TierRecallCostOnly`
- `RuntimeHandoffEquivalence`
- `UnknownUpstreamAuthorityRejected`
- `NonCanonicalScopeRejected`
- `UnsupportedFamilyKindRejected`
- `MultiDriftBasisPrecedence`
- `MultiDriftCompatibilityPrecedence`
- `RebuildBasisMissingNotResumable`
- `BackendAccessStructureDebt`
- `DecodedRowPublicationRejected`
- `OversizedPayloadRejectedBeforeDecode`
- `RestartShardBoundedReconstruction`
- `ResultCostSurfaceExact`
- `BatchClassificationDebt`

The matrix rejects missing lanes, duplicate lanes, and semantically mislabeled
lane evidence. Lane validation checks classification, primary cause, suppressed
causes, cost-surface posture, and required counter evidence where applicable.

## Verification

Final verification commands:

```text
cargo fmt -p worth-store
cargo test -p worth-store subscription_support_certification --lib
cargo test -p worth-store subscription_support --lib
cargo test -p worth-store --test phase_boundaries_compile_fail -- --test-threads=1
cargo test -p worth-store
git diff --check
```

Final observed results:

- focused certification suite: 4 tests passed
- focused subscription-support suite: 42 tests passed
- phase-boundary compile-fail suite: passed
- full `worth-store` suite: 522 tests passed plus compile-fail and doctest phases
- diff hygiene: no whitespace errors; only pre-existing CRLF warnings

## Remaining Named Debt

- Operational subscription-support rebuild execution remains
  `Milestone13_2Required`; 13.1 classifies rebuild-required posture and names
  the family/basis, but does not schedule hidden maintenance work.
- Retention, compatibility propagation, replication, and maintenance
  participation for support artifacts remain Milestone 13.2.
- Final accuracy/trust taxonomy and generic/domain certification coverage remain
  Milestone 13.3.
- Extension-defined support families remain out of catalog until Milestone 15.

## Confidence Statement

Milestone 13.1 is now closed with production-grade confidence for its intended
scope: durable support identity, family binding, restart visibility, typed
resumability classification, bounded reconstruction, handoff posture, and
machine-checkable certification. The remaining work is intentionally named
follow-on scope, not implicit behavior hidden inside this milestone.
