use super::CallbackDependencyPatchSummary;

const FRAMEWORK_HOST_BACKING_READ_PREFIX: &str = "__WorthSignal.host.";

pub(crate) fn public_callback_read_ids(reads: &[String]) -> Vec<String> {
    reads
        .iter()
        .filter(|read| !read.starts_with(FRAMEWORK_HOST_BACKING_READ_PREFIX))
        .cloned()
        .collect()
}

pub(crate) fn public_callback_dependency_patch_summary(
    previous_reads: &[String],
    current_reads: &[String],
    runtime_read_breadth: u64,
) -> CallbackDependencyPatchSummary {
    let previous_reads = public_callback_read_ids(previous_reads);
    let current_reads = public_callback_read_ids(current_reads);
    let previous_set = previous_reads
        .iter()
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    let current_set = current_reads
        .iter()
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    let retained_count = previous_set.intersection(&current_set).count() as u64;
    let added_count = current_set.difference(&previous_set).count() as u64;
    let removed_count = previous_set.difference(&current_set).count() as u64;
    CallbackDependencyPatchSummary {
        previous_reads,
        current_reads,
        added_count,
        removed_count,
        retained_count,
        runtime_read_breadth,
    }
}
