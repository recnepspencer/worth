use crate::branch::indexes::data::bounded_entity_field_lookup::{
    BoundedEntityFieldLookupAdmissionStop as Stop, BoundedEntityFieldLookupDenial,
    BoundedEntityFieldLookupDenialKind as Denial, BoundedEntityFieldLookupDenialOwnerPort as _,
    BoundedEntityFieldLookupOutcome, BoundedEntityFieldLookupOutcomeOwnerPort as _,
    BoundedIndexParityMode,
};

pub mod page;

use worth_foundational::facade::{AspectFieldLocator, AspectValue};

use crate::branch::indexes::data::{DerivedIndexEntries, SharedDerivedIndexGeneration};
use crate::branch::visibility::snapshot_states::resident_basis::ResidentVisibilitySnapshotBasis;
use crate::contracts::identity::data::primitives::KindId;
use crate::contracts::identity::data::record_ids::EntityId;
use crate::contracts::indexes::data::definition::{
    DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind,
};
use crate::contracts::storage::data::authoritative_field_comparison_key::AuthoritativeFieldComparisonKey;

use super::entity_field_collection::{collect, EntityFieldCollectionRead};

/// One installed field index pinned to an owner-issued exact root. It retains
/// the admitted observation, not a current-head claim or a registry lock.
pub struct PreparedEntityFieldLookup {
    runtime_identity: u64,
    instrumentation:
        crate::branch::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation,
    basis: ResidentVisibilitySnapshotBasis,
    definition: crate::branch::indexes::retained_definition::RetainedDerivedIndexDefinition,
    generation: SharedDerivedIndexGeneration,
    kind: KindId,
}

impl PreparedEntityFieldLookup {
    pub(super) fn applied_generation(&self) -> &SharedDerivedIndexGeneration {
        &self.generation
    }
    pub(super) fn source_root(&self) -> (u64, u64) {
        (self.runtime_identity, self.basis.root().id())
    }

    pub(super) fn belongs_to_owner(&self, runtime_identity: u64) -> bool {
        self.runtime_identity == runtime_identity
    }
    pub fn retain_definition(
        &self,
    ) -> crate::branch::indexes::retained_definition::RetainedDerivedIndexDefinition {
        self.definition.clone()
    }
    pub fn index_id(&self) -> DerivedIndexId {
        self.definition.index_id
    }
    pub fn entity_kind(&self) -> KindId {
        self.kind
    }
    pub fn field_locator(&self) -> &AspectFieldLocator {
        let DerivedIndexKind::EntityField { field_locator } = &self.definition.kind else {
            unreachable!(
                /* Proof: Prepared index admission checks the definition and generation discriminants together; the private prepared handle retains that same generation. */
                "prepared field definition was checked at admission"
            )
        };
        field_locator
    }
}

impl std::fmt::Debug for PreparedEntityFieldLookup {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PreparedEntityFieldLookup")
            .field("runtime_identity", &self.runtime_identity)
            .field("root_identity", &self.basis.root().id())
            .field("index_id", &self.index_id())
            .field("generation_id", &self.generation.generation_id)
            .finish()
    }
}

