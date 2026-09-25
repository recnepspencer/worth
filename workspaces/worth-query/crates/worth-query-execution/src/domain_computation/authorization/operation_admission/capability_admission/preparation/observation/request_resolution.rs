//! Exact typed capability-request resolution owned by capability admission.

use std::collections::BTreeMap;

use worth_query_declaration::facade::{
    application_capability::{
        ApplicationCapabilityContextEntitySlotBinding, ApplicationCapabilityEntitySelector,
        ApplicationCapabilityRequestProjection, ErasedApplicationCapabilityEntitySelector,
    },
    application_schema::ApplicationSchemaMember,
};
use worth_query_installation::facade::{ApplicationSchema, WorthQueryInstalledApplicationSchema};
use worth_relational::facade::identity::EntityId;

use crate::domain_computation::authorization::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationEntityIdentity, WorthQueryEntityResolutionTruth, WorthQueryResolvedEntity,
};

pub(in crate::domain_computation::authorization) struct WorthQueryResolvedCapabilityRequest<
    Schema,
    Scope,
> {
    resource: WorthQueryApplicationEntityIdentity<Schema, Scope>,
    elevation: Option<EntityId>,
    related: Option<EntityId>,
    context: BTreeMap<WorthQueryCapabilityContextKey, EntityId>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::domain_computation::authorization) struct WorthQueryCapabilityContextKey {
    context: String,
    context_type: String,
    slot: String,
    slot_type: String,
    entity: String,
}

impl<Schema, Scope> WorthQueryResolvedCapabilityRequest<Schema, Scope> {
    pub(in crate::domain_computation::authorization) fn resource_entity_id(&self) -> EntityId {
        self.resource.entity_id()
    }

    pub(super) fn resource_entity_kind(&self) -> worth_relational::facade::identity::KindId {
        self.resource.entity_kind()
    }

    pub(super) fn resource_entity_name(&self) -> &str {
        self.resource.entity_name()
    }

    pub(in crate::domain_computation::authorization) const fn elevation(&self) -> Option<EntityId> {
        self.elevation
    }

    pub(in crate::domain_computation::authorization) const fn related(&self) -> Option<EntityId> {
        self.related
    }

    pub(in crate::domain_computation::authorization) fn retained_context(
        &self,
    ) -> BTreeMap<WorthQueryCapabilityContextKey, EntityId> {
        self.context.clone()
    }

    pub(in crate::domain_computation::authorization) fn context_entity(
        &self,
        slot: &ApplicationCapabilityContextEntitySlotBinding,
    ) -> Option<EntityId> {
        self.context
            .get(&WorthQueryCapabilityContextKey::from_slot(slot))
            .copied()
    }
}

impl WorthQueryCapabilityContextKey {
    pub(in crate::domain_computation::authorization) fn from_anchor(
        anchor: &crate::domain_computation::authorization::capability_registry::WorthQueryCapabilityContextAnchor,
    ) -> Self {
        Self {
            context: anchor.context.clone(),
            context_type: anchor.context_type.clone(),
            slot: anchor.slot.clone(),
            slot_type: anchor.slot_type.clone(),
            entity: anchor.entity.clone(),
        }
    }

    pub(in crate::domain_computation::authorization) fn from_slot(
        slot: &ApplicationCapabilityContextEntitySlotBinding,
    ) -> Self {
        Self {
            context: slot.context().to_string(),
            context_type: slot.context_identity().as_str().to_string(),
            slot: slot.slot().to_string(),
            slot_type: slot.slot_identity().as_str().to_string(),
            entity: slot.entity().to_string(),
        }
    }
}

pub(super) fn resolve_capability_request<Schema, Scope, Context>(
    truth: &WorthQueryEntityResolutionTruth<'_>,
    schema: &WorthQueryInstalledApplicationSchema<Schema>,
    projection: &ApplicationCapabilityRequestProjection<Schema, Scope, Context>,
) -> Result<
    WorthQueryResolvedCapabilityRequest<Schema, Scope>,
    WorthQueryOperationAuthorizationDenial,
