use std::marker::PhantomData;
use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

use worth_relational::facade::identity::{EntityId, KindId};
use worth_relational::facade::indexes::{DerivedIndexGenerationId, DerivedIndexId};
use worth_relational::facade::transactions::RecordRef;

use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;

use super::super::WorthQueryPrincipalResolutionMode;

pub(super) struct WorthQueryEntityResolutionSubject<'subject> {
    entity: &'subject str,
    expected: AspectValue,
    mode: WorthQueryPrincipalResolutionMode,
}

impl<'subject> WorthQueryEntityResolutionSubject<'subject> {
    pub(super) const fn new(
        entity: &'subject str,
        expected: AspectValue,
        mode: WorthQueryPrincipalResolutionMode,
    ) -> Self {
        Self {
            entity,
            expected,
            mode,
        }
    }
}

/// Opaque, typed identity of one application entity resolved through an exact
/// equality-index generation.
///
/// This is descriptive scope identity only. Its fields and resolution result
/// are owner-sealed; callers cannot rebuild it from record/index axes.
///
/// ```compile_fail
/// use std::marker::PhantomData;
/// use worth_query_execution::facade::primary_graph::WorthQueryApplicationEntityIdentity;
///
/// fn forge<Schema, Entity>() -> WorthQueryApplicationEntityIdentity<Schema, Entity> {
///     WorthQueryApplicationEntityIdentity {
///         entity_id: todo!(),
///         entity_kind: todo!(),
///         entity_name: String::new(),
///         binding_identity: todo!(),
///         runtime_authority: todo!(),
///         identity_index_id: todo!(),
///         identity_index_generation: todo!(),
///         identity_locator: todo!(),
///         identity_value: todo!(),
///         examined_candidate_count: 0,
///         resolution_mode: todo!(),
///         _marker: PhantomData,
///     }
/// }
/// ```
pub struct WorthQueryApplicationEntityIdentity<Schema, Entity> {
    entity_id: EntityId,
    entity_kind: KindId,
    entity_name: String,
    binding_identity: ApplicationSchemaBindingIdentity,
    runtime_authority: WorthQueryRuntimeAuthorityIdentity,
    identity_index_id: DerivedIndexId,
    identity_index_generation: DerivedIndexGenerationId,
    identity_locator: AspectFieldLocator,
    identity_value: AspectValue,
    examined_candidate_count: usize,
    resolution_mode: WorthQueryPrincipalResolutionMode,
    _marker: PhantomData<fn() -> (Schema, Entity)>,
}

#[derive(Clone)]
pub struct WorthQueryResolvedEntity {
    entity_id: EntityId,
    entity_kind: KindId,
    entity_name: String,
    binding_identity: ApplicationSchemaBindingIdentity,
    runtime_authority: WorthQueryRuntimeAuthorityIdentity,
    identity_index_id: DerivedIndexId,
    identity_index_generation: DerivedIndexGenerationId,
    identity_locator: AspectFieldLocator,
    identity_value: AspectValue,
    examined_candidate_count: usize,
    resolution_mode: WorthQueryPrincipalResolutionMode,
}

impl WorthQueryResolvedEntity {
    pub(super) fn matches_source<Q>(
        &self,
        source: &super::super::WorthQueryObservedSource<Q>,
    ) -> bool {
        self.entity_id == source.model_root
            && self.runtime_authority.as_u64() == source.runtime_authority
            && self.binding_identity == source.schema_binding
    }
    pub(super) fn retained_capacity_bytes(&self) -> Option<usize> {
        self.entity_name
            .capacity()
            .checked_add(self.identity_locator.owned_allocation_capacity_bytes())?
            .checked_add(self.identity_value.owned_allocation_capacity_bytes())
    }
    pub(super) fn locator(&self) -> &AspectFieldLocator {
        &self.identity_locator
    }
    pub(super) fn value(&self) -> &AspectValue {
        &self.identity_value
    }

    pub(super) fn from_lookup(
        installed: &super::WorthQueryInstalledEntityResolutionContext,
        layout: &super::super::schema_layout::WorthQueryPrimaryFieldLayout,
        subject: WorthQueryEntityResolutionSubject<'_>,
        lookup: &worth_relational::facade::indexes::BoundedEntityFieldLookupOutcome,
    ) -> Self {
        Self {
            entity_id: lookup.candidate_entity_ids()[0],
            entity_kind: layout.entity_kind,
            entity_name: subject.entity.to_owned(),
            binding_identity: installed.binding_identity.clone(),
            runtime_authority: installed.runtime_authority,
            identity_index_id: layout
                .equality_index_id
                .expect("resolution accepts only an installed equality index"),
            identity_index_generation: lookup.generation_id(),
            identity_locator: layout.locator.clone(),
            identity_value: subject.expected,
            examined_candidate_count: lookup.examined_entry_count(),
            resolution_mode: subject.mode,
        }
    }

