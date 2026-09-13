use std::sync::Arc;

use worth_foundational::facade::{
    AspectBinding, AspectKey, AspectMask, AuthoritativeAspectChangeKind, FieldKey, ProjectionMask,
    ScalarAspectType,
};
use worth_relational::facade::{
    bridge::RuntimeBridgeRelationalSource, runtime::RelationalRuntimeApi,
};
use worth_runtime_bridge::facade::{
    BridgeAspectRegistration, BridgeAspectRegistrationId, BridgeDeliveryReceipt, BridgeMappingId,
    BridgeMappingRegistration, BridgeSemanticCorrespondenceRegistration,
    BridgeSemanticDependencyCandidate, BridgeSemanticDependencyCandidateParts,
    BridgeSemanticLocality, BridgeSignalAspectTargetDeclaration, CoarseRoutingMode,
    InvalidationSink, MappingSelector, RuntimeBridgeBuilder, SignalBridgeSinkError,
    SignalInvalidationScope, SliceWideningPolicy, SnapshotReadContract, SubscriptionSliceKind,
    TruthDeltaSurfaceKind, TruthPatchScope, TruthPatchTargetSelector,
};
use worth_signal::facade::{SignalGraph, SignalRuntime};

use crate::basis::AdmittedCompositeRuntimeWorldBasis;
use crate::identity::RuntimeWorldOwnerIdentity;
use crate::lifecycle::owner::RuntimeWorldOwnerConstructionContract;
use crate::retention::RuntimeWorldRetentionOwner;

mod artifacts;
mod owner_inputs;
pub(crate) use artifacts::{
    bootstrap_product_head_protection, initial_snapshot, initial_snapshot_named, install_ordinary,
    installed_root, product_head_protection, successor_snapshot,
};

#[derive(Clone)]
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

pub(crate) struct RealReferenceFixture {
    pub(super) owner: RuntimeWorldRetentionOwner<(), (), ()>,
    pub(super) owner_identity: RuntimeWorldOwnerIdentity,
    pub(super) basis: AdmittedCompositeRuntimeWorldBasis,
    pub(super) identities: RuntimeWorldOwnerConstructionContract,
    _signal_port: worth_signal::facade::branch::SignalBranchBasisPort<(), (), ()>,
    _relational_runtime: Arc<worth_relational::facade::runtime::RelationalRuntime>,
    _signal_runtime: SignalRuntime<(), (), (), (), ()>,
    _correspondence_port: worth_runtime_bridge::facade::RuntimeWorldCorrespondencePort,
}

