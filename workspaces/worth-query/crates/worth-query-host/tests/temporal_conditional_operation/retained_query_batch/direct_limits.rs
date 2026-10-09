//! Genuine direct plans must preserve the finite batch contract too.
use super::super::{product_query_support, schema::*, world::CourtroomWorld};
use std::num::NonZeroUsize;
use worth_query_host::facade::{
    declaration::application_query::ApplicationQueryParameterSet,
    primary_graph::{
        self, WorthQueryApplicationBatchReadDenial as ReadDenial,
        WorthQueryApplicationQueryBatchAdmission as Batch,
        WorthQueryApplicationQueryBatchLimits as Limits,
        WorthQueryApplicationQueryBatchResourceDenial as Resource,
    },
};

fn batch(inline: usize) -> Batch {
    Batch::new(Limits::new(
        NonZeroUsize::new(1).unwrap(),
        NonZeroUsize::new(1024).unwrap(),
        NonZeroUsize::new(128 * 1024).unwrap(),
        NonZeroUsize::new(inline).unwrap(),
    ))
}

fn execute(
    world: &CourtroomWorld,
    batch: &Batch,
    retained: bool,
) -> Result<
    primary_graph::WorthQueryApplicationBatchResult<TemporalIntentQuery, IntentQueryResult>,
    ReadDenial,
> {
    let request = super::super::world::request_scope();
    let principal = product_query_support::principal(world, &request);
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
    let mut basis = world
        .application
        .on_branch(world.application.current_world())
        .select()
        .unwrap();
    let plan = if retained {
        match selected.admit_retained_application_query_from_selection(
            &mut basis,
            &query,
            &access,
            ApplicationQueryParameterSet::new(),
            product_query_support::controls(&request),
            batch,
        ) {
            Ok(plan) => plan,
            Err(primary_graph::WorthQueryRetainedBatchQueryAdmissionDenial::Resource(cause)) => {
                return Err(ReadDenial::Resource(cause))
            }
            Err(other) => panic!("ordinary retained admission must remain valid: {other:?}"),
        }
    } else {
        selected
            .admit_application_query(
                &query,
                &access,
                ApplicationQueryParameterSet::new(),
                product_query_support::controls(&request),
            )
            .unwrap()
    };
    world
        .application
        .execute_application_query_one_shot_in_batch(plan, batch)
}

#[test]
fn direct_and_retained_batch_reads_share_one_finite_item_limit() {
    let world = CourtroomWorld::publish("ready");
    let observer = world.application.result_buffer_observer();
    let before = observer.observe();
    let head = world
        .application
        .on_branch(world.application.current_world())
        .select()
        .unwrap()
        .product()
        .selected_commit()
        .clone();
    for first_retained in [false, true] {
        let batch = batch(8 * 1024);
        let first = execute(&world, &batch, first_retained).unwrap();
        assert_eq!(first.result().rows()[0].input, "payload");
        let spent = batch.observe();
        assert!(spent.read_work_units() > 0);
        let custody = observer.observe();
        assert!(matches!(
            execute(&world, &batch, !first_retained),
            Err(ReadDenial::Resource(Resource::ItemLimit {
                required: 2,
                maximum: 1
            }))
        ));
        assert_eq!(
            batch.observe(),
            spent,
            "a refused second slot performs no read or buffer claim"
        );
        assert_eq!(observer.observe(), custody);
        assert_eq!(
            world
                .application
                .on_branch(world.application.current_world())
                .select()
                .unwrap()
                .product()
                .selected_commit(),
            &head
        );
        drop(first);
        assert_eq!(batch.observe().retained_bytes(), 0);
        assert_eq!(observer.observe().retained_bytes(), before.retained_bytes());
        assert_eq!(observer.observe().active_buffers(), before.active_buffers());
        assert!(
            matches!(
                execute(&world, &batch, false),
                Err(ReadDenial::Resource(Resource::ItemLimit {
                    required: 2,
                    maximum: 1
                }))
            ),
            "dropping a result cannot reopen an exhausted finite schedule"
        );
    }
}

#[test]
fn direct_batch_execution_narrows_the_actual_inline_buffer() {
    let mut world = CourtroomWorld::publish_with_result_bytes("ready", 320 * 1024 * 1024);
    let observer = world.application.result_buffer_observer();
    let before = observer.observe();
    // This independently admitted ordinary plan retains the broad host budget.
    // A small actual read still fits the batch's finite staging and inline loan.
    let small_batch = batch(8 * 1024);
    let small = execute(&world, &small_batch, false).unwrap();
    assert_eq!(small.result().rows()[0].input, "payload");
    drop(small);
    assert_eq!(small_batch.observe().retained_bytes(), 0);
    assert_eq!(observer.observe().retained_bytes(), before.retained_bytes());
    world.supersede_intent(2, 5, "active", &"x".repeat(16 * 1024), "ready");
    let head = world
        .application
        .on_branch(world.application.current_world())
        .select()
        .unwrap()
        .product()
        .selected_commit()
        .clone();
    let ample_batch = batch(32 * 1024);
    let control = execute(&world, &ample_batch, false).unwrap();
    assert_eq!(control.result().rows()[0].input, "x".repeat(16 * 1024));
    drop(control);
    let narrow_batch = batch(8 * 1024);
    assert!(matches!(
        execute(&world, &narrow_batch, false),
        Err(ReadDenial::Execution(cause)) if cause.kind()
            == primary_graph::WorthQueryApplicationOneShotDenialKind::ResultBufferLimitExceeded
    ));
    assert_eq!(narrow_batch.observe().retained_bytes(), 0);
    assert_eq!(observer.observe().retained_bytes(), before.retained_bytes());
    assert_eq!(observer.observe().active_buffers(), before.active_buffers());
    assert_eq!(
        world
            .application
            .on_branch(world.application.current_world())
            .select()
            .unwrap()
            .product()
            .selected_commit(),
        &head
    );
    let retry_batch = batch(32 * 1024);
    let retry = execute(&world, &retry_batch, false).unwrap();
    assert_eq!(retry.result().rows()[0].input, "x".repeat(16 * 1024));
    drop(retry);
    assert_eq!(observer.observe().retained_bytes(), before.retained_bytes());
    assert_eq!(observer.observe().active_buffers(), before.active_buffers());
}
