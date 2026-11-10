//! Durable support rows join the actual protected World, not a Native receipt.
use std::num::NonZeroU64;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{
    CanonicalDigestId, DurableCompositePublicationAttemptIdentity,
    ProgramSupportObligationIdentity, ProgramSupportObligationKind, WorldOriginalFate,
};
use worth_query_execution::facade::program_support::{
    QueryObligationPublicationOutcome as Outcome, QueryObligationsMedium,
    QueryProgramSupportDenial as Denial, QueryProgramSupportFrame, QueryProgramSupportLedger,
    QueryProgramSupportObligation, QueryProgramSupportPublicationGenerations,
    QueryProgramSupportRecord as Record, QUERY_PROGRAM_SUPPORT_FRAME_MAX_BYTES,
};
use worth_runtime_world::facade::*;

type Fixture = worth_store_testkit_application_oracle::world_home::WorldHomeFixture;
pub(super) struct MembershipJournal {
    medium: Arc<dyn QueryObligationsMedium>,
    pub(super) ledger: QueryProgramSupportLedger,
    generations: QueryProgramSupportPublicationGenerations,
}
impl MembershipJournal {
    fn read(fixture: &Fixture, execution: &ExecutionRequest<'_, '_>) -> Self {
        Self::read_home(&fixture.home, execution)
    }
    pub(super) fn read_home(
        home: &worth_store_application::WorldReadmittedApplicationHome<(), (), (), (), ()>,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Self {
        let medium = home.query_obligations_medium();
        let mut ledger = QueryProgramSupportLedger::empty(home.home_identity());
        while let Some(bytes) = medium.read_frame(execution, ledger.position() + 1).unwrap() {
            assert!(bytes.len() as u64 <= QUERY_PROGRAM_SUPPORT_FRAME_MAX_BYTES);
            ledger
                .readmit(QueryProgramSupportFrame::readmit(&bytes).unwrap())
                .unwrap();
        }
        let generations = QueryProgramSupportPublicationGenerations::from_journal(&ledger);
        Self {
            medium,
            ledger,
            generations,
        }
    }
    pub(super) fn read_frame(
        &self,
        position: u64,
        execution: &ExecutionRequest<'_, '_>,
    ) -> QueryProgramSupportFrame {
        let bytes = self
            .medium
            .read_frame(execution, position)
            .unwrap()
            .unwrap();
        QueryProgramSupportFrame::readmit(&bytes).unwrap()
    }
    fn publish(&mut self, record: Record, execution: &ExecutionRequest<'_, '_>) {
        let frame = self.ledger.prepare(record, &mut self.generations).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            match self.medium.advance_publication(execution, &frame) {
                Outcome::Committed(proof) => {
                    proof.consume(|publication| {
                        let subject = publication.subject();
                        assert_eq!(subject.admit_to(self.medium.authority()), Ok(()));
                        assert_eq!(subject.position(), frame.position());
                        assert_eq!(subject.payload_digest(), frame.payload_digest());
                    });
                    self.ledger.readmit(frame).unwrap();
                    break;
                }
                Outcome::Pending => {
                    assert!(std::time::Instant::now() < deadline);
                    std::thread::yield_now();
                }
                other => panic!("actual support publication: {other:?}"),
            }
        }
    }
    pub(super) fn acquire(
        &mut self,
        original: DurableCompositePublicationAttemptIdentity,
        revision: CanonicalDigestId,
        execution: &ExecutionRequest<'_, '_>,
    ) -> ProgramSupportObligationIdentity {
        if self.ledger.status(&revision).is_none() {
            self.publish(Record::Install { revision }, execution);
        }
        let identity = ProgramSupportObligationIdentity::from_recorded(
            self.ledger.home(),
            NonZeroU64::new(self.ledger.position() + 1).unwrap(),
        );
        self.publish(
            Record::Acquire(QueryProgramSupportObligation {
                identity,
                kind: ProgramSupportObligationKind::OriginalInterpretation,
                source: revision,
                target: revision,
                original,
            }),
            execution,
        );
        identity
    }
}
fn source(
    fixture: &Fixture,
    execution: &ExecutionRequest<'_, '_>,
) -> WorldProgramActivationHistory {
    fixture
        .home
        .world()
        .inspection_port()
        .observe_program_activation_history(execution)
        .unwrap()
}

