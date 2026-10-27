use std::sync::Arc;

mod page;

use worth_foundational::facade::{AspectFieldLocator, AspectValue};

use crate::identity::data::{EntityId, KindId};
use crate::indexes::data::{
    BoundedEntityFieldLookupAdmissionStop as Stop, BoundedEntityFieldLookupDenial,
    BoundedEntityFieldLookupDenialKind as Denial, BoundedEntityFieldLookupOutcome,
    BoundedIndexParityMode, DerivedIndexDefinition, DerivedIndexEntries, DerivedIndexGeneration,
    DerivedIndexId, DerivedIndexKind,
};
use crate::storage::data::AuthoritativeFieldComparisonKey;
use crate::visibility::snapshot_states::VisibilitySnapshotBasis;

use super::entity_field_collection::{collect, EntityFieldCollectionRead};

/// One installed field index pinned to an owner-issued exact root. It retains
/// the admitted observation, not a current-head claim or a registry lock.
pub struct PreparedEntityFieldLookup {
    runtime_identity: u64,
    instrumentation: crate::runtime::RuntimeInstrumentation,
    basis: VisibilitySnapshotBasis,
    definition: Arc<DerivedIndexDefinition>,
    generation: Arc<DerivedIndexGeneration>,
    kind: KindId,
}

impl PreparedEntityFieldLookup {
    pub(super) fn belongs_to_owner(&self, runtime_identity: u64) -> bool {
        self.runtime_identity == runtime_identity
    }
    pub fn index_id(&self) -> DerivedIndexId {
        self.definition.index_id
    }
    pub fn entity_kind(&self) -> KindId {
        self.kind
    }
    pub fn field_locator(&self) -> &AspectFieldLocator {
        let DerivedIndexKind::EntityField { field_locator } = &self.definition.kind else {
            unreachable!("prepared field definition was checked at admission")
        };
        field_locator
    }
}

impl std::fmt::Debug for PreparedEntityFieldLookup {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PreparedEntityFieldLookup")
            .field("runtime_identity", &self.runtime_identity)
            .field("root_identity", &self.basis.root_id())
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
            unreachable!("prepared generation is an entity field index")
        };
        Ok(entries.is_empty())
    }

    fn performance_access(&self) -> crate::performance::PerformanceAccess<'_> {
        crate::performance::PerformanceAccess::from_instrumentation(&self.instrumentation)
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
        self.performance_access().count_query_index_attempt();
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
            unreachable!("prepared generation is an entity field index")
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
            Arc::clone(&self.definition),
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
            self.performance_access()
                .count_query_index_parity_verification();
        }
        super::prepared_lookup_admission::admit(execution, self.index_id(), &mut admission)?;
        self.performance_access().count_query_index_path();
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
        Ok(
            crate::indexes::projected_field_values::exact_entity_field_matches(
                self.prepared.basis.root(),
                entity,
                self.prepared.field_locator(),
                self.value,
            ),
        )
    }
}

pub(super) fn denied(kind: Denial, index: DerivedIndexId) -> BoundedEntityFieldLookupDenial {
    BoundedEntityFieldLookupDenial::new(kind, index)
}

impl crate::visibility::materialization::read_records::ProjectionRecordSource
    for PreparedEntityFieldLookup
{
    // Construction already admitted and retained the complete immutable source.
    // Reading an accepted pin checks request life, without reacquiring cold truth.
    fn demand_all_full(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), crate::execution::RelationalExecutionDenialCause> {
        crate::execution::check_request(execution)
    }
    fn demand_regions_full(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        _partitions: impl IntoIterator<Item = crate::identity::data::PartitionId>,
    ) -> Result<(), crate::execution::RelationalExecutionDenialCause> {
        crate::execution::check_request(execution)
    }
    fn selected_root(&self) -> Option<&Arc<crate::branch::RelationalBranchRoot>> {
        Some(self.basis.root())
    }
    fn is_exact_basis(&self) -> bool {
        true
    }
    fn version_id(&self) -> crate::identity::data::VersionId {
        self.basis.version_id()
    }
}

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
    pub(crate) fn pin(
        runtime_identity: u64,
        basis: VisibilitySnapshotBasis,
        definition: Arc<DerivedIndexDefinition>,
        kind: KindId,
        locator: &AspectFieldLocator,
        instrumentation: crate::runtime::RuntimeInstrumentation,
        generation: impl FnOnce(&DerivedIndexDefinition) -> Option<Arc<DerivedIndexGeneration>>,
    ) -> Result<Self, BoundedEntityFieldLookupDenial> {
        let index = definition.index_id;
        Self::validate_definition(&definition, locator)?;
        let generation = generation(&definition)
            .ok_or_else(|| denied(Denial::ExactGenerationUnavailable, index))?;
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