    pub(in crate::domain_computation) const fn entity_id(&self) -> EntityId {
        self.entity_id
    }

    pub(in crate::domain_computation) const fn entity_kind(&self) -> KindId {
        self.entity_kind
    }

    pub(in crate::domain_computation) fn into_application_identity<Schema, Entity>(
        self,
    ) -> WorthQueryApplicationEntityIdentity<Schema, Entity> {
        WorthQueryApplicationEntityIdentity {
            entity_id: self.entity_id,
            entity_kind: self.entity_kind,
            entity_name: self.entity_name,
            binding_identity: self.binding_identity,
            runtime_authority: self.runtime_authority,
            identity_index_id: self.identity_index_id,
            identity_index_generation: self.identity_index_generation,
            identity_locator: self.identity_locator,
            identity_value: self.identity_value,
            examined_candidate_count: self.examined_candidate_count,
            resolution_mode: self.resolution_mode,
            _marker: PhantomData,
        }
    }
}

impl<Schema, Entity> WorthQueryApplicationEntityIdentity<Schema, Entity> {
    /// Retain the actual resolver's description, including its full generation.
    /// This copies no permission and makes no claim about a later basis.
    pub(super) fn retain_description(&self) -> WorthQueryResolvedEntity {
        WorthQueryResolvedEntity {
            entity_id: self.entity_id,
            entity_kind: self.entity_kind,
            entity_name: self.entity_name.clone(),
            binding_identity: self.binding_identity.clone(),
            runtime_authority: self.runtime_authority,
            identity_index_id: self.identity_index_id,
            identity_index_generation: self.identity_index_generation,
            identity_locator: self.identity_locator.clone(),
            identity_value: self.identity_value.clone(),
            examined_candidate_count: self.examined_candidate_count,
            resolution_mode: self.resolution_mode,
        }
    }

    pub fn binding_identity(&self) -> &ApplicationSchemaBindingIdentity {
        &self.binding_identity
    }

    pub const fn examined_candidate_count(&self) -> usize {
        self.examined_candidate_count
    }

    pub fn matches_record(&self, record: &RecordRef) -> bool {
        matches!(record, RecordRef::Entity(entity) if *entity == self.entity_id)
    }

    /// Describes the exact relational record selected by this resolved
    /// application identity for an audience integration adapter.
    ///
    /// The returned coordinates carry no resolution, read, mutation, or
    /// invalidation authority; the primary graph remains the authority owner.
    #[doc(hidden)]
    pub fn relational_record_identity_parts(
        &self,
    ) -> worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts {
        worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts::entity(
            self.entity_id.partition_value(),
            self.entity_id.local_slot_value(),
            self.entity_id.generation_value(),
        )
    }

    pub(crate) const fn entity_id(&self) -> EntityId {
        self.entity_id
    }

    pub(crate) const fn entity_kind(&self) -> KindId {
        self.entity_kind
    }

    pub(crate) fn entity_name(&self) -> &str {
        &self.entity_name
    }

    pub(crate) const fn runtime_authority(&self) -> WorthQueryRuntimeAuthorityIdentity {
        self.runtime_authority
    }

    pub(crate) const fn identity_index_id(&self) -> DerivedIndexId {
        self.identity_index_id
    }

    pub const fn identity_index_generation(&self) -> DerivedIndexGenerationId {
        self.identity_index_generation
    }

    pub(crate) fn identity_locator(&self) -> &AspectFieldLocator {
        &self.identity_locator
    }

    pub(crate) fn identity_value(&self) -> &AspectValue {
        &self.identity_value
    }

    pub(crate) const fn resolution_mode(&self) -> WorthQueryPrincipalResolutionMode {
        self.resolution_mode
    }
}

impl<Schema, Entity> std::fmt::Debug for WorthQueryApplicationEntityIdentity<Schema, Entity> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryApplicationEntityIdentity")
            .field("binding_identity", &self.binding_identity)
            .field("identity_index_id", &self.identity_index_id)
            .field("identity_index_generation", &self.identity_index_generation)
            .field("examined_candidate_count", &self.examined_candidate_count)
            .finish_non_exhaustive()
    }
}

mod court_doors_for_test;
