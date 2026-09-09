use worth_query_installation::facade::{
    ApplicationSchema, ApplicationSchemaMember, WorthQueryInstalledApplicationSchema,
};
use worth_relational::facade::bridge::RuntimeBridgeRelationalSource;
use worth_runtime_bridge::facade::{
    AspectKeySelector, BridgeAspectRegistration, BridgeAspectRegistrationId,
    BridgeConditionalRuntimeBuilder, BridgeDeliveryReceipt, BridgeMappingId,
    BridgeMappingRegistration, BridgeRuntimePolicy, BridgeSealedRuntimeAssembly,
    BridgeSourceAdapter, BridgeSourceCapability, BridgeSourceCapabilitySet,
    BridgeTruthViewSelector, CoarseRoutingMode, InvalidationSink, MappingSelector, RuntimeBridge,
    RuntimeBridgeBuilder, SignalBridgeSinkError, SignalInvalidationScope, SliceWideningPolicy,
    SnapshotReadContract, SnapshotReadSource, SourceDeclaration, SourceDeclarationIdentity,
    SubscriptionSliceKind, TruthDeltaSurfaceKind, TruthPatchScope, TruthPatchTargetSelector,
    TruthSnapshotIdentity, TruthSnapshotReader,
};

use super::{
    WorthQueryPrimaryGraphInstallationDenial, WorthQueryPrimaryGraphInstallationDenialKind,
};

#[derive(Clone)]
struct WorthQueryApplicationBridgeSource {
    source: RuntimeBridgeRelationalSource,
}

struct WorthQueryApplicationGranularOnlySink;

struct WorthQueryApplicationFieldMappings {
    routing: BridgeMappingRegistration,
    aspect: BridgeAspectRegistration,
}

pub(super) struct WorthQueryApplicationBridgeInstallation {
    ordinary: RuntimeBridge,
    conditional: BridgeConditionalRuntimeBuilder,
}

impl WorthQueryApplicationBridgeInstallation {
    pub(super) fn conditional_builder(&mut self) -> &mut BridgeConditionalRuntimeBuilder {
        &mut self.conditional
    }

    pub(super) fn seal(
        self,
    ) -> Result<
        WorthQueryInstalledApplicationBridge,
        worth_runtime_bridge::facade::BridgeConditionalDenial,
    > {
        Ok(WorthQueryInstalledApplicationBridge {
            ordinary: self.ordinary,
            conditional: std::sync::Arc::new(std::sync::RwLock::new(self.conditional.seal()?)),
        })
    }
}

pub(crate) struct WorthQueryInstalledApplicationBridge {
    ordinary: RuntimeBridge,
    conditional: std::sync::Arc<std::sync::RwLock<BridgeSealedRuntimeAssembly>>,
}

impl WorthQueryInstalledApplicationBridge {
    pub(super) fn ordinary(&self) -> &RuntimeBridge {
        &self.ordinary
    }

    pub(crate) fn conditional(
        &self,
    ) -> std::sync::RwLockReadGuard<'_, BridgeSealedRuntimeAssembly> {
        self.conditional
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub(super) fn conditional_lifecycle(
        &self,
    ) -> std::sync::RwLockWriteGuard<'_, BridgeSealedRuntimeAssembly> {
        self.conditional
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub(super) fn conditional_operations(
        &self,
    ) -> std::sync::Arc<std::sync::RwLock<BridgeSealedRuntimeAssembly>> {
        std::sync::Arc::clone(&self.conditional)
    }
}

pub(super) fn install_application_bridge<Schema>(
    schema: &WorthQueryInstalledApplicationSchema<Schema>,
    layout: &super::schema_layout::WorthQueryPrimaryGraphLayout,
    source: RuntimeBridgeRelationalSource,
    conditional_evaluation_budget: worth_signal::facade::runtime::SignalConditionalEvaluationBudget,
) -> Result<WorthQueryApplicationBridgeInstallation, WorthQueryPrimaryGraphInstallationDenial>
where
    Schema: ApplicationSchema,
{
    let mut mappings = application_mappings(schema, layout)?;
    let first = mappings.next().ok_or_else(|| {
        bridge_denial("installed application schema has no bridge-readable fields")
    })?;
    let builder = RuntimeBridgeBuilder::new()
        .with_policy(BridgeRuntimePolicy::operational())
        .with_relational_source(source.clone())
        .with_truth_branch_head_source(source.clone())
        .with_source_adapter(WorthQueryApplicationBridgeSource { source })
        .with_signal_sink(WorthQueryApplicationGranularOnlySink)
        .register_source(SourceDeclaration::new(
            SourceDeclarationIdentity::from_stable_name("primary-application-source"),
            BridgeTruthViewSelector::branch_head(
                super::application_branch::primary_truth_branch_identity(),
            ),
            BridgeSourceCapabilitySet::new(vec![
                BridgeSourceCapability::SnapshotRead,
                BridgeSourceCapability::BranchRead,
            ]),
        ))
        .register_aspect_mapping(first.aspect)
        .register_mapping(first.routing);
    let ordinary = mappings
        .fold(builder, |builder, mapping| {
            builder
                .register_aspect_mapping(mapping.aspect)
                .register_mapping(mapping.routing)
        })
        .build()
        .map_err(|error| bridge_denial(format!("{error:?}")))?;
    let conditional = BridgeConditionalRuntimeBuilder::with_owned_signal_graph(
        ordinary.clone(),
        conditional_evaluation_budget,
    )
    .map_err(|error| bridge_denial(format!("{error:?}")))?;
    Ok(WorthQueryApplicationBridgeInstallation {
        ordinary,
        conditional,
    })
}

