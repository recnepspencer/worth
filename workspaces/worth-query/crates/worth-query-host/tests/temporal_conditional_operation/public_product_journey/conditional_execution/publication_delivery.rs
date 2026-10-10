//! Combined publication and exact Relational delivery checks.
use super::*;

pub(super) fn publish_and_deliver(
    world: &CourtroomWorld,
    source_branch: worth_query_host::facade::product::WorthQueryProductBranch,
    transport: &CompletingExternalTransport,
) {
    let (replacement, _) = ReplacementPredicate::controlled(world.contacts.clone());
    let mut publication = world.change_input_and_conditional_definition_on_branch(
        source_branch,
        "changed-on-a",
        Arc::new(replacement),
        1,
    );
    assert_eq!(
        publication
            .committed_product_publication()
            .relational_posture(),
        runtime::CompositeComponentChangePosture::Published
    );
    assert_eq!(
        publication.committed_product_publication().signal_posture(),
        runtime::CompositeComponentChangePosture::Published
    );
    assert_eq!(
        publication
            .committed_product_publication()
            .conditional_definition_generation(),
        Some(2)
    );
    assert!(publication.dispatch_outbox().is_some());
    assert_eq!(
        publication.external_dispatch().unwrap().posture().kind(),
        primary_graph::WorthQueryExternalDispatchPostureKind::Completed
    );
    super::super::projection_and_outbox::assert_commit_publication(&publication);
    assert_eq!(transport.contact_count(), 1);
    let performed = publication
        .take_performed_relational_product_change()
        .expect("the combined publication retains its unique Relational delivery");
    let selected_source = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .on_branch(source_branch)
                .select(&phase)
                .unwrap()
        })
        .expect("installed host call opens");
    let failed = world
        .application
        .with_application_advancement(&world::request_scope(), |phase| {
            selected_source.deliver_relational_change_to_conditional(
                &phase,
                &world.clock,
                usize::MAX,
                performed,
            )
        })
        .expect("the declared host request admits delivery")
        .expect_err("an undeclared dependency ordinal must fail before retention");
    assert_eq!(
        failed.kind(),
        runtime::WorthQueryPerformedRelationalProductChangeDeliveryDenialKind::Bridge
    );
    assert_eq!(
        world
            .application
            .inspect_conditional_runtime()
            .retained_direct_delivery_count(),
        0
    );
    let delivery = world
        .application
        .with_application_advancement(&world::request_scope(), |phase| {
            selected_source.deliver_relational_change_to_conditional(
                &phase,
                &world.clock,
                0,
                failed.into_change(),
            )
        })
        .expect("the declared host request admits delivery")
        .expect("the performed patch belongs to the selected product");
    let runtime::WorthQueryPerformedRelationalProductChangeDeliveryOutcome::Success(delivery) =
        delivery
    else {
        panic!("unexpected public delivery posture: {delivery:?}")
    };
    assert_eq!(delivery.source_envelopes_loaded(), 1);
    assert_eq!(delivery.signal_seeds_emitted(), 1);
    drop(delivery);
    drop(selected_source);
    assert_eq!(
        world
            .application
            .inspect_conditional_runtime()
            .retained_direct_delivery_count(),
        1
    );
}
