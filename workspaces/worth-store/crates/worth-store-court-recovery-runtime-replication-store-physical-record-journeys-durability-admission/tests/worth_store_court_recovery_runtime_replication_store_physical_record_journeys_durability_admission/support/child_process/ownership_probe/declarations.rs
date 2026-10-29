pub(crate) use {std::io::Write, std::path::Path};

pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_store::physical_runtime::FilesystemMediaAdmission, worth_store::physical_runtime::PhysicalRuntimeAdmission, worth_store::physical_runtime::PhysicalStore};
pub(crate) use {worth_store_physical_backend::FilesystemAccessPosture};

pub(crate) fn run(root: &Path) {
    let admitted = worth_store::physical_runtime::PhysicalRuntimeAdmission::new(root)
        .ok()
        .and_then(|request| worth_store::physical_runtime::PhysicalStore::admit(request).ok());
    let admitted = admitted.is_some_and(|runtime| {
        matches!(
            runtime
                .try_admit_filesystem_media(worth_store::physical_runtime::FilesystemMediaAdmission::production(
                    worth_store_physical_backend::FilesystemAccessPosture::CoordinatedServiceAccount,
                ))
                .into_raw(),
            worth_proof::TransitionOutcome::Success(_)
        )
    });
    println!(
        "C5_SECOND_OWNER {}",
        if admitted { "admitted" } else { "denied" }
    );
    std::io::stdout().flush().unwrap();
}