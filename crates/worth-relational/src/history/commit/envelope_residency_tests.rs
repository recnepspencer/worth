use super::envelope_residency::RelationalCommitEnvelopeResidency;
use super::RetainedCanonicalIndex;
#[cfg(test)]
use crate::durability::canonical_commit_artifact::checkpoint_body::copy_original_for_test;
use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact;
use crate::durability::medium::{RelationalDurableColdSource, RelationalDurableMemberLocator};
use crate::publication::patch::data::PatchStreamPosition;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeFixturePort as _;
use worth_relational_branch::internal_port::history::data::positioned_canonical_commit::PositionedCanonicalCommitFixturePort as _;

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
) -> (RetainedCanonicalIndex, Arc<Source>) {
    use crate::branch::target::RelationalBranchRootDescriptor;
    use crate::durability::canonical_commit_artifact::journal_source_read::readmit_journal_commit;
    use crate::history::data::PositionedCanonicalCommit;
    use crate::identity::data::{EntityId, PartitionId};
    use crate::transactions::data::primitives::RecordRef;
    use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::tests::envelope_with_patch_and_update_for_test as envelope_with_patch_and_update;

    let entity = EntityId::new(PartitionId::main(), 1, 1);
    let canonical = Arc::new(envelope_with_patch_and_update(
        RecordRef::Entity(entity),
        entity,
    ));
    let positioned =
        PositionedCanonicalCommit::positioned_commit_for_test(PatchStreamPosition(1), canonical);
    let bytes = RelationalCanonicalCommitArtifact::from_positioned(
        &positioned,
        execution.allocation_policy(),
    )
    .unwrap();
    let index = RetainedCanonicalIndex::from_publication(
        &positioned,
        &RelationalBranchRootDescriptor::new(
            [3; 32],
            crate::schema::data::schema_authority_snapshot_digest_bytes(
                &positioned.schema_authority,
            ),
        ),
        *bytes.payload_digest().as_bytes(),
        execution,
        readmit_journal_commit,
    )
    .unwrap();
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
    let host = worth_harness::facade::execution_fixture::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let (entry, source) = selected(&execution, None);
    let expected = entry.commit.clone();
    let cell = RelationalCommitEnvelopeResidency::cold(entry, source.clone(), None, None);
    assert!(cell.get().is_none());
    assert_eq!(source.reads.load(Ordering::Relaxed), 0);
    let first = cell.demand(&execution, 1 << 30).unwrap();
    assert_eq!(first.commit, expected);
    assert!(Arc::ptr_eq(
        &first,
        &cell.demand(&execution, 1 << 30).unwrap()
    ));
    assert_eq!(source.reads.load(Ordering::Relaxed), 1);
}

