#[path = "temporal_conditional_operation/adapters.rs"]
mod adapters;
#[path = "temporal_conditional_operation/contract.rs"]
mod contract;
#[path = "temporal_conditional_operation/schema.rs"]
mod schema;
#[path = "temporal_conditional_operation/world.rs"]
mod world;

use std::num::NonZeroUsize;

use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationRequestExt, WorthQueryApplicationRequestQueryDenialKind,
    },
    publication::domain_computation::WorthQueryPublishedApplicationBasisPosture,
};

use adapters::block_on;
use schema::current_read::TemporalIntentReadRequest;
use world::CourtroomWorld;

#[test]
fn reusable_request_executes_fresh_published_reads_through_installed_principal_binding() {
    let world = CourtroomWorld::publish("ready");
    let scope = world::request_scope();
    let authentication = world::admit_identity_adapter(world.application.installed_schema());
    let external = block_on(authentication.authenticate((), &scope)).unwrap();
    let request = world.application.request(&external, &scope);
    let basis = world.application.application_query_basis_observer();

    let before_widening = basis.observe();
    let widening_denial = match request
        .query(TemporalIntentReadRequest::new("intent-1"))
        .limits(
            NonZeroUsize::new(2).unwrap(),
            NonZeroUsize::new(64).unwrap(),
        )
        .execute()
    {
        Ok(_) => panic!("a widened installed limit must be denied"),
        Err(denial) => denial,
    };
    assert_eq!(
        widening_denial.kind(),
        WorthQueryApplicationRequestQueryDenialKind::Limit
    );
    assert_eq!(
        basis.observe().acquisitions(),
        before_widening.acquisitions(),
        "a widened finite limit must be denied before selecting provider truth"
    );

    let first = request
        .query(TemporalIntentReadRequest::new("intent-1"))
        .execute()
        .unwrap();
    assert_eq!(first.rows()[0].input, "payload");
    let first_publication = first.receipt().inspect();
    assert_eq!(first_publication.result_count(), 1);
    assert_eq!(
        first_publication.basis().posture(),
        WorthQueryPublishedApplicationBasisPosture::SelectedProduct
    );
    assert!(!first_publication.query_identity().is_empty());
    assert!(first_publication.terminal_resources_released());

    let changed = world.change_input_on_branch(world.application.current_world(), "fresh-payload");
    changed
        .require_committed()
        .expect("the intervening publication must commit");
    let second = request
        .query(TemporalIntentReadRequest::new("intent-1"))
        .execute()
        .unwrap();
    assert_eq!(second.rows()[0].input, "fresh-payload");
    assert!(
        second.receipt().inspect().basis().snapshot() > first_publication.basis().snapshot(),
        "reusing request context must start a fresh current-World attempt"
    );

    world.revoke_principal_on_default_product();
    let principal_denial = match request
        .query(TemporalIntentReadRequest::new("intent-1"))
        .execute()
    {
        Ok(_) => panic!("revoked installed principal binding must deny the next attempt"),
        Err(denial) => denial,
    };
    assert_eq!(
        principal_denial.kind(),
        WorthQueryApplicationRequestQueryDenialKind::PrincipalResolution,
        "each attempt must resolve the authenticated proof through the installed binding"
    );
}
