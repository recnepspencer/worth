use worth_runtime_bridge::facade::{
    AspectKeySelector, BridgeAspectRegistration, BridgeAspectRegistrationId, BridgeDeliveryReceipt,
    BridgeMappingId, BridgeMappingRegistration, CoarseRoutingMode, InvalidationSink,
    MappingSelector, RuntimeBridge, RuntimeBridgeBuilder, SignalBridgeSinkError,
    SignalInvalidationScope, SliceWideningPolicy, SnapshotReadContract, SubscriptionSliceKind,
    TruthDeltaSurfaceKind, TruthPatchScope, TruthPatchTargetSelector,
};

struct Sink;

impl InvalidationSink for Sink {
    fn deliver_invalidation(
        &self,
        delivery: worth_runtime_bridge::facade::BridgeSignalInvalidationDelivery,
        _lease: worth_runtime_bridge::facade::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeDeliveryReceipt, SignalBridgeSinkError> {
        Ok(BridgeDeliveryReceipt::new(
            delivery.invalidation_targets().len(),
            delivery.source_snapshot().clone(),
        ))
    }
}

pub fn runtime_bridge_with_unrelated_mappings(
    dependency: &worth_query::facade::domain::WorthQuerySemanticTruthDependency,
    record: worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts,
    installation: &worth_query_execution::facade::primary_graph::WorthQueryGranularInvalidationInstallation,
    unrelated_mapping_count: usize,
) -> RuntimeBridge {
    runtime_bridge_for_dependency(
        dependency,
        record,
        installation,
        "temporal-primary-intent",
        unrelated_mapping_count,
    )
}

pub fn runtime_bridge_for_dependency(
    dependency: &worth_query::facade::domain::WorthQuerySemanticTruthDependency,
    record: worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts,
    installation: &worth_query_execution::facade::primary_graph::WorthQueryGranularInvalidationInstallation,
    mapping_identity: &str,
    unrelated_mapping_count: usize,
) -> RuntimeBridge {
    runtime_bridge_for_dependencies(
        std::slice::from_ref(dependency),
        record,
        installation,
        mapping_identity,
        unrelated_mapping_count,
    )
}

pub fn runtime_bridge_for_dependencies(
    dependencies: &[worth_query::facade::domain::WorthQuerySemanticTruthDependency],
    record: worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts,
    installation: &worth_query_execution::facade::primary_graph::WorthQueryGranularInvalidationInstallation,
    mapping_identity: &str,
    unrelated_mapping_count: usize,
) -> RuntimeBridge {
    let source = installation
        .retain_primary_graph_integration_handle()
        .relational_bridge_source();
    let dependency = dependencies
        .first()
        .expect("a certification bridge requires an installed dependency");
    let first_identity = mapping_identity_for_dependency(dependencies, 0, mapping_identity);
    let (mapping, aspect) = exact_field_mapping(
        dependency,
        record,
        &first_identity,
        truth_surface_requires_wildcard(dependencies, dependency),
    );
    let mut builder = RuntimeBridgeBuilder::new()
        .with_relational_source(source)
        .with_signal_sink(Sink)
        .register_mapping(mapping)
        .register_aspect_mapping(aspect);
    for (ordinal, dependency) in dependencies.iter().enumerate().skip(1) {
        let identity = mapping_identity_for_dependency(dependencies, ordinal, mapping_identity);
        if dependencies[..ordinal]
            .iter()
            .any(|prior| same_truth_surface(prior, dependency))
        {
            continue;
        }
        let (mapping, aspect) = exact_field_mapping(
            dependency,
            record,
            &identity,
            truth_surface_requires_wildcard(dependencies, dependency),
        );
        builder = builder
            .register_mapping(mapping)
            .register_aspect_mapping(aspect);
    }
    for ordinal in 0..unrelated_mapping_count {
        let identity = format!("temporal-unrelated-{ordinal}");
        let unrelated_record =
            worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts::entity(
                record.partition_id(),
                10_000 + ordinal as u64,
                1,
            );
        let mapping = BridgeMappingRegistration::new(
            BridgeMappingId::from_stable_name(identity.clone()),
            TruthPatchScope::new(
                MappingSelector::exact(unrelated_record.terminal_projection_for_reporting()),
                AspectKeySelector::exact(dependency.contract().key().clone()),
                TruthPatchTargetSelector::entity_field(
                    dependency.projection_mask().paths()[0].fields()[0].clone(),
                ),
            ),
            SnapshotReadContract::new(dependency.contract().clone()),
            SignalInvalidationScope::from_stable_name(identity.clone()),
            CoarseRoutingMode::Direct,
        );
        let aspect = BridgeAspectRegistration::new(
            BridgeAspectRegistrationId::from_stable_name(identity),
            mapping.truth_scope().clone(),
            mapping.snapshot_read_contract().clone(),
            TruthDeltaSurfaceKind::EntityField,
            SubscriptionSliceKind::SignalField,
            SliceWideningPolicy::Disallow,
        );
        builder = builder
            .register_mapping(mapping)
            .register_aspect_mapping(aspect);
    }
    builder
        .build()
        .expect("the certification conditional bridge must build")
}

pub fn mapping_identity_for_dependency(
    dependencies: &[worth_query::facade::domain::WorthQuerySemanticTruthDependency],
    ordinal: usize,
    base: &str,
) -> String {
    if dependencies.len() == 1 {
        return base.to_owned();
    }
    let dependency = &dependencies[ordinal];
    let first = dependencies
        .iter()
        .position(|candidate| same_truth_surface(candidate, dependency))
        .expect("the dependency is in its own installed set");
    format!("{base}-{first}")
}

fn same_truth_surface(
    left: &worth_query::facade::domain::WorthQuerySemanticTruthDependency,
    right: &worth_query::facade::domain::WorthQuerySemanticTruthDependency,
) -> bool {
    left.contract() == right.contract()
        && left.projection_mask() == right.projection_mask()
        && left.binding() == right.binding()
}

fn truth_surface_requires_wildcard(
    dependencies: &[worth_query::facade::domain::WorthQuerySemanticTruthDependency],
    surface: &worth_query::facade::domain::WorthQuerySemanticTruthDependency,
) -> bool {
    dependencies.iter().any(|dependency| {
        same_truth_surface(dependency, surface)
            && !matches!(
                dependency.locality(),
                worth_query::facade::domain::WorthQuerySemanticLocality::SourceRecord
            )
    })
}

fn exact_field_mapping(
    dependency: &worth_query::facade::domain::WorthQuerySemanticTruthDependency,
    record: worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts,
    identity: &str,
    wildcard: bool,
) -> (BridgeMappingRegistration, BridgeAspectRegistration) {
    let mapping = BridgeMappingRegistration::new(
        BridgeMappingId::from_stable_name(identity),
        TruthPatchScope::new(
            if wildcard {
                MappingSelector::any()
            } else {
                MappingSelector::exact(record.terminal_projection_for_reporting())
            },
            AspectKeySelector::exact(dependency.contract().key().clone()),
            TruthPatchTargetSelector::entity_field(
                dependency
                    .projection_mask()
                    .paths()
                    .first()
                    .expect("an exact dependency projects one field")
                    .fields()[0]
                    .clone(),
            ),
        ),
        SnapshotReadContract::new(dependency.contract().clone()),
        SignalInvalidationScope::from_stable_name(identity),
        CoarseRoutingMode::Direct,
    );
    let aspect = BridgeAspectRegistration::new(
        BridgeAspectRegistrationId::from_stable_name(identity),
        mapping.truth_scope().clone(),
        mapping.snapshot_read_contract().clone(),
        TruthDeltaSurfaceKind::EntityField,
        SubscriptionSliceKind::SignalField,
        SliceWideningPolicy::Disallow,
    );
    (mapping, aspect)
}
