//! Installed scalar/batch meaning and native currentness are consumed at the same basis.
use super::*;
use crate::application_program::discovered_contracts::CounterfeitDelivery;
use crate::application_program::discovered_delivery::{BatchDelivery, NoProjectionDelivery};
use worth_query_host::facade::application_entry::{
    WorthQueryOperationDeliveryDenial as DeliveryDenial,
    WorthQueryOperationDeliveryPolicy as Policy,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationQueryBatchLimits as Limits,
    WorthQueryApplicationQueryBatchResourceDenial as ResourceDenial,
};
use worth_query_topology_entry::PlanarOutputRead;
#[test]
fn discovered_ordinary_batch_is_complete_and_same_basis_after_aggregate_work_refusal() {
    declared_world!(world, request, BatchDelivery<2>);
    let input = PlanarSourceAdjustment {
        scope_key: "anchor-a".to_owned(),
        replacement_y: length(2),
    };
    reset_planar_source_adjustment_contacts(&input);
    let source = request
        .query(PlanarRead {
            body_key: input.scope_key.clone(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let mut operation = Some(
        request
            .mutate(Original(input.clone()))
            .expect_source(source)
            .idempotency_owned(110_031)
            .retain_operation::<Program<BatchDelivery<2>>, Action<BatchDelivery<2>>>(
                &world.application,
            )
            .unwrap_or_else(|(_, d)| panic!("construction: {d:?}")),
    );
    let settled = (0..32)
        .find_map(|_| match operation.take().unwrap().advance(&request) {
            Advance::Progress {
                operation: next, ..
            } => {
                operation = Some(next);
                None
            }
            Advance::Settled(settled) => Some(settled),
            Advance::Refused { denial, .. } | Advance::Terminal { denial, .. } => {
                panic!("native fanout: {denial:?}")
            }
        })
        .unwrap();
    let before = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let retained_commit = settled.settlement().observation().selected_commit().clone();
    let mut works = Vec::new();
    let mut historical_basis = None;
    for key in ["remote-b", "sibling-b"] {
        let read = request
            .at(settled.settlement().observation())
            .query(PlanarOutputRead {
                body_key: key.to_owned(),
            })
            .execute()
            .unwrap();
        historical_basis.get_or_insert_with(|| read.receipt().inspect().basis().clone());
        works.push(read.receipt().inspect().read_work().total_work_units());
    }
    let cap = *works.iter().max().unwrap();
    assert!(cap < works.iter().sum::<usize>());
    let policy = Policy::currentness(
        NonZeroUsize::new(
            world
                .application
                .runtime()
                .output_demand_resource_profile()
                .limits()
                .source_currentness_work(),
        )
        .expect("installed finite positive currentness allowance"),
    )
    .batch(Limits::new(
        NonZeroUsize::new(2).unwrap(),
        NonZeroUsize::new(cap).unwrap(),
        NonZeroUsize::MAX,
        NonZeroUsize::MAX,
    ));
    assert!(matches!(settled.read_batch_current_with_policy::<2>(&request,policy),Err(DeliveryDenial::Batch(worth_query_host::facade::application_entry::WorthQueryApplicationQueryBatchDenial::Resource{item:Some(1),denial:ResourceDenial::WorkLimit{..}}))));
    let batch = settled.read_batch_current::<2>(&request).unwrap();
    assert_eq!(batch.items().len(), 2);
    assert_eq!(batch.items()[0].rows()[0].value, length(42));
    assert_eq!(batch.items()[1].rows()[0].value, length(22));
    assert_eq!(
        batch.items()[0].receipt().inspect().basis(),
        batch.items()[1].receipt().inspect().basis()
    );
    let history = settled.read_batch_historical::<2>(&request).unwrap();
    for item in history.items() {
        let actual = item.receipt().inspect().basis().clone();
        let expected = historical_basis.as_ref().unwrap();
        assert_eq!(actual.runtime_instance(), expected.runtime_instance());
        assert_eq!(actual.branch(), expected.branch());
        assert_eq!(actual.version(), expected.version());
        assert_eq!(actual.posture(), expected.posture());
    }
    assert_eq!(
        history.items()[0].receipt().inspect().basis(),
        history.items()[1].receipt().inspect().basis()
    );
    assert_eq!(history.items()[0].rows()[0].value, length(42));
    assert_eq!(history.items()[1].rows()[0].value, length(22));
    assert_eq!(
        settled.settlement().observation().selected_commit(),
        &retained_commit
    );
    drop((batch, history));
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &before
    );
    assert_eq!(planar_source_adjustment_contacts(&input), (1, 1));
    settled
        .close(&request)
        .unwrap_or_else(|(_, d)| panic!("close: {d:?}"));
}
#[test]
fn discovered_ordinary_concrete_selector_refused_before_effect_and_valid_twin_settles() {
    declared_world!(world, request, QueryDelivery);
    let input = PlanarSourceAdjustment {
        scope_key: "anchor-a".to_owned(),
        replacement_y: length(2),
    };
    reset_planar_source_adjustment_contacts(&input);
    let source = request
        .query(PlanarRead {
            body_key: input.scope_key.clone(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let description = request
        .mutate(Original(input.clone()))
        .expect_source(source)
        .idempotency_owned(110_032);
    let (description, denial) = match description
        .retain_operation::<Program<QueryDelivery>, Action<CounterfeitDelivery>>(&world.application)
    {
        Err(refusal) => refusal,
        Ok(_) => panic!("counterfeit implementation accepted"),
    };
    assert!(matches!(
        denial,
        worth_query_host::facade::application_entry::WorthQueryOperationConstructionDenial::Action(
            worth_query_host::facade::domain::WorthQueryActionResolutionDenial::DeliveryMismatch
        )
    ));
    assert_eq!(planar_source_adjustment_contacts(&input), (0, 0));
    let mut operation = Some(
        description
            .retain_operation::<Program<QueryDelivery>, Action<QueryDelivery>>(&world.application)
            .unwrap_or_else(|(_, d)| panic!("valid original: {d:?}")),
    );
    let settled = (0..32)
        .find_map(|_| match operation.take().unwrap().advance(&request) {
            Advance::Progress {
                operation: next, ..
            } => {
                operation = Some(next);
                None
            }
            Advance::Settled(settled) => Some(settled),
            _ => panic!("valid original native progression"),
        })
        .unwrap();
    assert_eq!(
        settled.read_current(&request).unwrap().rows()[0].value,
        length(42)
    );
    settled
        .close(&request)
        .unwrap_or_else(|(_, d)| panic!("close: {d:?}"));
}
#[test]
fn discovered_ordinary_no_projection_closes_actual_nonempty_fanout_without_query_selector() {
    declared_world!(world, request, NoProjectionDelivery);
    let source = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let mut operation = Some(
        request
            .mutate(PlanarSourceAdjustment {
                scope_key: "anchor-a".to_owned(),
                replacement_y: length(2),
            })
            .expect_source(source)
            .idempotency_owned(110_033)
            .retain_operation::<Program<NoProjectionDelivery>, Action<NoProjectionDelivery>>(
                &world.application,
            )
            .unwrap_or_else(|(_, d)| panic!("no-projection construction: {d:?}")),
    );
    let settled = (0..32)
        .find_map(|_| match operation.take().unwrap().advance(&request) {
            Advance::Progress {
                operation: next, ..
            } => {
                operation = Some(next);
                None
            }
            Advance::Settled(settled) => Some(settled),
            _ => panic!("native fanout"),
        })
        .unwrap();
    assert_eq!(settled.settlement().output_count(), 6);
    let current = settled
        .deliver_current(
            &request,
            Policy::currentness(
                NonZeroUsize::new(
                    world
                        .application
                        .runtime()
                        .output_demand_resource_profile()
                        .limits()
                        .source_currentness_work(),
                )
                .expect("installed finite positive currentness allowance"),
            ),
        )
        .unwrap();
    assert_eq!(
        current.selected_commit(),
        settled.settlement().observation().selected_commit()
    );
    drop(current);
    settled
        .close(&request)
        .unwrap_or_else(|(_, d)| panic!("native cleanup: {d:?}"));
}