>
where
    Schema: ApplicationSchema,
{
    let resource = resolve_selector(truth, schema, projection.resource())?;
    let elevation = projection
        .elevation_selector()
        .map(|selector| {
            resolve_erased_selector(truth, schema, selector).map(|resolved| resolved.entity_id())
        })
        .transpose()?;
    let related = projection
        .related()
        .map(|related| {
            let selector = related.selector();
            resolve_erased_selector(truth, schema, selector).map(|resolved| resolved.entity_id())
        })
        .transpose()?;
    let mut context = BTreeMap::new();
    for selected in projection.context_value().entities() {
        let slot = selected.slot();
        let selector = selected.selector();
        if selector.entity() != slot.entity() {
            return Err(denial(slot.slot()));
        }
        let evidence = resolve_erased_selector(truth, schema, selector)?;
        let key = WorthQueryCapabilityContextKey {
            context: slot.context().to_string(),
            context_type: slot.context_identity().as_str().to_string(),
            slot: slot.slot().to_string(),
            slot_type: slot.slot_identity().as_str().to_string(),
            entity: slot.entity().to_string(),
        };
        if context.insert(key, evidence.entity_id()).is_some() {
            return Err(denial(slot.slot()));
        }
    }
    Ok(WorthQueryResolvedCapabilityRequest {
        resource: resource.into_application_identity(),
        elevation,
        related,
        context,
    })
}

pub(super) fn resolve_erased_selector<Schema>(
    truth: &WorthQueryEntityResolutionTruth<'_>,
    schema: &WorthQueryInstalledApplicationSchema<Schema>,
    selector: &ErasedApplicationCapabilityEntitySelector,
) -> Result<WorthQueryResolvedEntity, WorthQueryOperationAuthorizationDenial>
where
    Schema: ApplicationSchema,
{
    resolve_selector(truth, schema, selector)
}

fn resolve_selector<Schema>(
    truth: &WorthQueryEntityResolutionTruth<'_>,
    schema: &WorthQueryInstalledApplicationSchema<Schema>,
    selector: &impl WorthQueryCapabilityEntitySelector,
) -> Result<WorthQueryResolvedEntity, WorthQueryOperationAuthorizationDenial>
where
    Schema: ApplicationSchema,
{
    let installed = schema
        .installed_declaration()
        .members()
        .iter()
        .any(|member| {
            matches!(
                member,
                ApplicationSchemaMember::Field {
                    entity: installed_entity,
                    aspect: installed_aspect,
                    field: installed_field,
                    scalar_family: installed_scalar,
                    value_type: installed_value_type,
                    equality_queryable: true,
                    ..
                } if installed_entity == selector.entity()
                    && installed_aspect == selector.aspect()
                    && installed_field == selector.field()
                    && *installed_scalar == selector.scalar_family()
                    && installed_value_type == selector.value_type()
            )
        });
    if !installed {
        return Err(denial(selector.field()));
    }
    truth
        .resolve(
            selector.entity(),
            selector.aspect(),
            selector.field(),
            selector.value().clone(),
        )
        .map_err(|_| denial(selector.field()))
}

trait WorthQueryCapabilityEntitySelector {
    fn entity(&self) -> &str;
    fn aspect(&self) -> &str;
    fn field(&self) -> &str;
    fn scalar_family(&self) -> worth_foundational::facade::ScalarAspectType;
    fn value_type(&self) -> &str;
    fn value(&self) -> &worth_foundational::facade::AspectValue;
}

impl<Schema, Entity> WorthQueryCapabilityEntitySelector
    for ApplicationCapabilityEntitySelector<Schema, Entity>
{
    fn entity(&self) -> &str {
        self.entity()
    }
    fn aspect(&self) -> &str {
        self.aspect()
    }
    fn field(&self) -> &str {
        self.field()
    }
    fn scalar_family(&self) -> worth_foundational::facade::ScalarAspectType {
        self.scalar_family()
    }
    fn value_type(&self) -> &str {
        self.value_type()
    }
    fn value(&self) -> &worth_foundational::facade::AspectValue {
        self.value()
    }
}

impl WorthQueryCapabilityEntitySelector for ErasedApplicationCapabilityEntitySelector {
    fn entity(&self) -> &str {
        self.entity()
    }
    fn aspect(&self) -> &str {
        self.aspect()
    }
    fn field(&self) -> &str {
        self.field()
    }
    fn scalar_family(&self) -> worth_foundational::facade::ScalarAspectType {
        self.scalar_family()
    }
    fn value_type(&self) -> &str {
        self.value_type()
    }
    fn value(&self) -> &worth_foundational::facade::AspectValue {
        self.value()
    }
}

fn denial(subject: impl Into<String>) -> WorthQueryOperationAuthorizationDenial {
    WorthQueryOperationAuthorizationDenial::new(
        WorthQueryOperationAuthorizationDenialKind::CapabilityProjectionRejected,
        subject,
    )
}
