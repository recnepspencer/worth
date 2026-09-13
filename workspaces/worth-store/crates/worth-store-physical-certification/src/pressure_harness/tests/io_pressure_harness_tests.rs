use worth_store_io_scheduler::foreground_reservation::ForegroundIoLaneKind;
use worth_store_io_scheduler::BackgroundIoPressureClass;
use worth_store_physical_backend::{BackendTargetProfile, CapabilityEvidenceClass};

use crate::pressure_harness::fixtures::{
    fault_phase_for, io_pressure_oracle_denial_without_pressure_observation, replay_bundle_for,
};
use crate::{
    all_io_pressure_fault_evidence_classes, all_io_pressure_fault_kinds,
    IoPressureBackendSafetyQualificationDenial, IoPressureFaultKind, IoPressureHarnessEvidence,
    IoPressureHarnessScenario, OracleDenial, PhysicalFaultEvidenceClass, PhysicalSimulationProfile,
};

#[test]
fn replay_bundle_admits_io_pressure_evidence() {
    let scenario = IoPressureHarnessScenario::deterministic_read_under_repair_pressure();
    let replay = replay_bundle_for(scenario.clone(), PhysicalSimulationProfile::DeveloperSmoke);

    let evidence = IoPressureHarnessEvidence::from_replay_bundle(scenario, &replay).unwrap();
    assert_ne!(evidence.replay_identity(), &[0; 32]);
    assert_eq!(
        evidence.replay_profile(),
        PhysicalSimulationProfile::DeveloperSmoke
    );
}

#[test]
fn backend_safety_qualification_distinguishes_all_fault_evidence_classes() {
    let denials = [
        (
            PhysicalFaultEvidenceClass::Simulated,
            IoPressureBackendSafetyQualificationDenial::SimulatedBackendSuccess,
        ),
        (
            PhysicalFaultEvidenceClass::InjectedProductionBoundary,
            IoPressureBackendSafetyQualificationDenial::InjectedBoundaryOnly,
        ),
        (
            PhysicalFaultEvidenceClass::BackendEmulated,
            IoPressureBackendSafetyQualificationDenial::BackendEmulatedOnly,
        ),
        (
            PhysicalFaultEvidenceClass::ObservedHost,
            IoPressureBackendSafetyQualificationDenial::ObservedHostOnly,
        ),
    ];
    for (fault_class, expected_denial) in denials {
        let evidence = evidence_for_fault_class(fault_class);

        assert_eq!(
            evidence.require_real_backend_safety().unwrap_err(),
            expected_denial
        );
    }
    for fault_class in [
        PhysicalFaultEvidenceClass::CertifiedBackend,
        PhysicalFaultEvidenceClass::ExternallyGuaranteed,
    ] {
        let evidence = evidence_for_fault_class(fault_class);
        let qualification = evidence.require_real_backend_safety().unwrap();

        assert_eq!(
            qualification.backend_profile(),
            BackendTargetProfile::PosixFileFsyncDirSync
        );
        assert_eq!(qualification.evidence_class(), fault_class);
    }
}

#[test]
fn weak_backend_evidence_class_cannot_qualify_real_backend_safety() {
    for backend_evidence_class in [
        CapabilityEvidenceClass::DeclaredByConfig,
        CapabilityEvidenceClass::ObservedByProbe,
        CapabilityEvidenceClass::EstablishedByFilesystemAdmission,
        CapabilityEvidenceClass::ExternallyGuaranteed,
        CapabilityEvidenceClass::UnverifiableAssumption,
    ] {
        let scenario = IoPressureHarnessScenario::deterministic_read_under_repair_pressure()
            .with_fault_evidence_class(PhysicalFaultEvidenceClass::CertifiedBackend)
            .with_backend_evidence_class(backend_evidence_class);
        let replay = replay_bundle_for(scenario.clone(), PhysicalSimulationProfile::DeveloperSmoke);
        let evidence = IoPressureHarnessEvidence::from_replay_bundle(scenario, &replay).unwrap();

        assert_eq!(
            evidence.require_real_backend_safety().unwrap_err(),
            IoPressureBackendSafetyQualificationDenial::BackendEvidenceClassTooWeak {
                required: CapabilityEvidenceClass::CertifiedBackendProfile,
                actual: backend_evidence_class,
            }
        );
    }
}

