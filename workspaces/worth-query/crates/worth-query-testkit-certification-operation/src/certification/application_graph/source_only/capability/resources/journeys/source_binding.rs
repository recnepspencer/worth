//! Only the closed source equality probe is instrumentation; all actions are Host.
pub use super::*;
pub use worth_query_host::facade::admission::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;

pub fn run(
    host: &host::Host,
    actor: &WorthQueryAuthenticatedExternalPrincipal<CapabilitySchema>,
    scope: &WorthQueryRequestScope,
) {
    let runtime = host.application.runtime();
    let main = runtime.current_world();
    let fork = host
        .application
        .with_host_advancement(|phase| {
            runtime
                .branches()
                .fork(main)
                .components(|components| components.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
                .unwrap()
        })
        .unwrap();
    let original = runtime
        .request(actor, scope)
        .on_branch(fork)
        .retain_current_read()
        .unwrap();
    // A genuine sibling publication moves Native issuance without moving the fork.
    let sibling = runtime
        .request(actor, scope)
        .advanced()
        .mutate(super::super::grant_status::SetStatus {
            identity: "resource-action".to_owned(),
            active: false,
        })
        .without_source()
        .idempotency(&0x9181_8020_u64)
        .execute_in_program(
            &host.application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(matches!(
        sibling,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    let sibling = runtime.request(actor, scope).retain_current_read().unwrap();
    assert_ne!(original.selected_commit(), sibling.selected_commit());
    let reissue = |read: &worth_query_host::facade::application_entry::WorthQueryApplicationReadObservation| {
        runtime
            .with_host_advancement(|phase| {
                read.select_on(&phase, runtime)
                    .unwrap()
                    .retain_application_read()
            })
            .unwrap()
    };
    let original = reissue(&original);
    let sibling = reissue(&sibling);
    let request = runtime.request(actor, scope).on_branch(fork);
    let resources = request.resources();
    let index = host.contacts.resource.counts().0;
    let mut handle = start(&resources, 61);
    let (same, native, version) = resources
        .source_matches_read_for_test(
            &handle.receipt(),
            &original,
            metadata::admit_on(host, actor, scope, fork).unwrap(),
        )
        .unwrap();
    assert!(
        same,
        "the actual Native source is the independently retained fork truth"
    );
    assert_eq!(version, 0, "a populated fork resets its branch counter");
    assert_ne!(
        native, version,
        "the exact Native occurrence cannot be a branch version"
    );
    assert!(
        !resources
            .source_matches_read_for_test(
                &handle.receipt(),
                &sibling,
                metadata::admit_on(host, actor, scope, fork).unwrap()
            )
            .unwrap()
            .0
    );
    host.contacts.resource.send(index, 6161).unwrap();
    resources.advance(&mut handle).unwrap();
    assert_eq!(
        resources
            .with_result_governed(
                &handle.receipt(),
                metadata::admit_on(host, actor, scope, fork).unwrap(),
                |value| *value
            )
            .unwrap(),
        Some(6161)
    );
    resources.close(&mut handle).unwrap();
}