impl PreparedEntityFieldLookup {
    /// Tests the whole pinned field population without copying keys or rows.
    pub fn population_is_empty(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<bool, BoundedEntityFieldLookupDenial> {
        execution
            .consult()
            .map_err(|cause| denied(Denial::Execution(cause.into()), self.index_id()))?;
        let DerivedIndexEntries::EntityField(entries) = &self.generation.entries else {
            unreachable!(
                /* Proof: Prepared index admission checks the definition and generation discriminants together; the private prepared handle retains that same generation. */
                "prepared generation is an entity field index"
            )
        };
        Ok(entries.is_empty())
    }

    /// Complete finite selection. The callback controls interruption; it does
    /// not claim admission of temporary key or result Vec backing.
    pub fn execute_prepared_entity_field_lookup<E: worth_execution::ChargedBytes>(
        &self,
        value: &AspectValue,
        limit: usize,
        parity: BoundedIndexParityMode,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        mut admission: impl FnMut() -> Result<(), E>,
    ) -> Result<BoundedEntityFieldLookupOutcome, Stop<E>> {
        super::prepared_lookup_admission::admit(execution, self.index_id(), &mut admission)?;
        if limit == 0 || limit == usize::MAX {
            return Err(Stop::Lookup(denied(
                Denial::InvalidCandidateLimit,
                self.index_id(),
            )));
        }
        self.instrumentation.count_query_index_attempt();
        self.collect_prepared_entity_field_lookup(value, limit, parity, execution, admission)
    }

    pub(super) fn collect_prepared_entity_field_lookup<E: worth_execution::ChargedBytes>(
        &self,
        value: &AspectValue,
        limit: usize,
        parity: BoundedIndexParityMode,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        mut admission: impl FnMut() -> Result<(), E>,
    ) -> Result<BoundedEntityFieldLookupOutcome, Stop<E>> {
        let key = AuthoritativeFieldComparisonKey::from_aspect_value(value);
        super::prepared_lookup_admission::admit(execution, self.index_id(), &mut admission)?;
        let DerivedIndexEntries::EntityField(entries) = &self.generation.entries else {
            unreachable!(
                /* Proof: Prepared index admission checks the definition and generation discriminants together; the private prepared handle retains that same generation. */
                "prepared generation is an entity field index"
            )
        };
        let mut read = PreparedCollectionRead {
            prepared: self,
            value,
            admission: &mut admission,
            execution,
        };
        let (candidates, examined, overflowed) = collect(
            execution,
            entries.get(&key),
            self.index_id(),
            self.kind,
            limit,
            &mut read,
        )?;
        let outcome = BoundedEntityFieldLookupOutcome::new(
            self.definition.clone(),
            self.generation.generation_id,
            candidates,
            examined,
            overflowed,
            parity,
        );
        if parity == BoundedIndexParityMode::Certification {
            super::prepared_field_certification::certify(
                execution,
                self,
                value,
                limit,
                &outcome,
                &mut admission,
            )?;
            self.instrumentation.count_query_index_parity_verification();
        }
        super::prepared_lookup_admission::admit(execution, self.index_id(), &mut admission)?;
        self.instrumentation.count_query_index_path();
        Ok(outcome)
    }
}

struct PreparedCollectionRead<'a, 'request, 'authority, F> {
    execution: &'a worth_execution::ExecutionRequest<'request, 'authority>,
    prepared: &'a PreparedEntityFieldLookup,
    value: &'a AspectValue,
    admission: &'a mut F,
}
impl<E, F: FnMut() -> Result<(), E>> EntityFieldCollectionRead<E>
    for PreparedCollectionRead<'_, '_, '_, F>
{
    fn prepare_results(&mut self, _: usize) -> Result<(), Stop<E>> {
        super::prepared_lookup_admission::admit(
            self.execution,
            self.prepared.index_id(),
            self.admission,
        )
    }
    fn compare(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity: EntityId,
        _: usize,
    ) -> Result<Option<(KindId, bool)>, Stop<E>> {
        super::prepared_lookup_admission::admit(
            execution,
            self.prepared.index_id(),
            self.admission,
        )?;
        crate::indexes::projected_field_values::exact_entity_field_matches(
            &self.prepared.basis,
            execution,
            entity,
            self.prepared.field_locator(),
            self.value,
        )
        .map_err(|cause| Stop::Lookup(denied(Denial::Execution(cause), self.prepared.index_id())))
    }
}

pub fn denied(kind: Denial, index: DerivedIndexId) -> BoundedEntityFieldLookupDenial {
    BoundedEntityFieldLookupDenial::new(kind, index)
}

pub mod record_source;

impl PreparedEntityFieldLookup {
    pub(crate) fn validate_definition(
        definition: &DerivedIndexDefinition,
        locator: &AspectFieldLocator,
    ) -> Result<(), BoundedEntityFieldLookupDenial> {
        if !matches!(&definition.kind, DerivedIndexKind::EntityField { field_locator } if field_locator == locator)
        {
            return Err(denied(Denial::WrongIndexKind, definition.index_id));
        }
        Ok(())
    }
    pub(crate) fn pin<G: Into<SharedDerivedIndexGeneration>>(
        runtime_identity: u64,
        basis: ResidentVisibilitySnapshotBasis,
        definition: crate::branch::indexes::retained_definition::RetainedDerivedIndexDefinition,
        kind: KindId,
        locator: &AspectFieldLocator,
        instrumentation: crate::branch::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation,
        generation: impl FnOnce(&DerivedIndexDefinition) -> Option<G>,
    ) -> Result<Self, BoundedEntityFieldLookupDenial> {
        let index = definition.index_id;
        Self::validate_definition(&definition, locator)?;
        let generation: SharedDerivedIndexGeneration = generation(&definition)
            .ok_or_else(|| denied(Denial::ExactGenerationUnavailable, index))?
            .into();
        if !matches!(generation.entries, DerivedIndexEntries::EntityField(_)) {
            return Err(denied(Denial::WrongIndexKind, index));
        }
        Ok(Self {
            runtime_identity,
            instrumentation,
            basis,
            definition,
            generation,
            kind,
        })
    }
}

#[doc(hidden)]
pub trait PreparedEntityFieldLookupOwnerPort: Sized {
    fn belongs_to_owner(&self, runtime_identity: u64) -> bool;

    fn validate_definition(
        definition: &DerivedIndexDefinition,
        locator: &AspectFieldLocator,
    ) -> Result<(), BoundedEntityFieldLookupDenial>;
}

impl PreparedEntityFieldLookupOwnerPort for PreparedEntityFieldLookup {
    fn belongs_to_owner(&self, runtime_identity: u64) -> bool {
        self.belongs_to_owner(runtime_identity)
    }

    fn validate_definition(
        definition: &DerivedIndexDefinition,
        locator: &AspectFieldLocator,
    ) -> Result<(), BoundedEntityFieldLookupDenial> {
        Self::validate_definition(definition, locator)
    }
}
