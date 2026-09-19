use std::sync::Arc;

use super::super::*;
use crate::facade::{ProductBranchCreationIntent, RuntimeWorldOwner};
use worth_foundational::facade::{AspectKey, FieldKey, ScalarAspectType};
use worth_relational::facade::{
    bridge::RuntimeBridgeRelationalSource, runtime::RelationalRuntimeApi,
};
use worth_runtime_bridge::facade::{
    BridgeAspectRegistration, BridgeAspectRegistrationId, BridgeConditionalComputeProvider,
    BridgeConditionalCondition, BridgeConditionalContract, BridgeConditionalContractParts,
    BridgeConditionalLocation, BridgeConditionalProviderSemantics, BridgeConditionalProviderSet,
    BridgeConditionalRuntimeBuilder, BridgeDeliveryReceipt, BridgeInstalledConditionalLowering,
    BridgeMappingId, BridgeMappingRegistration, BridgeOwnedConditionalInstallationRequest,
    BridgeRuntimePolicy, BridgeSealedRuntimeAssembly, CoarseRoutingMode, InvalidationSink,
    MappingSelector, RuntimeBridge, SignalBridgeSinkError, SignalInvalidationScope,
    SliceWideningPolicy, SnapshotReadContract, SubscriptionSliceKind, TruthDeltaSurfaceKind,
    TruthPatchScope, TruthPatchTargetSelector,
};
use worth_signal::facade::{
    NodeEvaluationResult, SignalConditionalArtifactReuse, SignalConditionalVersionComparator,
};

struct AdmissionSink;

impl InvalidationSink for AdmissionSink {
    fn deliver_invalidation(
        &self,
        delivery: worth_runtime_bridge::facade::BridgeSignalInvalidationDelivery,
    ) -> Result<BridgeDeliveryReceipt, SignalBridgeSinkError> {
        Ok(BridgeDeliveryReceipt::new(
            delivery.invalidation_targets().len(),
            delivery.source_snapshot().clone(),
        ))
    }
}

struct Compute(u64);

impl BridgeConditionalProviderSemantics for Compute {
    type SemanticContract = u64;

    fn semantic_contract(&self) -> Self::SemanticContract {
        self.0
    }

