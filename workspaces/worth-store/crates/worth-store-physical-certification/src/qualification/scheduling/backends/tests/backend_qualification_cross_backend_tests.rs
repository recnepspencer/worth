use worth_store_physical_backend::{
    AdmittedBackendCapabilityWitness, BackendCapabilityAdmissionRequest,
    BackendCapabilityEvidenceBasis, BackendCapabilityKind, BackendCapabilitySupportPosture,
    BackendCapabilitySupportSet, BackendMediaAssumptionSet, BackendRebindTriggers,
    BackendTargetProfile, CapabilityEvidenceClass, PhysicalBackendCapabilityAdmissionAuthority,
};

use crate::pressure_harness::fixtures::replay_bundle_for;
use crate::{
    BackendQualificationMatrixDenial, BackendQualificationParityComparison,
    BackendQualificationRow, IoPressureHarnessEvidence, IoPressureHarnessScenario,
    PhysicalFaultEvidenceClass, PhysicalSimulationProfile, PublishedQualificationPosture,
    QualificationMatrixPublisher, QualificationResidualDebt, QualificationResidualDebtReason,
};

#[test]
fn ordinary_publisher_rejects_cross_backend_evidence_for_every_capability() {
    let posix_evidence = io_pressure_evidence(BackendTargetProfile::PosixFileFsyncDirSync);
    let windows_witness = admitted_backend(
        BackendTargetProfile::WindowsFlushFileBuffers,
        BackendCapabilitySupportSet::all_supported(),
    );

    for capability in backend_qualification_capabilities() {
        assert_eq!(
            publish_capability(
                QualificationMatrixPublisher::from_executed_store_evidence(),
                capability,
                &windows_witness,
                &posix_evidence,
            )
            .unwrap_err(),
            BackendQualificationMatrixDenial::ProfileMismatch {
                expected: BackendTargetProfile::WindowsFlushFileBuffers,
                actual: BackendTargetProfile::PosixFileFsyncDirSync,
            },
            "{capability:?} must bind proof to the executed backend profile"
        );
    }
}

#[test]
fn cross_backend_parity_preserves_causes_for_every_capability() {
    for capability in backend_qualification_capabilities() {
        let posix = parity_denial_row(BackendTargetProfile::PosixFileFsyncDirSync, capability);
        let windows = parity_denial_row(BackendTargetProfile::WindowsFlushFileBuffers, capability);
        let parity = BackendQualificationParityComparison::compare(&posix, &windows).unwrap();
        let expected_posture = expected_parity_posture(capability);

        assert!(parity.policy_equivalent(), "{capability:?}");
        assert_eq!(
            parity.left_profile(),
            BackendTargetProfile::PosixFileFsyncDirSync
        );
        assert_eq!(
            parity.right_profile(),
            BackendTargetProfile::WindowsFlushFileBuffers
        );
        assert_eq!(parity.left_posture(), expected_posture);
        assert_eq!(parity.right_posture(), expected_posture);
        assert_eq!(
            parity.left_residual_debt().reason(),
            QualificationResidualDebtReason::BackendSpecificDenial
        );
        assert_eq!(
            parity.right_residual_debt().reason(),
            QualificationResidualDebtReason::BackendSpecificDenial
        );
        assert_eq!(
            parity.left_residual_debt().affected_capability(),
            capability
        );
        assert_eq!(
            parity.right_residual_debt().affected_capability(),
            capability
        );
    }
}

fn parity_denial_row(
    profile: BackendTargetProfile,
    capability: BackendCapabilityKind,
) -> BackendQualificationRow {
    if capability == BackendCapabilityKind::BufferedFile {
        return buffered_degraded_row(profile);
    }
    let support = BackendCapabilitySupportSet::all_supported()
        .with_posture(capability, BackendCapabilitySupportPosture::Unsupported);
    let witness = admitted_backend(profile, support);
    let evidence = io_pressure_evidence(profile);
    let matrix = publish_capability(
        QualificationMatrixPublisher::from_executed_store_evidence(),
        capability,
        &witness,
        &evidence,
    )
    .unwrap()
    .publish()
    .unwrap();

    *matrix.rows_for_claim(profile, capability).next().unwrap()
}

