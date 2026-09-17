use bank_domain::{
    estate::{BankEstateWorld, DelegationLimit, EstateEmployeeAssignment},
    model::EmployeeRole,
};

use super::{super::*, DelegationParentContext};

pub(super) fn install_grants(
    estate: BankEstateWorld,
    command_authority: bool,
    parent_context: DelegationParentContext,
    parent_spec: GrantSpec,
) -> BankEstateWorld {
    let mut parent = grant(GRANT, SPECIALIST, parent_spec);
    parent.scope.delegation = DelegationLimit::generations(2);
    match parent_context {
        DelegationParentContext::Exact => {}
        DelegationParentContext::Branch => parent.scope.branch = ALTERNATE_BRANCH,
        DelegationParentContext::Institution => parent.scope.institution = ALTERNATE_INSTITUTION,
    }
    let estate = estate
        .with_assignment(EstateEmployeeAssignment {
            id: DELEGATION_EXECUTOR_ASSIGNMENT,
            principal: EXECUTOR,
            institution: INSTITUTION,
            branch: BRANCH,
            role: EmployeeRole::EstateSpecialist,
        })
        .with_estate_assignment(ESTATE, DELEGATION_EXECUTOR_ASSIGNMENT)
        .with_assignment(EstateEmployeeAssignment {
            id: DELEGATION_REVIEWER_ASSIGNMENT,
            principal: REVIEWER,
            institution: INSTITUTION,
            branch: BRANCH,
            role: EmployeeRole::EstateSpecialist,
        })
        .with_estate_assignment(ESTATE, DELEGATION_REVIEWER_ASSIGNMENT)
        .with_grant(parent)
        .with_grant(grant(
            UNRELATED_GOVERNANCE_GRANT,
            EXECUTOR,
            GrantSpec::governance_view(),
        ));
    if !command_authority {
        return estate;
    }
    estate
        .with_grant(grant(COMMAND_GRANT, SPECIALIST, GrantSpec::delegate()))
        .with_grant(grant(
            APPROVER_DELEGATION_GRANT,
            APPROVER,
            GrantSpec::delegate(),
        ))
        .with_grant(grant(
            REVOKE_CAPABILITY_GRANT,
            SPECIALIST,
            GrantSpec::revoke_capability(),
        ))
        .with_grant(grant(
            APPROVER_REVOKE_CAPABILITY_GRANT,
            APPROVER,
            GrantSpec::revoke_capability(),
        ))
}
