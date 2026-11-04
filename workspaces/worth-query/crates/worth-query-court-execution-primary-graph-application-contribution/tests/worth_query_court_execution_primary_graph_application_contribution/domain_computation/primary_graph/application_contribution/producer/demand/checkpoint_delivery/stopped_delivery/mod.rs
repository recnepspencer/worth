//! A real World authority mismatch stops delivery without freezing its peers.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputCheckpoint as Checkpoint;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryPendingOutputDelivery as Delivery;
use worth_query_execution::facade::product::WorthQueryPerformedRelationalProductChangeDeliveryDenialKind as DeliveryDenialKind;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::demand::checkpoint_delivery::PreparedSelectedDelivery;
use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandAdvance;
use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenialKind;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::demand::progression::denial;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::ready_backing::PreparedReadyBacking;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::publication_head::PublicationHeadSelection;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::lifecycle::selected_execution_finish::RunningSelectedExecution;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::advance_admission::WorthQueryOutputDemandAdvanceAdmission as Admission;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::demand_key::WorthQueryOutputDemandKey;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support;
use worth_runtime_bridge::facade::*;
use worth_signal::facade::SignalConditionalArtifactReuse;
use worth_signal::facade::SignalConditionalVersionComparator;

pub struct Compute;
impl BridgeConditionalProviderSemantics for Compute {
    type SemanticContract = ();
    fn semantic_contract(&self) {}
    fn retained_heap_bytes(
        &self,
        _: &(),
    ) -> Result<BridgeConditionalProviderHeapRetention, BridgeConditionalProviderRetentionOverflow>
    {
        Ok(BridgeConditionalProviderHeapRetention::none())
    }
}
impl BridgeConditionalComputeProvider for Compute {
    fn compute(
        &self,
        _: &mut dyn std::any::Any,
    ) -> Result<worth_signal::facade::NodeEvaluationResult, String> {
        panic!("foreign Product authority is refused before a conditional kernel")
    }
}

#[test]
fn a_terminal_delivery_stop_releases_the_head_with_a_peer_still_present() {
    let world = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let mut foreign_receipt = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let change = foreign_receipt
        .take_performed_relational_product_change()
        .unwrap();
    let branch = world
        .application
        .product_runtime()
        .admit_product_branch_for_test(
            world
                .application
                .product_runtime()
                .default_branch_for_test(),
        )
        .unwrap();
    let occurrence = branch.observation_for_test().lifecycle_incarnation();
    let root = worth_relational::facade::identity::EntityId::new(
        worth_relational::facade::identity::PartitionId::main(),
        1,
        1,
    );
    let key = WorthQueryOutputDemandKey::new_for_test("terminal-delivery-family", "terminal-delivery".into(),
        worth_query_execution::facade::application_contribution::WorthQueryProducerApplicability::new("terminal-delivery", worth_query_execution::facade::application_contribution::WorthQueryProducerLifecyclePosture::Preserve),
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::observed_source::source_identity::WorthQueryObservedSourceEpoch::new([1;32], [2;32], root, occurrence, 1, [3;32]));
    let registry = &world.application.runtime.output_demands;
    let scope = worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity(root);
    let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    // Build a genuine lowering, with no dependencies. The foreign root is
    // refused before this lowering or its compute can be used.
    let mut builder = BridgeConditionalRuntimeBuilder::with_owned_signal_graph(
        world.application.runtime.bridge.ordinary().clone(),
        worth_signal::facade::runtime::SignalConditionalEvaluationBudget::development(),
    )
    .unwrap();
    let lowering = builder
        .install_owned_conditional(BridgeOwnedConditionalInstallationRequest {
            contract: BridgeConditionalContract::new(BridgeConditionalContractParts {
                identity: std::sync::Arc::from("terminal-delivery"),
                dependency_count: 0,
                condition_dependency_ordinals: vec![],
                condition: BridgeConditionalCondition::Always,
                dependency_comparator: SignalConditionalVersionComparator::Exact,
                output_comparator: SignalConditionalVersionComparator::Exact,
                artifact_reuse: SignalConditionalArtifactReuse::NotReusable,
            }),
            location: BridgeConditionalLocation::operation("terminal-delivery"),
            dependencies: vec![],
            providers: BridgeConditionalProviderSet::new().compute(Compute),
        })
        .unwrap();
    world.application.with_application_advancement(&request, |phase| {
        let owner = registry.admit_for_test(key.clone(), None, scope, occurrence, worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandAdmissionKind::Ordinary, None, None).map(|(interest, _pause)| interest).unwrap();
        let peer = registry.admit_for_test(key.clone(), None, scope, occurrence, worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandAdmissionKind::Ordinary, None, None).map(|(interest, _pause)| interest).unwrap();
        assert!(matches!(registry.begin_for_test(&owner), Admission::Schedule(None)));
        registry.finish_scheduling_for_test(&owner, None, &mut Ok(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryOutputSchedulingResult::Scheduled));
        let finish = registry.prepare_selected_execution_finish_for_test(&owner).unwrap();
        assert!(matches!(registry.begin_for_test(&owner), Admission::Execute { .. }));
        assert!(RunningSelectedExecution::new(finish).publish_for_test(Checkpoint::Published {
            receipt: foreign_receipt, delivery: Delivery::Change(change),
            ready_backing: PreparedReadyBacking::for_registry_test_for_test(),
        }).is_ok());
        assert_eq!(registry.publication_counts_for_registry_test(), (1, 1));
        let (finish, claim, checkpoint) = registry.prepare_selected_checkpoint_finish_for_test(&owner).unwrap().claim_for_test().unwrap().unwrap();
        let Checkpoint::Published { receipt, delivery, ready_backing } = checkpoint else { panic!("published checkpoint") };
        let stop = world.application.deliver_output_checkpoint_with_finish_for_test(&phase, "terminal-delivery",
            receipt, delivery, ready_backing, true, Some(PreparedSelectedDelivery::new_for_test(&lowering, 0, false)),
            |checkpoint, denial| {
                assert!(finish.finish_for_test(claim, checkpoint, denial.as_ref()).is_ok());
                match denial { Some(stop) => Err(stop), None => Ok(WorthQueryOutputDemandAdvance::Pending) }
            }).err().expect("the foreign root refuses real delivery");
        assert!(matches!(stop.kind(), WorthQueryOutputDemandDenialKind::ProductDelivery(ref cause) if **cause == DeliveryDenialKind::ForeignProductRoot));
        assert_eq!(registry.publication_counts_for_registry_test(), (0, 0));
        assert!(matches!(registry.publication_head_selection_for_test(occurrence).unwrap(), PublicationHeadSelection::Live(_)));
        assert!(matches!(registry.begin_for_test(&peer), Admission::Failed(ref cause) if cause.kind() == stop.kind()));
    }).unwrap();
}
