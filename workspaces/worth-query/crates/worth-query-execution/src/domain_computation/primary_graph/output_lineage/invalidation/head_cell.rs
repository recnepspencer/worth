//! Prepaid branch lookup installation under Native true-head publication exclusion.
use super::{
    admission::IndexAdmission, index_capacity, owner::BranchCells, retention,
    source_alignment::BranchMarkRoot, InvalidationEditAdmission, SourceInvalidationOwner,
};
use crate::domain_computation::execution_runtime::source_invalidation::RetainedInvalidationCapacity;
use im::OrdMap;
use std::sync::Arc;
use worth_relational::facade::{
    branch::RelationalBranchIdentity,
    history::BranchId,
    mvcc::{
        CompanionBranchCell, CompanionBranchCellSlot, CompanionPreflightStop,
        PublicationCompanionRegistrationStop,
    },
    runtime::RelationalRuntime,
};

#[derive(Debug)]
pub(in crate::domain_computation) enum HeadCellRegistrationStop {
    Admission(CompanionPreflightStop),
    /// In particular, Native HeadCellPublicationContended stays its own cause.
    Native(PublicationCompanionRegistrationStop),
    LookupChanged,
}
impl From<CompanionPreflightStop> for HeadCellRegistrationStop {
    fn from(stop: CompanionPreflightStop) -> Self {
        Self::Admission(stop)
    }
}

pub(super) struct PreparedBranchLookupInsertion {
    pub(super) name_bytes: u64,
    pub(super) capacity: Arc<RetainedInvalidationCapacity>,
}
impl PreparedBranchLookupInsertion {
    pub(super) fn reserve(
        owner: &SourceInvalidationOwner,
        branches: &BranchCells,
        branch: &BranchId,
        admission: &mut impl IndexAdmission,
    ) -> Result<Self, CompanionPreflightStop> {
        let present = branches.cells.contains_key(branch);
        let branch_bytes = if present { 0 } else { branch.0.len() as u64 };
        admission.bytes(branch_bytes)?;
        admission.work(branch_bytes)?;
        let name_bytes = branches
            .branch_name_bytes
            .checked_add(branch_bytes)
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        let next_count = branches
            .cells
            .len()
            .checked_add(usize::from(!present))
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        let bound = index_capacity::retained_map_bytes::<
            BranchId,
            CompanionBranchCellSlot<BranchMarkRoot>,
        >(next_count)
        .and_then(|n| n.checked_add(name_bytes))
        .and_then(|n| {
            n.checked_add(
                (next_count as u64).checked_mul(
                    CompanionBranchCellSlot::<BranchMarkRoot>::maximum_deferred_retained_bytes(),
                )?,
            )
        })
        .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        let capacity = retention::reserve(&owner.resources, bound, admission)?;
        admission.ordered_edit::<BranchId, CompanionBranchCellSlot<BranchMarkRoot>>(
            branches.cells.len(),
        )?;
        Ok(Self {
            name_bytes,
            capacity,
        })
    }
}

/// The pinned lookup keeps its old ledger ticket through an intervening edit.
struct BranchLookupBasis {
    cells: OrdMap<BranchId, CompanionBranchCellSlot<BranchMarkRoot>>,
    _retained_capacity: Option<Arc<RetainedInvalidationCapacity>>,
}

impl SourceInvalidationOwner {
    /// A missing derived lookup says nothing about the Native head. Reserve
    /// the lookup insertion first, without holding its lock across Native's
    /// exclusion. The callback takes only this lookup and installs prepaid
    /// state; it performs no Native read, publication, admission or request wait.
    pub(in crate::domain_computation) fn mint_cell_at_head(
        &self,
        registration: &worth_runtime_bridge::facade::RelationalBridgeCanonicalSubscription,
        runtime: &RelationalRuntime,
        branch: &RelationalBranchIdentity,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), HeadCellRegistrationStop> {
        if branch.runtime_instance_id() != self.runtime_instance_id {
            return Err(HeadCellRegistrationStop::Native(
                PublicationCompanionRegistrationStop::ForeignRuntime,
            ));
        }
        let (expected, initial, prepared) = {
            let mut branches = self
                .branches
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            admission.ordered_read(branches.cells.len())?;
            if branches
                .cells
                .get(branch.branch_id())
                .and_then(|slot| slot.admitted())
                .is_some()
            {
                return Ok(());
            }
            let initial = self.vacant_root(&mut branches, admission)?;
            let prepared = PreparedBranchLookupInsertion::reserve(
                self,
                &branches,
                branch.branch_id(),
                admission,
            )?;
            admission.work(3)?; // map snapshot, capacity pin, same-map install fence
            let expected = BranchLookupBasis {
                cells: branches.cells.clone(),
                _retained_capacity: branches.retained_capacity.as_ref().map(Arc::clone),
            };
            (expected, initial, prepared)
        };
        // This copy is prepared before exclusion as part of the reservation.
        let install_branch = branch.branch_id().clone();
        registration
            .with_branch_cell_at_head(runtime, branch, initial, |cell| {
                self.install_head_lookup(&expected, install_branch, cell, prepared)
            })
            .map_err(HeadCellRegistrationStop::Native)?
    }

    fn install_head_lookup(
        &self,
        expected: &BranchLookupBasis,
        branch: BranchId,
        cell: CompanionBranchCell<BranchMarkRoot>,
        prepared: PreparedBranchLookupInsertion,
    ) -> Result<(), HeadCellRegistrationStop> {
        let mut branches = self
            .branches
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !branches.cells.ptr_eq(&expected.cells) {
            // An intervening publisher may have added another branch. Its map
            // needs a different reservation, so decline this derived insertion.
            return Err(HeadCellRegistrationStop::LookupChanged);
        }
        branches.cells.insert(branch, cell.into_lookup_slot());
        branches.branch_name_bytes = prepared.name_bytes;
        branches.retained_capacity = Some(prepared.capacity);
        Ok(())
    }
}
