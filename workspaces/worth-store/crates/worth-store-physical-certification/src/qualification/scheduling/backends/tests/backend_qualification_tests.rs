use worth_store_physical_backend::{
    AdmittedBackendCapabilityWitness, BackendCapabilityAdmissionRequest,
    BackendCapabilityEvidenceBasis, BackendCapabilityKind, BackendCapabilitySupportPosture,
    BackendCapabilitySupportSet, BackendMediaAssumptionSet, BackendRebindTriggers,
    BackendTargetProfile, CapabilityEvidenceClass, PhysicalBackendCapabilityAdmissionAuthority,
};

use crate::pressure_harness::fixtures::replay_bundle_for;
use crate::{
    evaluate_row_rebind, reject_copied_backend_qualification_row,
    reject_environment_name_backend_qualification, reject_log_output_backend_qualification,
    reject_test_only_backend_label_qualification, require_profile_local_row,
    BackendQualificationMatrixDenial, BackendQualificationParityComparison,
    BackendQualificationRow, IoPressureHarnessEvidence, IoPressureHarnessScenario,
    PhysicalFaultEvidenceClass, PhysicalSimulationProfile, PublishedQualificationPosture,
    QualificationHarnessProofClaim, QualificationMatrixPublisher, QualificationPublicationShortcut,
    QualificationRebindEvaluation, QualificationResidualDebt, QualificationResidualDebtReason,
};

#[test]
fn matrix_row_binds_backend_support_and_harness_proof() {
    let evidence = io_pressure_evidence(BackendTargetProfile::PosixFileFsyncDirSync);
    let witness = admitted_backend(
        BackendTargetProfile::PosixFileFsyncDirSync,
        BackendCapabilityEvidenceBasis::certified_backend_profile(),
        BackendCapabilitySupportSet::all_supported(),
    );

    let matrix = QualificationMatrixPublisher::from_executed_store_evidence()
        .with_executed_flush_durability_row(&witness, &evidence)
        .unwrap()
        .publish()
        .unwrap();
    let published = matrix
        .rows_for_claim(
            BackendTargetProfile::PosixFileFsyncDirSync,
            BackendCapabilityKind::Fsync,
        )
        .next()
        .unwrap();
    let identity = published.identity();

    let published = matrix.require_row(identity).unwrap();
    let certified = published.require_certified_backend_support().unwrap();

    assert_eq!(
        published.profile(),
        BackendTargetProfile::PosixFileFsyncDirSync
    );
    assert_eq!(published.capability(), BackendCapabilityKind::Fsync);
    assert_eq!(
        published.evidence_class(),
        CapabilityEvidenceClass::CertifiedBackendProfile
    );
    assert_eq!(
        published.support_posture(),
        BackendCapabilitySupportPosture::Supported
    );
    assert!(published
        .media_assumptions()
        .supports(BackendCapabilityKind::Fsync));
    assert_eq!(published.rebind_triggers(), witness.rebind_triggers());
    assert_eq!(
        published.harness_proof().replay_identity(),
        *evidence.replay_identity()
    );
    assert_eq!(
        published.harness_proof().claim(),
        QualificationHarnessProofClaim::FlushDurability
    );
    assert_eq!(certified.capability(), BackendCapabilityKind::Fsync);
}

#[test]
fn weak_and_stale_rows_cannot_satisfy_certified_backend_support() {
    for basis in [
        BackendCapabilityEvidenceBasis::declared_by_config(1),
        BackendCapabilityEvidenceBasis::observed_by_probe(1),
        BackendCapabilityEvidenceBasis::externally_guaranteed(1),
        BackendCapabilityEvidenceBasis::unverifiable_assumption(),
    ] {
        let row = row_for_basis(basis, BackendCapabilitySupportSet::all_supported());

        assert_eq!(
            row.require_certified_backend_support().unwrap_err(),
            BackendQualificationMatrixDenial::EvidenceClassTooWeak {
                required: CapabilityEvidenceClass::CertifiedBackendProfile,
                actual: basis.evidence_class(),
            }
        );
    }

    let stale = row_for_basis(
        BackendCapabilityEvidenceBasis::certified_backend_profile(),
        BackendCapabilitySupportSet::all_supported().with_posture(
            BackendCapabilityKind::Fsync,
            BackendCapabilitySupportPosture::Stale,
        ),
    );
    let rebind = row_for_basis(
        BackendCapabilityEvidenceBasis::certified_backend_profile(),
        BackendCapabilitySupportSet::all_supported().with_posture(
            BackendCapabilityKind::Fsync,
            BackendCapabilitySupportPosture::RebindRequired,
        ),
    );

    assert_eq!(
        stale.require_certified_backend_support().unwrap_err(),
        BackendQualificationMatrixDenial::StaleRow {
            capability: BackendCapabilityKind::Fsync
        }
    );
    assert!(matches!(
        rebind.require_certified_backend_support().unwrap_err(),
        BackendQualificationMatrixDenial::RebindRequired { .. }
    ));
    assert_eq!(
        evaluate_row_rebind(&stale),
        QualificationRebindEvaluation::Stale
    );
    assert!(evaluate_row_rebind(&rebind).requires_rebind());
    assert_eq!(
        stale.residual_debt().reason(),
        QualificationResidualDebtReason::StaleEvidence
    );
}