#[test]
fn original_interpretation_membership_matches_the_protected_original_after_graceful_reopen() {
    let directory = tempfile::tempdir().unwrap();
    let foreign_directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let fixture = super::world_occurrence_readmission::open(directory.path(), &execution);
    let root = fixture.bootstrap_root(&execution);
    let expected = root.product_branch().clone();
    drop(root);
    let cancellation = RuntimeWorldCancellationSource::new();
    let prepared = fixture.prepare_publication(
        &execution,
        expected,
        crate::batches::create(0),
        &cancellation.token(),
    );
    let attempt = DurableCompositePublicationAttemptIdentity::from_recorded(
        fixture.home.world().owner_identity().durable().unwrap(),
        prepared.attempt_identity().ordinal(),
    );
    let originals = source(&fixture, &execution);
    let revision = originals
        .original_state(attempt)
        .unwrap()
        .program_revision();
    let mut journal = MembershipJournal::read(&fixture, &execution);
    let identity = journal.acquire(attempt, revision, &execution);
    assert_eq!(journal.ledger.counts(&revision), (1, 0));
    assert!(journal
        .ledger
        .released_original_interpretations(&originals)
        .unwrap()
        .is_empty());
    drop(originals);
    let performed = fixture.publish(&execution, prepared, &cancellation.token());
    drop(performed.consume());
    drop(journal);
    super::world_occurrence_readmission::close(fixture.home, &execution);
    let fixture = super::world_occurrence_readmission::open(directory.path(), &execution);
    let originals = source(&fixture, &execution);
    assert_eq!(
        originals.original_state(attempt).unwrap().fate(),
        WorldOriginalFate::Published
    );
    let journal = MembershipJournal::read(&fixture, &execution);
    assert_eq!(journal.ledger.counts(&revision), (1, 0));
    assert_eq!(
        journal.ledger.obligations().next().unwrap().identity,
        identity
    );
    assert!(journal
        .ledger
        .released_original_interpretations(&originals)
        .unwrap()
        .is_empty());
    // A different real directory source cannot validate this original, even
    // though its owner has an actual admitted bootstrap.
    let foreign = super::world_occurrence_readmission::open(foreign_directory.path(), &execution);
    drop(foreign.bootstrap_root(&execution));
    assert!(
        matches!(journal.ledger.released_original_interpretations(&source(&foreign, &execution)),
        Err(Denial::MissingOriginalInterpretationSource { identity: refused }) if refused == identity)
    );
    // The row is DATA, so counterfeit revision content must be refused against
    // the actual original instead of being promoted to interpretation authority.
    let changed = CanonicalDigestId::new([0xe7; 32]);
    assert_ne!(changed, revision);
    let mut false_ledger = QueryProgramSupportLedger::empty(fixture.home.home_identity());
    let mut generations = QueryProgramSupportPublicationGenerations::from_journal(&false_ledger);
    let install = false_ledger
        .prepare(Record::Install { revision: changed }, &mut generations)
        .unwrap();
    false_ledger.readmit(install).unwrap();
    let false_identity = ProgramSupportObligationIdentity::from_recorded(
        false_ledger.home(),
        NonZeroU64::new(2).unwrap(),
    );
    let acquired = false_ledger
        .prepare(
            Record::Acquire(QueryProgramSupportObligation {
                identity: false_identity,
                kind: ProgramSupportObligationKind::OriginalInterpretation,
                source: changed,
                target: changed,
                original: attempt,
            }),
            &mut generations,
        )
        .unwrap();
    false_ledger.readmit(acquired).unwrap();
    assert!(
        matches!(false_ledger.released_original_interpretations(&originals),
        Err(Denial::OriginalInterpretationProgramMismatch { identity: refused }) if refused == false_identity)
    );
    assert_eq!(journal.ledger.counts(&revision), (1, 0));
    drop((journal, originals));
    super::world_occurrence_readmission::close(fixture.home, &execution);
    super::world_occurrence_readmission::close(foreign.home, &execution);
}

