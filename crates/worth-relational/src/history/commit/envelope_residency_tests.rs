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
    let runtime = runtime_with_test_schema();
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
fn an_indexed_envelope_remains_cold_until_explicit_readmission_and_loads_once() {
    let host = crate::execution::test_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let (entry, source) = selected(&execution, None);
    let expected = entry.commit.clone();
    let cell = RelationalCommitEnvelopeResidency::cold(
        super::RetainedCanonicalIndex::from_ordinary_owned(entry),
        source.clone(),
        None,
        None,
    );
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
    let host = crate::execution::test_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let (entry, source) = selected(&execution, Some(Cause::Damaged));
    let position = entry.position;
    let cell = RelationalCommitEnvelopeResidency::cold(
        super::RetainedCanonicalIndex::from_ordinary_owned(entry),
        source.clone(),
        None,
        None,
    );
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
    let host = crate::execution::test_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let (entry, source) = selected(&execution, None);
    let id = entry.commit.commit_id;
    let mut catalog = super::RelationalCommitCatalog::default();
    catalog
        .append_preencoded_recovery(Arc::new(
            super::RelationalCommitArtifact::from_index(
                super::RetainedCanonicalIndex::from_ordinary_owned(entry),
                source,
            )
            .unwrap(),
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
    let host = crate::execution::test_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    for journal_backed in [false, true] {
        let (entry, source) = selected(&execution, None);
        let performed = RelationalCommitEnvelopeResidency::readmit_positioned(
            &entry,
            source.as_ref(),
            &execution,
            1 << 30,
        )
        .unwrap();
        let position = performed.position();
        let body = performed.canonical_arc().clone();
        let weak = Arc::downgrade(&body);
        let original =
            crate::branch::RelationalOriginalRootCommitEvidence::prepare(&execution, body).unwrap();
        let cell = RelationalCommitEnvelopeResidency::resident(original);
        let reads = source.reads.load(Ordering::Relaxed);
        cell.bind_performed(
            &performed,
            journal_backed.then(|| {
                (
                    super::RetainedCanonicalIndex::from_ordinary_owned(entry.clone()),
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
            journal_backed.then_some(&entry)
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
    let host = crate::execution::test_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let (entry, source) = selected(&execution, None);
    let cell = RelationalCommitEnvelopeResidency::cold(
        super::RetainedCanonicalIndex::from_ordinary_owned(entry.clone()),
        source.clone(),
        None,
        None,
    );
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
    let host = crate::execution::test_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let (entry, source) = selected(&execution, None);
    let id = entry.commit.commit_id;
    let mut catalog = super::RelationalCommitCatalog::default();
    catalog
        .append_preencoded_recovery(Arc::new(
            super::RelationalCommitArtifact::from_index(
                super::RetainedCanonicalIndex::from_ordinary_owned(entry),
                source,
            )
            .unwrap(),
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
    let mut conflicting = envelope.copy_original_for_test(&execution);
    conflicting.body_mut_for_test().commit.version_id = crate::identity::data::VersionId(u64::MAX);
    assert_eq!(artifact.matches_envelope(&conflicting), Ok(false));
    let original =
        crate::branch::RelationalOriginalRootCommitEvidence::prepare(&execution, envelope).unwrap();
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

// Fixture adaptation (C2.4/C4): restored from 86e84dc5cd:memory_publication_tests.rs
// using case-owned `OwnedTestDurableMedium` (no runtime_attachment).
#[test]
fn journal_publication_keeps_journal_placement_without_memory_frame() {
    let host = crate::execution::test_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    let result = create_entity_outcome(&execution, &runtime, "unchanged-journal-frame");
    let cell = runtime
        .history
        .commit_artifact(result.commit.commit_id)
        .unwrap()
        .envelope_residency();
    let (entry, source) = cell.journal_placement().unwrap();
    assert!(entry.same_owner(&cell.journal_placement().unwrap().0));
    assert!(cell.resident_payload_bytes().is_none());
    assert_eq!(cell.payload_storage_bytes(), 0);
    let original = medium.latest_committed_artifact().unwrap();
    let exported = cell.export_artifact(&execution).unwrap();
    assert_eq!(exported.as_bytes(), original.as_bytes());
    assert!(Arc::ptr_eq(
        &source,
        &runtime.durability.cold_source().unwrap()
    ));
    release_test_commit_snapshot(&runtime, &result);
}

#[test]
fn recovered_nonhead_tail_envelopes_release_demanded_journal_body_on_drop() {
    let host = crate::execution::test_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    let entity = create_entity(&execution, &runtime, "tail-body-0");
    for n in 1..4 {
        update_entity_and_release_snapshot(&execution, &runtime, entity, &format!("tail-body-{n}"));
    }
    let mut recovered = durable_runtime_for_medium(&medium);
    recover_runtime_from_medium(&execution, &mut recovered, &medium);
    recovered.history.reclaim_retired_branch_roots(1024);
    let receipt = recovered
        .history()
        .immutable_commit_receipt_at_patch_stream_position(PatchStreamPosition(2))
        .unwrap();
    let body = recovered
        .history()
        .demand_commit_envelope(&execution, receipt.commit_id)
        .unwrap()
        .unwrap();
    let weak = Arc::downgrade(&body);
    drop(body);
    assert!(
        weak.upgrade().is_none(),
        "replay must not retain a recovered nonhead journal-backed body after drop"
    );
    assert_eq!(
        recovered
            .history()
            .demand_commit_envelope(&execution, receipt.commit_id)
            .unwrap()
            .unwrap()
            .commit,
        receipt
    );
}
