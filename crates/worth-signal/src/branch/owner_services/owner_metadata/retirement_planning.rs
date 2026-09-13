use crate::branch::{SignalBranchRetirementDenial, SignalOwnerUnavailable};
use crate::state::SignalBranchId;

use super::super::SignalOwnerOperationAdmission;
use super::{SignalOwnerMetadata, SignalOwnerMetadataAuthorizationDenial};

pub(in crate::branch::owner_services) struct SignalOwnerRetirementPlanningFacts {
    pub(in crate::branch::owner_services) child_branch_ids: Vec<SignalBranchId>,
    pub(in crate::branch::owner_services) merge_participant: bool,
}

impl<D, I, T> SignalOwnerMetadata<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn retirement_planning_facts(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        branch_id: SignalBranchId,
    ) -> Result<SignalOwnerRetirementPlanningFacts, SignalBranchRetirementDenial> {
        let _hold = self
            .authorize(admission)
            .map_err(|denial| map_retirement_planning_metadata_denial(denial, branch_id))?;
        let state = self.lock();
        Ok(SignalOwnerRetirementPlanningFacts {
            child_branch_ids: state.branch_children(branch_id),
            merge_participant: state.is_merge_participant(branch_id),
        })
    }
}

fn map_retirement_planning_metadata_denial(
    denial: SignalOwnerMetadataAuthorizationDenial,
    branch_id: SignalBranchId,
) -> SignalBranchRetirementDenial {
    match denial {
        SignalOwnerMetadataAuthorizationDenial::OwnerUnavailable => {
            SignalBranchRetirementDenial::OwnerUnavailable(SignalOwnerUnavailable)
        }
        SignalOwnerMetadataAuthorizationDenial::OwnerCellMisuse => {
            SignalBranchRetirementDenial::OwnerCellMisuse { branch_id }
        }
        SignalOwnerMetadataAuthorizationDenial::OwnerReentry => {
            SignalBranchRetirementDenial::OwnerReentry
        }
    }
}
