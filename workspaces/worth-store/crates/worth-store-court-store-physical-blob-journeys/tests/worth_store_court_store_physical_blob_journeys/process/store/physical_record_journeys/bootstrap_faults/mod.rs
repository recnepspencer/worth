#[allow(unused_imports)]
use crate::physical_record_journeys::bootstrap_faults::*;
use worth_proof::TransitionOutcome;
use {
    worth_store::physical_runtime::FilesystemMediaAdmission,
    worth_store::physical_runtime::PhysicalRecordInitialization,
    worth_store::physical_runtime::PhysicalRecordOpen,
    worth_store::physical_runtime::PhysicalRuntimeAdmission,
    worth_store::physical_runtime::PhysicalStore,
    worth_store::physical_runtime::RecordBootstrapDenial,
};
use {
    worth_store_physical_backend::FilesystemAccessPosture,
    worth_store_physical_backend::MediaFaultDirective,
    worth_store_physical_backend::MediaOperationRole,
};

use {
    worth_store_testkit_store::physical_record_journeys::configuration,
    worth_store_testkit_store::physical_record_journeys::media,
    worth_store_testkit_store::physical_record_journeys::serving_from_initialization,
    worth_store_testkit_store::physical_record_journeys::success,
};

#[test]
fn zero_store_identity_remains_catalog_damage_not_rebind_authority() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("zero-store");
    serving_from_initialization(&root).close();
    let catalog = root.join("families/records/bootstrap.catalog");
    let mut bytes = std::fs::read(&catalog).unwrap();
    bytes[48..64].fill(0);
    worth_store_testkit_store::durable_frame_oracle::reseal(&mut bytes);
    std::fs::write(catalog, bytes).unwrap();

    let (format, _, access) = configuration();
    let outcome = open_record_store!(media(&root), |durability| {
        worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability)
    })
    .into_raw();
    let worth_proof::TransitionOutcome::Denied(denial) = outcome else {
        panic!("a structurally invalid Store identity must be denied as catalog damage")
    };
    assert_eq!(
        denial.reason(),
        worth_store::physical_runtime::RecordBootstrapDenial::CatalogDamaged
    );
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
        worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, durability,
        )
    }));
    let after = serving.media_counters();
    serving.close();

    let effect_roles = [
        worth_store_physical_backend::MediaOperationRole::CreateDirectory,
        worth_store_physical_backend::MediaOperationRole::CreateNew,
        worth_store_physical_backend::MediaOperationRole::PositionedWrite,
        worth_store_physical_backend::MediaOperationRole::SynchronizeFileState,
        worth_store_physical_backend::MediaOperationRole::SynchronizeDirectoryPublication,
        worth_store_physical_backend::MediaOperationRole::AtomicReplace,
    ];
    let mut cut_index = 0;
    for role in effect_roles {
        let first = before.attempts_for(role) + 1;
        let last = after.attempts_for(role);
        for ordinal in first..=last {
            let mutation_started = role
                != worth_store_physical_backend::MediaOperationRole::CreateDirectory
                || ordinal != first;
            exercise_cut(
                cut_index,
                role,
                ordinal,
                worth_store_physical_backend::MediaFaultDirective::FailBefore {
                    kind: std::io::ErrorKind::Other,
                    raw_os_error: None,
                },
                mutation_started,
            );
            cut_index += 1;
        }
    }

    for role in [
        worth_store_physical_backend::MediaOperationRole::AtomicReplace,
        worth_store_physical_backend::MediaOperationRole::SynchronizeDirectoryPublication,
    ] {
        exercise_cut(
            cut_index,
            role,
            after.attempts_for(role),
            worth_store_physical_backend::MediaFaultDirective::IndeterminateAfterEffect,
            true,
        );
        cut_index += 1;
    }
}
