//! Query admission for the actual selected Bridge truth-basis construction.

use super::*;
use worth_relational::facade::mvcc::CompanionPreflightStop as Stop;

impl WorthQueryConditionalTruthBasis {
    pub(in crate::domain_computation::primary_graph) fn from_selected_admitted<Schema>(
        selected: crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<
            '_,
            Schema,
        >,
        admission: &mut crate::domain_computation::primary_graph::InvalidationEditAdmission,
    ) -> Result<Self, Stop> {
        // The selected snapshot and Bridge source are shallow Arc retains.
        // The primary branch projection builds one raw Arc, one framed String,
        // and one framed Arc before the existing truth-basis constructor.
        admission.charge_external_work(6)?;
        let branch = super::super::super::application_branch::PRIMARY_APPLICATION_BRANCH;
        let framed = TruthBranchIdentity::relational_branch_framed_len(branch)
            .ok_or(Stop::WorkCounterOverflow)?;
        let backing = crate::domain_computation::arc_str_layout::backing_bytes(branch.len())
            .and_then(|raw| {
                crate::domain_computation::arc_str_layout::backing_bytes(framed)
                    .and_then(|framed_arc| raw.checked_add(framed_arc))
            })
            .and_then(|bytes| bytes.checked_add(framed))
            .ok_or(Stop::PreparationMemoryCounterOverflow)?;
        let copy_work = branch
            .len()
            .checked_mul(2)
            .and_then(|bytes| {
                framed
                    .checked_mul(2)
                    .and_then(|framed| bytes.checked_add(framed))
            })
            .and_then(|work| {
                work.checked_add(
                    2 * crate::domain_computation::arc_str_layout::initialized_header_work(),
                )
            })
            .and_then(|work| work.checked_add(std::mem::size_of::<Self>() + 6))
            .ok_or(Stop::WorkCounterOverflow)?;
        admission.charge_external_work(
            u64::try_from(copy_work).map_err(|_| Stop::WorkCounterOverflow)?,
        )?;
        admission.admit_read_scratch(
            u64::try_from(backing).map_err(|_| Stop::PreparationMemoryCounterOverflow)?,
        )?;
        Ok(Self::from_selected(selected))
    }
}
