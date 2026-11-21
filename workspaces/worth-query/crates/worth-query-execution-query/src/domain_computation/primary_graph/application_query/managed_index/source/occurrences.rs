//! Immutable material from actual Native Source loans, never a retained read grant.
use super::super::source_rows::{SourceImageRows, SourceImageView};
use super::projection::{project, project_observed, Entry, ProjectionMembership};
use crate::domain_computation::primary_graph::{
    application_query::{
        disclosure::WorthQueryApplicationQueryGovernance,
        projection::WorthQueryApplicationDisclosedProjectionNode as Node,
    },
    source_facts::SourceFactsAllocation,
};
use crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity;
use std::{collections::BTreeSet, sync::Arc};
use worth_execution::{ExecutionOrderedMap, ExecutionRequest};
mod delta;
mod postings;
use postings::{ordered_denial, OccurrencePostings};
use worth_query_installation::facade::{
    WorthQueryInstalledApplicationValueBindingCatalog, WorthQueryInstalledGraphReadContract,
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
pub enum MaterialDenial
{
    Native(ManagedIndexSourcePreparationDenial),
    Storage(RelationalExecutionDenialCause),
}
impl MaterialDenial {
    pub fn into_native(
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

pub struct PersistentSourceRows
{
    owner: WorthQueryGraphWorkSessionIdentity,
    program: ManagedIndexSourceProgram,
    scope: EntityId,
    entries: ExecutionOrderedMap<EntityId, Arc<Entry>>,
    dependencies: OccurrencePostings,
}

impl PersistentSourceRows {
    pub fn program(
        &self,
    ) -> &ManagedIndexSourceProgram {
        &self.program
    }
    pub fn scope(
        &self,
    ) -> EntityId {
        self.scope
    }
    pub fn entry(
        &self,
        entity: EntityId,
    ) -> Option<&Arc<Entry>> {
        self.entries.get(&entity)
    }
    pub fn acquire(
        execution: &ExecutionRequest<'_, '_>,
        rows: &ManagedIndexSourceRows<'_>,
        owner: WorthQueryGraphWorkSessionIdentity,
        graph: &WorthQueryInstalledGraphReadContract,
        maximum: usize,
    ) -> Result<Self, MaterialDenial> {
        Self::acquire_with_support(execution, rows, owner, graph, maximum, None)
            .map(|(rows, _)| rows)
    }
    pub fn acquire_observed(
        execution: &ExecutionRequest<'_, '_>,
        rows: &ManagedIndexSourceRows<'_>,
        owner: WorthQueryGraphWorkSessionIdentity,
        graph: &WorthQueryInstalledGraphReadContract,
        maximum: usize,
    ) -> Result<
        (
            Self,
            crate::domain_computation::provider_session::WorthQueryObservedGraphReadWork,
            super::projection_support::SourceProjectionSupport,
        ),
        MaterialDenial,
    > {
        let mut captured = super::projection_support::SourceProjectionSupport::new(execution)?;
        let (rows, work) = Self::acquire_with_support(
            execution,
            rows,
            owner,
            graph,
            maximum,
            Some(&mut captured),
        )?;
        Ok((rows, work, captured))
    }
    fn acquire_with_support(
        execution: &ExecutionRequest<'_, '_>,
        rows: &ManagedIndexSourceRows<'_>,
        owner: WorthQueryGraphWorkSessionIdentity,
        graph: &WorthQueryInstalledGraphReadContract,
        maximum: usize,
        mut captured: Option<&mut super::projection_support::SourceProjectionSupport>,
    ) -> Result<
        (
            Self,
            crate::domain_computation::provider_session::WorthQueryObservedGraphReadWork,
        ),
        MaterialDenial,
    > {
        let support = rows.acquire_all()?;
        let mut work = super::read_work::SourceProjectionWork::default();
        if let Some(captured) = &mut captured {
            let membership = rows.membership_support(&support)?;
            captured.membership(execution, membership)?;
            // Report the actual compiled membership acquisition, including
            // explored nonmatches. This borrows the same read, never scans again.
            work.relation_records_examined = membership.reads().relation_records;
        }
        let occurrences = rows.occurrences(&support)?;
        let examined_candidates = occurrences.len();
        if occurrences.len() > maximum {
            return Err(ManagedIndexSourcePreparationDenial::Source(
                DerivedRelationReadDenial::Definition(DerivedRelationDenial::WorkBudgetExceeded),
            )
            .into());
        }
        let mut entries = ExecutionOrderedMap::default();
        let mut dependencies = OccurrencePostings::default();
        for occurrence in occurrences {
            let entity = occurrence.entity();
            let (entry, observed) = project_observed(
                execution,
                rows.program(),
                rows,
                occurrence,
                graph,
                ProjectionMembership::Forward(&support),
            )?;
            work.append(observed)?;
            if let Some(captured) = &mut captured {
                captured.row(execution, &entry.dependencies)?;
            }
            if entries.contains_key(&entity) {
                return Err(ManagedIndexSourcePreparationDenial::ForeignSource.into());
            }
            dependencies.insert_entry(entity, &entry.dependencies, execution)?;
            entries
                .insert_cloned_native_for_request(&entity, &entry.into_arc(execution)?, *execution)
                .map_err(|cause| ordered_denial(execution, cause))?;
        }
        Ok((
            Self {
                owner,
                program: rows.program().clone(),
                scope: rows.scope(),
                entries,
                dependencies,
            },
            work.complete(examined_candidates),
        ))
    }

    /// Reassociate immutable material only through Native's declared predecessor.
    pub fn handoff(
        &self,
        scope: &ManagedIndexSourceScope<'_>,
    ) -> Result<Self, ManagedIndexSourcePreparationDenial> {
        if self.program != *scope.program() || scope.ordered_predecessor()? != Some(self.scope) {
            return Err(ManagedIndexSourcePreparationDenial::ForeignSource);
        }
        // Slots embedded in immutable shared nodes keep their build-session
        // coordinates. The successor owns this row set and its distinct key.
        Ok(Self {
            owner: self.owner,
            program: self.program.clone(),
            scope: scope.scope(),
            entries: self.entries.clone(),
            dependencies: self.dependencies.clone(),
        })
    }

    /// The actual changed before/after occurrences come from Native's inverse
    /// path. Unchanged rows and subtrees are shared; no full roster is visited.
    pub fn update(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        scope: &ManagedIndexSourceScope<'_>,
        changed: &[(
            Option<ManagedIndexSourceOccurrence>,
            Option<worth_relational::facade::runtime::PreparedManagedIndexSourceOccurrence>,
        )],
        graph: &WorthQueryInstalledGraphReadContract,
        inherited: bool,
    ) -> Result<
        (
            Self,
            worth_relational::facade::runtime::ManagedIndexSourceChanges,
        ),
        MaterialDenial,
    > {
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
        for entity in changed.iter().flat_map(|(a, b)| {
            a.iter()
                .map(|row| row.entity())
                .chain(b.iter().map(|row| row.entity()))
        }) {
            scope
                .read_budget()
                .checkpoint(1)
                .map_err(DerivedRelationReadDenial::Structural)?;
            allocation.claim_tree::<EntityId>(execution, changed_keys.len())?;
            changed_keys.insert(entity);
        }
        for entity in scope.changed_entities().filter(|_| !inherited) {
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
        let mut delta = delta::SourceDelta::new(inherited);
        for (before, proposed) in changed {
            execution.consult().map_err(|cause| {
                MaterialDenial::Storage(RelationalExecutionDenialCause::from(cause))
            })?;
            let retained = before
                .as_ref()
                .and_then(|before| self.entries.get(&before.entity()).cloned());
            if let Some(before) = before {
                // This also convicts a stale/missing retained full-generation
                // occurrence; a later generation never aliases an earlier slot.
                if entries
                    .get(&before.entity())
                    .map(|entry| entry.occurrence.entity())
                    != Some(before.entity())
                {
                    return Err(ManagedIndexSourcePreparationDenial::ForeignSource.into());
                }
                let old = entries
                    .get(&before.entity())
                    .expect("checked retained occurrence");
                dependencies.remove_entry(before.entity(), &old.dependencies, execution)?;
            }
            if let Some(proposed) = proposed {
                let entity = proposed.entity();
                let entry = project(
                    execution,
                    &self.program,
                    &after,
                    proposed.occurrence().clone(),
                    graph,
                    ProjectionMembership::Inverse(proposed),
                )?;
                delta.observe(execution, scope, retained.as_deref(), Some(&entry))?;
                dependencies.insert_entry(entity, &entry.dependencies, execution)?;
                entries
                    .insert_cloned_native_for_request(
                        &entity,
                        &entry.into_arc(execution)?,
                        *execution,
                    )
                    .map_err(|cause| ordered_denial(execution, cause))?;
            } else if let Some(before) = before {
                delta.observe(execution, scope, retained.as_deref(), None)?;
                entries
                    .remove_for_request(&before.entity(), *execution)
                    .map_err(|cause| ordered_denial(execution, cause))?;
            }
        }
        let rows = Self {
            owner: self.owner,
            program: self.program.clone(),
            scope: self.scope,
            entries,
            dependencies,
        };
        Ok((rows, delta.finish()))
    }

    pub fn len(
        &self,
    ) -> usize {
        self.entries.len()
    }
    pub fn owner(
        &self,
    ) -> WorthQueryGraphWorkSessionIdentity {
        self.owner
    }
    pub fn contains(
        &self,
        entity: EntityId,
    ) -> bool {
        self.entries.contains_key(&entity)
    }
    pub fn keys(
        &self,
    ) -> impl Iterator<Item = EntityId> + '_ {
        self.entries.keys().copied()
    }
    pub fn at(
        &self,
        ordinal: usize,
    ) -> Option<(EntityId, Node<'_>)> {
        let (entity, entry) = self.entries.entry_at(ordinal)?;
        Some((*entity, entry.row.only()?))
    }
    pub fn get(
        &self,
        entity: EntityId,
    ) -> Option<Node<'_>> {
        self.entries.get(&entity)?.row.only()
    }
    pub fn view<
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
