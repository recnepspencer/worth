//! Public ordinary batch courts using the existing installed temporal world.
#[path = "retained_query_batch/inline_result.rs"]
mod inline_result;
#[path = "retained_query_batch/installed_profile.rs"]
mod installed_profile;
use super::{
    adapters::block_on,
    schema::current_read::TemporalIntentReadRequest,
    world::{self, CourtroomWorld},
};
use std::num::NonZeroUsize;
use worth_query_host::facade::{
    application_entry::{WorthQueryApplicationQueryBatchDenial, WorthQueryApplicationRequestExt},
    primary_graph::{
        WorthQueryApplicationQueryBatchLimits, WorthQueryApplicationQueryBatchResourceDenial,
    },
};

fn read(identity: &str) -> TemporalIntentReadRequest {
    TemporalIntentReadRequest {
        identity: identity.to_owned(),
    }
}
fn limits(work: usize, bytes: usize) -> WorthQueryApplicationQueryBatchLimits {
    WorthQueryApplicationQueryBatchLimits::new(
        NonZeroUsize::new(8).unwrap(),
        NonZeroUsize::new(work).unwrap(),
        NonZeroUsize::new(bytes).unwrap(),
        NonZeroUsize::new(2048).unwrap(),
    )
}

#[test]
fn retained_batch_orders_independent_scopes_and_bills_duplicates_at_one_basis() {
    let world = CourtroomWorld::publish_with_intent_population("ready", 2);
    let scope = world::request_scope();
    let auth = world::admit_identity_adapter(world.application.installed_schema());
    let external = block_on(auth.authenticate((), &scope)).unwrap();
    let observation = world
        .application
        .request(&external, &scope)
        .retain_read()
        .unwrap();
    let request = world
        .application
        .request(&external, &scope)
        .at(&observation);
    let single = request.query(read("intent-1")).execute().unwrap();
    let scalar_work = single.receipt().inspect().read_work().total_work_units();
    let batch = request
        .query_batch(vec![read("intent-1"), read("intent-2"), read("intent-1")])
        .limits(limits(1024, 1_048_576))
        .execute()
        .unwrap();
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
        batch.receipt().read_work().read_work_units(),
        3 * scalar_work
    );
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
        .request(&external, &scope)
        .retain_read()
        .unwrap();
    let observer = world.application.result_buffer_observer();
    let before = observer.observe();
    let denial = world
        .application
        .request(&external, &scope)
        .at(&observation)
        .query_batch(vec![read("intent-1"), read("absent")])
        .limits(limits(1024, 1_048_576))
        .execute();
    assert!(matches!(denial, Err(WorthQueryApplicationQueryBatchDenial::Item { index: 1, cause: worth_query_host::facade::application_entry::WorthQueryApplicationRequestQueryDenial::ScopeResolution(_) })));
    assert_eq!(observer.observe().retained_bytes(), before.retained_bytes());
    assert_eq!(observer.observe().active_buffers(), before.active_buffers());
    let current = world
        .application
        .request(&external, &scope)
        .retain_read()
        .unwrap();
    assert_eq!(current.selected_commit(), observation.selected_commit());
}

#[test]
fn aggregate_work_refuses_second_read_without_returning_the_first() {
    let world = CourtroomWorld::publish("ready");
    let scope = world::request_scope();
    let auth = world::admit_identity_adapter(world.application.installed_schema());
    let external = block_on(auth.authenticate((), &scope)).unwrap();
    let observation = world
        .application
        .request(&external, &scope)
        .retain_read()
        .unwrap();
    let request = world
        .application
        .request(&external, &scope)
        .at(&observation);
    let scalar = request.query(read("intent-1")).execute().unwrap();
    let work = scalar.receipt().inspect().read_work().total_work_units();
    drop(scalar);
    let observer = world.application.result_buffer_observer();
    let before = observer.observe().retained_bytes();
    let outcome = request
        .query_batch(vec![read("intent-1"), read("intent-1")])
        .limits(limits(work, 1_048_576))
        .execute();
    assert!(matches!(
        outcome,
        Err(WorthQueryApplicationQueryBatchDenial::Resource {
            item: Some(1),
            denial: WorthQueryApplicationQueryBatchResourceDenial::WorkLimit { .. }
        })
    ));
    assert_eq!(observer.observe().retained_bytes(), before);
    assert_eq!(observer.observe().active_buffers(), 0);
}

