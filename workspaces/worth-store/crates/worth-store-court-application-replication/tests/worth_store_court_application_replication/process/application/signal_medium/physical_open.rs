//! Open actual physical media before constructing its exclusive Signal adapter.
use std::path::Path;
use worth_store::physical_runtime::{
    AdmittedRecordPlacementPolicy, FilesystemAccessPosture, FilesystemMediaAdmission,
    MediaOwnedPhysicalRuntime, OrderedJournalRegistry, PhysicalRecordInitialization,
    PhysicalRecordOpen, PhysicalRuntimeAdmission, PhysicalStore, ServingPhysicalRuntime,
};
use worth_store_testkit_store::physical_record_journeys as physical;

pub(super) fn open(
    root: &Path,
    fresh: bool,
) -> (Box<ServingPhysicalRuntime>, AdmittedRecordPlacementPolicy) {
    let (format, placement, access) = physical::configuration();
    let registry =
        OrderedJournalRegistry::declare(3, &["signal.definitions", "signal.advances"], &[])
            .unwrap();
    let media = admit_media(root);
    let durability = physical::durability(&media);
    let serving = if fresh {
        Box::new(physical::success(
            media.initialize_record_store(
                PhysicalRecordInitialization::new(format, placement, access, durability)
                    .with_journal_registry(registry),
            ),
        ))
    } else {
        Box::new(physical::success(media.open_record_store(
            PhysicalRecordOpen::new(format, access, durability).with_journal_registry(registry),
        )))
    };
    (serving, placement)
}

fn admit_media(root: &Path) -> MediaOwnedPhysicalRuntime {
    let admitted = PhysicalStore::admit(PhysicalRuntimeAdmission::new(root).unwrap()).unwrap();
    match admitted
        .try_admit_filesystem_media(FilesystemMediaAdmission::production(
            FilesystemAccessPosture::CoordinatedServiceAccount,
        ))
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(media) => media,
        worth_proof::TransitionOutcome::Denied(denial) => panic!(
            "Signal filesystem admission denied at {root:?}: {:?}",
            denial.reason()
        ),
        worth_proof::TransitionOutcome::Deferred(denial) => panic!(
            "Signal filesystem admission deferred at {root:?}: {:?}",
            denial.reason()
        ),
        worth_proof::TransitionOutcome::Stale(denial) => panic!(
            "Signal filesystem admission stale at {root:?}: {:?}",
            denial.reason()
        ),
        worth_proof::TransitionOutcome::RebindRequired(denial) => panic!(
            "Signal filesystem admission requires rebind at {root:?}: {:?}",
            denial.reason()
        ),
        worth_proof::TransitionOutcome::Failed(denial) => panic!(
            "Signal filesystem admission requires inspection at {root:?}: {:?}",
            denial.cause()
        ),
    }
}
