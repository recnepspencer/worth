use super::*;

pub(crate) fn already_committed_replace_and_preserve_revalidate(
    foreign: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    let world = installation::install(foreign);
    let scope = authentication::request_scope();
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .expect("the application authenticates its principal");
    let request = world.application.request(&principal, &scope);

    let replaced = settle_and_close(&request, "anchor-a");
    let replayed = settle_and_close(&request, "anchor-a");
    assert_eq!(
        replayed
            .0
            .committed_product_publication()
            .composite_commit(),
        replaced
            .0
            .committed_product_publication()
            .composite_commit(),
        "Replace retry must reuse the real World commit"
    );
    assert_eq!(
        replayed.1, 0,
        "AlreadyCommitted Replace has no second delivery"
    );

    let related = request
        .query(PlanarRead {
            body_key: "anchor-b".to_owned(),
        })
        .execute()
        .expect("the related source is observed");
    let outcome = request
        .mutate(PlanarMutation {
            scope_key: "anchor-b".to_owned(),
            operation: worth_query_consumer_values::PlanarOperation::Adjust(vec![
                worth_query_consumer_values::PlanarAdjustment {
                    body_key: "anchor-b".to_owned(),
                    replacement_y: length(5),
                },
            ]),
            validator_work: 4_096,
        })
        .expect_source(related.observed_sources()[0].clone())
        .idempotency(&10_050)
        .execute()
        .expect("the related dependency changes");
    assert!(matches!(
        outcome,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));

    let preserved = settle_and_close(&request, "anchor-a");
    assert_eq!(
        preserved.1, 1,
        "the first Preserve execution delivers its exact no-op"
    );
    let replayed_preserve = settle_and_close(&request, "anchor-a");
    assert_eq!(
        replayed_preserve
            .0
            .committed_product_publication()
            .composite_commit(),
        preserved
            .0
            .committed_product_publication()
            .composite_commit(),
        "Preserve retry must reuse its committed World output",
    );
    assert_eq!(
        replayed_preserve.1, 0,
        "AlreadyCommitted Preserve has no second delivery",
    );
}

fn settle_and_close(
    request: &worth_query_host::facade::application_entry::WorthQueryApplicationRequest<
        '_,
        '_,
        '_,
        ConsumerSchema,
    >,
    source: &str,
) -> (
    worth_query_host::facade::primary_graph::WorthQueryApplicationCommitReceipt,
    usize,
) {
    let mut demand = request
        .demand(PlanarOutputDemand::new(source))
        .controls(controls())
        .start()
        .expect("the output demand starts");
    let settled = (0..32)
        .find_map(
            |_| match demand.advance(request).expect("the output demand settles") {
                WorthQueryApplicationOutputDemandProgress::Pending => None,
                WorthQueryApplicationOutputDemandProgress::Settled(value) => Some(value),
            },
        )
        .expect("the output settles within the bounded progression");
    let commit = settled.receipt().clone();
    let delivery_contacts = settled
        .readiness_delivery()
        .unwrap()
        .delivery_contact_count();
    drop(settled);
    demand.close();
    (commit, delivery_contacts)
}
