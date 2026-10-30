//! Immutable material from actual Native Source loans, never a retained read grant.
use super::super::source_rows::{SourceImageRows, SourceImageView};
use crate::domain_computation::primary_graph::{
    application_query::{
        derived_view::{DependencyIndex, ViewDependency},
        disclosure::WorthQueryApplicationQueryGovernance,
        projection::{
            WorthQueryApplicationDisclosedProjectionNode as Node,
            WorthQueryApplicationDisclosedProjectionTree as Tree,
            WorthQueryApplicationProjectedField as Field,
            WorthQueryApplicationProjectionNode as Row,
            WorthQueryApplicationWorkingProjectionTree as WorkingTree,
        },
    },
    source_facts::SourceFactsAllocation,
};
use crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity;
use std::{collections::BTreeSet, sync::Arc};
use worth_execution::ExecutionRequest;
use worth_query_installation::facade::{
    WorthQueryInstalledApplicationValueBindingCatalog, WorthQueryInstalledGraphProjection,
};
use worth_relational::facade::{
    identity::EntityId,
    runtime::{
        DerivedRelationReadDenial, ManagedIndexSourceOccurrence,
        ManagedIndexSourcePreparationDenial, ManagedIndexSourceProgram, ManagedIndexSourceRows,
        ManagedIndexSourceScope,
    },
    schema::DerivedRelationDenial,
    transactions::RelationalExecutionDenialCause,
};

#[derive(Debug)]
pub(in crate::domain_computation::primary_graph::application_query::managed_index) enum MaterialDenial
{
    Native(ManagedIndexSourcePreparationDenial),
    Storage(RelationalExecutionDenialCause),
}
impl MaterialDenial {
    pub(in crate::domain_computation::primary_graph::application_query::managed_index) fn into_native(
        self,
    ) -> ManagedIndexSourcePreparationDenial {
        match self {
            Self::Native(cause) => cause,
            Self::Storage(cause) => DerivedRelationReadDenial::Request(cause).into(),
        }
    }
}
impl From<ManagedIndexSourcePreparationDenial> for MaterialDenial {
    fn from(cause: ManagedIndexSourcePreparationDenial) -> Self {
        Self::Native(cause)
    }
}
impl From<DerivedRelationReadDenial> for MaterialDenial {
    fn from(cause: DerivedRelationReadDenial) -> Self {
        Self::Native(cause.into())
    }
}
impl From<RelationalExecutionDenialCause> for MaterialDenial {
    fn from(cause: RelationalExecutionDenialCause) -> Self {
        Self::Storage(cause)
    }
}

struct Entry {
    occurrence: ManagedIndexSourceOccurrence,
    row: Tree,
    dependencies: BTreeSet<ViewDependency>,
    // Each changed path keeps only its own allocation, never a predecessor image.
    _allocation: Arc<SourceFactsAllocation>,
}

pub(in crate::domain_computation::primary_graph::application_query::managed_index) struct PersistentSourceRows
{
    owner: WorthQueryGraphWorkSessionIdentity,
    program: ManagedIndexSourceProgram,
    scope: EntityId,
    entries: im::OrdMap<EntityId, Arc<Entry>>,
    dependencies: DependencyIndex<EntityId>,
    _allocation: Arc<SourceFactsAllocation>,
}

impl PersistentSourceRows {
    pub(in crate::domain_computation::primary_graph::application_query::managed_index) fn acquire(
        execution: &ExecutionRequest<'_, '_>,
        scope: &ManagedIndexSourceScope<'_>,
        rows: &ManagedIndexSourceRows<'_>,
        owner: WorthQueryGraphWorkSessionIdentity,
        projections: &[WorthQueryInstalledGraphProjection],
        maximum: usize,
    ) -> Result<Self, MaterialDenial> {
        let allocation = SourceFactsAllocation::open(execution)?;
        let support = rows.acquire_all()?;
        let occurrences = rows.occurrences(&support)?;
        if occurrences.len() > maximum {
            return Err(ManagedIndexSourcePreparationDenial::Source(
                DerivedRelationReadDenial::Definition(DerivedRelationDenial::WorkBudgetExceeded),
            )
            .into());
        }
        let mut entries = im::OrdMap::new();
        let mut dependencies = DependencyIndex::default();
        for occurrence in occurrences {
            let entity = occurrence.entity();
            let entry = project(execution, scope.program(), rows, occurrence, projections)?;
            index_entry(execution, &mut dependencies, entity, &entry, &allocation)?;
            allocation.claim_tree::<(EntityId, Arc<Entry>)>(execution, entries.len())?;
            if entries.insert(entity, Arc::new(entry)).is_some() {
                return Err(ManagedIndexSourcePreparationDenial::ForeignSource.into());
            }
        }
        Ok(Self {
            owner,
            program: scope.program().clone(),
            scope: scope.scope(),
            entries,
            dependencies,
            _allocation: allocation,
        })
    }

