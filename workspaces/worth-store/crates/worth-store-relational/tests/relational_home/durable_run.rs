//! Running a script against a durable home and comparing it with the
//! reference.

use std::path::Path;

use worth_execution::{ExecutionAllocationPolicy, ExecutionRequest};
use worth_store_relational::{
    RelationalStoreHome, RelationalStoreHomeOpenRefusal, RelationalStoreHomeRequest,
};

use super::workload::{self, Script, Step};

pub(crate) fn open(
    location: &Path,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<RelationalStoreHome, RelationalStoreHomeOpenRefusal> {
    RelationalStoreHome::open(
        RelationalStoreHomeRequest::new(location, workload::runtime_builder()),
        execution,
    )
}

/// Apply `steps` to the home, returning how many commits were acknowledged.
pub(crate) fn apply(
    home: &RelationalStoreHome,
    execution: &ExecutionRequest<'_, '_>,
    steps: &[Step],
) -> usize {
    let mut acknowledged = 0;
    for step in steps {
        match step {
            Step::Commit { branch, batch } => {
                workload::commit(home.runtime(), execution, branch, batch.clone())
                    .unwrap_or_else(|error| panic!("durable commit {acknowledged}: {error:?}"));
                acknowledged += 1;
            }
            Step::Fork { target } => {
                workload::fork_branch(
                    home.runtime(),
                    &worth_relational::facade::history::BranchId("main".to_owned()),
                    target,
                );
            }
            Step::Checkpoint => {
                home.checkpoint(ExecutionAllocationPolicy::SystemAllocation)
                    .expect("home checkpoint");
            }
        }
    }
    acknowledged
}

/// Require the home to hold exactly the commits the durable run itself
/// acknowledged (`acknowledged`, byte-identical canonical artifacts), and the
/// reference's record states after that many commits. Artifacts embed the
/// producing runtime's process-local identity, so they compare against the
/// durable run's own; record states compare against the reference.
pub(crate) fn assert_holds(home: &RelationalStoreHome, script: &Script, acknowledged: &[Vec<u8>]) {
    let artifacts = workload::artifacts_of(home.runtime());
    let commits = acknowledged.len();
    assert_eq!(artifacts.len(), commits, "recovered commit count");
    for (index, (found, expected)) in artifacts.iter().zip(acknowledged).enumerate() {
        assert!(
            found == expected,
            "artifact at position {} differs",
            index + 1
        );
    }
    assert_eq!(
        workload::states_of(home.runtime(), &script.records),
        script.states[commits],
        "branch states after {commits} commits"
    );
}

/// The steps that follow the `commits`-th commit (and any checkpoint after it).
pub(crate) fn steps_after(script: &Script, commits: usize) -> &[Step] {
    let mut seen = 0;
    for (index, step) in script.steps.iter().enumerate() {
        if seen == commits && !matches!(step, Step::Checkpoint) {
            return &script.steps[index..];
        }
        if matches!(step, Step::Commit { .. }) {
            seen += 1;
        }
    }
    &[]
}

/// The steps up to and including the `commits`-th commit.
pub(crate) fn steps_before(script: &Script, commits: usize) -> &[Step] {
    let rest = steps_after(script, commits).len();
    &script.steps[..script.steps.len() - rest]
}
