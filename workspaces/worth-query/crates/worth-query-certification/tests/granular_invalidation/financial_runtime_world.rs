#[path = "financial_runtime_world/adapters.rs"]
mod adapters;
#[path = "financial_runtime_world/contract.rs"]
mod contract;
#[path = "financial_runtime_world/frontier_slope.rs"]
mod frontier_slope;
#[path = "financial_runtime_world/host.rs"]
mod host;
#[path = "financial_runtime_world/portfolio.rs"]
mod portfolio;
#[path = "financial_runtime_world/query.rs"]
mod query;
#[path = "financial_runtime_world/query_domain.rs"]
mod query_domain;
#[path = "financial_runtime_world/query_source.rs"]
mod query_source;
#[path = "financial_runtime_world/schema.rs"]
mod schema;
#[path = "financial_runtime_world/shared.rs"]
mod shared;

pub fn assert_financial_host_curve_delivery() {
    use worth_query_host::facade::primary_graph::WorthQueryConditionalClockObservationOutcome;

    let mut world = host::FinancialCourtroomWorld::publish_curve();
    let baseline = world
        .conditional_clock(&world.curve_clock)
        .unwrap()
        .observe();
    assert!(matches!(
        baseline,
        WorthQueryConditionalClockObservationOutcome::Accepted(_)
    ));
    world.amend_curve(2, 4_260, 5_100);
    world.curve_gate.release();
    world.curve_clock_control.push(2, 11);
    let observed = world
        .conditional_clock(&world.curve_clock)
        .unwrap()
        .observe();
    let mut receipt = match observed {
        WorthQueryConditionalClockObservationOutcome::Accepted(receipt) => receipt,
        WorthQueryConditionalClockObservationOutcome::Duplicate(_) => {
            panic!("the 5y curve observation was duplicate")
        }
        WorthQueryConditionalClockObservationOutcome::Stale => {
            panic!("the 5y curve observation was stale")
        }
        WorthQueryConditionalClockObservationOutcome::Reordered => {
            panic!("the 5y curve observation was reordered")
        }
        WorthQueryConditionalClockObservationOutcome::Closed => {
            panic!("the 5y curve observation found a closed runtime")
        }
        WorthQueryConditionalClockObservationOutcome::Failed(failure) => {
            panic!("the 5y curve observation failed: {}", failure.detail())
        }
    };
    let batch = receipt.take_granular_invalidation_batch();
    assert!(!batch.is_empty());
    let declared_curve_dependencies = contract::operation_definition()
        .semantics()
        .conditional_nodes
        .iter()
        .find(|node| node.identity() == "curve-risk")
        .expect("the financial courtroom declares its curve node")
        .dependencies()
        .len();
    assert_eq!(declared_curve_dependencies, 5);
    assert_eq!(batch.observation().direct_truth_delivery_count(), 3);
    assert_eq!(batch.observation().signal_performed_delivery_count(), 1);
    let delivered_dependencies = batch
        .bridge_deliveries()
        .iter()
        .map(|delivery| {
            let dependency = delivery.truth().change_set().dependency();
            assert_eq!(dependency.contract().key().as_str(), "CurveFacts");
            dependency.dependency_ordinal()
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(delivered_dependencies.len(), 3);
    let localities = batch
        .bridge_deliveries()
        .iter()
        .map(
            |delivery| match delivery.truth().change_set().dependency().locality() {
                worth_runtime_bridge::facade::BridgeSemanticLocality::ManagedSourceRecord => {
                    "record"
                }
                worth_runtime_bridge::facade::BridgeSemanticLocality::SourcePartition(role)
                    if role.as_str() == "usd-rates" =>
                {
                    "partition"
                }
                worth_runtime_bridge::facade::BridgeSemanticLocality::WholeLogicalGraph => {
                    "unscoped"
                }
                _ => "unexpected",
            },
        )
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(localities, ["partition", "record", "unscoped"].into());
}

pub fn assert_sibling_curve_record_does_no_query_work() {
    use worth_query::facade::domain::{
        bind_primary_runtime_granular_invalidations, maintain_primary_runtime_granular_batch,
        WorthQueryPrimaryGranularMaintenanceOutcome,
    };
    use worth_query_host::facade::primary_graph::WorthQueryConditionalClockObservationOutcome;

    let mut host = host::FinancialCourtroomWorld::publish_curve();
    let mut sibling = query::build_sibling_curve_record(&host);
    assert!(matches!(
        host.conditional_clock(&host.curve_clock).unwrap().observe(),
        WorthQueryConditionalClockObservationOutcome::Accepted(_)
    ));
    host.amend_curve(2, 4_260, 5_100);
    host.curve_gate.release();
    host.curve_clock_control.push(2, 11);
    let WorthQueryConditionalClockObservationOutcome::Accepted(mut receipt) =
        host.conditional_clock(&host.curve_clock).unwrap().observe()
    else {
        panic!("the 5y curve change must be observed")
    };
    let binding = bind_primary_runtime_granular_invalidations(
        &sibling.live,
        host.application.granular_invalidation_installation(),
    );
    let outcome = maintain_primary_runtime_granular_batch(
        &sibling.live,
        &mut sibling.workspace,
        &binding,
        receipt.take_granular_invalidation_batch(),
    )
    .expect("an off-record delivery is a lawful irrelevant outcome");
    let WorthQueryPrimaryGranularMaintenanceOutcome::NoRelevantChange(no_change) = outcome else {
        panic!("the 10y-only consumer must not maintain from a 5y change")
    };
    assert_eq!(no_change.lower_truth_delivery_count(), 3);
    assert_eq!(no_change.lower_signal_performed_delivery_count(), 1);
    assert_eq!(no_change.irrelevant_delivery_count(), 3);
}

pub fn assert_financial_curve_query_patch() {
    use worth_query::facade::domain::{
        bind_primary_runtime_granular_invalidations, maintain_primary_runtime_granular_batch,
        WorthQueryPrimaryGranularMaintenanceOutcome,
    };
    use worth_query_host::facade::primary_graph::WorthQueryConditionalClockObservationOutcome;

    let mut host = host::FinancialCourtroomWorld::publish_curve();
    let mut query = query::build_curve(&host);
    let baseline = host.conditional_clock(&host.curve_clock).unwrap().observe();
    assert!(matches!(
        baseline,
        WorthQueryConditionalClockObservationOutcome::Accepted(_)
    ));
    host.amend_curve(2, 4_260, 5_100);
    host.curve_gate.release();
    host.curve_clock_control.push(2, 11);
    let WorthQueryConditionalClockObservationOutcome::Accepted(mut receipt) =
        host.conditional_clock(&host.curve_clock).unwrap().observe()
    else {
        panic!("the financial curve release must be accepted")
    };
    let binding = bind_primary_runtime_granular_invalidations(
        &query.live,
        host.application.granular_invalidation_installation(),
    );
    let outcome = maintain_primary_runtime_granular_batch(
        &query.live,
        &mut query.workspace,
        &binding,
        receipt.take_granular_invalidation_batch(),
    )
    .expect("the exact 5y curve delivery must maintain the financial Query projection");
    let WorthQueryPrimaryGranularMaintenanceOutcome::Performed(performed) = outcome else {
        panic!("the curve bump must perform Query-owned risk maintenance")
    };
    assert_eq!(performed.admitted_impact_count(), 3);
    assert_eq!(performed.maintenance_operation_count(), 1);
    assert_eq!(performed.consumer_publication_count(), 1);
    assert_eq!(performed.maintenance_counters().coalesced_impacts(), 2);
    let effect = performed.deliveries()[0].effect();
    let fields = effect
        .projection_patch()
        .map(|patch| patch.fields())
        .or_else(|| effect.indexed_live_patch().map(|patch| patch.fields()))
        .expect("performed financial maintenance must publish a concrete Query patch");
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].as_uint64(), Ok(&5_120));
}

