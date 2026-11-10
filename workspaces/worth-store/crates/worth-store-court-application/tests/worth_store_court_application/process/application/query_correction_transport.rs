//! The real Home transports opaque Query DATA; no correction admission is asserted.
use std::num::NonZeroU64;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::RetentionClass;
use worth_query_execution::facade::correction_material::{
    QueryCorrectionMaterialDenial as Denial, QueryCorrectionMaterialFrame as Frame,
    QueryCorrectionMaterialPublicationOutcome as Outcome, QueryCorrectionsMedium,
};
fn publish(
    medium: &dyn QueryCorrectionsMedium,
    frame: &Frame,
    execution: &ExecutionRequest<'_, '_>,
) -> worth_proof::DurablyCommitted<worth_proof::CommittedHomeQueryCorrection> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        match medium.advance_publication(execution, frame) {
            Outcome::Committed(proof) => return proof,
            Outcome::Pending => {
                assert!(std::time::Instant::now() < deadline);
                std::thread::yield_now();
            }
            Outcome::ProvenNoEffect(proof) => {
                panic!("ordinary publication unexpectedly had no effect: {proof:?}")
            }
            Outcome::Refused(cause) => panic!("ordinary opaque publication refused: {cause:?}"),
            Outcome::Unresolved => panic!("ordinary opaque publication has unresolved custody"),
        }
    }
}
#[test]
fn correction_journal_publication_survives_graceful_reopen_without_retaining_the_home() {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let fixture = super::world_occurrence_readmission::open(directory.path(), &execution);
    drop(fixture.bootstrap_root(&execution));
    let medium = fixture.home.query_corrections_medium();
    let home = medium.home();
    let incarnation = medium.incarnation();
    // These bytes test transport only, not a retained input/program association.
    let first = Frame::prepare(
        home,
        NonZeroU64::MIN,
        [0; 32],
        NonZeroU64::MIN,
        RetentionClass::OptionalRetainedMaterial,
        b"opaque-query-body-one",
    )
    .unwrap();
    publish(medium.as_ref(), &first, &execution).consume(|committed| {
        assert_eq!(committed.subject().admit_to(medium.authority()), Ok(()));
        assert_eq!(committed.subject().position(), first.position());
        assert_eq!(committed.subject().payload_digest(), first.payload_digest());
    });
    publish(medium.as_ref(), &first, &execution).consume(|committed| {
        assert_eq!(committed.subject().payload_digest(), first.payload_digest());
    });
    let second = Frame::prepare(
        home,
        NonZeroU64::new(2).unwrap(),
        first.digest(),
        NonZeroU64::new(2).unwrap(),
        RetentionClass::MandatoryObligation,
        b"opaque-query-body-two",
    )
    .unwrap();
    let prior_commit = publish(medium.as_ref(), &second, &execution);
    assert!(medium
        .read_frame(&execution, 3, u64::MAX)
        .unwrap()
        .is_none());
    super::world_occurrence_readmission::close(fixture.home, &execution);
    let fixture = super::world_occurrence_readmission::open(directory.path(), &execution);
    let current = fixture.home.query_corrections_medium();
    assert_eq!(current.home(), home);
    assert_ne!(current.incarnation(), incarnation);
    prior_commit.consume(|committed| {
        assert_eq!(
            committed.subject().admit_to(current.authority()),
            Err(worth_proof::HomeIncarnationDenial::StaleIncarnation)
        );
    });
    assert_eq!(
        medium.read_frame(&execution, 1, u64::MAX),
        Err(Denial::OwnerUnavailable)
    );
    for original in [&first, &second] {
        let bytes = current
            .read_frame(
                &execution,
                original.position(),
                original.bytes().len() as u64,
            )
            .unwrap()
            .unwrap();
        assert_eq!(bytes, original.bytes());
        let read = Frame::readmit(bytes).unwrap();
        assert_eq!(&read, original);
    }
    assert!(current
        .read_frame(&execution, 3, u64::MAX)
        .unwrap()
        .is_none());
    super::world_occurrence_readmission::close(fixture.home, &execution);
}
#[test]
fn correction_prefix_proofs_refuse_foreign_and_prior_incarnations_before_admission() {
    let directory = tempfile::tempdir().unwrap();
    let other_directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let fixture = super::world_occurrence_readmission::open(directory.path(), &execution);
    drop(fixture.bootstrap_root(&execution));
    let medium = fixture.home.query_corrections_medium();
    assert!(medium
        .read_frame(&execution, 1, u64::MAX)
        .unwrap()
        .is_none());
    let prior = medium
        .authority()
        .admit_query_correction_prefix(0, [0; 32], [0; 32]);
    fixture
        .home
        .admit_query_corrections(prior.clone(), &execution)
        .unwrap();
    super::world_occurrence_readmission::close(fixture.home, &execution);
    let fixture = super::world_occurrence_readmission::open(directory.path(), &execution);
    let current = fixture.home.query_corrections_medium();
    assert_eq!(
        fixture.home.admit_query_corrections(prior, &execution),
        Err(Denial::StaleIncarnation)
    );
    let foreign = super::world_occurrence_readmission::open(other_directory.path(), &execution);
    drop(foreign.bootstrap_root(&execution));
    let foreign_medium = foreign.home.query_corrections_medium();
    let proof = foreign_medium
        .authority()
        .admit_query_correction_prefix(0, [0; 32], [0; 32]);
    assert_eq!(
        fixture.home.admit_query_corrections(proof, &execution),
        Err(Denial::ForeignHome)
    );
    let foreign_issuer = worth_proof::HomeQueryCorrectionsOwner::fresh(
        current.home().bytes(),
        current.incarnation().fence(),
    );
    let proof = foreign_issuer
        .authority()
        .admit_query_correction_prefix(0, [0; 32], [0; 32]);
    assert_eq!(
        fixture.home.admit_query_corrections(proof, &execution),
        Err(Denial::ForeignHome)
    );
    // Matching descriptive fields cannot substitute the Store's actual issuer.
    // The actual current empty journal admits after all refused presentations.
    let proof = current
        .authority()
        .admit_query_correction_prefix(0, [0; 32], [0; 32]);
    fixture
        .home
        .admit_query_corrections(proof, &execution)
        .unwrap();
    super::world_occurrence_readmission::close(fixture.home, &execution);
    super::world_occurrence_readmission::close(foreign.home, &execution);
}
