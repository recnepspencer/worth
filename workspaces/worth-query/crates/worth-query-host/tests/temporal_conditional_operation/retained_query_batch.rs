//! Public ordinary batch courts using the existing installed temporal world.
#[path = "retained_query_batch/installed_profile.rs"]
mod installed_profile;
use super::{
    adapters::block_on,
    schema::current_read::TemporalIntentReadRequest,
    world::{self, CourtroomWorld},
};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationQueryBatchDenial, WorthQueryApplicationRequestExt,
};

fn read(identity: &str) -> TemporalIntentReadRequest {
    TemporalIntentReadRequest {
        identity: identity.to_owned(),
    }
}

#[test]
fn retained_batch_orders_independent_scopes_and_duplicate_reads_at_one_basis() {
    let world = CourtroomWorld::publish_with_intent_population("ready", 2);
    let scope = world::request_scope();
    let auth = world::admit_identity_adapter(world.application.installed_schema());
    let external = block_on(auth.authenticate((), &scope)).unwrap();
    let observation = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .request(&external, &scope)
                .retain_read(&phase)
                .unwrap()
        })
        .expect("installed host call opens");
    let request = world
        .application
        .request(&external, &scope)
        .at(&observation);
    let single = request.query(read("intent-1")).execute().unwrap();
    let batch = world
        .application
        .with_host_advancement(|phase| {
            request
                .query_batch(vec![read("intent-1"), read("intent-2"), read("intent-1")])
                .execute(&phase)
                .unwrap()
        })
        .expect("installed host call opens");
    assert_eq!(
        batch
            .items()
            .iter()
            .map(|item| item.rows()[0].identity.as_str())
            .collect::<Vec<_>>(),
        ["intent-1", "intent-2", "intent-1"]
    );
    let batch_basis = batch.items()[0].receipt().inspect().basis().clone();
    let scalar_inspection = single.receipt().inspect();
    let scalar_basis = scalar_inspection.basis();
    assert_eq!(
        batch_basis.runtime_instance(),
        scalar_basis.runtime_instance()
    );
    assert_eq!(batch_basis.branch(), scalar_basis.branch());
    assert_eq!(batch_basis.version(), scalar_basis.version());
    assert_eq!(batch_basis.posture(), scalar_basis.posture());
    for item in batch.items() {
        let inspection = item.receipt().inspect();
        assert_eq!(inspection.basis(), &batch_basis);
        assert_eq!(inspection.read_work().projected_record_count(), 1);
        assert_eq!(inspection.read_work().projected_field_count(), 5);
        assert!(inspection.read_work().total_work_units() > 0);
        assert_eq!(inspection.read_work().fallback_count(), 0);
        assert_eq!(item.observed_sources().len(), 1);
    }
    assert_eq!(batch.receipt().item_count(), 3);

    assert_eq!(
        batch.receipt().authorization_work_units(),
        batch
            .items()
            .iter()
            .map(|item| item.receipt().inspect().authorization_work_units())
            .sum::<usize>()
    );
}

#[test]
fn late_missing_scope_publishes_no_collection_and_releases_staged_custody() {
    let world = CourtroomWorld::publish("ready");
    let scope = world::request_scope();
    let auth = world::admit_identity_adapter(world.application.installed_schema());
    let external = block_on(auth.authenticate((), &scope)).unwrap();
    let observation = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .request(&external, &scope)
                .retain_read(&phase)
                .unwrap()
        })
        .expect("installed host call opens");

    let denial = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .request(&external, &scope)
                .at(&observation)
                .query_batch(vec![read("intent-1"), read("absent")])
                .execute(&phase)
        })
        .expect("installed host call opens");
    assert!(matches!(denial, Err(WorthQueryApplicationQueryBatchDenial::Item { index: 1, cause: worth_query_host::facade::application_entry::WorthQueryApplicationRequestQueryDenial::ScopeResolution(_) })));

    let current = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .request(&external, &scope)
                .retain_read(&phase)
                .unwrap()
        })
        .expect("installed host call opens");
    assert_eq!(current.selected_commit(), observation.selected_commit());
}

#[test]
fn historical_batch_reads_original_data_but_current_revocation_still_refuses() {
    let world = CourtroomWorld::publish("ready");
    let scope = world::request_scope();
    let auth = world::admit_identity_adapter(world.application.installed_schema());
    let external = block_on(auth.authenticate((), &scope)).unwrap();
    let observation = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .request(&external, &scope)
                .retain_read(&phase)
                .unwrap()
        })
        .expect("installed host call opens");
    world.change_input_after_query_admission("new-payload");
    let historical = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .request(&external, &scope)
                .at(&observation)
                .query_batch(vec![read("intent-1"), read("intent-1")])
                .execute(&phase)
                .unwrap()
        })
        .expect("installed host call opens");
    assert!(historical
        .items()
        .iter()
        .all(|item| item.rows()[0].input == "payload"));
    drop(historical);
    world.revoke_principal_on_default_product();
    let revoked = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .request(&external, &scope)
                .at(&observation)
                .query_batch(vec![read("intent-1")])
                .execute(&phase)
        })
        .expect("installed host call opens");
    assert!(matches!(
        revoked,
        Err(WorthQueryApplicationQueryBatchDenial::Item {
            index: 0,
            cause: worth_query_host::facade::application_entry::WorthQueryApplicationRequestQueryDenial::PrincipalResolution(_)
        })
    ));
}

#[path = "retained_query_batch/pre_kernel_cancellation_keeps_its_real_execution_cause.rs"]
mod pre_kernel_cancellation_keeps_its_real_execution_cause;

#[path = "retained_query_batch/direct_limits.rs"]
mod direct_limits;