fn buffered_degraded_row(profile: BackendTargetProfile) -> BackendQualificationRow {
    let witness = admitted_backend(profile, BackendCapabilitySupportSet::all_supported());
    let evidence = io_pressure_evidence(profile);
    let debt = QualificationResidualDebt::backend_specific_denial(
        BackendCapabilityKind::BufferedFile,
        CapabilityEvidenceClass::CertifiedBackendProfile,
        witness.rebind_triggers(),
    );
    let matrix = QualificationMatrixPublisher::from_executed_store_evidence()
        .with_executed_buffered_file_row_and_residual_debt(&witness, &evidence, debt)
        .unwrap()
        .publish()
        .unwrap();

    *matrix
        .rows_for_claim(profile, BackendCapabilityKind::BufferedFile)
        .next()
        .unwrap()
}

fn expected_parity_posture(capability: BackendCapabilityKind) -> PublishedQualificationPosture {
    match capability {
        BackendCapabilityKind::BufferedFile => PublishedQualificationPosture::Degraded,
        _ => PublishedQualificationPosture::Unsupported,
    }
}

fn publish_capability(
    publisher: QualificationMatrixPublisher,
    capability: BackendCapabilityKind,
    witness: &AdmittedBackendCapabilityWitness,
    evidence: &IoPressureHarnessEvidence,
) -> Result<QualificationMatrixPublisher, BackendQualificationMatrixDenial> {
    match capability {
        BackendCapabilityKind::BufferedFile => {
            publisher.with_executed_buffered_file_row(witness, evidence)
        }
        BackendCapabilityKind::DirectIo => publisher.with_executed_direct_io_row(witness, evidence),
        BackendCapabilityKind::Mmap => publisher.with_executed_mmap_row(witness, evidence),
        BackendCapabilityKind::AsyncIo => publisher.with_executed_async_io_row(witness, evidence),
        BackendCapabilityKind::Fsync => {
            publisher.with_executed_flush_durability_row(witness, evidence)
        }
        BackendCapabilityKind::DirectorySync => {
            publisher.with_executed_directory_sync_row(witness, evidence)
        }
        BackendCapabilityKind::DurableRename => {
            publisher.with_executed_durable_rename_row(witness, evidence)
        }
        BackendCapabilityKind::SecureFrameIo => {
            publisher.with_executed_secure_frame_io_row(witness, evidence)
        }
    }
}

fn backend_qualification_capabilities() -> [BackendCapabilityKind; 8] {
    [
        BackendCapabilityKind::BufferedFile,
        BackendCapabilityKind::DirectIo,
        BackendCapabilityKind::Mmap,
        BackendCapabilityKind::AsyncIo,
        BackendCapabilityKind::Fsync,
        BackendCapabilityKind::DirectorySync,
        BackendCapabilityKind::DurableRename,
        BackendCapabilityKind::SecureFrameIo,
    ]
}

fn admitted_backend(
    profile: BackendTargetProfile,
    support: BackendCapabilitySupportSet,
) -> AdmittedBackendCapabilityWitness {
    PhysicalBackendCapabilityAdmissionAuthority::store_owned()
        .admit_backend_capability(BackendCapabilityAdmissionRequest::new(
            profile,
            BackendCapabilityEvidenceBasis::certified_backend_profile(),
            support,
            BackendMediaAssumptionSet::platform_file_defaults()
                .with_direct_io_alignment()
                .with_sector_atomicity()
                .with_page_cache_policy()
                .with_mmap_coherence()
                .with_async_ordering()
                .with_secure_frame_io(),
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
