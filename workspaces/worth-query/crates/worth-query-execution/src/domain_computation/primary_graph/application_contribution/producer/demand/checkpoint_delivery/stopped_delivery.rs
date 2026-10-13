//! A real World authority mismatch stops delivery without freezing its peers.
use super::*;
use crate::domain_computation::primary_graph::{
    application_output_demand::{
        PreparedReadyBacking, PublicationHeadSelection, RunningSelectedExecution,
        WorthQueryOutputDemandAdvanceAdmission as Admission, WorthQueryOutputDemandKey,
    },
    output_lineage::invalidation::InvalidationEditAdmission,
    tests::{fixture, recoverable_commit_support},
};
use worth_runtime_bridge::facade::*;
use worth_signal::facade::{SignalConditionalArtifactReuse, SignalConditionalVersionComparator};

struct Compute;
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
fn admission() -> InvalidationEditAdmission {
    InvalidationEditAdmission::new(worth_relational::facade::mvcc::CompanionPreflightBudget {
        maximum_work_visits: 1_000_000,
        maximum_preparation_bytes: 8 * 1024 * 1024,
    })
}

#[test]
fn a_terminal_delivery_stop_releases_the_head_with_a_peer_still_present() {
    let world = fixture::installed_authorization_world(true);
    let mut foreign_receipt = recoverable_commit_support::committed_recoverable_application();
    let change = foreign_receipt
        .take_performed_relational_product_change()
        .unwrap();
    let branch = world
        .application
        .product_runtime()
        .admit_product_branch(world.application.product_runtime().default_branch())
        .unwrap();
    let occurrence = branch.observation().lifecycle_incarnation();
    let root = worth_relational::facade::identity::EntityId::new(
        worth_relational::facade::identity::PartitionId::main(),
        1,
        1,
    );
    let key = WorthQueryOutputDemandKey::new("terminal-delivery-family", "terminal-delivery".into(),
        crate::domain_computation::primary_graph::application_contribution::WorthQueryProducerApplicability::new("terminal-delivery", crate::domain_computation::primary_graph::application_contribution::WorthQueryProducerLifecyclePosture::Preserve),
        crate::domain_computation::primary_graph::application_query::WorthQueryObservedSourceEpoch::new([1;32], [2;32], root, occurrence, 1, [3;32]));
    let registry = &world.application.output_demands;
    let scope = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(root);
    let request = fixture::live_scope();
    // Build a genuine lowering, with no dependencies. The foreign root is
    // refused before this lowering or its compute can be used.
    let mut builder = BridgeConditionalRuntimeBuilder::with_owned_signal_graph(
        world.application.bridge.ordinary().clone(),
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
        let owner = registry.admit(key.clone(), None, scope, occurrence,
            crate::domain_computation::primary_graph::application_output_demand::DemandAdmissionKind::Ordinary, None, None, &mut admission()).map(|(interest, _pause)| interest).unwrap();
        let peer = registry.admit(key.clone(), None, scope, occurrence,
            crate::domain_computation::primary_graph::application_output_demand::DemandAdmissionKind::Ordinary, None, None, &mut admission()).map(|(interest, _pause)| interest).unwrap();
        assert!(matches!(registry.begin(&owner), Admission::Schedule(None)));
        registry.finish_scheduling(&owner, None, &mut Ok(crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputSchedulingResult::Scheduled));
        let finish = registry.prepare_selected_execution_finish(&owner, &mut admission()).unwrap();
        assert!(matches!(registry.begin(&owner), Admission::Execute { .. }));
        assert!(RunningSelectedExecution::new(finish).publish(Checkpoint::Published {
            receipt: foreign_receipt, delivery: Delivery::Change(change),
            ready_backing: PreparedReadyBacking::for_registry_test(registry),
        }).is_ok());
        assert_eq!(registry.publication_counts_for_registry_test(), (1, 1));
        let (finish, claim, checkpoint) = registry.prepare_selected_checkpoint_finish(&owner, &mut admission()).unwrap().claim().unwrap().unwrap();
        let Checkpoint::Published { receipt, delivery, ready_backing } = checkpoint else { panic!("published checkpoint") };
        let stop = world.application.deliver_output_checkpoint_with_finish(&phase, "terminal-delivery",
            receipt, delivery, ready_backing, true, Some(PreparedSelectedDelivery { lowering: &lowering, ordinal: 0, preserved: false }),
            |checkpoint, denial| {
                assert!(finish.finish(claim, checkpoint, denial.as_ref()).is_ok());
                match denial { Some(stop) => Err(stop), None => Ok(WorthQueryOutputDemandAdvance::Pending) }
            }).err().expect("the foreign root refuses real delivery");
        assert!(matches!(stop.kind(), WorthQueryOutputDemandDenialKind::ProductDelivery(ref cause) if **cause == DeliveryDenialKind::ForeignProductRoot));
        assert_eq!(registry.publication_counts_for_registry_test(), (0, 0));
        assert!(matches!(registry.publication_head_selection(occurrence, &mut admission()).unwrap(), PublicationHeadSelection::Live(_)));
        assert!(matches!(registry.begin(&peer), Admission::Failed(ref cause) if cause.kind() == stop.kind()));
    }).unwrap();
}
