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
pub struct SourceInvalidationOwner {
    pub runtime_instance_id: u64,
    pub resources: WorthQueryInvalidationResources,
    pub managed_views: Arc<dyn crate::ports::ManagedDerivedViewPort>,
    pub branches: Mutex<BranchCells>,
    pub released: Mutex<Vec<Arc<super::super::RecordedSettlementIdentity>>>,
}
pub struct BranchCells {
    pub cells: OrdMap<BranchId, CompanionBranchCellSlot<BranchMarkRoot>>,
    pub vacant: Option<Arc<BranchMarkRoot>>,
}
impl fmt::Debug for SourceInvalidationOwner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SourceInvalidationOwner")
            .finish_non_exhaustive()
    }
}
impl SourceInvalidationOwner {
    pub fn new(resources: WorthQueryInvalidationResources, runtime_instance_id: u64) -> Self {
        Self::with_managed_views(
            resources,
            runtime_instance_id,
            Arc::new(crate::ports::EmptyManagedDerivedViewPort),
        )
    }
    pub fn with_managed_views(
        resources: WorthQueryInvalidationResources,
        runtime_instance_id: u64,
        managed_views: Arc<dyn crate::ports::ManagedDerivedViewPort>,
    ) -> Self {
        resources.attach_runtime_observations();
        Self {
            runtime_instance_id,
            resources,
            managed_views,
            branches: Mutex::new(BranchCells {
                cells: OrdMap::new(),
                vacant: None,
            }),
            released: Mutex::new(Vec::new()),
        }
    }
    pub fn selected_cell(
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
    pub fn vacant_root(&self, branches: &mut BranchCells) -> Arc<BranchMarkRoot> {
        Arc::clone(
            branches
                .vacant
                .get_or_insert_with(|| Arc::new(BranchMarkRoot::initial())),
        )
    }
}

impl worth_query_execution_runtime::facade::ports::RelationalSourceInvalidationPort
    for SourceInvalidationOwner
{
    fn mint_cell_at_head(
        &self,
        subscription: &worth_runtime_bridge::facade::RelationalBridgeCanonicalSubscription,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        branch: &worth_relational::facade::branch::RelationalBranchIdentity,
    ) -> Result<(), worth_query_execution_contracts::facade::HeadCellRegistrationStop> {
        SourceInvalidationOwner::mint_cell_at_head(self, subscription, runtime, branch)
    }

    fn as_publication_companion(
        self: Arc<Self>,
    ) -> Arc<dyn worth_relational::facade::mvcc::RelationalPublicationCompanion> {
        self
    }

    fn as_any(&self) -> &(dyn std::any::Any + Send + Sync) {
        self
    }

    fn into_any_arc(self: Arc<Self>) -> Arc<dyn std::any::Any + Send + Sync> {
        self
    }

    fn latest_position_for_test(
        &self,
    ) -> Option<worth_relational::facade::publication::PatchStreamPosition> {
        SourceInvalidationOwner::latest_position(self)
    }

    fn touched_commits_after_for_test(
        &self,
        branch: &worth_relational::facade::history::BranchId,
        cursor: Option<worth_relational::facade::publication::PatchStreamPosition>,
        ceiling: Option<worth_relational::facade::history::CommitId>,
        limit: usize,
        interest: super::commit_touches::CommitTouchInterest<'_>,
    ) -> super::commit_touches::TouchedCommits {
        SourceInvalidationOwner::touched_commits_after(
            self, branch, cursor, ceiling, limit, interest,
        )
    }
}

mod court_doors_for_test;
