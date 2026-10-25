use super::envelope_residency::RelationalCommitEnvelopeResidency;
use super::RelationalCommitIndexEntry;
use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact;
use crate::durability::medium::{RelationalDurableColdSource, RelationalDurableMemberLocator};
use crate::publication::patch::data::PatchStreamPosition;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::tests::support::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

#[derive(Debug)]
struct Source {
    artifact: RelationalCanonicalCommitArtifact,
    reads: AtomicUsize,
    failure: Option<Cause>,
}
impl RelationalDurableColdSource for Source {
    fn read_member(&self, _: RelationalDurableMemberLocator, _: u64) -> Result<Vec<u8>, Cause> {
        panic!("envelope demand reads the journal only")
    }
    fn read_commit(&self, position: PatchStreamPosition, limit: u64) -> Result<Vec<u8>, Cause> {
        self.reads.fetch_add(1, Ordering::Relaxed);
        assert_eq!(position, self.artifact.position());
        if let Some(failure) = self.failure {
            return Err(failure);
        }
        assert!(self.artifact.as_bytes().len() as u64 <= limit);
        Ok(self.artifact.as_bytes().to_vec())
    }
}

fn selected(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    failure: Option<Cause>,
) -> (RelationalCommitIndexEntry, Arc<Source>) {
    let runtime = durable_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "indexed-envelope-record");
    let commit = runtime
        .history
        .positioned_canonical_commits_snapshot()
        .pop()
        .unwrap();
    let artifact = runtime
        .history
        .commit_artifact(commit.commit.commit_id)
        .unwrap();
    let bytes = RelationalCanonicalCommitArtifact::from_positioned(
        &commit,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .unwrap();
    let index = RelationalCommitIndexEntry {
        commit: commit.commit.clone(),
        position: commit.position(),
        root: artifact.roots().clone(),
        payload_digest: *bytes.payload_digest().as_bytes(),
        schema_version: commit.schema_version,
    };
    (
        index,
        Arc::new(Source {
            artifact: bytes,
            reads: AtomicUsize::new(0),
            failure,
        }),
    )
}

#[test]
fn an_indexed_envelope_remains_cold_until_explicit_readmission_and_loads_once() {
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let (entry, source) = selected(&execution, None);
    let expected = entry.commit.clone();
    let cell = RelationalCommitEnvelopeResidency::cold(entry, source.clone(), None);
    assert!(cell.get().is_none());
    assert_eq!(source.reads.load(Ordering::Relaxed), 0);
    let first = cell.demand(&execution, 1 << 30).unwrap();
    assert_eq!(first.commit, expected);
    assert!(Arc::ptr_eq(
        first,
        cell.demand(&execution, 1 << 30).unwrap()
    ));
    assert_eq!(source.reads.load(Ordering::Relaxed), 1);
}

#[test]
fn an_indexed_envelope_storage_failure_is_typed_and_sticky() {
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let (entry, source) = selected(&execution, Some(Cause::Damaged));
    let position = entry.position;
    let cell = RelationalCommitEnvelopeResidency::cold(entry, source.clone(), None);
    for _ in 0..2 {
        let stop = cell.demand(&execution, 1 << 30).unwrap_err();
        let crate::storage::residency::RelationalResidencyDemandFailure::StorageUnavailable(
            failure,
        ) = stop
        else {
            panic!("typed storage refusal")
        };
        assert_eq!(
            failure,
            crate::storage::data::RelationalStorageUnavailable {
                subject: crate::storage::data::RelationalStorageSubject::Envelope(position),
                cause: Cause::Damaged,
            }
        );
    }
    assert_eq!(source.reads.load(Ordering::Relaxed), 1);
}
