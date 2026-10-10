//! lost terminal index denies until bounded owner reconstruction.
use super::*;

#[test]
fn lost_terminal_index_denies_until_bounded_owner_reconstruction() {
    let world = installed_world();
    let dispatch = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world.commit_dispatch(&phase, 31, "notice-index-rebuild")
        })
        .expect("the fixture read owns its installed host call");
    let record = dispatch.dispatch_outbox().unwrap();
    let correlation = *record.correlation();
    let envelope = signed_envelope(record, [0x96; 32], record.payload(), false);
    let request = super::super::super::fixture::live_scope();
    assert_eq!(
        world
            .application
            .receive_inbound_occurrence(&world.verifier, &envelope, &request)
            .unwrap()
            .posture(),
        Posture::Performed
    );
    let provider = &world.application.primary_provider;
    let sealed = provider
        .lookup_completed_inbound(&correlation)
        .unwrap()
        .expect("World Performed must seal canonical terminal history");
    assert_ne!(
        sealed.original_world_commit(),
        sealed.completion_world_commit()
    );
    provider.destroy_completed_inbound_index_for_test();
    assert!(matches!(
        provider.lookup_completed_inbound(&correlation),
        Err(super::super::super::super::provider::WorthQueryInboundTerminalIndexDenial::IndexUnavailable)
    ));
    let one = std::num::NonZeroUsize::new(1).unwrap();
    let records = std::num::NonZeroUsize::new(32).unwrap();
    let ancestry = std::num::NonZeroUsize::new(32).unwrap();
    assert!(matches!(
        world.application.with_host_advancement(|phase| world.application.repair_completed_inbound_index(
            &phase, &world.verifier, std::num::NonZeroUsize::new(32).unwrap(), records, one,
        )).expect("installed host call opens"),
        Err(crate::domain_computation::primary_graph::WorthQueryInboundIndexRepairDenial::ReconstructionWorkExhausted)
    ));
    assert!(provider.lookup_completed_inbound(&correlation).is_err());
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
        assert!(provider.lookup_completed_inbound(&correlation).is_err());
        assert!(
            pages < 32,
            "small fixture must complete within finite page budget"
        );
    }
    assert!(
        pages > 1,
        "one slot cannot silently scan the whole World history"
    );
    assert_eq!(
        provider.lookup_completed_inbound(&correlation).unwrap(),
        Some(sealed)
    );
}
