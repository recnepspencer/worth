use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    FilesystemMediaAdmission, PhysicalRecordInitialization, PhysicalRecordOpen,
    PhysicalRuntimeAdmission, PhysicalStore, RecordBootstrapDenial,
};
use worth_store_physical_backend::{
    FilesystemAccessPosture, MediaFaultDirective, MediaOperationRole,
};

use super::{configuration, media, serving_from_initialization, success};

#[test]
fn zero_store_identity_remains_catalog_damage_not_rebind_authority() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("zero-store");
    serving_from_initialization(&root).close();
    let catalog = root.join("families/records/bootstrap.catalog");
    let mut bytes = std::fs::read(&catalog).unwrap();
    bytes[48..64].fill(0);
    super::durable_frame_oracle::reseal(&mut bytes);
    std::fs::write(catalog, bytes).unwrap();

    let (format, _, access) = configuration();
    let outcome = open_record_store!(media(&root), |durability| PhysicalRecordOpen::new(
        format, access, durability
    ))
    .into_raw();
    let TransitionOutcome::Denied(denial) = outcome else {
        panic!("a structurally invalid Store identity must be denied as catalog damage")
    };
    assert_eq!(denial.reason(), RecordBootstrapDenial::CatalogDamaged);
    denial.into_runtime().close();
}

#[test]
fn incomplete_bootstrap_publication_never_returns_reusable_authority() {
    let baseline_parent = tempfile::tempdir().unwrap();
    let baseline_root = baseline_parent.path().join("baseline");
    let baseline = media(&baseline_root);
    let before = baseline.media_counters();
    let (format, placement, access) = configuration();
    let serving = success(initialize_record_store!(baseline, |durability| {
        PhysicalRecordInitialization::new(format, placement, access, durability)
    }));
    let after = serving.media_counters();
    serving.close();

    let effect_roles = [
        MediaOperationRole::CreateDirectory,
        MediaOperationRole::CreateNew,
        MediaOperationRole::PositionedWrite,
        MediaOperationRole::SynchronizeFileState,
        MediaOperationRole::SynchronizeDirectoryPublication,
        MediaOperationRole::AtomicReplace,
    ];
    let mut cut_index = 0;
    for role in effect_roles {
        let first = before.attempts_for(role) + 1;
        let last = after.attempts_for(role);
        for ordinal in first..=last {
            let mutation_started = role != MediaOperationRole::CreateDirectory || ordinal != first;
            exercise_cut(
                cut_index,
                role,
                ordinal,
                MediaFaultDirective::FailBefore {
                    kind: std::io::ErrorKind::Other,
                    raw_os_error: None,
                },
                mutation_started,
            );
            cut_index += 1;
        }
    }

    for role in [
        MediaOperationRole::AtomicReplace,
        MediaOperationRole::SynchronizeDirectoryPublication,
    ] {
        exercise_cut(
            cut_index,
            role,
            after.attempts_for(role),
            MediaFaultDirective::IndeterminateAfterEffect,
            true,
        );
        cut_index += 1;
    }
}

fn exercise_cut(
    index: usize,
    role: MediaOperationRole,
    ordinal: u64,
    directive: MediaFaultDirective,
    mutation_started: bool,
) {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join(format!("store-{index}"));
    let admission =
        FilesystemMediaAdmission::production(FilesystemAccessPosture::CoordinatedServiceAccount);
    let authority = admission.fault_schedule_authority();
    let directive_label = format!("{directive:?}");
    let schedule = authority
        .schedule(vec![authority.rule(role, ordinal, directive)])
        .unwrap();
    let runtime = PhysicalStore::admit(PhysicalRuntimeAdmission::new(&root).unwrap()).unwrap();
    let faulted_media = match runtime
        .try_admit_filesystem_media(admission.with_fault_schedule(schedule))
        .into_raw()
    {
        TransitionOutcome::Success(media) => media,
        _ => panic!("cut point must be after media admission"),
    };
    let (format, placement, access) = configuration();
    let outcome = initialize_record_store!(faulted_media, |durability| {
        PhysicalRecordInitialization::new(format, placement, access, durability)
    })
    .into_raw();
    if !mutation_started {
        let TransitionOutcome::Denied(denial) = outcome else {
            panic!("a pre-effect cut must return reusable media authority");
        };
        denial.into_runtime().close();
        assert!(!root.join("families/records").exists());
        return;
    }
    assert!(matches!(outcome, TransitionOutcome::Failed(_)));
    let catalog_published = root.join("families/records/bootstrap.catalog").is_file();

    let observed = open_record_store!(media(&root), |durability| PhysicalRecordOpen::new(
        format, access, durability
    ))
    .into_raw();
    match observed {
        TransitionOutcome::Success(serving) if catalog_published => {
            serving.close();
        }
        TransitionOutcome::Denied(denial) => {
            denial.into_runtime().close();
        }
        TransitionOutcome::Success(serving) => {
            serving.close();
            panic!(
                "fresh observation returned Success without the durable catalog marker at cut \
                 {index}: {role:?} ordinal {ordinal} with {directive_label}"
            )
        }
        TransitionOutcome::Deferred(never) => match never {},
        TransitionOutcome::Stale(stale) => {
            let reason = stale.reason();
            stale.into_runtime().close();
            panic!("fresh observation returned Stale({reason:?})")
        }
        TransitionOutcome::RebindRequired(rebind) => {
            let reason = rebind.reason();
            rebind.into_runtime().close();
            panic!("fresh observation returned RebindRequired({reason:?})")
        }
        TransitionOutcome::Failed(failure) => {
            panic!("fresh observation returned Failed({:?})", failure.cause())
        }
    }
}
