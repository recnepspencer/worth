use std::mem::size_of;

use super::WorthQueryGraphIndexSupportRow;
use crate::graph_read_access::graph_index_inventory::{
    support_row_defaults::{
        default_bases_for_requirement, support_posture_for_requirement, support_state_for_posture,
    },
    WorthQueryGraphIndexInventoryAdmissionStop, WorthQueryGraphIndexLifecycleClass,
    WorthQueryGraphIndexLifecycleOwner, WorthQueryGraphIndexPosture,
    WorthQueryGraphIndexSupportState,
};
use crate::graph_read_access::{
    WorthQueryGraphReadAccessRequirementKind, WorthQueryGraphReadAccessRequirementRow,
};

fn supported_row_without_digest(
    requirement: &WorthQueryGraphReadAccessRequirementRow,
) -> WorthQueryGraphIndexSupportRow {
    let kind = requirement.kind().clone();
    let (rebuild_basis, invalidation_basis, complexity_contract) =
        default_bases_for_requirement(&kind);
    let (lifecycle_owner, lifecycle_class, posture, support_state, owning_milestone) =
        if kind == WorthQueryGraphReadAccessRequirementKind::LiveMaintenanceSupport {
            (
                WorthQueryGraphIndexLifecycleOwner::QueryRuntime,
                WorthQueryGraphIndexLifecycleClass::RuntimeMaintained,
                WorthQueryGraphIndexPosture::Verified,
                WorthQueryGraphIndexSupportState::Available,
                None,
            )
        } else {
            let (owner, lifecycle, posture, milestone) = support_posture_for_requirement(&kind);
            let state = support_state_for_posture(&posture);
            (owner, lifecycle, posture, state, milestone)
        };
    WorthQueryGraphIndexSupportRow {
        digest: String::new(),
        requirement_kind: kind,
        supported_relation_direction: requirement.relation_direction().cloned(),
        supported_predicate_family: requirement.predicate_family().cloned(),
        supported_ordering_posture: requirement.ordering_posture().cloned(),
        supported_requirement_lifecycle: requirement.lifecycle_class().cloned(),
        lifecycle_owner,
        lifecycle_class,
        rebuild_basis,
        invalidation_basis,
        complexity_contract,
        posture,
        support_state,
        owning_milestone,
    }
}

impl WorthQueryGraphIndexSupportRow {
    pub fn for_supported_requirement(
        requirement: &WorthQueryGraphReadAccessRequirementRow,
    ) -> Self {
        let mut row = supported_row_without_digest(requirement);
        row.digest = super::digest::ordinary_digest(&row);
        row
    }

    pub fn for_supported_requirement_admitted<Stop>(
        requirement: &WorthQueryGraphReadAccessRequirementRow,
        admit: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<Self, WorthQueryGraphIndexInventoryAdmissionStop<Stop>> {
        use WorthQueryGraphIndexInventoryAdmissionStop as Refusal;
        // Kind/default basis, four supported axes and final posture metadata.
        admit(8, 0).map_err(Refusal::Admission)?;
        if requirement.kind()
            == &WorthQueryGraphReadAccessRequirementKind::DomainOperationCapabilityRegistration
        {
            let bytes = "worth-query-9.10-"
                .len()
                .checked_add(requirement.kind().as_str().len())
                .and_then(|bytes| u64::try_from(bytes).ok())
                .ok_or(Refusal::AccountingOverflow)?;
            admit(bytes, bytes).map_err(Refusal::Admission)?;
        }
        admit(size_of::<Self>() as u64, 0).map_err(Refusal::Admission)?;
        let mut row = supported_row_without_digest(requirement);
        row.digest = super::digest::admitted_digest(&row, admit)?;
        Ok(row)
    }
}
