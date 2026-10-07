use std::fmt;
use std::sync::{Arc, Mutex};

use im::OrdMap;
use worth_relational::facade::{
    history::BranchId,
    mvcc::{CompanionBranchCellSlot, CompanionPreflightStop, PublicationCompanionPreflight},
};

use crate::domain_computation::execution_runtime::WorthQueryInvalidationResources;

use super::admission::IndexAdmission;
use super::index_capacity;
use super::retention;
use super::source_alignment::BranchMarkRoot;

/// Callback custody contains derived cells and capacity only. It cannot enter
/// the runtime mutex, retain a source provider, or create a registration cycle.
pub(in crate::domain_computation) struct SourceInvalidationOwner {
    pub(super) runtime_instance_id: u64,
    pub(super) resources: WorthQueryInvalidationResources,
    pub(super) branches: Mutex<BranchCells>,
    /// Released rows that have not left yet. The next release retries them.
    pub(super) released: Mutex<Vec<Arc<super::super::RecordedSettlementIdentity>>>,
}

pub(super) struct BranchCells {
    pub(super) cells: OrdMap<BranchId, CompanionBranchCellSlot<BranchMarkRoot>>,
    pub(super) branch_name_bytes: u64,
    /// Shared, pre-admitted empty image used when publication evicts derived state.
    pub(super) vacant: Option<Arc<BranchMarkRoot>>,
    pub(super) retained_capacity: Option<Arc<crate::domain_computation::execution_runtime::source_invalidation::RetainedInvalidationCapacity>>,
}

impl fmt::Debug for SourceInvalidationOwner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SourceInvalidationOwner")
            .field("resources", &self.resources)
            .finish_non_exhaustive()
    }
}

impl SourceInvalidationOwner {
    pub(in crate::domain_computation) fn new(
        resources: WorthQueryInvalidationResources,
        runtime_instance_id: u64,
    ) -> Self {
        Self {
            runtime_instance_id,
            resources,
            branches: Mutex::new(BranchCells {
                cells: OrdMap::new(),
                branch_name_bytes: 0,
                vacant: None,
                retained_capacity: None,
            }),
            released: Mutex::new(Vec::new()),
        }
    }

    /// Fund the empty publication image at source installation, before any
    /// computation can exhaust the derived ledger.
    pub(in crate::domain_computation) fn admit_publication_fallback(
        &self,
    ) -> Result<(), CompanionPreflightStop> {
        let mut branches = self
            .branches
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        self.vacant_root(&mut branches, &mut self.edit_admission())
            .map(|_| ())
    }

    pub(super) fn selected_cell(
        &self,
        context: &mut PublicationCompanionPreflight<'_>,
    ) -> Result<super::publication_cell::SelectedPublicationCell, CompanionPreflightStop> {
        // The writer waits for the map like readers do. Every holder only
        // looks up, or mints and inserts one cell through atomic capacity
        // counters and non-blocking registration probes; none waits on
        // another lock, so no cycle can form.
        let mut branches = self
            .branches
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        context.work(
            (context.branch_id().0.len() as u64)
                .checked_mul(
                    index_capacity::ordered_navigation_work(branches.cells.len())
                        .ok_or(CompanionPreflightStop::WorkCounterOverflow)?,
                )
                .ok_or(CompanionPreflightStop::WorkCounterOverflow)?,
        )?;
        context.ordered_read(branches.cells.len())?;
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
        let initial = self.vacant_root(&mut branches, context)?;
        let cell = context.mint_selected_branch_cell(initial)?;
        let branch = context.branch_id().clone();
        let slot = cell.lookup_slot();
        match self.retain_slot(&mut branches, branch, slot, context) {
            Ok(()) | Err(CompanionPreflightStop::RetainedCompanionCapacityExhausted { .. }) => Ok(
                super::publication_cell::SelectedPublicationCell::Prepared(cell),
            ),
            Err(stop) => Err(stop),
        }
    }

    pub(super) fn vacant_root(
        &self,
        branches: &mut BranchCells,
        admission: &mut impl IndexAdmission,
    ) -> Result<Arc<BranchMarkRoot>, CompanionPreflightStop> {
        if let Some(vacant) = &branches.vacant {
            return Ok(Arc::clone(vacant));
        }
        let vacant = self.initial_root(admission)?;
        branches.vacant = Some(Arc::clone(&vacant));
        Ok(vacant)
    }

    fn initial_root(
        &self,
        admission: &mut impl IndexAdmission,
    ) -> Result<Arc<BranchMarkRoot>, CompanionPreflightStop> {
        admission.bytes(
            index_capacity::arc_bytes::<BranchMarkRoot>()
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
        )?;
        admission.bytes(
            index_capacity::arc_bytes::<super::mark_state::MarkState>()
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
        )?;
        let mut initial = BranchMarkRoot::initial();
        retention::admit_first(
            Arc::get_mut(&mut initial.current).expect("new state is exclusive"),
            &self.resources,
            admission,
        )?;
        retention::admit_root(&mut initial, None, &self.resources, admission)?;
        Ok(Arc::new(initial))
    }

    fn retain_slot(
        &self,
        branches: &mut BranchCells,
        branch: BranchId,
        slot: CompanionBranchCellSlot<BranchMarkRoot>,
        context: &mut impl IndexAdmission,
    ) -> Result<(), CompanionPreflightStop> {
        let prepared = super::head_cell::PreparedBranchLookupInsertion::reserve(
            self, branches, &branch, context,
        )?;
        branches.cells.insert(branch, slot);
        branches.branch_name_bytes = prepared.name_bytes;
        branches.retained_capacity = Some(prepared.capacity);
        Ok(())
    }
}
