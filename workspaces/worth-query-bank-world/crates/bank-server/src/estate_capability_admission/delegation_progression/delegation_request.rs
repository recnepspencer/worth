//! Honest delegation commands reused by the public-entry contracts.
use super::*;

pub(super) fn delegated_action(delegation: DelegationLimit) -> EstateAction {
    delegated_action_for(CHILD, delegation)
}

pub(super) fn delegated_action_for(
    child_id: CapabilityGrantId,
    delegation: DelegationLimit,
) -> EstateAction {
    delegated_action_from(GRANT, child_id, APPROVER, delegation)
}

pub(super) fn delegated_action_from(
    parent: CapabilityGrantId,
    child_id: CapabilityGrantId,
    grantee: BankPrincipalId,
    delegation: DelegationLimit,
) -> EstateAction {
    EstateAction::DelegateCapability {
        estate: ESTATE,
        parent,
        child: EstateCapabilityDelegationRequest {
            id: child_id,
            grantee,
            scope: EstateCapabilityScope {
                account: None,
                estate: ESTATE,
                institution: INSTITUTION,
                branch: BRANCH,
                operation: EstateCapabilityOperation::ViewRestrictedEstate,
                purpose: EstateCapabilityPurpose::EstateAdministration,
                field: Some(RestrictedBankField::GovernanceMetadata),
                amount_ceiling: None,
                validity: CapabilityValidity::new(
                    EstateMoment::from_epoch_seconds(0),
                    EstateMoment::from_epoch_seconds(u64::MAX),
                )
                .unwrap(),
                delegation,
                workflow_stage: EstateWorkflowStage::Administration,
            },
        },
    }
}
