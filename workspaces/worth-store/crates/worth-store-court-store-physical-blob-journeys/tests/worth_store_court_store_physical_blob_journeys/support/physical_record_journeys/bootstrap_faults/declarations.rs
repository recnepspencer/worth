pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_store::physical_runtime::FilesystemMediaAdmission, worth_store::physical_runtime::PhysicalRecordInitialization, worth_store::physical_runtime::PhysicalRecordOpen, worth_store::physical_runtime::PhysicalRuntimeAdmission, worth_store::physical_runtime::PhysicalStore, worth_store::physical_runtime::RecordBootstrapDenial};
pub(crate) use {worth_store_physical_backend::FilesystemAccessPosture, worth_store_physical_backend::MediaFaultDirective, worth_store_physical_backend::MediaOperationRole};

pub(crate) use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::media, worth_store_testkit_store::physical_record_journeys::serving_from_initialization, worth_store_testkit_store::physical_record_journeys::success};

pub(crate) fn exercise_cut(
    index: usize,
    role: MediaOperationRole,
    ordinal: u64,
    directive: MediaFaultDirective,
    mutation_started: bool,
) {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join(format!("store-{index}"));
    let admission =
        worth_store::physical_runtime::FilesystemMediaAdmission::production(worth_store_physical_backend::FilesystemAccessPosture::CoordinatedServiceAccount);
    let authority = admission.fault_schedule_authority();
    let directive_label = format!("{directive:?}");
    let schedule = authority
        .schedule(vec![authority.rule(role, ordinal, directive)])
        .unwrap();
    let runtime = worth_store::physical_runtime::PhysicalStore::admit(worth_store::physical_runtime::PhysicalRuntimeAdmission::new(&root).unwrap()).unwrap();
    let faulted_media = match runtime
        .try_admit_filesystem_media(admission.with_fault_schedule(schedule))
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(media) => media,
        _ => panic!("cut point must be after media admission"),
    };
    let (format, placement, access) = configuration();
    let outcome = initialize_record_store!(faulted_media, |durability| {
        worth_store::physical_runtime::PhysicalRecordInitialization::new(format, placement, access, durability)
    })
    .into_raw();
    if !mutation_started {
        let worth_proof::TransitionOutcome::Denied(denial) = outcome else {
            panic!("a pre-effect cut must return reusable media authority");
        };
        denial.into_runtime().close();
        assert!(!root.join("families/records").exists());
        return;
    }
    assert!(matches!(outcome, worth_proof::TransitionOutcome::Failed(_)));
    let catalog_published = root.join("families/records/bootstrap.catalog").is_file();

    let observed = open_record_store!(media(&root), |durability| worth_store::physical_runtime::PhysicalRecordOpen::new(
        format, access, durability
    ))
    .into_raw();
    match observed {
        worth_proof::TransitionOutcome::Success(serving) if catalog_published => {
            serving.close();
        }
        worth_proof::TransitionOutcome::Denied(denial) => {
            denial.into_runtime().close();
        }
        worth_proof::TransitionOutcome::Success(serving) => {
            serving.close();
            panic!(
                "fresh observation returned Success without the durable catalog marker at cut \
                 {index}: {role:?} ordinal {ordinal} with {directive_label}"
            )
        }
        worth_proof::TransitionOutcome::Deferred(never) => match never {},
        worth_proof::TransitionOutcome::Stale(stale) => {
            let reason = stale.reason();
            stale.into_runtime().close();
            panic!("fresh observation returned Stale({reason:?})")
        }
        worth_proof::TransitionOutcome::RebindRequired(rebind) => {
            let reason = rebind.reason();
            rebind.into_runtime().close();
            panic!("fresh observation returned RebindRequired({reason:?})")
        }
        worth_proof::TransitionOutcome::Failed(failure) => {
            panic!("fresh observation returned Failed({:?})", failure.cause())
        }
    }
}