pub fn assert_suppressed_quote_has_no_query_patch() {
    use worth_query::facade::domain::{
        bind_primary_runtime_granular_invalidations, maintain_primary_runtime_granular_batch,
        WorthQueryPrimaryGranularMaintenanceOutcome,
    };
    use worth_query_host::facade::primary_graph::WorthQueryConditionalClockObservationOutcome;

    let mut host = host::FinancialCourtroomWorld::publish_quote();
    let mut query = query::build_quote(&host);
    host.quote_gate.release();
    let WorthQueryConditionalClockObservationOutcome::Accepted(_) =
        host.conditional_clock(&host.quote_clock).unwrap().observe()
    else {
        panic!("the quote baseline must commit its producer output")
    };
    let binding = bind_primary_runtime_granular_invalidations(
        &query.live,
        host.application.granular_invalidation_installation(),
    );

    host.amend_quote(2, 102, 5_120);
    host.quote_clock_control.push(2, 11);
    let WorthQueryConditionalClockObservationOutcome::Accepted(mut small) =
        host.conditional_clock(&host.quote_clock).unwrap().observe()
    else {
        panic!("the small quote move must be observed")
    };
    let small_batch = small.take_granular_invalidation_batch();
    assert_eq!(small_batch.observation().direct_truth_delivery_count(), 1);
    let small = maintain_primary_runtime_granular_batch(
        &query.live,
        &mut query.workspace,
        &binding,
        small_batch,
    )
    .expect("the suppressed quote delivery must remain a lawful no-change outcome");
    let WorthQueryPrimaryGranularMaintenanceOutcome::NoRelevantChange(small) = small else {
        panic!("a quote move within producer tolerance must publish no Query patch")
    };
    assert_eq!(small.suppressed_impact_count(), 1);

    host.amend_quote(3, 110, 5_120);
    host.quote_clock_control.push(3, 12);
    let large_observation = host.conditional_clock(&host.quote_clock).unwrap().observe();
    let mut large = match large_observation {
        WorthQueryConditionalClockObservationOutcome::Accepted(receipt) => receipt,
        WorthQueryConditionalClockObservationOutcome::Duplicate(_) => {
            panic!("the large quote move was duplicate")
        }
        WorthQueryConditionalClockObservationOutcome::Stale => {
            panic!("the large quote move was stale")
        }
        WorthQueryConditionalClockObservationOutcome::Reordered => {
            panic!("the large quote move was reordered")
        }
        WorthQueryConditionalClockObservationOutcome::Closed => {
            panic!("the large quote move found a closed runtime")
        }
        WorthQueryConditionalClockObservationOutcome::Failed(failure) => {
            panic!("the large quote move failed: {}", failure.detail())
        }
    };
    let large = maintain_primary_runtime_granular_batch(
        &query.live,
        &mut query.workspace,
        &binding,
        large.take_granular_invalidation_batch(),
    )
    .expect("the meaningful quote move must maintain Query risk");
    let large = match large {
        WorthQueryPrimaryGranularMaintenanceOutcome::Performed(performed) => performed,
        WorthQueryPrimaryGranularMaintenanceOutcome::NoRelevantChange(no_change) => panic!(
            "the quote move outside tolerance produced no Query patch: truth={}, signal={}, duplicate={}, settled={}, irrelevant={}, suppressed={}",
            no_change.lower_truth_delivery_count(),
            no_change.lower_signal_performed_delivery_count(),
            no_change.duplicate_delivery_count(),
            no_change.already_settled_delivery_count(),
            no_change.irrelevant_delivery_count(),
            no_change.suppressed_impact_count(),
        ),
    };
    let effect = large.deliveries()[0].effect();
    let fields = effect
        .projection_patch()
        .map(|patch| patch.fields())
        .or_else(|| effect.indexed_live_patch().map(|patch| patch.fields()))
        .expect("meaningful quote maintenance must carry Query-owned fields");
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].as_uint64(), Ok(&5_120));
}

pub use frontier_slope::assert_frontier_expansion_slope;
pub use portfolio::assert_ordered_portfolio_membership;
pub use shared::{
    assert_shared_financial_disclosure_revalidation,
    assert_shared_financial_execution_and_publication,
};
