//! Recovery keeps published product truth without restoring an undelivered head hold.
use super::*;
use worth_query_host::facade::application_installation::WorthQueryCheckpointCapturePolicy;

#[test]
fn a_checkpoint_of_a_frozen_head_restores_truth_without_a_publication_hold() {
    let _guard = checkpoint_recovery_test_guard();
    let application = support::install_program_with_seed::<application::Program>(
        None,
        Default::default(),
        4096,
        128 * 1024 * 1024,
        1_000_000,
        source_world::seed,
    );
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut owner = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .start_in_program::<application::Program, program::ChainRoot>(&application)
        .unwrap();
    assert!(matches!(
        owner.advance(&request).unwrap(),
        WorthQueryApplicationOutputDemandProgress::Settled(_)
    ));
    let source = request
        .query(PlanarRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    request
        .advanced()
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".into(),
            replacement_y: length(7),
        })
        .expect_source(source.observed_sources()[0].clone())
        .idempotency(&0x9176_78e0_u64)
        .execute_performed::<application::Program, program::ChainRoot>(
            &application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    application.delay_next_output_readiness_delivery_for_test();
    assert!(matches!(
        owner.advance(&request).unwrap(),
        WorthQueryApplicationOutputDemandProgress::Pending
    ));
    assert_eq!(
        application.undelivered_publication_counts_for_test(),
        (1, 1)
    );
    let published = request
        .query(PlanarOutputRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(published.rows().len(), 1);
    let last_published_truth = published.rows()[0].value;
    let checkpoint = application
        .with_host_advancement(|phase| {
            application
                .capture_application_checkpoint(
                    &phase,
                    WorthQueryCheckpointCapturePolicy::SystemAllocation,
                )
                .unwrap()
        })
        .expect("checkpoint capture owns its host request");
    assert_eq!(
        application.undelivered_publication_counts_for_test(),
        (1, 1),
        "capture neither delivers nor abandons the owner's publication"
    );
    drop((published, source, owner, principal, scope));
    drop(application);

    let restored = support::install_program_with_seed::<application::Program>(
        Some(checkpoint),
        Default::default(),
        4096,
        128 * 1024 * 1024,
        1_000_000,
        source_world::seed,
    );
    assert_eq!(restored.undelivered_publication_counts_for_test(), (0, 0));
    let (scope, principal) = authenticate(&restored);
    let request = restored.request(&principal, &scope);
    let published = request
        .query(PlanarOutputRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(published.rows().len(), 1);
    assert_eq!(published.rows()[0].value, last_published_truth);
    assert_eq!(restored.undelivered_publication_counts_for_test(), (0, 0));
}
