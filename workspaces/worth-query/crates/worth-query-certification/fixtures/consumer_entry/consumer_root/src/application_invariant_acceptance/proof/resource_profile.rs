use std::sync::atomic::Ordering;

use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationRequestExt, WorthQueryApplicationRequestMutationDenial,
    },
    domain::WorthQueryInstalledApplicationSchema,
    primary_graph::WorthQueryOperationAuthorizationDenialKind,
};

use super::{adjust, authentication, installation, read_y, source_version, ConsumerSchema};

pub(super) fn candidate_bytes_beyond_host_limit_are_denied(
    foreign: &WorthQueryInstalledApplicationSchema<ConsumerSchema>,
) {
    let world = installation::install_with_candidate_bytes(foreign, 8191);
    let scope = authentication::request_scope();
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .expect("the bounded host authenticates the same declared principal");
    let request = world.application.request(&principal, &scope);
    let before = source_version(&request);
    let calls = world.invariant_calls.load(Ordering::SeqCst);
    assert_eq!(read_y(&request, "anchor-a"), 1);

    // The ordinary world runs this exact adjustment with its 8192-byte host limit.
    let outcome = request
        .mutate(adjust("anchor-a", 2, 4096))
        .idempotency(&10)
        .execute();
    let Err(WorthQueryApplicationRequestMutationDenial::Authorization(denial)) = outcome else {
        panic!("8192 declared candidate bytes must exceed the 8191-byte host: {outcome:?}")
    };
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::GraphWorkAdmissionUnavailable
    );
    assert_eq!(world.invariant_calls.load(Ordering::SeqCst), calls);
    assert_eq!(source_version(&request), before);
    assert_eq!(read_y(&request, "anchor-a"), 1);
    assert_eq!(read_y(&request, "sibling-a"), 21);
}
