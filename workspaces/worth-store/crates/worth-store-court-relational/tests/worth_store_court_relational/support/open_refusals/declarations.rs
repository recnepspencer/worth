//! Open refusals: a home that is not this runtime's, a location holding
//! something other than a Store, and a home whose checkpoint or journal is
//! damaged all refuse before service, with a typed refusal, and none of them
//! is treated as an empty home.

pub(crate) use std::fs;
pub(crate) use {std::path::Path, std::path::PathBuf};

pub(crate) use worth_execution::ExecutionRequest;
pub(crate) use worth_relational::facade::runtime::RelationalRuntimeApi;
pub(crate) use {
    worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
    worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    worth_store::physical_runtime::AnchoredRecordName,
    worth_store::physical_runtime::AnchoredRecordPublication,
    worth_store::physical_runtime::ManifestEntryCapacity,
    worth_store::physical_runtime::OrderedJournalPublication,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalRecordFormatDeclaration,
    worth_store::physical_runtime::PhysicalRecordPlacementPolicy,
};
pub(crate) use {
    worth_store_relational::RelationalStoreHome,
    worth_store_relational::RelationalStoreHomeOpenRefusal,
    worth_store_relational::RelationalStoreHomeRequest,
    worth_store_relational::RelationalStoreJournalRefusal,
};

pub(crate) use crate::schema_transition;
pub(crate) use crate::selected_checkpoint;
pub(crate) use {
    crate::durable_run::apply, crate::durable_run::assert_holds, crate::durable_run::open,
    crate::durable_run::steps_before,
};
pub(crate) use {crate::workload, crate::workload::Script};

pub(crate) const SEED: u64 = 0x5eed_0501;
pub(crate) const COMMITS: usize = 6;
pub(crate) const CHECKPOINT_MAGIC: &[u8] = b"WSRCKPT1";
pub(crate) const ARTIFACT_MAGIC: &[u8] = b"WRTHRCCA";

/// A closed home holding `commits` commits of the script, with a checkpoint
/// after the third.
pub(crate) fn home_with(
    commits: usize,
    execution: &ExecutionRequest<'_, '_>,
) -> (tempfile::TempDir, PathBuf, Script, Vec<Vec<u8>>) {
    let script = crate::workload::Script::generate(SEED, COMMITS, 3);
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let home = open(&location, execution).expect("a new home opens");
    apply(&home, execution, steps_before(&script, commits));
    let acknowledged = crate::workload::artifacts_of(home.runtime());
    home.close();
    (directory, location, script, acknowledged)
}

/// Flip one byte `offset` bytes after every occurrence of `pattern` in every
/// file under `root`. Returns how many bytes were flipped.
pub(crate) fn flip_after(
    root: &Path,
    pattern: &[u8],
    offset: usize,
    matches: &dyn Fn(&[u8]) -> bool,
) -> usize {
    let mut flipped = 0;
    let paths = if root.is_file() {
        vec![root.to_owned()]
    } else {
        std::fs::read_dir(root)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect()
    };
    for path in paths {
        if path.is_dir() {
            flipped += flip_after(&path, pattern, offset, matches);
            continue;
        }
        let mut bytes = std::fs::read(&path).unwrap();
        let mut changed = false;
        let mut start = 0;
        while let Some(found) = bytes[start..]
            .windows(pattern.len())
            .position(|window| window == pattern)
        {
            let at = start + found;
            if at + offset < bytes.len() && matches(&bytes[at..]) {
                bytes[at + offset] ^= 0x5a;
                changed = true;
                flipped += 1;
            }
            start = at + pattern.len();
        }
        if changed {
            std::fs::write(&path, bytes).unwrap();
        }
    }
    flipped
}

pub(crate) fn assert_refuses_before_service(
    location: &Path,
    case: &str,
) -> RelationalStoreHomeOpenRefusal {
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    match open(location, &execution) {
        Ok(home) => panic!(
            "{case}: a damaged home served {} commits",
            crate::workload::artifacts_of(home.runtime()).len()
        ),
        Err(refusal) => refusal,
    }
}

/// The placement a home publishes under: the default record format with a
/// manifest capacity of 64.
pub(crate) fn home_placement() -> AdmittedRecordPlacementPolicy {
    worth_store::physical_runtime::PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(worth_store::physical_runtime::ManifestEntryCapacity::new(64).unwrap())
        .admit(
            worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(
                worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder()
                    .admit()
                    .unwrap(),
            ),
        )
        .unwrap()
}

pub(crate) fn deadline() -> PhysicalMutationDeadline {
    worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000).unwrap()
}
