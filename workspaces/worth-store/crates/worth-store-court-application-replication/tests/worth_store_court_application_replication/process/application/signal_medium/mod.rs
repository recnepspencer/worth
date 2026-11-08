//! Transport and owner readmission courts cross actual Store journals.
mod bridge_owner;
mod composite_owner;
mod medium_fixture;
mod owner_admission;
mod owner_candidate_refusal;
mod owner_definition_projection;
mod owner_definition_publication;
mod owner_fixture;
mod owner_fork;
mod owner_installation;
mod owner_node_issuance;
mod owner_quiescent_kill;
mod owner_readmission;
mod owner_source_points;
mod physical_open;
use sha2::{Digest, Sha256};
use std::time::{Duration, Instant};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_foundational::facade::{DurablePublicationFate, SignalDurableJournal};
use worth_signal::facade::durability::{
    SignalBranchAdvanceArtifact, SignalDefinitionTransitionArtifact, SignalDurableMedium,
    SignalDurableMediumDenial, SignalPublicationCompletion, SignalPublicationObservation,
};

fn drive(mut step: impl FnMut() -> SignalPublicationObservation) -> SignalPublicationCompletion {
    let deadline = Instant::now() + Duration::from_secs(120);
    loop {
        match step() {
            SignalPublicationObservation::Pending => {
                assert!(
                    Instant::now() < deadline,
                    "original Signal publication did not reach a terminal observation"
                );
                std::thread::yield_now();
            }
            SignalPublicationObservation::Finished(done) => return done,
        }
    }
}
fn committed(
    done: SignalPublicationCompletion,
    journal: SignalDurableJournal,
    position: u64,
    bytes: &[u8],
) {
    assert!(
        matches!(done.fate(), DurablePublicationFate::Committed),
        "{done:?}"
    );
    let digest: [u8; 32] = Sha256::digest(bytes).into();
    assert_eq!(done.committed_location(), Some((journal, position, digest)));
}
#[test]
fn signal_journals_reopen_exact_owner_frames_and_preserve_the_original_read_cut() {
    let directory = tempfile::tempdir().unwrap();
    let funding = medium_fixture::serial_request();
    let execution = ExecutionRequest::serial(&funding);
    let (serving, medium) = medium_fixture::open(directory.path(), true, &execution);
    let first = medium_fixture::definition(&execution, 7, None);
    committed(
        drive(|| medium.advance_definition(&first, &execution)),
        SignalDurableJournal::Definitions,
        1,
        first.as_bytes(),
    );
    let original = medium.capture_recovery_source(&execution).unwrap();
    assert_eq!(
        original
            .frontier(SignalDurableJournal::Definitions, &execution)
            .unwrap(),
        1
    );
    let second = medium_fixture::definition(&execution, 9, Some(&first));
    committed(
        drive(|| medium.advance_definition(&second, &execution)),
        SignalDurableJournal::Definitions,
        2,
        second.as_bytes(),
    );
    let advance = medium_fixture::advance(&execution, &second);
    committed(
        drive(|| medium.advance_branch(&advance, &execution)),
        SignalDurableJournal::Advances,
        1,
        advance.as_bytes(),
    );
    assert_eq!(
        original
            .frontier(SignalDurableJournal::Definitions, &execution)
            .unwrap(),
        1
    );
    assert_eq!(
        original
            .frontier(SignalDurableJournal::Advances, &execution)
            .unwrap(),
        0
    );
    assert_eq!(
        original
            .read(SignalDurableJournal::Definitions, 1, &execution)
            .unwrap()
            .bytes(),
        first.as_bytes()
    );
    drop(original);
    medium_fixture::close(serving, medium, &execution);
    let (serving, medium) = medium_fixture::open(directory.path(), false, &execution);
    let source = medium.capture_recovery_source(&execution).unwrap();
    assert_eq!(source.owner(), medium_fixture::owner());
    assert_eq!(
        source
            .frontier(SignalDurableJournal::Definitions, &execution)
            .unwrap(),
        2
    );
    assert_eq!(
        source
            .frontier(SignalDurableJournal::Advances, &execution)
            .unwrap(),
        1
    );
    for (position, expected) in [(1, &first), (2, &second)] {
        let bytes = source
            .read(SignalDurableJournal::Definitions, position, &execution)
            .unwrap();
        assert_eq!(bytes.bytes(), expected.as_bytes());
        let read =
            SignalDefinitionTransitionArtifact::readmit_bytes(&execution, bytes.bytes()).unwrap();
        assert_eq!(read.reference(), expected.reference());
        assert_eq!(read.digest(), expected.digest());
    }
    let bytes = source
        .read(SignalDurableJournal::Advances, 1, &execution)
        .unwrap();
    let read = SignalBranchAdvanceArtifact::readmit_bytes(&execution, bytes.bytes()).unwrap();
    assert_eq!(read.digest(), advance.digest());
    assert_eq!(read.descriptor(), advance.descriptor());
    assert_eq!(read.as_bytes(), advance.as_bytes());
    drop(source);
    medium_fixture::close(serving, medium, &execution);
}
#[test]
fn stopped_repeat_call_observes_original_commit_and_stopped_fresh_call_proves_no_effect() {
    let directory = tempfile::tempdir().unwrap();
    let funding = medium_fixture::serial_request();
    let execution = ExecutionRequest::serial(&funding);
    let (serving, medium) = medium_fixture::open(directory.path(), true, &execution);
    let first = medium_fixture::definition(&execution, 7, None);
    committed(
        drive(|| medium.advance_definition(&first, &execution)),
        SignalDurableJournal::Definitions,
        1,
        first.as_bytes(),
    );
    let cancellation = CancellationSource::new();
    let stopped =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    cancellation.cancel();
    let stopped = ExecutionRequest::serial(&stopped);
    committed(
        drive(|| medium.advance_definition(&first, &stopped)),
        SignalDurableJournal::Definitions,
        1,
        first.as_bytes(),
    );
    let second = medium_fixture::definition(&execution, 9, Some(&first));
    let refused = drive(|| medium.advance_definition(&second, &stopped));
    assert!(
        matches!(
            refused.fate(),
            DurablePublicationFate::ProvenNoEffect {
                detail: SignalDurableMediumDenial::Request(_)
            }
        ),
        "{refused:?}"
    );
    let source = medium.capture_recovery_source(&execution).unwrap();
    assert_eq!(
        source
            .frontier(SignalDurableJournal::Definitions, &execution)
            .unwrap(),
        1
    );
    drop(source);
    medium_fixture::close(serving, medium, &execution);
}
