use super::*;
use crate::durability::medium::{RelationalDurableColdSource, RelationalDurableMemberLocator};
use crate::publication::patch::data::PatchStreamPosition;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::tests::support::*;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug)]
struct SelectedJournal {
    last: Vec<u8>,
    reads: AtomicUsize,
}
impl RelationalDurableColdSource for SelectedJournal {
    fn read_member(&self, _: RelationalDurableMemberLocator, _: u64) -> Result<Vec<u8>, Cause> {
        Err(Cause::SourceUnavailable)
    }
    fn read_commit(&self, position: PatchStreamPosition, _: u64) -> Result<Vec<u8>, Cause> {
        self.reads.fetch_add(1, Ordering::Relaxed);
        if position == PatchStreamPosition(4) {
            Ok(self.last.clone())
        } else {
            Err(Cause::SourceUnavailable)
        }
    }
}

#[test]
fn seeking_past_an_acknowledged_index_prefix_never_demands_its_journal_bodies() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let runtime = durable_runtime_with_test_schema();
    for name in ["prefix-a", "prefix-b", "prefix-c", "selected-tail"] {
        create_entity(execution, &runtime, name);
    }
    let artifacts = runtime
        .durability()
        .canonical_commit_artifacts(ExecutionAllocationPolicy::SystemAllocation)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let source = Arc::new(SelectedJournal {
        last: artifacts[3].as_bytes().to_vec(),
        reads: AtomicUsize::new(0),
    });
    let catalog = runtime.history.commit_artifacts();
    let retained = artifacts
        .iter()
        .map(|artifact| {
            let owner = catalog
                .iter()
                .find(|owner| owner.identity().commit_id() == artifact.commit_id())
                .unwrap();
            RetainedArtifact::Cold {
                entry: crate::history::RelationalCommitIndexEntry {
                    commit: owner.receipt(),
                    position: artifact.position(),
                    root: owner.roots().clone(),
                    payload_digest: *artifact.payload_digest().as_bytes(),
                    schema_version: owner.schema_version(),
                },
                source: source.clone(),
            }
        })
        .collect::<Vec<_>>();
    let iter = RelationalCanonicalCommitArtifacts {
        retained: retained.into_iter(),
        policy: ExecutionAllocationPolicy::SystemAllocation,
    };
    let selected = iter
        .after_position(PatchStreamPosition(3))
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].as_bytes(), artifacts[3].as_bytes());
    assert_eq!(source.reads.load(Ordering::Relaxed), 1);
}
