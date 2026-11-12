//! A real associated Signal effect survives an occurrence publication refusal.
use super::super::artifact_source::Fixture;
use super::*;

#[test]
fn signal_effect_without_occurrence_or_disposition_refuses_before_serving() {
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let mut fixture = Fixture::journal(&request);
    let root = match fixture.offer(&request, "primary") {
        WorldDurableBootstrapOutcome::Performed(value) => value.product_branch().clone(),
        other => panic!("real bootstrap: {other:?}"),
    };
    let cancellation = RuntimeWorldCancellationSource::new();
    let port = fixture.world.publication_port();
    let opened = match port
        .prepare_query_publication_with_signal(
            &request,
            root.clone(),
            fixture.recorder.program_source(&root),
            None,
            &cancellation.token(),
            None,
        )
        .unwrap()
        .advance(&request)
    {
        WorldOpenAttemptOutcome::Opened(value) => value,
        other => panic!("real open: {other:?}"),
    };
    let prepared = opened
        .prepare_signal(&fixture.native, &request, None)
        .unwrap();
    fixture.recorder.refuse_next_occurrence();
    let outcome =
        port.execute_durable_with_signal(request, prepared, &mut (), &cancellation.token(), |_| {
            Ok(())
        });
    assert!(matches!(
        outcome,
        WorldDurablePublicationOutcome::RecoveryRequired {
            cause: WorldDurablePublicationDenial::Medium(WorldDurableMediumDenial::Unavailable),
            ..
        }
    ));
    let digest = fixture.associated_signal_digest(&request);
    drop((outcome, port, root));
    let denial = match fixture.try_reopen(&request) {
        Ok(_) => panic!("unmatched Signal effect must never serve"),
        Err(cause) => cause,
    };
    assert!(
        matches!(denial, WorldReadmissionDenial::UncleanRecoveryUnsupported(
        UncleanRecoveryUnsupported { owner: DurableOwnerFamily::Signal, effect }
    ) if effect == CanonicalDigestId::new(digest))
    );
}