#[test]
fn released_no_effect_original_disposes_its_durable_interpretation_membership() {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let fixture = super::world_occurrence_readmission::open(directory.path(), &execution);
    let root = fixture.bootstrap_root(&execution);
    let expected = root.product_branch().clone();
    drop(root);
    let cancellation = RuntimeWorldCancellationSource::new();
    let prepared = fixture.prepare_publication(
        &execution,
        expected,
        crate::batches::create(0),
        &cancellation.token(),
    );
    let attempt = DurableCompositePublicationAttemptIdentity::from_recorded(
        fixture.home.world().owner_identity().durable().unwrap(),
        prepared.attempt_identity().ordinal(),
    );
    let revision = source(&fixture, &execution)
        .original_state(attempt)
        .unwrap()
        .program_revision();
    let mut journal = MembershipJournal::read(&fixture, &execution);
    let identity = journal.acquire(attempt, revision, &execution);
    // Establish a real released original through the current recovery owner.
    // This setup is not the graceful membership-readmission claim below.
    drop((prepared, journal, fixture));
    let fixture = super::world_occurrence_readmission::open(directory.path(), &execution);
    let page = fixture
        .home
        .world()
        .inspection_port()
        .recovery_page(None, std::num::NonZeroUsize::MIN)
        .unwrap();
    let [row] = page.rows() else {
        panic!("one actual acknowledged original");
    };
    let recovery = fixture.home.world().recovery_port();
    let claim = recovery.inspect_recovered_effects(row.handle()).unwrap();
    assert!(claim.native_receipt().is_none());
    let mut outcome = recovery.release_recovered_effects(&execution, claim);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        match outcome {
            WorldRecoveredReleaseOutcome::Released(released) => {
                drop(released);
                break;
            }
            WorldRecoveredReleaseOutcome::Pending(claim) => {
                assert!(std::time::Instant::now() < deadline);
                std::thread::yield_now();
                outcome = recovery.release_recovered_effects(&execution, claim);
            }
            other => panic!("actual no-effect original release: {other:?}"),
        }
    }
    drop(page);
    let retained = MembershipJournal::read(&fixture, &execution);
    assert_eq!(retained.ledger.counts(&revision), (1, 0));
    assert_eq!(
        source(&fixture, &execution)
            .original_state(attempt)
            .unwrap()
            .fate(),
        WorldOriginalFate::Released
    );
    drop(retained);
    // H9 consumer journey: the released original and its still-unreleased
    // Query obligation cross a genuine graceful close before disposal.
    super::world_occurrence_readmission::close(fixture.home, &execution);
    let fixture = super::world_occurrence_readmission::open(directory.path(), &execution);
    let originals = source(&fixture, &execution);
    let mut journal = MembershipJournal::read(&fixture, &execution);
    assert_eq!(journal.ledger.counts(&revision), (1, 0));
    let released = journal
        .ledger
        .released_original_interpretations(&originals)
        .unwrap();
    assert_eq!(released, vec![identity]);
    for identity in released {
        journal.publish(Record::Release { identity }, &execution);
    }
    assert_eq!(journal.ledger.counts(&revision), (0, 0));
    drop((journal, originals));
    super::world_occurrence_readmission::close(fixture.home, &execution);
    let fixture = super::world_occurrence_readmission::open(directory.path(), &execution);
    let journal = MembershipJournal::read(&fixture, &execution);
    assert_eq!(journal.ledger.position(), 3);
    assert_eq!(journal.ledger.counts(&revision), (0, 0));
    assert!(journal.ledger.obligations().next().is_none());
    drop(journal);
    super::world_occurrence_readmission::close(fixture.home, &execution);
}
