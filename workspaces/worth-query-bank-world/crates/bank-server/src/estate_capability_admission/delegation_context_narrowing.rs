use bank_domain::{
    estate::{
        CapabilityGrantId, CapabilityValidity, DelegationLimit, EstateAction,
        EstateCapabilityDelegationRequest, EstateCapabilityOperation, EstateCapabilityPurpose,
        EstateCapabilityScope, EstateMoment, EstateWorkflowStage, RestrictedBankField,
    },
    proposals::BankIdempotencyKey,
};

use super::fixture::{
    delegation_world_with_parent_branch_mismatch,
    delegation_world_with_parent_institution_mismatch, request_scope, CapabilityFixture, APPROVER,
    BRANCH, ESTATE, GRANT, INSTITUTION,
};
use crate::{BankEstateProgressionDenial, BankReadControls};

const BRANCH_CHILD: CapabilityGrantId = CapabilityGrantId::new(411).unwrap();
const INSTITUTION_CHILD: CapabilityGrantId = CapabilityGrantId::new(412).unwrap();

#[test]
fn child_branch_must_be_carried_by_the_exact_parent() {
    assert_context_denied(
        delegation_world_with_parent_branch_mismatch("delegation-parent-branch-mismatch"),
        BRANCH_CHILD,
        131,
    );
}

#[test]
fn child_institution_must_be_carried_by_the_exact_parent() {
    assert_context_denied(
        delegation_world_with_parent_institution_mismatch("delegation-parent-institution-mismatch"),
        INSTITUTION_CHILD,
        133,
    );
}

fn assert_context_denied(fixture: CapabilityFixture, child: CapabilityGrantId, seed: u8) {
    let denial = fixture
        .runtime
        .delegate_estate_capability_with_key(
            &fixture.authenticate(),
            delegated_action(child),
            &BankIdempotencyKey::new(format!("delegation-context-{seed}")).unwrap(),
            &request_scope(),
        )
        .expect_err("a child cannot replace an exact parent activation context");
    let BankEstateProgressionDenial::Authorization(denial) = denial else {
        panic!("context narrowing must deny at Query delegation authorization: {denial:#?}");
    };
    assert_eq!(
        denial.kind(),
        crate::BankAuthorizationDenialKind::DelegationRejected
    );
    let readback = fixture
        .runtime
        .query(crate::queries::estate_governance_context(ESTATE))
        .as_principal(&fixture.authenticate_executor())
        .controls(BankReadControls::current(request_scope(), 1, 20_000).unwrap())
        .execute()
        .expect("independent governance authority must inspect the denied poststate");
    assert!(readback.rows()[0]
        .capabilities()
        .iter()
        .all(|capability| capability.id() != child));
}

fn delegated_action(child_id: CapabilityGrantId) -> EstateAction {
    EstateAction::DelegateCapability {
        estate: ESTATE,
        parent: GRANT,
        child: EstateCapabilityDelegationRequest {
            id: child_id,
            grantee: APPROVER,
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
                delegation: DelegationLimit::generations(1),
                workflow_stage: EstateWorkflowStage::Administration,
            },
        },
    }
}