fn application_mappings<Schema>(
    schema: &WorthQueryInstalledApplicationSchema<Schema>,
    layout: &super::schema_layout::WorthQueryPrimaryGraphLayout,
) -> Result<
    impl Iterator<Item = WorthQueryApplicationFieldMappings>,
    WorthQueryPrimaryGraphInstallationDenial,
>
where
    Schema: ApplicationSchema,
{
    schema
        .installed_declaration()
        .members()
        .iter()
        .filter_map(|member| match member {
            ApplicationSchemaMember::Field {
                entity,
                aspect,
                field,
                scalar_family,
                ..
            } => Some((entity, aspect, field, scalar_family)),
            _ => None,
        })
        .map(|(entity, aspect, field, _scalar_family)| {
            let aspect_key = worth_foundational::facade::AspectKey::new(aspect)
                .ok_or_else(|| bridge_denial(aspect))?;
            let contract = layout
                .aspect_contract(entity, &aspect_key)
                .cloned()
                .ok_or_else(|| bridge_denial(aspect))?;
            let field_key = worth_foundational::facade::FieldKey::new(field)
                .ok_or_else(|| bridge_denial(field))?;
            let identity = format!(
                "application-field:{}:{}:{entity}:{aspect}:{field}",
                schema.owner(),
                schema.schema_name(),
            );
            let snapshot = SnapshotReadContract::new(contract);
            let routing = BridgeMappingRegistration::new(
                BridgeMappingId::from_stable_name(identity),
                TruthPatchScope::for_entity_field(
                    MappingSelector::exact(entity.as_str()),
                    aspect_key.clone(),
                    field_key,
                ),
                snapshot.clone(),
                SignalInvalidationScope::from_stable_name(format!(
                    "application-field-signal:{entity}:{aspect}:{field}"
                )),
                CoarseRoutingMode::Direct,
            );
            let aspect = BridgeAspectRegistration::new(
                BridgeAspectRegistrationId::from_stable_name(format!(
                    "application-field:{entity}:{aspect}:{field}"
                )),
                TruthPatchScope::new(
                    MappingSelector::any(),
                    AspectKeySelector::exact(aspect_key),
                    TruthPatchTargetSelector::entity_field(
                        worth_foundational::facade::FieldKey::new(field)
                            .expect("installed schema field keys are validated"),
                    ),
                ),
                snapshot,
                TruthDeltaSurfaceKind::EntityField,
                SubscriptionSliceKind::SignalField,
                SliceWideningPolicy::Disallow,
            );
            Ok(WorthQueryApplicationFieldMappings { routing, aspect })
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_iter)
}

impl BridgeSourceAdapter for WorthQueryApplicationBridgeSource {
    fn declared_capabilities(&self) -> BridgeSourceCapabilitySet {
        BridgeSourceCapabilitySet::new(vec![
            BridgeSourceCapability::SnapshotRead,
            BridgeSourceCapability::BranchRead,
        ])
    }

    fn open_snapshot(
        &self,
        identity: &TruthSnapshotIdentity,
    ) -> Result<
        Box<dyn TruthSnapshotReader>,
        worth_runtime_bridge::facade::RelationalBridgeSourceError,
    > {
        SnapshotReadSource::open_snapshot(&self.source, identity)
    }
}

impl InvalidationSink for WorthQueryApplicationGranularOnlySink {
    fn deliver_invalidation(
        &self,
        _delivery: worth_runtime_bridge::facade::BridgeSignalInvalidationDelivery,
    ) -> Result<BridgeDeliveryReceipt, SignalBridgeSinkError> {
        Err(SignalBridgeSinkError::new(
            "the primary Query runtime accepts only installed granular correspondence delivery",
        ))
    }
}

fn bridge_denial(subject: impl Into<String>) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(
        WorthQueryPrimaryGraphInstallationDenialKind::RuntimeBridgeRejected,
        subject,
    )
}
