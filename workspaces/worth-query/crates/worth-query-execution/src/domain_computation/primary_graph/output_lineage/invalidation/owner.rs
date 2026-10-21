use super::source_alignment::BranchMarkRoot;
use crate::domain_computation::execution_runtime::WorthQueryInvalidationResources;
use im::OrdMap;
use std::fmt;
use std::sync::Arc;
use std::sync::Mutex;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::mvcc::CompanionBranchCellSlot;
use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_relational::facade::mvcc::PublicationCompanionPreflight;
pub(in crate::domain_computation) struct SourceInvalidationOwner {
    pub(super) runtime_instance_id: u64,
    pub(super) resources: WorthQueryInvalidationResources,
    pub(super) branches: Mutex<BranchCells>,
    pub(super) released: Mutex<Vec<Arc<super::super::RecordedSettlementIdentity>>>,
}
pub(super) struct BranchCells {
    pub(super) cells: OrdMap<BranchId, CompanionBranchCellSlot<BranchMarkRoot>>,
    pub(super) vacant: Option<Arc<BranchMarkRoot>>,
}
impl fmt::Debug for SourceInvalidationOwner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SourceInvalidationOwner")
            .finish_non_exhaustive()
    }
}
impl SourceInvalidationOwner {
    pub(in crate::domain_computation) fn new(
        resources: WorthQueryInvalidationResources,
        runtime_instance_id: u64,
    ) -> Self {
        resources.attach_runtime_observations();
        Self {
            runtime_instance_id,
            resources,
            branches: Mutex::new(BranchCells {
                cells: OrdMap::new(),
                vacant: None,
            }),
            released: Mutex::new(Vec::new()),
        }
    }
    pub(super) fn selected_cell(
        &self,
        context: &mut PublicationCompanionPreflight<'_>,
    ) -> Result<super::publication_cell::SelectedPublicationCell, CompanionPreflightStop> {
        context.checkpoint()?;
        let mut branches = self
            .branches
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(slot) = branches.cells.get(context.branch_id()) {
            if let Some(cell) = slot.admitted() {
                return Ok(super::publication_cell::SelectedPublicationCell::Admitted(
                    cell,
                ));
            }
            if let Some(cell) = slot.pending_for_preflight(context)? {
                return Ok(super::publication_cell::SelectedPublicationCell::Prepared(
                    cell,
                ));
            }
        }
        let initial = self.vacant_root(&mut branches);
        let cell = context.mint_selected_branch_cell(initial)?;
        branches
            .cells
            .insert(context.branch_id().clone(), cell.lookup_slot());
        Ok(super::publication_cell::SelectedPublicationCell::Prepared(
            cell,
        ))
    }
    pub(super) fn vacant_root(&self, branches: &mut BranchCells) -> Arc<BranchMarkRoot> {
        Arc::clone(
            branches
                .vacant
                .get_or_insert_with(|| Arc::new(BranchMarkRoot::initial())),
        )
    }
}