pub(crate) fn real_fixture(unique_pin_limit: u64, reservation_limit: u64) -> RealReferenceFixture {
    let relational_runtime = Arc::new(RelationalRuntimeApi::builder().build());
    let relational_port = relational_runtime.owner_component_services().basis_port();
    let relational_identity = relational_runtime.main_branch_identity();
    let (_, relational) = relational_runtime
        .observe_branch(&relational_identity)
        .expect("real Relational owner admits its main basis");

    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let worth_proof::TransitionOutcome::Success(node) = graph.admit_installed_node(node) else {
        panic!("real Signal owner admits its installed node");
    };
    let aspect_key = AspectKey::new("profile").expect("valid aspect key");
    let field_key = FieldKey::new("name").expect("valid field key");
    let snapshot_contract =
        SnapshotReadContract::scalar(aspect_key.clone(), ScalarAspectType::String);
    let dependency_contract = snapshot_contract.aspect_contract().clone();
    let mapping = BridgeMappingRegistration::new(
        BridgeMappingId::from_stable_name("profile-name"),
        TruthPatchScope::for_target(
            MappingSelector::exact("relational-record:entity:0:1:1"),
            aspect_key.clone(),
            TruthPatchTargetSelector::authoritative_aspect(),
        ),
        snapshot_contract.clone(),
        SignalInvalidationScope::from_stable_name("signal.profile.name"),
        CoarseRoutingMode::Direct,
    );
    let aspect_registration = BridgeAspectRegistration::new(
        BridgeAspectRegistrationId::from_stable_name("profile-name"),
        mapping.truth_scope().clone(),
        snapshot_contract,
        TruthDeltaSurfaceKind::AuthoritativeAspect,
        SubscriptionSliceKind::SignalField,
        SliceWideningPolicy::Disallow,
    );
    let target = BridgeSignalAspectTargetDeclaration::allocate(
        BridgeAspectRegistrationId::from_stable_name("profile-name"),
        worth_signal::facade::PartitionToken::new("bridge-main"),
        node,
    );
    let source = RuntimeBridgeRelationalSource::for_graph_role(
        Arc::clone(&relational_runtime),
        "runtime-world-reference-test",
    )
    .expect("real Relational owner provides the Bridge source");
    let source_profile = source.authoritative_source_profile();
    let dependency =
        BridgeSemanticDependencyCandidate::admit(BridgeSemanticDependencyCandidateParts {
            source_installation_identity: Arc::from("relational-installation:reference-test"),
            source_basis: Arc::from("relational-main"),
            source_runtime_authority: source_profile.runtime_instance_id(),
            source_installation_generation: 1,
            source_authority_binding_identity: Arc::from("relational-owner-binding:reference"),
            source_stage_identity: None,
            source_node_identity: Arc::from("runtime-world-reference-test"),
            dependency_ordinal: 0,
            declared_graph_role: Arc::from("runtime-world-reference-test"),
            graph_participation_identity: Arc::from("runtime-world-reference-graph"),
            graph_adapter_identity: Arc::from("relational-bridge-adapter"),
            source_record_identity: Some(
                worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts::entity(0, 1, 1),
            ),
            observation_record_identity: Some(
                worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts::entity(0, 1, 1),
            ),
            contract: dependency_contract,
            projection_mask: AspectMask::<ProjectionMask>::whole_aspect(),
            binding: AspectBinding::EntityField { field: field_key },
            locality: BridgeSemanticLocality::SourceRecord,
            relevant_changes: vec![AuthoritativeAspectChangeKind::FieldSet],
        })
        .expect("real Bridge owner admits its semantic dependency");
    let registration =
        BridgeSemanticCorrespondenceRegistration::new(dependency.clone(), vec![target])
            .expect("real Bridge registration is coherent");
    let bridge = RuntimeBridgeBuilder::new()
        .with_relational_source(source)
        .with_signal_sink(AdmissionSink)
        .register_mapping(mapping)
        .register_aspect_mapping(aspect_registration)
        .register_semantic_correspondence(registration)
        .build()
        .expect("real Bridge owner builds the installed correspondence");
    let installed = {
        let mut binding = bridge
            .bind_signal_graph(&mut graph)
            .expect("Bridge binds the real Signal graph");
        match binding.install_semantic_correspondence(dependency) {
            worth_proof::TransitionOutcome::Success(installed) => installed,
            other => panic!("real Bridge owner admits its correspondence: {other:?}"),
        }
    };
    let correspondence_port = bridge.runtime_world_correspondence_port();
    let correspondence = correspondence_port
        .admit_installed_basis(&installed)
        .expect("Bridge port admits its installed correspondence basis");

    let mut signal_runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    let signal_port = signal_runtime
        .owner_component_services()
        .expect("real Signal owner issues its basis service")
        .basis_port();
    let signal = signal_runtime
        .observe_signal_branch_basis(signal_runtime.current_branch())
        .expect("real Signal owner admits its current basis");

    let identities = RuntimeWorldOwnerConstructionContract::new()
        .expect("explicit Runtime World owner identity issuance");
    let owner_identity = identities.owner_identity();
    let basis = crate::basis::admit_current(
        identities.issuer(),
        &relational_port,
        &signal_port,
        &correspondence_port,
        relational.clone(),
        signal.clone(),
        correspondence,
    )
    .expect("real Relational+Signal+Bridge tuple is admitted");
    let budgets = crate::budget::RuntimeWorldBudgets::install(
        crate::budget::RuntimeWorldBudgetInstallation {
            branches: crate::budget::RuntimeWorldBranchBudgetInstallation {
                live_product_branches: 1,
            },
            history: crate::budget::RuntimeWorldHistoryBudgetInstallation {
                retained_composite_commits: 1,
                history_metadata_bytes: 1,
            },
            observations: crate::budget::RuntimeWorldObservationBudgetInstallation {
                active_observations: 3,
            },
            publication: crate::budget::RuntimeWorldPublicationBudgetInstallation {
                active_publication_attempts: 1,
            },
            recovery: crate::budget::RuntimeWorldRecoveryBudgetInstallation {
                retained_product_unpublished_records: 1,
                retained_partial_metadata_bytes: 1,
            },
            retention: crate::budget::RuntimeWorldRetentionBudgetInstallation {
                unique_exact_component_pins: unique_pin_limit,
                in_flight_pin_acquisition_reservations: reservation_limit,
            },
            custody: crate::budget::RuntimeWorldCustodyBudgetInstallation {
                owner_created_component_custody_records: 1,
            },
        },
    )
    .expect("positive Runtime World retention budgets");
    let owner = RuntimeWorldRetentionOwner::new(
        owner_identity,
        relational_port,
        signal_port.clone(),
        budgets.unique_exact_component_pins(),
        budgets.in_flight_pin_acquisition_reservations(),
        budgets.active_observations(),
    );
    RealReferenceFixture {
        owner,
        owner_identity,
        basis,
        identities,
        _signal_port: signal_port,
        _relational_runtime: relational_runtime,
        _signal_runtime: signal_runtime,
        _correspondence_port: correspondence_port,
    }
}