#[test]
fn residual_debt_keeps_supported_row_degraded_until_requalified() {
    let evidence = io_pressure_evidence(BackendTargetProfile::PosixFileFsyncDirSync);
    let witness = admitted_backend(
        BackendTargetProfile::PosixFileFsyncDirSync,
        BackendCapabilityEvidenceBasis::certified_backend_profile(),
        BackendCapabilitySupportSet::all_supported(),
    );
    let debt = QualificationResidualDebt::missing_evidence(
        BackendCapabilityKind::Fsync,
        CapabilityEvidenceClass::CertifiedBackendProfile,
        witness.rebind_triggers(),
    );
    let matrix = QualificationMatrixPublisher::from_executed_store_evidence()
        .with_executed_flush_durability_row_and_residual_debt(&witness, &evidence, debt)
        .unwrap()
        .publish()
        .unwrap();
    let row = matrix
        .rows_for_claim(
            BackendTargetProfile::PosixFileFsyncDirSync,
            BackendCapabilityKind::Fsync,
        )
        .next()
        .unwrap();

    assert_eq!(
        row.published_posture(),
        PublishedQualificationPosture::Degraded
    );
    assert_eq!(
        row.require_certified_backend_support().unwrap_err(),
        BackendQualificationMatrixDenial::ResidualDebtPresent {
            capability: BackendCapabilityKind::Fsync
        }
    );
    assert_eq!(
        row.residual_debt().missing_evidence_class(),
        debt.missing_evidence_class()
    );
}

#[test]
fn cross_backend_parity_preserves_backend_specific_causes() {
    let unsupported = BackendCapabilitySupportSet::all_supported().with_posture(
        BackendCapabilityKind::Fsync,
        BackendCapabilitySupportPosture::Unsupported,
    );
    let posix = row_for_profile(
        BackendTargetProfile::PosixFileFsyncDirSync,
        BackendCapabilityEvidenceBasis::certified_backend_profile(),
        unsupported,
    );
    let windows = row_for_profile(
        BackendTargetProfile::WindowsFlushFileBuffers,
        BackendCapabilityEvidenceBasis::certified_backend_profile(),
        unsupported,
    );

    let parity = BackendQualificationParityComparison::compare(&posix, &windows).unwrap();

    assert!(parity.policy_equivalent());
    assert_eq!(
        parity.left_profile(),
        BackendTargetProfile::PosixFileFsyncDirSync
    );
    assert_eq!(
        parity.right_profile(),
        BackendTargetProfile::WindowsFlushFileBuffers
    );
    assert_eq!(
        parity.left_posture(),
        PublishedQualificationPosture::Unsupported
    );
    assert_eq!(
        parity.left_residual_debt().reason(),
        QualificationResidualDebtReason::BackendSpecificDenial
    );
    assert_eq!(
        parity.right_residual_debt().reason(),
        QualificationResidualDebtReason::BackendSpecificDenial
    );
    assert_ne!(
        parity.left_profile(),
        parity.right_profile(),
        "profile-specific cause identity must remain visible"
    );
    assert_eq!(
        require_profile_local_row(BackendTargetProfile::PosixFileFsyncDirSync, &windows)
            .unwrap_err(),
        BackendQualificationMatrixDenial::CrossBackendEvidenceSubstitution {
            expected: BackendTargetProfile::PosixFileFsyncDirSync,
            actual: BackendTargetProfile::WindowsFlushFileBuffers,
        }
    );
}

