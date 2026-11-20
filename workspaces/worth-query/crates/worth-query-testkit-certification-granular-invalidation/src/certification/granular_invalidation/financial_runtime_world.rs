#[path = "financial_runtime_world/adapters.rs"]
pub mod adapters;
#[path = "financial_runtime_world/contract.rs"]
pub mod contract;
#[path = "financial_runtime_world/host.rs"]
pub mod host;
#[path = "financial_runtime_world/schema.rs"]
pub mod schema;

pub fn assert_financial_host_curve_delivery(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    use worth_query_host::facade::primary_graph::WorthQueryConditionalClockObservationOutcome;

    let mut world = host::FinancialCourtroomWorld::publish_curve();
    let baseline = world
        .conditional_clock(&world.curve_clock)
        .unwrap()
        .observe(request_execution);
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
        .observe(request_execution);
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
            assert_eq!(
                dependency.contract().key().as_str(),
                schema::CurveFacts::reference().name()
            );
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
