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
    fn retain_members(
        &self,
        _members: &[RelationalDurableMemberLocator],
    ) -> Result<crate::durability::medium::RelationalDurableMemberCustody, Cause> {
        Ok(crate::durability::medium::RelationalDurableMemberCustody::new(()))
    }

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
        association_discovery: commit.envelope().association_discovery(),
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
fn unpositioned_memory_envelopes_and_cold_digests_refuse_explicitly() {
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let (entry, source) = selected(&execution, None);
    let id = entry.commit.commit_id;
    let mut catalog = super::RelationalCommitCatalog::default();
    catalog
        .append_indexed_recovery(
            super::RetainedCanonicalIndex::from_ordinary_owned(entry),
            source,
            None,
        )
        .unwrap();
    let artifact = catalog.get(id).unwrap();
    assert_eq!(
        artifact.canonical_payload_digest(),
        Err(super::RelationalCommitArtifactDenial::CanonicalPayloadEncoding)
    );
    let envelope = artifact.demand_envelope(&execution, 1 << 30).unwrap();
    assert!(artifact.canonical_payload_digest().is_ok());
    assert_eq!(artifact.matches_envelope(&envelope), Ok(true));
    let mut conflicting = envelope.copy_original_for_test(&execution);
    conflicting.body_mut_for_test().commit.version_id = crate::identity::data::VersionId(u64::MAX);
    assert_eq!(artifact.matches_envelope(&conflicting), Ok(false));
    let payload = envelope.authoritative_payload_observation().unwrap();
    let cell = RelationalCommitEnvelopeResidency::resident(envelope, payload);
    let Err(crate::durability::data::RelationalRecoveryStop::Durability(failure)) =
        cell.demand_positioned(&execution, 1 << 30)
    else {
        panic!("an unpositioned memory envelope must refuse typed checkpoint reconstruction");
    };
    assert_eq!(
        failure.class,
        crate::durability::data::RecoveryFailureClass::CorruptCheckpoint
    );
}
