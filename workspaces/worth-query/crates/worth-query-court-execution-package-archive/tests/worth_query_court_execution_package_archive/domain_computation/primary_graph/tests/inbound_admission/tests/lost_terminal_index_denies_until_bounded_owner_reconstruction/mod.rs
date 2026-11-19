//! lost terminal index denies until bounded owner reconstruction.
use crate::domain_computation::primary_graph::tests::inbound_admission::tests::*;

#[test]
fn lost_terminal_index_denies_until_bounded_owner_reconstruction() {
    let world = installed_world();
    let dispatch = { world.commit_dispatch(31, "notice-index-rebuild") };
    let record = dispatch.dispatch_outbox().unwrap();
    let correlation = *record.correlation();
    let envelope = signed_envelope(record, [0x96; 32], record.payload(), false);
    let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    assert_eq!(
        world
            .application
            .receive_inbound_occurrence(&world.verifier, &envelope, &request)
            .unwrap()
            .posture(),
        Posture::Performed
    );
    // Reconstruction pages over completed terminals, so a one-slot page is
    // only incomplete when a second completion exists.
    let other = { world.commit_dispatch(32, "notice-index-rebuild-other") };
    let other_record = other.dispatch_outbox().unwrap();
    let other_correlation = *other_record.correlation();
    let other_envelope = signed_envelope(other_record, [0x97; 32], other_record.payload(), false);
    assert_eq!(
        world
            .application
            .receive_inbound_occurrence(&world.verifier, &other_envelope, &request)
            .unwrap()
            .posture(),
        Posture::Performed
    );
    let provider = (*world.application.runtime_for_test()).primary_provider_for_test();
    let other_sealed = provider
        .lookup_completed_inbound_for_test(&other_correlation)
        .unwrap()
        .expect("the second World Performed seals its own terminal");
    let sealed = provider
        .lookup_completed_inbound_for_test(&correlation)
        .unwrap()
        .expect("World Performed must seal canonical terminal history");
    assert_ne!(
        sealed.original_world_commit_for_test(),
        sealed.completion_world_commit_for_test()
    );
    provider.destroy_completed_inbound_index_for_test();
    assert!(matches!(
        provider.lookup_completed_inbound_for_test(&correlation),
        Err(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::inbound_terminal_index::WorthQueryInboundTerminalIndexDenial::IndexUnavailable)
    ));
    let one = std::num::NonZeroUsize::new(1).unwrap();
    let records = std::num::NonZeroUsize::new(32).unwrap();
    let ancestry = std::num::NonZeroUsize::new(32).unwrap();
    assert!(matches!(
        world.application.with_host_advancement(|phase| world.application.repair_completed_inbound_index(
            &phase, &world.verifier, std::num::NonZeroUsize::new(32).unwrap(), records, one,
        )).expect("installed host call opens"),
        Err(worth_query_execution::facade::domain_computation::WorthQueryInboundIndexRepairDenial::ReconstructionWorkExhausted)
    ));
    assert!(provider
        .lookup_completed_inbound_for_test(&correlation)
        .is_err());
    let mut pages = 0;
    loop {
        pages += 1;
        let complete = world
            .application
            .with_host_advancement(|phase| {
                world.application.repair_completed_inbound_index(
                    &phase,
                    &world.verifier,
                    one,
                    records,
                    ancestry,
                )
            })
            .expect("installed host call opens")
            .expect("each explicit World page is bounded and authoritative");
        if complete {
            break;
        }
        assert!(provider
            .lookup_completed_inbound_for_test(&correlation)
            .is_err());
        assert!(
            pages < 32,
            "small fixture must complete within finite page budget"
        );
    }
    assert!(
        pages > 1,
        "one slot cannot silently reconstruct every completed terminal"
    );
    assert_eq!(
        provider
            .lookup_completed_inbound_for_test(&other_correlation)
            .unwrap(),
        Some(other_sealed)
    );
    assert_eq!(
        provider
            .lookup_completed_inbound_for_test(&correlation)
            .unwrap(),
        Some(sealed)
    );
}
