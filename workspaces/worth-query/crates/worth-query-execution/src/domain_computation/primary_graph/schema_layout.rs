use std::collections::{BTreeMap, BTreeSet};
#[cfg(test)]
pub(in crate::domain_computation::primary_graph) use unique_fields::WorthQueryUniqueFieldFixture;

use worth_foundational::facade::{AspectContract, AspectFieldLocator, AspectKey, FieldKey};
use worth_query_installation::facade::{
    ErasedApplicationSchemaDeclaration, WorthQueryInstalledApplicationSchemaContractCatalog,
};
use worth_relational::facade::identity::KindId;
use worth_relational::facade::indexes::DerivedIndexId;
use worth_relational::facade::schema::{DeclaredAspectContractBinding, RelationalSchemaRegistry};

use super::WorthQueryPrimaryGraphInstallationDenial;

mod aggregate_identity;
mod aspect_contract;
mod content;
mod current_publication_ownership;
pub(in crate::domain_computation::primary_graph) use aggregate_identity::ComponentSetLayout;
pub(in crate::domain_computation::primary_graph) use current_publication_ownership::install_current_publication_ownership;
mod application_layout_lowering;
mod authorization_selectors;
mod bootstrap_publication;
mod capability_grant_join;
mod continuation_ordering;
pub(crate) mod derived_relation;
mod equality_fields;
pub(crate) mod field_coordinate;
mod field_optionality;
pub(in crate::domain_computation::primary_graph) use field_coordinate::InstalledFieldCoordinate;
mod history_ownership;
mod installation_primitives;
mod kind_lookup;
mod live_control_roles;
mod managed_source;
mod member_domain;
pub(in crate::domain_computation::primary_graph) use member_domain::MemberDomainBinding;
mod lowering;
mod native_outputs;
mod physical_identity;
mod platform_entity_lowering;
mod platform_identity_allocator;
pub(crate) mod principal_binding;
pub(crate) mod program_activation;
mod projected_member_domain;
mod provider_aftermath_causality;
mod provider_dispatch_outbox;
pub(crate) mod provider_idempotency;
pub(crate) mod provider_inbound_completion;
mod record_selection;
mod registry_lowering;
mod relation_payload;
mod signed_entities;
mod signing_lifecycle;
mod source_registration;
mod support_admission;

pub(crate) mod unique_fields;
mod workflow_reads;

pub(in crate::domain_computation::primary_graph) use unique_fields::{
    WorthQueryUniqueFieldIndex, WorthQueryUniqueFields,
};

use super::workflow::schema::WorthQueryWorkflowLayout;
use crate::domain_computation::application_aftermath::WorthQueryDispatchOutboxLayout;
use application_layout_lowering::{field_capability_keys, lower_fields, lower_relation_layouts};
use capability_grant_join::{lower_capability_grant_joins, WorthQueryCapabilityGrantJoinLayout};
use continuation_ordering::{
    lower_continuation_orderings, WorthQueryPrimaryContinuationOrderingLayout,
};
pub(in crate::domain_computation::primary_graph) use installation_primitives::{
    contract_space_exhausted, invalid_member, kind_space_exhausted, planned_field_locator,
    relational_schema_denial, required_kind, valid_aspect_key, valid_field_key,
};
use platform_entity_lowering::lower_platform_entities;
use principal_binding::lower_principal_bindings;
pub(in crate::domain_computation) use principal_binding::WorthQueryPrimaryPrincipalBindingLayout;
pub(in crate::domain_computation::primary_graph) use program_activation::WorthQueryProgramActivationLayout;
pub(super) use provider_aftermath_causality::WorthQueryAftermathCausalityLayout;
pub(super) use provider_idempotency::WorthQueryProviderIdempotencyLayout;
pub(in crate::domain_computation::primary_graph) use provider_inbound_completion::WorthQueryInboundCompletionLayout;
use registry_lowering::{
    lower_application_contract_bindings, lower_kind_ids, next_provider_kind_id,
    register_application_relation, relational_schema_basis,
};
pub(in crate::domain_computation::primary_graph) use registry_lowering::{
    register_entity, register_relation, register_relation_to_kinds,
};

