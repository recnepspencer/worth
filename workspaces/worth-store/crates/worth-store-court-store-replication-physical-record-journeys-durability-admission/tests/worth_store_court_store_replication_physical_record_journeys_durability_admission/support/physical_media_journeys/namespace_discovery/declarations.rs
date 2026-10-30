pub(crate) use std::path::Path;

pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {
    worth_store::physical_runtime::MediaOwnedPhysicalRuntime,
    worth_store::physical_runtime::MediaShutdownOutcome,
    worth_store::physical_runtime::ObservationError,
};
pub(crate) use {
    worth_store_physical_backend::MediaCounterSnapshot,
    worth_store_physical_backend::MediaOperationRole,
};

pub(crate) use {
    crate::child_dispatch::emit, crate::child_dispatch::run_role,
    crate::child_dispatch::ChildReport,
};
pub(crate) use {
    crate::physical_media_journeys::admit_runtime, crate::physical_media_journeys::media_admission,
};

pub(crate) use crate::namespace_discovery::canonical_equivalence::canonical_mismatch_loci;
pub(crate) use {
    crate::namespace_discovery::observer_report::assert_namespace_shape,
    crate::namespace_discovery::observer_report::observe_namespace,
};

pub(crate) fn run_child(role: &str, root: &Path) {
    let media = match role {
        "namespace-writer" => admit_writer(root),
        "namespace-reopener" => admit_media(root, media_admission()),
        _ => panic!("unsupported namespace role"),
    };
    let observation = media.observer().snapshot().unwrap();
    let canonical_loci = (role == "namespace-reopener").then(|| canonical_mismatch_loci(&media));
    let store_identity = media.store_identity();
    let runtime_identity = media.runtime_identity();
    let observer = media.observer();
    assert!(matches!(
        media.close(),
        worth_store::physical_runtime::MediaShutdownOutcome::Released(_)
    ));
    assert!(matches!(
        observer.snapshot(),
        Err(worth_store::physical_runtime::ObservationError::Closed { .. })
    ));
    let mut fields = report_fields(
        store_identity,
        runtime_identity,
        observation.mutation_owner(),
        observer.media_counters(),
    );
    if let Some(loci) = canonical_loci {
        fields.push(("canonical", "equivalent".into()));
        fields.push(("mutant_loci", loci));
    }
    emit(&fields);
}

pub(crate) fn admit_writer(root: &Path) -> MediaOwnedPhysicalRuntime {
    let admission = worth_store::physical_runtime::FilesystemMediaAdmission::certification(
        worth_store_physical_backend::FilesystemAccessPosture::CoordinatedServiceAccount,
    );
    #[cfg(not(all()))]
    let admission = media_admission();
    admit_media(root, admission)
}

pub(crate) fn admit_media(
    root: &Path,
    admission: worth_store::physical_runtime::FilesystemMediaAdmission,
) -> MediaOwnedPhysicalRuntime {
    match admit_runtime(root)
        .try_admit_filesystem_media(admission)
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(media) => media,
        _ => panic!("fresh-process media admission failed"),
    }
}