    fn retained_heap_bytes(
        &self,
        _: &Self::SemanticContract,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention,
        worth_runtime_bridge::facade::BridgeConditionalProviderRetentionOverflow,
    > {
        Ok(worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention::none())
    }
}

impl BridgeConditionalComputeProvider for Compute {
    fn compute(&self, _context: &mut dyn std::any::Any) -> Result<NodeEvaluationResult, String> {
        Ok(NodeEvaluationResult::from_version(
            worth_signal::facade::AspectVersion::from_updates([(
                worth_signal::facade::Aspect::new(0),
                self.0,
            )]),
        ))
    }
}

pub(super) struct DefinitionWorld {
    pub(super) _relational: Arc<worth_relational::facade::runtime::RelationalRuntime>,
    pub(super) owner: RuntimeWorldOwner<(), (), (), (), ()>,
    pub(super) bridge: BridgeSealedRuntimeAssembly,
    pub(super) lowering: Arc<BridgeInstalledConditionalLowering>,
    pub(super) root: ProductBranchObservation,
}

pub(super) fn source_free_request(version: u64) -> BridgeOwnedConditionalInstallationRequest {
    BridgeOwnedConditionalInstallationRequest {
        contract: BridgeConditionalContract::new(BridgeConditionalContractParts {
            identity: Arc::from("query:definition-generation"),
            dependency_count: 0,
            condition_dependency_ordinals: Vec::new(),
            condition: BridgeConditionalCondition::Always,
            dependency_comparator: SignalConditionalVersionComparator::Exact,
            output_comparator: SignalConditionalVersionComparator::Exact,
            artifact_reuse: SignalConditionalArtifactReuse::NotReusable,
        }),
        location: BridgeConditionalLocation::operation("query:definition-generation"),
        dependencies: Vec::new(),
        providers: BridgeConditionalProviderSet::new().compute(Compute(version)),
    }
}

pub(super) fn definition_world() -> DefinitionWorld {
    definition_world_with_bridge_budget(
        worth_runtime_bridge::facade::BridgeConditionalRetentionBudget::development(),
    )
}

pub(super) fn definition_world_with_bridge_budget(
    retention: worth_runtime_bridge::facade::BridgeConditionalRetentionBudget,
) -> DefinitionWorld {
    let relational = Arc::new(RelationalRuntimeApi::builder().build());
    let relational_identity = relational.main_branch_identity();
    let (_, relational_basis) = relational.observe_branch(&relational_identity).unwrap();
    let source = RuntimeBridgeRelationalSource::for_graph_role(
        Arc::clone(&relational),
        "definition-generation-test",
    )
    .unwrap();
    let aspect = AspectKey::new("definition-test").unwrap();
    let read = SnapshotReadContract::scalar(aspect.clone(), ScalarAspectType::String);
    let scope = TruthPatchScope::for_target(
        MappingSelector::exact("definition-test-record"),
        aspect,
        TruthPatchTargetSelector::entity_field(FieldKey::new("value").unwrap()),
    );
    let mapping = BridgeMappingRegistration::new(
        BridgeMappingId::from_stable_name("definition-test"),
        scope.clone(),
        read.clone(),
        SignalInvalidationScope::from_stable_name("definition-test"),
        CoarseRoutingMode::Direct,
    );
    let bridge = RuntimeBridge::builder()
        .with_policy(BridgeRuntimePolicy::development().with_conditional_retention(retention))
        .with_relational_source(source)
        .with_signal_sink(AdmissionSink)
        .register_mapping(mapping)
        .register_aspect_mapping(BridgeAspectRegistration::new(
            BridgeAspectRegistrationId::from_stable_name("definition-test"),
            scope,
            read,
            TruthDeltaSurfaceKind::EntityField,
            SubscriptionSliceKind::SignalField,
            SliceWideningPolicy::Disallow,
        ))
        .build()
        .unwrap();
    let mut builder = BridgeConditionalRuntimeBuilder::with_owned_signal_graph(
        bridge,
        worth_signal::facade::runtime::SignalConditionalEvaluationBudget::development(),
    )
    .unwrap();
    let lowering = builder
        .install_owned_conditional(source_free_request(1))
        .unwrap();
    let mut bridge = builder.seal().unwrap();
    let signal_basis = bridge.admitted_signal_basis().clone();
    let correspondence_basis = bridge.admitted_runtime_world_correspondence_basis().clone();
    let owner = RuntimeWorldOwner::builder()
        .with_bridge_correspondence(bridge.runtime_world_correspondence_port())
        .with_relational_services(relational.owner_component_services())
        .with_signal_services(bridge.signal_owner_services())
        .with_signal_definition_publication(
            bridge
                .take_runtime_world_signal_definition_publication()
                .unwrap(),
        )
        .with_budgets(definition_budgets())
        .with_clock(RuntimeWorldClock::from_source(FixedClock))
        .build()
        .unwrap();
    let root = match owner
        .lifecycle_port()
        .bootstrap_root(crate::facade::RuntimeWorldBootstrapIntent::new(
            ProductBranchCreationIntent::named("definition-root").unwrap(),
            relational_basis,
            signal_basis,
            correspondence_basis,
        ))
        .unwrap()
    {
        RuntimeWorldBootstrapOutcome::Performed(performed) => performed.product_branch().clone(),
        other => panic!("definition World bootstrap must perform: {other:?}"),
    };
    DefinitionWorld {
        _relational: relational,
        owner,
        bridge,
        lowering,
        root,
    }
}

fn definition_budgets() -> RuntimeWorldBudgets {
    RuntimeWorldBudgets::install(RuntimeWorldBudgetInstallation {
        branches: RuntimeWorldBranchBudgetInstallation {
            live_product_branches: 2,
        },
        history: RuntimeWorldHistoryBudgetInstallation {
            retained_composite_commits: 12,
            history_metadata_bytes: 65_536,
        },
        observations: RuntimeWorldObservationBudgetInstallation {
            active_observations: 6,
        },
        publication: RuntimeWorldPublicationBudgetInstallation {
            active_publication_attempts: 4,
        },
        recovery: RuntimeWorldRecoveryBudgetInstallation {
            retained_product_unpublished_records: 4,
            retained_partial_metadata_bytes:
                (crate::recovery::ProductUnpublishedOwnerEffects::metadata_charge_hint() * 4)
                    as u64,
        },
        retention: RuntimeWorldRetentionBudgetInstallation {
            unique_exact_component_pins: 16,
            in_flight_pin_acquisition_reservations: 16,
        },
        custody: RuntimeWorldCustodyBudgetInstallation {
            owner_created_component_custody_records: 4,
        },
    })
    .unwrap()
}
