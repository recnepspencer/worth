//! Preparation and publication use the same registered Native branch cells.
use super::*;
use worth_relational::facade::{
    mvcc::{
        CompanionPreflightStop as Stop, PublicationCompanionPreflight, ReservedCompanionBranchCell,
    },
    runtime::{
        ManagedIndexBranchImage, ManagedIndexSourcePreparation,
        ManagedIndexSourcePreparationDenial as Denial, PreparedManagedIndexSourceRoot,
    },
};
mod fork;
pub(super) mod initialization;

/// This lookup is mutated only under the registry mutex; it has no persistent
/// map clones. Its dedicated grant covers the actual key and conservative node.
pub(super) struct SourceBranchSlot {
    slot: worth_relational::facade::mvcc::CompanionBranchCellSlot<ManagedIndexBranchImage>,
    allocation: Arc<crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation>,
}
impl SourceBranchSlot {
    fn new(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        branch: &worth_relational::facade::history::BranchId,
        slot: worth_relational::facade::mvcc::CompanionBranchCellSlot<ManagedIndexBranchImage>,
        count: usize,
    ) -> Result<Self, worth_relational::facade::transactions::RelationalExecutionDenialCause> {
        let allocation =
            crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation::open(
                execution,
            )?;
        allocation
            .claim_tree::<(worth_relational::facade::history::BranchId, Self)>(execution, count)?;
        allocation.claim(execution, branch.0.len() as u64)?;
        Ok(Self { slot, allocation })
    }
}

impl ManagedDerivedViewRegistry {
    pub(in crate::domain_computation::primary_graph) fn manages_native_sources(
        &self,
        branch: &worth_relational::facade::branch::RelationalBranchIdentity,
    ) -> bool {
        self.native_runtime == Some(branch.runtime_instance_id()) && !self.sources.is_empty()
    }
    /// Authenticate this actual Native receipt against the installed resource owner.
    /// Notification is carried beyond all operation guards; final Drop is here.
    pub(in crate::domain_computation::primary_graph) fn retire_source_branch(
        &self,
        deleted: &worth_relational::facade::branch::DeletedRelationalBranch,
    ) {
        if self.native_runtime != Some(deleted.identity().runtime_instance_id()) {
            return;
        }
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let retired = state.source_branches.remove(deleted.identity().branch_id());
        let empty = state
            .source_branches
            .is_empty()
            .then(|| std::mem::take(&mut state.source_branches));
        drop(state);
        drop(empty);
        drop(retired);
    }
    pub(in crate::domain_computation::primary_graph) fn prepare_sources(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &ManagedIndexSourcePreparation<'_>,
    ) -> Result<Vec<PreparedManagedIndexSourceRoot>, Denial> {
        if self.native_runtime != Some(source.selected_basis().identity().runtime_instance_id()) {
            return Err(Denial::ForeignSource);
        }
        let service = self
            .sources
            .service(source.program())
            .ok_or(Denial::MissingParticipant)?;
        let selected = source.selected_basis();
        let cell = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .source_branches
            .get(selected.identity().branch_id())
            .and_then(|entry| entry.slot.admitted());
        let image = cell.as_ref().map(|cell| cell.read_image());
        if image
            .as_ref()
            .is_some_and(|image| image.root_id() != selected.descriptor().root_identity())
        {
            return Err(Denial::ForeignSource);
        }
        let initialization = self
            .initialization
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .upgrade();
        let initializing = initialization
            .as_ref()
            .map(|session| session.applies_to(source))
            .transpose()?
            .unwrap_or(false);
        let mut roots = Vec::new();
        roots.try_reserve_exact(source.scopes().len()).map_err(|_| {
            Denial::Source(worth_relational::facade::runtime::DerivedRelationReadDenial::Request(
                worth_relational::facade::transactions::RelationalExecutionDenialCause::AllocationFailed,
            ))
        })?;
        for scope in source.scopes() {
            execution.consult().map_err(|cause| {
                Denial::Source(
                    worth_relational::facade::runtime::DerivedRelationReadDenial::Request(
                        cause.into(),
                    ),
                )
            })?;
            let retained = image
                .as_ref()
                .map(|image| image.payload().retained(source.program(), scope.scope()))
                .transpose()?
                .flatten();
            let retained = retained.as_ref().and_then(|entry| entry.payload());
            let predecessor = if retained.is_none() && scope.scope_exists()? {
                match scope.ordered_predecessor()? {
                    Some(parent) => image
                        .as_ref()
                        .map(|image| image.payload().retained(source.program(), parent))
                        .transpose()?
                        .flatten(),
                    _ => None,
                }
            } else {
                None
            };
            let predecessor = predecessor.as_ref().filter(|entry| entry.physical_availability() == worth_relational::facade::runtime::ManagedIndexSourceAvailability::Available)
                .and_then(|entry| entry.payload());
            if initializing && !scope.committed_scope_exists()? && scope.scope_exists()? {
                roots.push(
                    service.prepare_initial(
                        execution,
                        &initialization
                            .as_ref()
                            .expect("admitted active session")
                            .request
                            .proof,
                        scope,
                        retained,
                    )?,
                );
            } else {
                roots.push(service.prepare(execution, scope, retained, predecessor)?);
            }
        }
        execution.consult().map_err(|cause| {
            Denial::Source(
                worth_relational::facade::runtime::DerivedRelationReadDenial::Request(cause.into()),
            )
        })?;
        Ok(roots)
    }

    pub(in crate::domain_computation::primary_graph) fn reserve_source_image(
        &self,
        context: &mut PublicationCompanionPreflight<'_>,
    ) -> Result<Option<ReservedCompanionBranchCell<ManagedIndexBranchImage>>, Stop> {
        if self.sources.is_empty() {
            return Ok(None);
        }
        if self.native_runtime != Some(context.runtime_instance_id()) {
            return Err(Stop::ForeignCell);
        }
        context.checkpoint()?;
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(entry) = state.source_branches.get(context.branch_id()) {
            let slot = &entry.slot;
            if let Some(cell) = slot.admitted() {
                return cell.reserve_preflight(context).map(Some);
            }
            if let Some(cell) = slot.pending_for_preflight(context)? {
                return cell.reserve_preflight(context).map(Some);
            }
        }
        let cell = context.mint_managed_index_source_cell()?;
        let entry = SourceBranchSlot::new(
            &context.execution_request(),
            context.branch_id(),
            cell.lookup_slot(),
            state.source_branches.len(),
        )
        .map_err(Stop::Execution)?;
        let reserved = cell.reserve_preflight(context);
        let retired = if reserved.is_ok() {
            state
                .source_branches
                .insert(context.branch_id().clone(), entry)
        } else {
            Some(entry)
        };
        drop(state);
        drop(retired);
        reserved.map(Some)
    }
}