    /// The actual changed before/after occurrences come from Native's inverse
    /// path. Unchanged rows and subtrees are shared; no full roster is visited.
    pub(in crate::domain_computation::primary_graph::application_query::managed_index) fn update(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        scope: &ManagedIndexSourceScope<'_>,
        changed: &[(
            Option<ManagedIndexSourceOccurrence>,
            Option<ManagedIndexSourceOccurrence>,
        )],
        projections: &[WorthQueryInstalledGraphProjection],
    ) -> Result<Self, MaterialDenial> {
        if self.program != *scope.program() || self.scope != scope.scope() {
            return Err(ManagedIndexSourcePreparationDenial::ForeignSource.into());
        }
        let allocation = SourceFactsAllocation::open(execution)?;
        let after = scope.proposed();
        let mut entries = self.entries.clone();
        let mut dependencies = self.dependencies.clone();
        // Native closes structural pullback; Query's existing field/dependency
        // index closes scalar/lifecycle entry reads. Neither substitutes for the
        // other. A routed retained entry missing from Native's real delta refuses.
        let mut changed_keys = BTreeSet::new();
        for entity in changed
            .iter()
            .flat_map(|(a, b)| a.iter().chain(b))
            .map(|row| row.entity())
        {
            scope
                .read_budget()
                .checkpoint(1)
                .map_err(DerivedRelationReadDenial::Structural)?;
            allocation.claim_tree::<EntityId>(execution, changed_keys.len())?;
            changed_keys.insert(entity);
        }
        for entity in scope.changed_entities() {
            for key in self.dependencies.entries_for_entity(entity) {
                scope
                    .read_budget()
                    .checkpoint(1)
                    .map_err(DerivedRelationReadDenial::Structural)?;
                if !changed_keys.contains(key) {
                    return Err(ManagedIndexSourcePreparationDenial::IncompletePreparation.into());
                }
            }
        }
        for (before, proposed) in changed {
            execution.consult().map_err(|cause| {
                MaterialDenial::Storage(RelationalExecutionDenialCause::from(cause))
            })?;
            if let Some(before) = before {
                // This also convicts a stale/missing retained full-generation
                // occurrence; a later generation never aliases an earlier slot.
                if entries.get(&before.entity()).map(|entry| &entry.occurrence) != Some(before) {
                    return Err(ManagedIndexSourcePreparationDenial::ForeignSource.into());
                }
                let old = entries
                    .get(&before.entity())
                    .expect("checked retained occurrence");
                dependencies.remove_entry(&before.entity(), &old.dependencies);
            }
            allocation.claim_tree::<(EntityId, Arc<Entry>)>(execution, entries.len())?;
            if let Some(proposed) = proposed {
                let entity = proposed.entity();
                let entry = project(
                    execution,
                    &self.program,
                    &after,
                    proposed.clone(),
                    projections,
                )?;
                index_entry(execution, &mut dependencies, entity, &entry, &allocation)?;
                entries.insert(entity, Arc::new(entry));
            } else if let Some(before) = before {
                entries.remove(&before.entity());
            }
        }
        Ok(Self {
            owner: self.owner,
            program: self.program.clone(),
            scope: self.scope,
            entries,
            dependencies,
            _allocation: allocation,
        })
    }

    pub(in crate::domain_computation::primary_graph::application_query::managed_index) fn len(
        &self,
    ) -> usize {
        self.entries.len()
    }
    pub(in crate::domain_computation::primary_graph::application_query::managed_index) fn owner(
        &self,
    ) -> WorthQueryGraphWorkSessionIdentity {
        self.owner
    }
    pub(in crate::domain_computation::primary_graph::application_query::managed_index) fn contains(
        &self,
        entity: EntityId,
    ) -> bool {
        self.entries.contains_key(&entity)
    }
    pub(in crate::domain_computation::primary_graph::application_query::managed_index) fn keys(
        &self,
    ) -> impl Iterator<Item = EntityId> + '_ {
        self.entries.keys().copied()
    }
    pub(in crate::domain_computation::primary_graph::application_query::managed_index) fn at(
        &self,
        ordinal: usize,
    ) -> Option<(EntityId, Node<'_>)> {
        let (entity, entry) = self.entries.iter().nth(ordinal)?;
        Some((*entity, entry.row.only()?))
    }
    pub(in crate::domain_computation::primary_graph::application_query::managed_index) fn get(
        &self,
        entity: EntityId,
    ) -> Option<Node<'_>> {
        self.entries.get(&entity)?.row.only()
    }
    pub(in crate::domain_computation::primary_graph::application_query::managed_index) fn view<
        'a,
    >(
        &'a self,
        bindings: &'a WorthQueryInstalledApplicationValueBindingCatalog,
        source_budget: Option<&'a worth_relational::facade::runtime::CustomInvariantReadBudget<'a>>,
    ) -> SourceImageView<'a> {
        SourceImageView {
            owner: self.owner,
            rows: SourceImageRows::Maintained(self),
            // This controls the Source's compiled projection slots only. The
            // material carries no public admission; each reader needs its loan.
            governance: &WorthQueryApplicationQueryGovernance::Public,
            bindings,
            source_budget,
        }
    }
}