#[test]
fn all_pressure_faults_require_io_pressure_fault_delivery() {
    for fault_kind in all_io_pressure_fault_kinds() {
        let scenario = IoPressureHarnessScenario::deterministic_read_under_repair_pressure()
            .with_fault_kind(fault_kind);
        let replay = replay_bundle_for(scenario.clone(), PhysicalSimulationProfile::DeveloperSmoke);
        let evidence =
            IoPressureHarnessEvidence::from_replay_bundle(scenario.clone(), &replay).unwrap();

        assert_eq!(evidence.fault_phase(), fault_phase_for(fault_kind));
        assert_eq!(evidence.scenario().fault_kind(), fault_kind);
    }
}

#[test]
fn missing_pressure_observation_denies_io_pressure_oracle_and_evidence() {
    let scenario = IoPressureHarnessScenario::deterministic_read_under_repair_pressure();

    let denial = io_pressure_oracle_denial_without_pressure_observation(scenario);

    assert_eq!(denial, OracleDenial::MissingIoPressureObservation);
}

#[test]
fn deterministic_and_large_profiles_preserve_io_pressure_evidence_topology() {
    let deterministic = IoPressureHarnessScenario::deterministic_read_under_repair_pressure();
    let large = deterministic
        .clone()
        .with_foreground_lane(ForegroundIoLaneKind::CommitCriticalWalWrite)
        .with_background_pressure(BackgroundIoPressureClass::CheckpointFlush)
        .with_fault_kind(IoPressureFaultKind::DelayedSync);

    let deterministic_replay = replay_bundle_for(
        deterministic.clone(),
        PhysicalSimulationProfile::DeveloperSmoke,
    );
    let large_replay = replay_bundle_for(large.clone(), PhysicalSimulationProfile::CiCertification);
    let deterministic_evidence =
        IoPressureHarnessEvidence::from_replay_bundle(deterministic, &deterministic_replay)
            .unwrap();
    let large_evidence =
        IoPressureHarnessEvidence::from_replay_bundle(large.clone(), &large_replay).unwrap();

    assert_eq!(
        deterministic_replay.plan().scenario_family(),
        large_replay.plan().scenario_family()
    );
    assert_eq!(
        deterministic_replay.plan().actors(),
        large_replay.plan().actors()
    );
    assert_eq!(
        deterministic_replay.plan().drivers(),
        large_replay.plan().drivers()
    );
    assert_eq!(
        deterministic_replay.plan().oracle_families(),
        large_replay.plan().oracle_families()
    );
    assert_eq!(
        deterministic_replay.plan().counter_contracts(),
        large_replay.plan().counter_contracts()
    );
    assert_eq!(
        deterministic_replay
            .schedule()
            .actor_steps()
            .iter()
            .map(|step| step.actor_role())
            .collect::<Vec<_>>(),
        large_replay
            .schedule()
            .actor_steps()
            .iter()
            .map(|step| step.actor_role())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        large_evidence.replay_profile(),
        PhysicalSimulationProfile::CiCertification
    );
    assert_eq!(
        large_evidence.scenario().fault_kind(),
        IoPressureFaultKind::DelayedSync
    );
    assert_ne!(
        deterministic_evidence.replay_identity(),
        large_evidence.replay_identity()
    );
}

#[test]
fn all_fault_evidence_classes_are_explicit() {
    assert_eq!(all_io_pressure_fault_evidence_classes().len(), 6);
    assert!(all_io_pressure_fault_evidence_classes()
        .contains(&PhysicalFaultEvidenceClass::BackendEmulated));
}

fn evidence_for_fault_class(fault_class: PhysicalFaultEvidenceClass) -> IoPressureHarnessEvidence {
    let scenario = IoPressureHarnessScenario::deterministic_read_under_repair_pressure()
        .with_fault_evidence_class(fault_class);
    let replay = replay_bundle_for(scenario.clone(), PhysicalSimulationProfile::DeveloperSmoke);
    IoPressureHarnessEvidence::from_replay_bundle(scenario, &replay).unwrap()
}
