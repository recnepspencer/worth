pub use {sha2::Digest};
pub use {tempfile::tempdir};
pub use {worth_proof::TransitionOutcome};
pub use {worth_store::physical_runtime::PhysicalExecutorCommand, worth_store::physical_runtime::PhysicalStoreCloseOutcome, worth_store::physical_runtime::PhysicalWorkOperationFamily, worth_store::physical_runtime::PhysicalWorkRecoveryAdmissionOutcome, worth_store::physical_runtime::PhysicalWorkRecoveryIngressRejection};
pub use {worth_store_physical_backend::MediaFaultDirective};

pub use {crate::physical_record_journeys::physical_work::executor::admitted_write, crate::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile, crate::physical_record_journeys::physical_work::fault_fixture::serving_from_open_with_positioned_write_fault, crate::physical_record_journeys::physical_work::fixture::semantic_work::work_fixture};

pub fn crash_writer(root: &std::path::Path) {
    let (profile, _, mutation_request) = work_fixture();
    serving_from_initialization_with_work_profile(root, profile.clone()).close();
    let catalog = std::fs::read(root.join("families/records/bootstrap.catalog")).unwrap();
    let payload = catalog[8..16].to_vec();
    let serving = serving_from_open_with_positioned_write_fault(
        root,
        profile,
        worth_store_physical_backend::MediaFaultDirective::PanicAfter,
    );
    let admitted = admitted_write(&serving, mutation_request);
    let command = worth_store::physical_runtime::PhysicalExecutorCommand::exact_write(admitted, payload).unwrap();
    let _ = serving.execute_physical_work(command);
    panic!("fault schedule did not terminate the target effect");
}

pub fn crash_reopener(root: &std::path::Path) {
    let (profile, _, _) = work_fixture();
    let (format, _, access) = crate::physical_record_journeys::configuration();
    let serving = match open_record_store!(crate::physical_record_journeys::media(root), |durability| {
        worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability)
            .with_physical_work_profile(profile)
    },)
    .into_raw()
    {
        worth_proof::TransitionOutcome::Success(serving) => serving,
        worth_proof::TransitionOutcome::Denied(_) => panic!("recovery reopener admission denied"),
        worth_proof::TransitionOutcome::Deferred(_) => panic!("recovery reopener admission deferred"),
        worth_proof::TransitionOutcome::Stale(_) => panic!("recovery reopener admission stale"),
        worth_proof::TransitionOutcome::RebindRequired(_) => {
            panic!("recovery reopener admission requires rebind")
        }
        worth_proof::TransitionOutcome::Failed(_) => panic!("recovery reopener admission requires inspection"),
    };
    let obligations = serving.physical_recovery_obligations();
    assert_eq!(obligations.len(), 1);
    let locator = obligations[0];
    assert_eq!(locator.store(), serving.store_identity());
    assert_ne!(locator.runtime(), 0);
    assert_ne!(locator.generation(), 0);
    assert_ne!(locator.operation(), 0);
    assert_eq!(
        locator.family(),
        worth_store::physical_runtime::PhysicalWorkOperationFamily::ArtifactRangeWrite
    );
    assert_eq!(
        locator.coordinate(),
        Some(
            worth_store_physical_format::RecordFrameCoordinate::new(
                worth_store_physical_format::RecordArtifactFile::BootstrapCatalog,
                8,
                8,
            )
            .unwrap()
        )
    );
    assert!(matches!(
        locator.target(),
        worth_store::physical_runtime::PhysicalWorkRecoveryTarget::Range(_)
    ));
    let payload = &std::fs::read(root.join("families/records/bootstrap.catalog")).unwrap()[8..16];
    assert_eq!(
        locator.payload_digest(),
        Some(sha2::Sha256::digest(payload).into())
    );
    assert_eq!(
        locator.recovery_disposition(),
        worth_store::physical_runtime::PhysicalWorkRecoveryDisposition::InspectionRequired
    );
    assert!(!serving.physical_recovery_evidence_damaged());
    assert!(matches!(
        serving.close_plan().execute(),
        worth_store::physical_runtime::PhysicalStoreCloseOutcome::InspectionRequired { .. }
    ));
    println!("C5_PHYSICAL_RECOVERY_INSPECTION");
}

pub fn child(role: &str, root: &std::path::Path) -> std::process::Output {
    crate::physical_record_journeys::child_process::invocation::child_command(role, root)
        .output()
        .unwrap()
}