fn project(
    execution: &ExecutionRequest<'_, '_>,
    program: &ManagedIndexSourceProgram,
    rows: &ManagedIndexSourceRows<'_>,
    occurrence: ManagedIndexSourceOccurrence,
    projections: &[WorthQueryInstalledGraphProjection],
) -> Result<Entry, MaterialDenial> {
    let allocation = SourceFactsAllocation::open(execution)?;
    allocation.claim_slice::<Field>(execution, projections.len())?;
    let mut fields = Vec::new();
    fields
        .try_reserve_exact(projections.len())
        .map_err(|_| RelationalExecutionDenialCause::AllocationFailed)?;
    for projection in projections {
        let column = program
            .fields()
            .iter()
            .position(|field| {
                field.locator.aspect().aspect_key() == projection.aspect_key()
                    && field.locator.field_path().fields()
                        == std::slice::from_ref(projection.field_key())
            })
            .ok_or(DerivedRelationReadDenial::Definition(
                DerivedRelationDenial::InvalidExpression,
            ))?;
        let value = match rows.field(&occurrence, column)? {
            Some(value) => {
                allocation.claim(execution, value.owned_allocation_capacity_bytes() as u64)?;
                value.clone()
            }
            None if projection.presence() == worth_query_declaration::facade::application_schema::ApplicationFieldPresence::Optional => {
                worth_foundational::facade::AspectValue::Null
            }
            None => return Err(DerivedRelationReadDenial::Definition(DerivedRelationDenial::RecordUnavailable).into()),
        };
        fields.push(Field::new(projection, value));
    }
    allocation.claim_slice::<Row>(execution, 1)?;
    let row = Row::new(occurrence.entity(), None, true, fields, Vec::new());
    let row =
        WorkingTree::new(vec![row]).into_disclosed(&WorthQueryApplicationQueryGovernance::Public);
    let mut dependencies = BTreeSet::new();
    for dependency in std::iter::once(ViewDependency::Entity(occurrence.entity())).chain(
        program
            .fields()
            .iter()
            .map(|field| ViewDependency::Field(occurrence.entity(), field.locator.clone())),
    ) {
        allocation.claim_tree::<ViewDependency>(execution, dependencies.len())?;
        allocation.claim(execution, dependency.retained_bytes() as u64)?;
        dependencies.insert(dependency);
    }
    Ok(Entry {
        occurrence,
        row,
        dependencies,
        _allocation: allocation,
    })
}

fn index_entry(
    execution: &ExecutionRequest<'_, '_>,
    index: &mut DependencyIndex<EntityId>,
    entity: EntityId,
    entry: &Entry,
    allocation: &Arc<SourceFactsAllocation>,
) -> Result<(), MaterialDenial> {
    for dependency in &entry.dependencies {
        allocation.claim(
            execution,
            u64::try_from(DependencyIndex::<EntityId>::entry_insertion_bound(
                dependency,
            ))
            .map_err(|_| RelationalExecutionDenialCause::ChargedBytesOverflow)?,
        )?;
        index.insert_entry(dependency, &entity);
    }
    Ok(())
}

pub(in crate::domain_computation::primary_graph::application_query::managed_index) fn project_row(
    execution: &ExecutionRequest<'_, '_>,
    program: &ManagedIndexSourceProgram,
    rows: &ManagedIndexSourceRows<'_>,
    occurrence: ManagedIndexSourceOccurrence,
    projections: &[WorthQueryInstalledGraphProjection],
) -> Result<(Tree, Arc<SourceFactsAllocation>), MaterialDenial> {
    let entry = project(execution, program, rows, occurrence, projections)?;
    Ok((entry.row, entry._allocation))
}