pub(crate) fn report_fields(
    store_identity: worth_store_physical_format::store_namespace::StableStoreIdentity,
    runtime_identity: worth_store::physical_runtime::RuntimeIdentity,
    owner: worth_store_physical_backend::MutationOwnerObservation,
    counters: MediaCounterSnapshot,
) -> Vec<(&'static str, String)> {
    vec![
        ("store", hex(&store_identity.bytes())),
        ("runtime", runtime_identity.get().to_string()),
        ("process", owner.process_id().to_string()),
        ("owner", hex(&owner.owner().bytes())),
        ("attempt", hex(&owner.attempt().bytes())),
        ("attempted", counters.attempted_operations().to_string()),
        ("completed", counters.completed_operations().to_string()),
        ("denied", counters.denied_before_effect().to_string()),
        ("partial", counters.partial_effects().to_string()),
        (
            "indeterminate",
            counters.indeterminate_effects().to_string(),
        ),
        ("requested_bytes", counters.requested_bytes().to_string()),
        ("completed_bytes", counters.completed_bytes().to_string()),
        (
            "qualification",
            counters.qualification_transactions().to_string(),
        ),
        ("file_syncs", counters.file_syncs().to_string()),
        ("directory_syncs", counters.directory_syncs().to_string()),
        (
            "file_state_sync_attempts",
            counters
                .attempts_for(worth_store_physical_backend::MediaOperationRole::SynchronizeFileState)
                .to_string(),
        ),
        (
            "directory_publication_sync_attempts",
            counters
                .attempts_for(worth_store_physical_backend::MediaOperationRole::SynchronizeDirectoryPublication)
                .to_string(),
        ),
        ("replacements", counters.replacements().to_string()),
        ("deletions", counters.deletions().to_string()),
        ("cleanup", counters.cleanup_actions().to_string()),
        (
            "ownership_releases",
            counters.ownership_releases().to_string(),
        ),
        ("live_files", counters.live_file_handles().to_string()),
        (
            "live_directories",
            counters.live_directory_handles().to_string(),
        ),
        (
            "create_new",
            counters
                .attempts_for(worth_store_physical_backend::MediaOperationRole::CreateNew)
                .to_string(),
        ),
        (
            "positioned_write",
            counters
                .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite)
                .to_string(),
        ),
        (
            "append",
            counters
                .attempts_for(worth_store_physical_backend::MediaOperationRole::Append)
                .to_string(),
        ),
        (
            "truncate",
            counters
                .attempts_for(worth_store_physical_backend::MediaOperationRole::Truncate)
                .to_string(),
        ),
        (
            "allocate",
            counters
                .attempts_for(worth_store_physical_backend::MediaOperationRole::Allocate)
                .to_string(),
        ),
        (
            "atomic_replace",
            counters
                .attempts_for(worth_store_physical_backend::MediaOperationRole::AtomicReplace)
                .to_string(),
        ),
        (
            "delete",
            counters
                .attempts_for(worth_store_physical_backend::MediaOperationRole::Delete)
                .to_string(),
        ),
        ("conserved", counters.is_conserved().to_string()),
    ]
}

pub(crate) fn assert_initialization_counters(report: &ChildReport) {
    assert_eq!(report.value("conserved"), "true");
    assert_eq!(
        report.number("attempted"),
        report.number("completed")
            + report.number("denied")
            + report.number("partial")
            + report.number("indeterminate")
    );
    assert!(report.number("completed_bytes") <= report.number("requested_bytes"));
    assert!(report.number("file_syncs") >= 1);
    assert!(report.number("directory_syncs") >= 1);
    assert!(report.number("replacements") >= 1);
    assert_closed_resource_conservation(report);
    {
        assert_eq!(report.number("qualification"), 1);
        assert_eq!(report.number("cleanup"), 3);
        assert_eq!(report.number("positioned_write"), 18);
        assert_eq!(report.number("append"), 1);
        assert_eq!(report.number("truncate"), 1);
        assert_eq!(report.number("allocate"), 1);
        assert_eq!(report.number("file_state_sync_attempts"), 3);
        assert_eq!(report.number("directory_publication_sync_attempts"), 4);
    }
    #[cfg(not(all()))]
    assert_eq!(report.number("qualification"), 0);
}

pub(crate) fn assert_reopen_has_no_initialization_effects(report: &ChildReport) {
    assert_eq!(report.value("conserved"), "true");
    assert_eq!(report.number("create_new"), 0);
    assert_eq!(report.number("positioned_write"), 0);
    assert_eq!(report.number("append"), 0);
    assert_eq!(report.number("truncate"), 0);
    assert_eq!(report.number("allocate"), 0);
    assert_eq!(report.number("atomic_replace"), 0);
    assert_eq!(report.number("delete"), 0);
    assert_eq!(report.number("qualification"), 0);
    assert_eq!(report.number("file_syncs"), 0);
    assert_eq!(report.number("directory_syncs"), 3);
    assert_eq!(report.number("replacements"), 0);
    assert_eq!(report.number("deletions"), 0);
    assert_eq!(report.number("cleanup"), 0);
    assert_closed_resource_conservation(report);
}

pub(crate) fn assert_closed_resource_conservation(report: &ChildReport) {
    assert_eq!(report.number("ownership_releases"), 1);
    assert_eq!(report.number("live_files"), 0);
    assert_eq!(report.number("live_directories"), 0);
}

pub(crate) fn assert_reopener_identity_independence(first: &ChildReport, second: &ChildReport) {
    for identity in ["process", "runtime", "owner", "attempt"] {
        assert_ne!(
            first.value(identity),
            second.value(identity),
            "fresh process reused {identity} identity"
        );
    }
}

pub(crate) fn sorted_child_paths(root: &Path) -> Vec<String> {
    let mut names = std::fs::read_dir(root)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    names.sort();
    names
}

pub(crate) fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
