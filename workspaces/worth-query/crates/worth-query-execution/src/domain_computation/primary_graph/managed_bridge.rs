use worth_query_installation::facade::{
    ApplicationSchema, ApplicationSchemaMember, WorthQueryInstalledApplicationSchema,
};
use worth_runtime_bridge::facade::{
    AspectKeySelector, BridgeAspectRegistration, BridgeAspectRegistrationId,
    BridgeConditionalRuntimeBuilder, BridgeDeliveryReceipt, BridgeMappingId,
    BridgeMappingRegistration, BridgeRuntimePolicy, BridgeSealedRuntimeAssembly,
    BridgeSourceCapability, BridgeSourceCapabilitySet, BridgeTruthViewSelector, CoarseRoutingMode,
    InvalidationSink, MappingSelector, RuntimeBridge, RuntimeBridgeBuilder,
    RuntimeBridgeRelationalSource, SignalBridgeSinkError, SignalInvalidationScope,
    SliceWideningPolicy, SnapshotReadContract, SourceDeclaration, SourceDeclarationIdentity,
    SubscriptionSliceKind, TruthDeltaSurfaceKind, TruthPatchScope, TruthPatchTargetSelector,
};

pub(in crate::domain_computation) mod conditional_execution;
pub(in crate::domain_computation) mod conditional_owner;
pub(in crate::domain_computation::primary_graph) mod home_definition;
mod home_retirement;
mod installed_custody;
use installed_custody::InstalledBridgeCustody;

use super::{
    WorthQueryPrimaryGraphInstallationDenial, WorthQueryPrimaryGraphInstallationDenialKind,
};

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
        let conditional = self.conditional.seal()?;
        Ok(finish_application_bridge(self.ordinary, conditional))
    }
}
#[inline(never)]
fn finish_application_bridge(
    ordinary: RuntimeBridge,
    conditional: BridgeSealedRuntimeAssembly,
) -> WorthQueryInstalledApplicationBridge {
    WorthQueryInstalledApplicationBridge::attach_home_bridge(
        ordinary,
        std::sync::Arc::new(std::sync::RwLock::new(conditional)),
    )
}

pub struct WorthQueryInstalledApplicationBridge {
    custody: Option<InstalledBridgeCustody>,
}

impl WorthQueryInstalledApplicationBridge {
    pub(in crate::domain_computation::primary_graph) fn attach_home_bridge(
        ordinary: RuntimeBridge,
        conditional: std::sync::Arc<std::sync::RwLock<BridgeSealedRuntimeAssembly>>,
    ) -> Self {
        Self {
            custody: Some(InstalledBridgeCustody::Serving {
                ordinary,
                conditional: conditional_owner::ConditionalOwner::Memory(conditional),
            }),
        }
    }

    pub(super) fn ordinary(&self) -> &RuntimeBridge {
        self.custody
            .as_ref()
            .expect("original Bridge custody")
            .ordinary()
            .expect("retired Query exposes no ordinary Bridge")
    }
    pub(in crate::domain_computation::primary_graph) fn conditional_owner(
        &self,
    ) -> Option<&conditional_owner::ConditionalOwner> {
        self.custody
            .as_ref()
            .and_then(InstalledBridgeCustody::conditional)
    }
    pub(in crate::domain_computation) fn conditional(
        &self,
    ) -> conditional_execution::ConditionalExecutionOwner<'_> {
        match self.conditional_owner() {
            Some(owner) => owner
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            None => conditional_execution::ConditionalExecutionOwner::Stopped,
        }
    }
    pub(super) fn conditional_lifecycle(
        &self,
    ) -> Result<
        std::sync::RwLockWriteGuard<'_, BridgeSealedRuntimeAssembly>,
        worth_runtime_bridge::facade::BridgeConditionalDenial,
    > {
        use worth_runtime_bridge::facade::BridgeConditionalDenial;
        match self.conditional_owner() {
            Some(conditional_owner::ConditionalOwner::Memory(owner)) => Ok(owner
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner)),
            Some(conditional_owner::ConditionalOwner::Authored(_)) => {
                Err(BridgeConditionalDenial::operation_route_refused(
                    "authored publication requires the original durable route",
                ))
            }
            Some(conditional_owner::ConditionalOwner::Stopped) | None => {
                Err(BridgeConditionalDenial::owner_stopped())
            }
        }
    }
    pub(super) fn conditional_operations(&self) -> conditional_owner::ConditionalOwner {
        self.conditional_owner()
            .cloned()
            .unwrap_or(conditional_owner::ConditionalOwner::Stopped)
    }
    pub(in crate::domain_computation::primary_graph) fn attach_authored_home_bridge(
        ordinary: RuntimeBridge,
        conditional: std::sync::Arc<worth_runtime_bridge::facade::BridgeReadmittedAuthoredAssembly>,
    ) -> Self {
        Self {
            custody: Some(InstalledBridgeCustody::Serving {
                ordinary,
                conditional: conditional_owner::ConditionalOwner::Authored(conditional),
            }),
        }
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
    let ordinary = build_application_runtime_bridge(schema, layout, source)?;
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

pub(super) fn build_application_runtime_bridge<Schema>(
    schema: &WorthQueryInstalledApplicationSchema<Schema>,
    layout: &super::schema_layout::WorthQueryPrimaryGraphLayout,
    source: RuntimeBridgeRelationalSource,
) -> Result<RuntimeBridge, WorthQueryPrimaryGraphInstallationDenial>
where
    Schema: ApplicationSchema,
{
    home_definition::prepare_home_bridge_definition(schema, layout)?
        .with_relational_source(source.clone())
        .with_truth_branch_head_source(source.clone())
        .with_source_adapter(source)
        .build()
        .map_err(|error| bridge_denial(format!("{error:?}")))
}

pub(crate) fn build_primary_graph_product_bridge<Schema>(
    schema: &WorthQueryInstalledApplicationSchema<Schema>,
    integration: &super::WorthQueryPrimaryGraphIntegrationHandle,
) -> Result<RuntimeBridge, WorthQueryPrimaryGraphInstallationDenial>
where
    Schema: ApplicationSchema,
{
    build_application_runtime_bridge(
        schema,
        &integration.layout,
        integration.source_owner.bridge_source(),
    )
}

fn application_mappings<Schema>(
    schema: &WorthQueryInstalledApplicationSchema<Schema>,
    layout: &super::schema_layout::WorthQueryPrimaryGraphLayout,
) -> Result<
    impl Iterator<Item = WorthQueryApplicationFieldMappings> + use<Schema>,
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

impl InvalidationSink for WorthQueryApplicationGranularOnlySink {
    fn deliver_invalidation(
        &self,
        _delivery: worth_runtime_bridge::facade::BridgeSignalInvalidationDelivery,
        _lease: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeDeliveryReceipt, SignalBridgeSinkError> {
        Err(SignalBridgeSinkError::new(
            worth_runtime_bridge::facade::SignalBridgeSinkErrorKind::WorldPublicationRequired,
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

mod court_doors_for_test;
