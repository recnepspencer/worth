use std::sync::Arc;

use worth_foundational::facade::{AspectFieldLocator, AspectValue};

use crate::identity::data::{EntityId, KindId};
use crate::indexes::data::{
    BoundedEntityFieldLookupAdmissionStop as Stop, BoundedEntityFieldLookupDenial,
    BoundedEntityFieldLookupDenialKind as Denial, BoundedEntityFieldLookupOutcome,
    BoundedIndexParityMode, DerivedIndexDefinition, DerivedIndexEntries, DerivedIndexGeneration,
    DerivedIndexId, DerivedIndexKind,
};
use crate::runtime::VisibilityProjectionView;
use crate::storage::data::AuthoritativeFieldComparisonKey;
use crate::visibility::snapshot_states::VisibilitySnapshotBasis;

use super::entity_field_collection::{collect, EntityFieldCollectionRead};
use super::IndexAccess;

/// One installed field index pinned to an owner-issued exact root. It retains
/// the admitted observation, not a current-head claim or a registry lock.
pub struct PreparedEntityFieldLookup {
    runtime_identity: u64,
    basis: VisibilitySnapshotBasis,
    definition: Arc<DerivedIndexDefinition>,
    generation: Arc<DerivedIndexGeneration>,
    kind: KindId,
}

impl PreparedEntityFieldLookup {
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

impl IndexAccess<'_> {
    pub fn prepare_entity_field_lookup(
        &self,
        view: &VisibilityProjectionView<'_>,
        index: DerivedIndexId,
        kind: KindId,
        locator: &AspectFieldLocator,
    ) -> Result<PreparedEntityFieldLookup, BoundedEntityFieldLookupDenial> {
        if !view.is_exact_basis() || !view.is_from_runtime(self.runtime) {
            return Err(denied(Denial::SnapshotUnavailable, index));
        }
        let definition = self
            .runtime
            .indexes
            .definition(index)
            .ok_or_else(|| denied(Denial::IndexNotInstalled, index))?;
        if !matches!(&definition.kind,
            DerivedIndexKind::EntityField { field_locator } if field_locator == locator)
        {
            return Err(denied(Denial::WrongIndexKind, index));
        }
        let schema = view
            .selected_schema_authority()
            .expect("exact projection carries its root schema")
            .schema_version();
        let generation = self
            .runtime
            .indexes
            .exact_generation(
                index,
                definition
                    .branch_scoped
                    .then_some(view.selected_branch_id()),
                view.version_id(),
                schema,
            )
            .ok_or_else(|| denied(Denial::ExactGenerationUnavailable, index))?;
        if !matches!(generation.entries, DerivedIndexEntries::EntityField(_)) {
            return Err(denied(Denial::WrongIndexKind, index));
        }
        Ok(PreparedEntityFieldLookup {
            runtime_identity: self.runtime.runtime_instance_id(),
            basis: view
                .retain_exact_basis()
                .expect("exact basis checked above"),
            definition,
            generation,
            kind,
        })
    }

    /// Complete finite selection. The callback controls interruption; it does
    /// not claim admission of temporary key or result Vec backing.
    pub fn execute_prepared_entity_field_lookup<E>(
        &self,
        prepared: &PreparedEntityFieldLookup,
        value: &AspectValue,
        limit: usize,
        parity: BoundedIndexParityMode,
        mut check_live: impl FnMut() -> Result<(), E>,
    ) -> Result<BoundedEntityFieldLookupOutcome, Stop<E>> {
        if prepared.runtime_identity != self.runtime.runtime_instance_id() {
            return Err(Stop::ExactBasisRequired);
        }
        check_live().map_err(Stop::Admission)?;
        if limit == 0 || limit == usize::MAX {
            return Err(Stop::Lookup(denied(
                Denial::InvalidCandidateLimit,
                prepared.index_id(),
            )));
        }
        self.runtime
            .performance_access()
            .count_query_index_attempt();
        self.collect_prepared_entity_field_lookup(prepared, value, limit, parity, check_live)
    }

    pub(super) fn collect_prepared_entity_field_lookup<E>(
        &self,
        prepared: &PreparedEntityFieldLookup,
        value: &AspectValue,
        limit: usize,
        parity: BoundedIndexParityMode,
        mut check_live: impl FnMut() -> Result<(), E>,
    ) -> Result<BoundedEntityFieldLookupOutcome, Stop<E>> {
        let key = AuthoritativeFieldComparisonKey::from_aspect_value(value);
        check_live().map_err(Stop::Admission)?;
        let DerivedIndexEntries::EntityField(entries) = &prepared.generation.entries else {
            unreachable!("prepared generation is an entity field index")
        };
        let mut read = PreparedCollectionRead {
            prepared,
            value,
            check_live: &mut check_live,
        };
        let (candidates, examined, overflowed) = collect(
            entries.get(&key),
            prepared.index_id(),
            prepared.kind,
            limit,
            &mut read,
        )?;
        let outcome = BoundedEntityFieldLookupOutcome::new(
            Arc::clone(&prepared.definition),
            prepared.generation.generation_id,
            candidates,
            examined,
            overflowed,
            parity,
        );
        if parity == BoundedIndexParityMode::Certification {
            super::prepared_field_certification::certify(
                self.runtime,
                prepared,
                value,
                limit,
                &outcome,
                &mut check_live,
            )?;
            self.runtime
                .performance_access()
                .count_query_index_parity_verification();
        }
        check_live().map_err(Stop::Admission)?;
        self.runtime.performance_access().count_query_index_path();
        Ok(outcome)
    }
}

struct PreparedCollectionRead<'a, F> {
    prepared: &'a PreparedEntityFieldLookup,
    value: &'a AspectValue,
    check_live: &'a mut F,
}
impl<E, F: FnMut() -> Result<(), E>> EntityFieldCollectionRead<E>
    for PreparedCollectionRead<'_, F>
{
    fn prepare_results(&mut self, _: usize) -> Result<(), Stop<E>> {
        (self.check_live)().map_err(Stop::Admission)
    }
    fn compare(&mut self, entity: EntityId, _: usize) -> Result<Option<(KindId, bool)>, Stop<E>> {
        (self.check_live)().map_err(Stop::Admission)?;
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

impl PreparedEntityFieldLookup {
    pub(super) fn basis(&self) -> &VisibilitySnapshotBasis {
        &self.basis
    }
}
