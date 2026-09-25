use super::*;

pub(crate) fn already_committed_replace_reuses_readiness(
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

    let replaced = settle_and_close(&request, &world.application, "anchor-a");
    let replace_readiness_attempts = world.application.output_readiness_attempt_count_for_test();
    let replayed = settle_ready(&request, "anchor-a");
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
    assert_eq!(replayed.1, replaced.1);
    assert_eq!(
        world.application.output_readiness_attempt_count_for_test(),
        replace_readiness_attempts,
        "a ready Replace read must not revalidate the producer",
    );
}

fn settle_and_close(
    request: &worth_query_host::facade::application_entry::WorthQueryApplicationRequest<
        '_,
        '_,
        '_,
        ConsumerSchema,
    >,
    application: &worth_query_host::facade::application_installation::WorthQueryProgramApplicationRuntime<
        ConsumerSchema,
        crate::ConsumerProgram,
    >,
    source: &str,
) -> (
    worth_query_host::facade::primary_graph::WorthQueryApplicationCommitReceipt,
    usize,
) {
    let mut demand = request
        .start_program_outputs::<crate::ConsumerProgram, crate::ConsumerProgramRoot>(
            application,
            PlanarOutputDemand::new(source),
            controls(),
        )
        .expect("the program output demand starts");
    let settled = (0..32)
        .find_map(|_| {
            match demand
                .advance(request)
                .expect("the program output demand settles")
            {
                WorthQueryApplicationProgramOutputProgress::Pending => None,
                WorthQueryApplicationProgramOutputProgress::Settled(value) => Some(value),
            }
        })
        .expect("the output settles within the bounded progression");
    let commit = settled
        .root_receipt()
        .expect("the produced root retains its commit")
        .clone();
    let delivery_contacts = settled
        .root_readiness_delivery()
        .unwrap()
        .delivery_contact_count();
    drop(settled);
    drop(demand);
    (commit, delivery_contacts)
}

fn settle_ready(
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
        .expect("the settled output opens for a ready read");
    let settled = (0..32)
        .find_map(|_| {
            match demand
                .advance(request)
                .expect("the ready output is readable")
            {
                WorthQueryApplicationOutputDemandProgress::Pending => None,
                WorthQueryApplicationOutputDemandProgress::Settled(value) => Some(value),
            }
        })
        .expect("the ready output settles within bounded progression");
    let receipt = settled
        .application_commit_receipt()
        .expect("the ready output retains its commit")
        .clone();
    let contacts = settled
        .readiness_delivery()
        .expect("a ready output retains delivery evidence")
        .delivery_contact_count();
    drop(settled);
    demand.close();
    (receipt, contacts)
}
