//! Genuine earlier and foreign grants reach the real receiving incarnation.
use super::progression::CurrentOriginal;
use super::*;
use worth_runtime_world::facade::WorldHomeTransitionEffectDenial as Denial;

#[test]
fn original_native_continuation_refuses_stale_and_foreign_grants_before_use() {
    let directory = tempfile::tempdir().unwrap();
    let foreign_directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let (_, ids, receipt, earlier) = native_original::publish(directory.path(), &execution);
    let (_, _, _, foreign) = native_original::publish(foreign_directory.path(), &execution);
    let mut current = CurrentOriginal::open(directory.path(), &execution);
    let cancellation = RuntimeWorldCancellationSource::new();
    for (grant, expected) in [
        (
            &earlier,
            worth_proof::HomeProgramTransitionAdmissionDenial::StaleIncarnation,
        ),
        (
            &foreign,
            worth_proof::HomeProgramTransitionAdmissionDenial::ForeignOwner,
        ),
    ] {
        let outcome = current
            .fixture
            .home
            .world()
            .publication_port()
            .continue_home_transition_effect(
                &execution,
                grant.plan(),
                HomeProgramTransitionStepAssociation::from_recorded(grant.plan().identity(), 0),
                grant.original(),
                grant.admission(),
                current.claim.take().unwrap(),
                &cancellation.token(),
                None,
            );
        current.claim = Some(match outcome {
            Outcome::Refused {
                cause: Denial::Admission(cause),
                custody,
            } => {
                assert_eq!(cause, expected);
                custody
            }
            other @ (Outcome::Continued(_)
            | Outcome::Pending(_)
            | Outcome::EvidenceRequired(_)
            | Outcome::Indeterminate(_)
            | Outcome::ReopenRequired(_)
            | Outcome::Refused { .. }) => {
                panic!("the actual receiver must refuse the presented grant first: {other:?}")
            }
        });
        assert_eq!(current.executions.load(Ordering::SeqCst), 0);
        assert_eq!(
            current
                .claim
                .as_ref()
                .unwrap()
                .home_transition_effect_progress()
                .unwrap(),
            None
        );
        assert_eq!(
            current.claim.as_ref().unwrap().native_receipt().as_ref(),
            Some(&receipt)
        );
        let actual = current
            .fixture
            .home
            .world()
            .observation_port()
            .observe_root_product_branch()
            .unwrap();
        assert_eq!(actual.reference_generation().get(), 0);
    }
    let publication = current.finish(&execution);
    assert_eq!(publication.attempt_identity().ordinal(), ids.0);
    assert_eq!(publication.native_receipt(), &receipt);
    progression::assert_live_head(&current.fixture, &publication);
    assert_eq!(current.executions.load(Ordering::SeqCst), 0);
    drop((publication, current.contracts, current.grant, current.claim));
    close(current.fixture.home, &execution);
}
