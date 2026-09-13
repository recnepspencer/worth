use super::{CoverageBasisKind, MaterializationDenial, MaterializationStateClass};
use crate::observation::AccessShape;
use crate::{
    access_planning, AccessAuthorityPosture, AccessStaleDisposition, ExpectedCounterClass,
};
use worth_store_physical_format::CheckpointWalSourceRange;
use worth_store_physical_format::PhysicalEpoch;
use worth_store_wal::LogSequenceNumber;

fn format_family() -> &'static crate::PhysicalArtifactFamilyDeclaration {
    crate::layout_declarations().seed_family()
}

#[test]
fn checkpoint_exactness_preserves_full_range_identity() {
    let left = access_planning()
        .exact_checkpoint_coverage(
            crate::bootstrap::test_support::bootstrap_exact_materialization(
                format_family().family(),
            ),
            CheckpointWalSourceRange::new(21, 29).expect("left checkpoint fixture should be valid"),
        )
        .expect("left checkpoint coverage should admit");
    let right = access_planning()
        .exact_checkpoint_coverage(
            crate::bootstrap::test_support::bootstrap_exact_materialization(
                format_family().family(),
            ),
            CheckpointWalSourceRange::new(24, 29)
                .expect("right checkpoint fixture should be valid"),
        )
        .expect("right checkpoint coverage should admit");

    assert_ne!(left, right);
    assert_eq!(left.upper_bound().value(), right.upper_bound().value());
    assert_ne!(
        left.upper_bound().start_inclusive(),
        right.upper_bound().start_inclusive()
    );
}

#[test]
fn bounded_scan_absence_is_separate_from_exact_index_absence() {
    let coverage = access_planning()
        .exact_root_epoch_coverage(
            crate::bootstrap::test_support::bootstrap_exact_materialization(
                format_family().family(),
            ),
            PhysicalEpoch::from_raw(17).expect("epoch fixture should be valid"),
        )
        .expect("exact coverage should build");
    coverage
        .require_exact()
        .expect("degraded scan fixture coverage should be exact");
    let degraded = crate::access_shapes()
        .explicit_degraded_exact_scan(
            crate::DegradedExactScanRequest::new().with_budget_rows(8_192),
        )
        .expect("bounded exact scan should stay admitted as degraded access");

    assert_eq!(degraded.shape(), AccessShape::DegradedExactScan);
    assert_eq!(
        degraded.authority_posture(),
        AccessAuthorityPosture::ExplicitDegradedExactScan
    );
    assert_eq!(
        degraded.stale_disposition(),
        AccessStaleDisposition::ExplicitDegradedFallback
    );
    assert_eq!(
        degraded.expected_counters(),
        ExpectedCounterClass::DegradedExactScan
    );
    assert_eq!(degraded.budget_rows(), Some(8_192));
}

#[test]
fn lagged_and_quarantined_states_deny_exact_completeness() {
    let lagged = access_planning()
        .lagged_wal_lsn_coverage(
            format_family(),
            LogSequenceNumber::new(40),
            LogSequenceNumber::new(44),
        )
        .expect("lagged coverage should build");
    assert!(matches!(
        lagged.require_exact(),
        Err(MaterializationDenial::LayoutCoverageIsLagged {
            family,
            basis_kind: CoverageBasisKind::WalLsn,
        }) if family == format_family().family()
    ));

    let quarantined = access_planning()
        .quarantined_wal_lsn_coverage(
            format_family(),
            LogSequenceNumber::new(49),
            LogSequenceNumber::new(53),
            CheckpointWalSourceRange::new(50, 52).expect("quarantine fixture should be valid"),
        )
        .expect("quarantined coverage should build");
    assert_eq!(
        quarantined.require_exact(),
        Err(MaterializationDenial::LayoutRangeIsQuarantined {
            gap: quarantined
                .gap()
                .expect("quarantined coverage retains its gap"),
        })
    );
    assert_eq!(
        quarantined.state().class(),
        MaterializationStateClass::Quarantined
    );
}

#[test]
fn reversed_lagged_intervals_are_denied_at_admission() {
    assert_eq!(
        access_planning().lagged_wal_lsn_coverage(
            format_family(),
            LogSequenceNumber::new(44),
            LogSequenceNumber::new(40),
        ),
        Err(MaterializationDenial::CoverageIntervalIsReversed {
            family: format_family().family(),
            basis_kind: CoverageBasisKind::WalLsn,
            lower_bound: 44,
            upper_bound: 40,
        })
    );
}

#[test]
fn persisted_lsm_membership_cannot_be_readmitted_under_another_store_family() {
    let replacement =
        crate::strategy::tests_support::certification_published_lsm_membership_replacement();
    let (foreign_family, _) = crate::strategy::tests_support::admit_strategy_scope(
        worth_store_contracts::DurableArtifactFamilyId::PublicationWalIntent,
        worth_store_security::StoreKeyScope::WalCheckpointEnvelope,
        worth_store_security::StoreTenantScope::StoreInternal,
        worth_store_security::StoreAuthenticityRequirement::required(
            worth_store_security::StoreAuthenticityRequirementClass::AuthenticatedWalRecord,
        ),
        worth_store_security::StoreCustodyPosture::InternalStoreCustody,
    );

    assert_eq!(
        crate::lsm_strategy().readmit_lookup_source(foreign_family, &replacement),
        Err(crate::BaselineLsmExecutionAdmissionDenial::RecordKeyScopeMismatch)
    );
}
