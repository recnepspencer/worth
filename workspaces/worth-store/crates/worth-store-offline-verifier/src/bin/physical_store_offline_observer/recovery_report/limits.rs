use worth_store_offline_verifier::RecoveryObserverLimits;

use super::arguments::Invocation;

pub(super) fn build(invocation: &Invocation) -> Result<RecoveryObserverLimits, String> {
    let maximum_directory_entries = parse(
        "maximum-directory-entries",
        &invocation.maximum_directory_entries,
    )?;
    let maximum_directories = parse("maximum-directories", &invocation.maximum_directories)?;
    let maximum_artifacts = parse("maximum-artifacts", &invocation.maximum_artifacts)?;
    let maximum_bytes = parse("maximum-bytes", &invocation.maximum_bytes)?;
    RecoveryObserverLimits::new(
        maximum_directory_entries,
        maximum_directories,
        maximum_artifacts,
        maximum_bytes,
    )
    .map_err(|_| "observer limits must be nonzero".to_owned())
}

fn parse(name: &str, value: &str) -> Result<u64, String> {
    value
        .parse::<u64>()
        .map_err(|_| format!("{name} must be a positive integer"))
}