#[test]
fn matrix_rejects_duplicate_claims_and_publication_shortcuts() {
    let evidence = io_pressure_evidence(BackendTargetProfile::PosixFileFsyncDirSync);
    let certified = admitted_backend(
        BackendTargetProfile::PosixFileFsyncDirSync,
        BackendCapabilityEvidenceBasis::certified_backend_profile(),
        BackendCapabilitySupportSet::all_supported(),
    );
    let weaker_same_claim = admitted_backend(
        BackendTargetProfile::PosixFileFsyncDirSync,
        BackendCapabilityEvidenceBasis::externally_guaranteed(1),
        BackendCapabilitySupportSet::all_supported(),
    );

    let matrix = QualificationMatrixPublisher::from_executed_store_evidence()
        .with_executed_flush_durability_row(&certified, &evidence)
        .unwrap()
        .with_executed_flush_durability_row(&weaker_same_claim, &evidence)
        .unwrap()
        .publish()
        .unwrap();
    assert_eq!(
        matrix
            .rows_for_claim(
                BackendTargetProfile::PosixFileFsyncDirSync,
                BackendCapabilityKind::Fsync
            )
            .count(),
        2
    );
    assert_eq!(
        QualificationMatrixPublisher::from_executed_store_evidence()
            .with_executed_flush_durability_row(&certified, &evidence)
            .unwrap()
            .with_executed_flush_durability_row(&certified, &evidence)
            .unwrap_err(),
        BackendQualificationMatrixDenial::DuplicateRow {
            profile: BackendTargetProfile::PosixFileFsyncDirSync,
            capability: BackendCapabilityKind::Fsync,
            evidence_class: CapabilityEvidenceClass::CertifiedBackendProfile,
        }
    );
    assert_eq!(
        reject_copied_backend_qualification_row().unwrap_err(),
        BackendQualificationMatrixDenial::PublicationShortcut(
            QualificationPublicationShortcut::CopiedRow
        )
    );
    assert_eq!(
        reject_log_output_backend_qualification().unwrap_err(),
        BackendQualificationMatrixDenial::PublicationShortcut(
            QualificationPublicationShortcut::LogOutput
        )
    );
    assert_eq!(
        reject_environment_name_backend_qualification().unwrap_err(),
        BackendQualificationMatrixDenial::PublicationShortcut(
            QualificationPublicationShortcut::EnvironmentName
        )
    );
    assert_eq!(
        reject_test_only_backend_label_qualification().unwrap_err(),
        BackendQualificationMatrixDenial::PublicationShortcut(
            QualificationPublicationShortcut::TestOnlyBackendLabel
        )
    );
}

fn row_for_basis(
    basis: BackendCapabilityEvidenceBasis,
    support: BackendCapabilitySupportSet,
) -> BackendQualificationRow {
    row_for_profile(BackendTargetProfile::PosixFileFsyncDirSync, basis, support)
}

fn row_for_profile(
    profile: BackendTargetProfile,
    basis: BackendCapabilityEvidenceBasis,
    support: BackendCapabilitySupportSet,
) -> BackendQualificationRow {
    let evidence = io_pressure_evidence(profile);
    let witness = admitted_backend(profile, basis, support);
    let matrix = QualificationMatrixPublisher::from_executed_store_evidence()
        .with_executed_flush_durability_row(&witness, &evidence)
        .unwrap()
        .publish()
        .unwrap();

    *matrix
        .rows_for_claim(profile, BackendCapabilityKind::Fsync)
        .next()
        .unwrap()
}

fn admitted_backend(
    profile: BackendTargetProfile,
    basis: BackendCapabilityEvidenceBasis,
    support: BackendCapabilitySupportSet,
) -> AdmittedBackendCapabilityWitness {
    PhysicalBackendCapabilityAdmissionAuthority::store_owned()
        .admit_backend_capability(BackendCapabilityAdmissionRequest::new(
            profile,
            basis,
            support,
            BackendMediaAssumptionSet::platform_file_defaults(),
            BackendRebindTriggers::kernel_filesystem_mount_firmware_and_backend()
                .with_cloud_volume()
                .with_sector_alignment()
                .with_security_posture(),
        ))
        .unwrap()
}

fn io_pressure_evidence(profile: BackendTargetProfile) -> IoPressureHarnessEvidence {
    let scenario = IoPressureHarnessScenario::deterministic_read_under_repair_pressure()
        .with_backend_profile(profile)
        .with_fault_evidence_class(PhysicalFaultEvidenceClass::CertifiedBackend);
    let replay = replay_bundle_for(
        scenario.clone(),
        PhysicalSimulationProfile::HardwareQualification,
    );
    IoPressureHarnessEvidence::from_replay_bundle(scenario, &replay).unwrap()
}
