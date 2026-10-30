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

impl ManagedDerivedViewRegistry {
    pub(in crate::domain_computation::primary_graph) fn prepare_sources(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &ManagedIndexSourcePreparation<'_>,
    ) -> Result<Vec<PreparedManagedIndexSourceRoot>, Denial> {
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
            .and_then(|slot| slot.admitted());
        let image = cell.as_ref().map(|cell| cell.read_image());
        if image
            .as_ref()
            .is_some_and(|image| image.root_id() != selected.descriptor().root_identity())
        {
            return Err(Denial::ForeignSource);
        }
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
                .and_then(|image| image.payload().retained(source.program(), scope.scope()))
                .map(|entry| entry.payload());
            roots.push(service.prepare(execution, scope, retained)?);
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
        context.checkpoint()?;
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(slot) = state.source_branches.get(context.branch_id()) {
            if let Some(cell) = slot.admitted() {
                return cell.reserve_preflight(context).map(Some);
            }
            if let Some(cell) = slot.pending_for_preflight(context)? {
                return cell.reserve_preflight(context).map(Some);
            }
        }
        let cell =
            context.mint_selected_branch_cell(Arc::new(ManagedIndexBranchImage::default()))?;
        state
            .source_branches
            .insert(context.branch_id().clone(), cell.lookup_slot());
        cell.reserve_preflight(context).map(Some)
    }
}