#[derive(Debug)]
pub struct WorthQueryPrimaryGraphLayout {
    history_ownership: history_ownership::HistoryOwnershipLayouts,
    value_bindings:
        worth_query_installation::facade::WorthQueryInstalledApplicationValueBindingCatalog,
    principal_bindings: BTreeMap<String, WorthQueryPrimaryPrincipalBindingLayout>,
    live_control_entity_kinds: BTreeSet<KindId>,
    authorization_selectors: authorization_selectors::WorthQueryAuthorizationSelectors,
    pub(in crate::domain_computation::primary_graph) physical_roles:
        physical_identity::PhysicalIdentityBindings,
    relation_kinds: BTreeMap<String, WorthQueryPrimaryRelationLayout>,
    component_sets: BTreeMap<String, ComponentSetLayout>,
    derived_relations: BTreeMap<String, derived_relation::DerivedRelationLayout>,
    managed_sources: BTreeMap<String, managed_source::ManagedSource>,
    projected_domains: BTreeMap<String, projected_member_domain::ProjectedMemberDomainLayout>,
    signing: signed_entities::SignedEntityLayout,
    pub(in crate::domain_computation::primary_graph) signing_lifecycle:
        signing_lifecycle::SigningLifecycleLayout,
    fields: BTreeMap<(String, String, String), WorthQueryPrimaryFieldLayout>,
    relation_payload: relation_payload::ApplicationRelationPayloadLayout,
    unique_fields: unique_fields::WorthQueryUniqueFieldNames,
    aspect_contracts: BTreeMap<String, BTreeMap<AspectKey, DeclaredAspectContractBinding>>,
    equality_field_keys: BTreeMap<AspectKey, BTreeSet<FieldKey>>,
    projection_field_keys: BTreeMap<AspectKey, BTreeSet<FieldKey>>,
    continuation_orderings: Vec<WorthQueryPrimaryContinuationOrderingLayout>,
    capability_grant_joins: BTreeMap<(String, String), WorthQueryCapabilityGrantJoinLayout>,
    provider_idempotency: WorthQueryProviderIdempotencyLayout,
    provider_dispatch_outbox: WorthQueryDispatchOutboxLayout,
    provider_inbound_completion: WorthQueryInboundCompletionLayout,
    provider_aftermath_causality: WorthQueryAftermathCausalityLayout,
    program_activation: WorthQueryProgramActivationLayout,
    workflow: WorthQueryWorkflowLayout,
    workflow_reads: Option<super::application_discovery::WorkflowReadIndexes>,
    pub(in crate::domain_computation::primary_graph) bootstrap_publication:
        bootstrap_publication::BootstrapPublicationLayout,
}

#[derive(Clone, Debug)]
pub struct WorthQueryPrimaryRelationLayout {
    pub(in crate::domain_computation) kind: KindId,
    pub(in crate::domain_computation) from: KindId,
    pub(in crate::domain_computation) to: KindId,
    pub(in crate::domain_computation) source_max: Option<u64>,
    pub(in crate::domain_computation) orbit: bool,
}

#[derive(Clone, Debug)]
pub struct WorthQueryPrimaryFieldLayout {
    pub(super) record_presence: Option<AspectFieldLocator>,
    pub(super) entity_kind: KindId,
    pub(super) locator: AspectFieldLocator,
    /// Whether the schema declares the field equality-queryable.
    pub(super) equality_queryable: bool,
    /// The installed equality index; `None` until installation registers it.
    pub(super) equality_index_id: Option<DerivedIndexId>,
    pub(super) unique: bool,
}

impl WorthQueryPrimaryGraphLayout {
    pub fn relation(&self, relation: &str) -> Option<&WorthQueryPrimaryRelationLayout> {
        self.relation_kinds.get(relation)
    }

    pub(super) fn field(
        &self,
        entity: &str,
        aspect: &str,
        field: &str,
    ) -> Option<&WorthQueryPrimaryFieldLayout> {
        self.fields
            .get(&(entity.to_owned(), aspect.to_owned(), field.to_owned()))
    }

    pub(in crate::domain_computation) fn application_entity_kind_without_create_scope(
        &self,
        touches: &worth_query_installation::facade::WorthQueryOperationTouchContract,
    ) -> Option<KindId> {
        self.physical_roles.entities
            .iter()
            .find(|(entity, _)| {
                !touches.scopes().iter().any(|scope| matches!(
                    scope,
                    worth_query_installation::facade::WorthQueryOperationTouchScope::CreateEntity(scope)
                        if scope.entity() == *entity
                ))
            })
            .map(|(_, kind)| *kind)
    }

    pub(in crate::domain_computation) fn field_locator(
        &self,
        entity: &str,
        aspect: &str,
        field: &str,
    ) -> Option<&AspectFieldLocator> {
        self.fields
            .get(&(entity.to_string(), aspect.to_string(), field.to_string()))
            .map(|layout| &layout.locator)
    }

    pub(super) const fn provider_idempotency(&self) -> &WorthQueryProviderIdempotencyLayout {
        &self.provider_idempotency
    }

    pub(super) fn provider_idempotency_mut(&mut self) -> &mut WorthQueryProviderIdempotencyLayout {
        &mut self.provider_idempotency
    }

    pub(super) const fn provider_dispatch_outbox(&self) -> &WorthQueryDispatchOutboxLayout {
        &self.provider_dispatch_outbox
    }

    pub const fn provider_inbound_completion(&self) -> &WorthQueryInboundCompletionLayout {
        &self.provider_inbound_completion
    }

    pub(super) const fn provider_aftermath_causality(&self) -> &WorthQueryAftermathCausalityLayout {
        &self.provider_aftermath_causality
    }

    pub(super) fn provider_aftermath_causality_mut(
        &mut self,
    ) -> &mut WorthQueryAftermathCausalityLayout {
        &mut self.provider_aftermath_causality
    }

    pub const fn program_activation(&self) -> &WorthQueryProgramActivationLayout {
        &self.program_activation
    }

    pub const fn workflow(&self) -> &WorthQueryWorkflowLayout {
        &self.workflow
    }

    pub(super) fn workflow_mut(&mut self) -> &mut WorthQueryWorkflowLayout {
        &mut self.workflow
    }
}

mod court_doors_for_test;
