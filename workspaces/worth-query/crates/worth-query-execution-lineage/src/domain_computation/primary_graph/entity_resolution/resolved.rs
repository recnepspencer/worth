use std::marker::PhantomData;
use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

use worth_relational::facade::identity::{EntityId, KindId};
use worth_relational::facade::indexes::{DerivedIndexGenerationId, DerivedIndexId};
use worth_relational::facade::transactions::RecordRef;

use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;

use super::super::WorthQueryPrincipalResolutionMode;

pub struct WorthQueryEntityResolutionSubject<'subject> {
    entity: &'subject str,
    expected: AspectValue,
    mode: WorthQueryPrincipalResolutionMode,
}

impl<'subject> WorthQueryEntityResolutionSubject<'subject> {
    pub const fn new(
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

#[derive(Clone, Copy, Debug)]
enum EntityResolutionOrigin {
    Committed(DerivedIndexGenerationId),
    Applied(worth_relational::facade::runtime::RelationalAppliedCandidateReadQualification),
}

/// Opaque descriptive identity resolved at an exact committed index generation
/// or an exact applied candidate. Neither origin grants current scope authority.
///
/// This is descriptive scope identity only. Its fields and resolution result
/// are owner-sealed; callers cannot rebuild it from record/index axes.
pub struct WorthQueryApplicationEntityIdentity<Schema, Entity> {
    entity_id: EntityId,
    entity_kind: KindId,
    entity_name: String,
    binding_identity: ApplicationSchemaBindingIdentity,
    runtime_authority: WorthQueryRuntimeAuthorityIdentity,
    identity_index_id: DerivedIndexId,
    origin: EntityResolutionOrigin,
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
    origin: EntityResolutionOrigin,
    identity_locator: AspectFieldLocator,
    identity_value: AspectValue,
    examined_candidate_count: usize,
    resolution_mode: WorthQueryPrincipalResolutionMode,
}

impl WorthQueryResolvedEntity {
    #[allow(clippy::too_many_arguments)]
    pub fn from_applied_parts(
        installed: &super::WorthQueryInstalledEntityResolutionContext,
        layout: &super::super::schema_layout::WorthQueryPrimaryFieldLayout,
        entity: &str,
        expected: AspectValue,
        entity_id: EntityId,
        entity_kind: worth_relational::facade::identity::KindId,
        candidate: worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        examined: usize,
    ) -> Self {
        Self {
            entity_id,
            entity_kind,
            entity_name: entity.to_owned(),
            binding_identity: installed.binding_identity.clone(),
            runtime_authority: installed.runtime_authority,
            identity_index_id: layout
                .equality_index_id
                .expect("applied equality resolution"),
            origin: EntityResolutionOrigin::Applied(candidate.qualification()),
            identity_locator: layout.locator.clone(),
            identity_value: expected,
            examined_candidate_count: examined,
            resolution_mode: WorthQueryPrincipalResolutionMode::Ordinary,
        }
    }

    pub fn matches_source_identity(
        &self,
        model_root: EntityId,
        runtime_authority: u64,
        schema_binding: &worth_query_installation::facade::ApplicationSchemaBindingIdentity,
    ) -> bool {
        self.entity_id == model_root
            && self.runtime_authority.as_u64() == runtime_authority
            && &self.binding_identity == schema_binding
    }
    pub fn retained_capacity_bytes(&self) -> Option<usize> {
        self.entity_name
            .capacity()
            .checked_add(self.identity_locator.owned_allocation_capacity_bytes())?
            .checked_add(self.identity_value.owned_allocation_capacity_bytes())
    }
    pub fn locator(&self) -> &AspectFieldLocator {
        &self.identity_locator
    }
    pub fn value(&self) -> &AspectValue {
        &self.identity_value
    }

    pub fn from_lookup(
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
            origin: EntityResolutionOrigin::Committed(lookup.generation_id()),
            identity_locator: layout.locator.clone(),
            identity_value: subject.expected,
            examined_candidate_count: lookup.examined_entry_count(),
            resolution_mode: subject.mode,
        }
    }

    pub const fn entity_id(&self) -> EntityId {
        self.entity_id
    }

    pub const fn entity_kind(&self) -> KindId {
        self.entity_kind
    }

    pub fn into_application_identity<Schema, Entity>(
        self,
    ) -> WorthQueryApplicationEntityIdentity<Schema, Entity> {
        WorthQueryApplicationEntityIdentity {
            entity_id: self.entity_id,
            entity_kind: self.entity_kind,
            entity_name: self.entity_name,
            binding_identity: self.binding_identity,
            runtime_authority: self.runtime_authority,
            identity_index_id: self.identity_index_id,
            origin: self.origin,
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
    pub fn retain_description(&self) -> WorthQueryResolvedEntity {
        WorthQueryResolvedEntity {
            entity_id: self.entity_id,
            entity_kind: self.entity_kind,
            entity_name: self.entity_name.clone(),
            binding_identity: self.binding_identity.clone(),
            runtime_authority: self.runtime_authority,
            identity_index_id: self.identity_index_id,
            origin: self.origin,
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
        worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts::from_entity(
            self.entity_id,
        )
    }

    pub const fn entity_id(&self) -> EntityId {
        self.entity_id
    }

    pub const fn entity_kind(&self) -> KindId {
        self.entity_kind
    }

    pub fn entity_name(&self) -> &str {
        &self.entity_name
    }

    pub const fn runtime_authority(&self) -> WorthQueryRuntimeAuthorityIdentity {
        self.runtime_authority
    }

    pub const fn identity_index_id(&self) -> DerivedIndexId {
        self.identity_index_id
    }

    /// Applied resolution has no committed index generation.
    pub const fn identity_index_generation(&self) -> Option<DerivedIndexGenerationId> {
        match self.origin {
            EntityResolutionOrigin::Committed(generation) => Some(generation),
            EntityResolutionOrigin::Applied(_) => None,
        }
    }

    pub const fn applied_qualification(
        &self,
    ) -> Option<worth_relational::facade::runtime::RelationalAppliedCandidateReadQualification>
    {
        match self.origin {
            EntityResolutionOrigin::Applied(qualification) => Some(qualification),
            EntityResolutionOrigin::Committed(_) => None,
        }
    }

    pub fn identity_locator(&self) -> &AspectFieldLocator {
        &self.identity_locator
    }

    pub fn identity_value(&self) -> &AspectValue {
        &self.identity_value
    }

    pub const fn resolution_mode(&self) -> WorthQueryPrincipalResolutionMode {
        self.resolution_mode
    }
}

impl<Schema, Entity> std::fmt::Debug for WorthQueryApplicationEntityIdentity<Schema, Entity> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut identity = formatter.debug_struct("WorthQueryApplicationEntityIdentity");
        identity
            .field("binding_identity", &self.binding_identity)
            .field("identity_index_id", &self.identity_index_id);
        match &self.origin {
            EntityResolutionOrigin::Committed(generation) => {
                identity.field("identity_index_generation", generation);
            }
            EntityResolutionOrigin::Applied(candidate) => {
                identity.field("applied_candidate", candidate);
            }
        }
        identity
            .field("examined_candidate_count", &self.examined_candidate_count)
            .finish_non_exhaustive()
    }
}

mod court_doors_for_test;