#[test]
fn an_indexed_envelope_storage_failure_is_typed_and_sticky() {
    let host = worth_harness::facade::execution_fixture::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let (entry, source) = selected(&execution, Some(Cause::Damaged));
    let position = entry.position;
    let cell = RelationalCommitEnvelopeResidency::cold(entry, source.clone(), None, None);
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

#[test]
fn relinking_other_envelopes_preserves_an_index_even_while_its_body_is_in_flight() {
    let host = worth_harness::facade::execution_fixture::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let (entry, source) = selected(&execution, None);
    let id = entry.commit.commit_id;
    let mut catalog = super::RelationalCommitCatalog::default();
    catalog
        .append_preencoded_recovery(Arc::new(
            super::RelationalCommitArtifact::from_index(entry, source).unwrap(),
        ))
        .unwrap();
    let body = catalog
        .get(id)
        .unwrap()
        .demand_envelope(&execution, 1 << 30)
        .unwrap();
    let retained = catalog.retained_entries_except(&std::collections::BTreeSet::new());
    assert_eq!(retained.get(id).unwrap().receipt(), body.commit);
    let weak = Arc::downgrade(&body);
    drop(body);
    assert!(weak.upgrade().is_none());
    assert!(retained.get(id).unwrap().resident_envelope().is_none());
    assert!(
        catalog
            .retained_entries_except(&std::collections::BTreeSet::from([id]))
            .get(id)
            .is_none(),
        "only explicitly replaced identities leave the projection"
    );
}

#[test]
fn settlement_releases_a_journal_flight_but_preserves_memory_history_and_position() {
    let host = worth_harness::facade::execution_fixture::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    for journal_backed in [false, true] {
        let (entry, source) = selected(&execution, None);
        let performed = entry
            .readmit_journal_for_test(source.as_ref(), &execution, 1 << 30)
            .unwrap();
        let position = performed.position();
        let body = performed.canonical_arc().clone();
        let weak = Arc::downgrade(&body);
        let original =
            crate::history::commit::original_payload::RelationalOriginalRootCommitEvidence::prepare(&execution, body).unwrap();
        let cell = RelationalCommitEnvelopeResidency::resident(original);
        let reads = source.reads.load(Ordering::Relaxed);
        cell.bind_performed(
            &performed,
            journal_backed.then(|| {
                (
                    entry.clone(),
                    source.clone() as Arc<dyn RelationalDurableColdSource>,
                )
            }),
            None,
        );
        drop(performed);
        assert_eq!(cell.positioned_if_resident().unwrap().position(), position);
        assert!(
            weak.upgrade().is_some(),
            "unsettled publication retains its exact body"
        );
        assert_eq!(
            source.reads.load(Ordering::Relaxed),
            reads,
            "binding never loads"
        );
        cell.settle();
        assert_eq!(weak.upgrade().is_none(), journal_backed);
        assert_eq!(
            source.reads.load(Ordering::Relaxed),
            reads,
            "settlement never loads"
        );
        assert_eq!(
            cell.journal_index().as_deref(),
            journal_backed.then_some(entry.as_ref())
        );
        let flight = cell.demand(&execution, 1 << 30).unwrap();
        assert_eq!(flight.commit, entry.commit);
        assert_eq!(cell.positioned_if_resident().unwrap().position(), position);
        if journal_backed {
            assert!(source.reads.load(Ordering::Relaxed) > reads);
        } else {
            assert_eq!(source.reads.load(Ordering::Relaxed), reads);
        }
    }
}

#[test]
fn positioned_demand_owns_its_result_across_journal_settlement() {
    let host = worth_harness::facade::execution_fixture::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let (entry, source) = selected(&execution, None);
    let cell = RelationalCommitEnvelopeResidency::cold(entry.clone(), source.clone(), None, None);
    let first = cell.demand_positioned(&execution, 1 << 30).unwrap();
    cell.settle();
    assert_eq!(first.position(), entry.position);
    assert_eq!(first.commit, entry.commit);
    assert_eq!(source.reads.load(Ordering::Relaxed), 1);
    assert_eq!(cell.demand_positioned(&execution, 1 << 30).unwrap(), first);
    assert_eq!(source.reads.load(Ordering::Relaxed), 1);
}

#[test]
fn unpositioned_memory_envelopes_and_cold_digests_refuse_explicitly() {
    let host = worth_harness::facade::execution_fixture::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let (entry, source) = selected(&execution, None);
    let id = entry.commit.commit_id;
    let mut catalog = super::RelationalCommitCatalog::default();
    catalog
        .append_preencoded_recovery(Arc::new(
            super::RelationalCommitArtifact::from_index(entry, source).unwrap(),
        ))
        .unwrap();
    let artifact = catalog.get(id).unwrap();
    assert_eq!(
        artifact.canonical_payload_digest(),
        Err(super::RelationalCommitArtifactDenial::CanonicalPayloadEncoding)
    );
    let envelope = artifact.demand_envelope(&execution, 1 << 30).unwrap();
    assert!(artifact.canonical_payload_digest().is_ok());
    assert_eq!(artifact.matches_envelope(&envelope), Ok(true));
    let mut conflicting = copy_original_for_test(&envelope, &execution);
    conflicting.body_mut_for_test().commit.version_id = crate::identity::data::VersionId(u64::MAX);
    assert_eq!(artifact.matches_envelope(&conflicting), Ok(false));
    let original =
        crate::history::commit::original_payload::RelationalOriginalRootCommitEvidence::prepare(
            &execution, envelope,
        )
        .unwrap();
    let cell = RelationalCommitEnvelopeResidency::resident(original);
    let Err(crate::durability::data::recovery_stop::RelationalRecoveryStop::Durability(failure)) =
        cell.demand_positioned(&execution, 1 << 30)
    else {
        panic!("an unpositioned memory envelope must refuse typed checkpoint reconstruction");
    };
    assert_eq!(
        failure.class,
        crate::durability::data::recovery_errors::RecoveryFailureClass::CorruptCheckpoint
    );
}