#[test]
fn aggregate_custody_refusal_releases_every_earlier_item() {
    let world = CourtroomWorld::publish("ready");
    let scope = world::request_scope();
    let auth = world::admit_identity_adapter(world.application.installed_schema());
    let external = block_on(auth.authenticate((), &scope)).unwrap();
    let observation = world
        .application
        .request(&external, &scope)
        .retain_read()
        .unwrap();
    let request = world
        .application
        .request(&external, &scope)
        .at(&observation);
    let single = request
        .query_batch(vec![read("intent-1")])
        .limits(limits(1024, 1_048_576))
        .execute()
        .unwrap();
    let bytes = single.receipt().read_work().peak_bytes();
    drop(single);
    let observer = world.application.result_buffer_observer();
    let before = observer.observe().retained_bytes();
    let outcome = request
        .query_batch(vec![read("intent-1"), read("intent-1")])
        .limits(limits(1024, bytes))
        .execute();
    assert!(matches!(
        outcome,
        Err(WorthQueryApplicationQueryBatchDenial::Resource {
            denial: WorthQueryApplicationQueryBatchResourceDenial::MemoryLimit { .. },
            ..
        })
    ));
    assert_eq!(observer.observe().retained_bytes(), before);
    assert_eq!(observer.observe().active_buffers(), 0);
}

#[test]
fn historical_batch_reads_original_data_but_current_revocation_still_refuses() {
    let world = CourtroomWorld::publish("ready");
    let scope = world::request_scope();
    let auth = world::admit_identity_adapter(world.application.installed_schema());
    let external = block_on(auth.authenticate((), &scope)).unwrap();
    let observation = world
        .application
        .request(&external, &scope)
        .retain_read()
        .unwrap();
    world.change_input_after_query_admission("new-payload");
    let historical = world
        .application
        .request(&external, &scope)
        .at(&observation)
        .query_batch(vec![read("intent-1"), read("intent-1")])
        .limits(limits(1024, 1_048_576))
        .execute()
        .unwrap();
    assert!(historical
        .items()
        .iter()
        .all(|item| item.rows()[0].input == "payload"));
    drop(historical);
    world.revoke_principal_on_default_product();
    let revoked = world
        .application
        .request(&external, &scope)
        .at(&observation)
        .query_batch(vec![read("intent-1")])
        .limits(limits(1024, 1_048_576))
        .execute();
    assert!(matches!(
        revoked,
        Err(WorthQueryApplicationQueryBatchDenial::Item {
            index: 0,
            cause: worth_query_host::facade::application_entry::WorthQueryApplicationRequestQueryDenial::PrincipalResolution(_)
        })
    ));
}

#[test]
fn pre_kernel_cancellation_keeps_its_real_execution_cause() {
    use super::{product_query_support, schema::*};
    use worth_query_host::facade::{
        admission::authenticated_principal::*,
        declaration::application_query::ApplicationQueryParameterSet, primary_graph,
    };
    let world = CourtroomWorld::publish("ready");
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(
        std::time::Instant::now() + std::time::Duration::from_secs(60),
        cancellation.token(),
    );
    let principal = product_query_support::principal(&world, &request);
    let query = world
        .application
        .installed_schema()
        .certification_query(TemporalIntentQuery::reference())
        .unwrap();
    let selected = world
        .application
        .on_branch(world.application.current_world())
        .select()
        .unwrap();
    let scope = selected
        .resolve_entity(
            IntentIdentityField::reference(),
            "intent-1".to_owned(),
            &request,
            primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let access = primary_graph::WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
    let plan = selected
        .admit_application_query(
            &query,
            &access,
            ApplicationQueryParameterSet::new(),
            product_query_support::controls(&request),
        )
        .unwrap();
    let batch =
        primary_graph::WorthQueryApplicationQueryBatchAdmission::new(limits(1024, 1_048_576));
    cancellation.cancel();
    let result = world
        .application
        .execute_application_query_one_shot_in_batch(plan, &batch);
    assert!(
        matches!(result, Err(primary_graph::WorthQueryApplicationBatchReadDenial::Execution(error)) if error.kind() == primary_graph::WorthQueryApplicationOneShotDenialKind::Cancelled)
    );
    assert_eq!(batch.observe().retained_bytes(), 0);
    assert_eq!(
        world
            .application
            .result_buffer_observer()
            .observe()
            .active_buffers(),
        0
    );
}

#[test]
fn genuine_source_clone_keeps_custody_after_batch_drop_and_refunds_on_last_drop() {
    let world = CourtroomWorld::publish("ready");
    let scope = world::request_scope();
    let auth = world::admit_identity_adapter(world.application.installed_schema());
    let external = block_on(auth.authenticate((), &scope)).unwrap();
    let observation = world
        .application
        .request(&external, &scope)
        .retain_read()
        .unwrap();
    let observer = world.application.result_buffer_observer();
    let before = observer.observe().retained_bytes();
    let batch = world
        .application
        .request(&external, &scope)
        .at(&observation)
        .query_batch(vec![read("intent-1")])
        .limits(limits(1024, 1_048_576))
        .execute()
        .unwrap();
    let source = batch.items()[0].observed_sources()[0].clone();
    drop(batch);
    assert!(observer.observe().retained_bytes() > before);
    assert_eq!(observer.observe().active_buffers(), 0);
    let second = source.clone();
    drop(source);
    assert!(observer.observe().retained_bytes() > before);
    drop(second);
    assert_eq!(observer.observe().retained_bytes(), before);
}

#[path = "retained_query_batch/direct_limits.rs"]
mod direct_limits;
