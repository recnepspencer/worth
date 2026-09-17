use bank_domain::proposals::BankIdempotencyKey;
use bank_domain::{
    estate::{
        CapabilityGrantId, CapabilityValidity, DelegationLimit, EstateAction,
        EstateCapabilityDelegationRequest, EstateCapabilityOperation, EstateCapabilityPurpose,
        EstateCapabilityScope, EstateMoment, EstateWorkflowStage, RestrictedBankField,
    },
    model::{Money, USD},
};

use super::fixture::{
    delegation_world, delegation_world_with_parent_spec, request_scope, CapabilityFixture,
    GrantSpec, ACCOUNT, APPROVER, BRANCH, ESTATE, GRANT, INSTITUTION, OTHER_ACCOUNT,
    UNRELATED_GOVERNANCE_GRANT,
};
use crate::{BankEstateProgressionDenial, BankReadControls};

const CHILD: CapabilityGrantId = CapabilityGrantId::new(421).unwrap();

#[test]
fn a_real_alternate_grant_cannot_substitute_for_the_selected_parent() {
    assert_denied(
        delegation_world("delegation-substituted-parent"),
        delegated_action(UNRELATED_GOVERNANCE_GRANT, governance_scope()),
        141,
    );
}

#[test]
fn an_expired_parent_cannot_activate_a_child() {
    let mut parent = GrantSpec::governance_view();
    parent.not_after = 0;
    assert_denied(
        delegation_world_with_parent_spec("delegation-expired-parent", parent),
        delegated_action(GRANT, governance_scope()),
        143,
    );
}

#[test]
fn every_governance_scope_axis_is_narrowed_by_the_effectful_transition() {
    let mut cases = Vec::new();
    let mut purpose = governance_scope();
    purpose.purpose = EstateCapabilityPurpose::LegalCompliance;
    cases.push(("purpose", purpose));
    let mut field = governance_scope();
    field.field = Some(RestrictedBankField::CustomerIdentity);
    cases.push(("field", field));
    let mut workflow = governance_scope();
    workflow.workflow_stage = EstateWorkflowStage::AuthorityReview;
    cases.push(("workflow", workflow));
    let mut action = governance_scope();
    action.operation = EstateCapabilityOperation::FreezeAccount;
    action.account = Some(ACCOUNT);
    action.field = None;
    cases.push(("action", action));

    for (ordinal, (axis, child)) in cases.into_iter().enumerate() {
        assert_denied(
            delegation_world(&format!("delegation-{axis}-widening")),
            delegated_action(GRANT, child),
            145 + u8::try_from(ordinal * 2).unwrap(),
        );
    }
}

#[test]
fn related_amount_and_validity_bounds_cannot_widen() {
    let mut freeze_child = governance_scope();
    freeze_child.operation = EstateCapabilityOperation::FreezeAccount;
    freeze_child.account = Some(OTHER_ACCOUNT);
    freeze_child.field = None;
    assert_denied(
        delegation_world_with_parent_spec("delegation-related-widening", GrantSpec::freeze()),
        delegated_action(GRANT, freeze_child),
        155,
    );

    let mut amount_child = governance_scope();
    amount_child.operation = EstateCapabilityOperation::DisburseEstate;
    amount_child.purpose = EstateCapabilityPurpose::EstateDisbursement;
    amount_child.account = Some(ACCOUNT);
    amount_child.field = None;
    amount_child.amount_ceiling = Some(Money::<USD>::from_minor(60_000).unwrap());
    assert_denied(
        delegation_world_with_parent_spec(
            "delegation-amount-widening",
            GrantSpec::disburse(50_000),
        ),
        delegated_action(GRANT, amount_child),
        157,
    );

    let mut validity_parent = GrantSpec::governance_view();
    validity_parent.not_before = 100;
    assert_denied(
        delegation_world_with_parent_spec("delegation-validity-widening", validity_parent),
        delegated_action(GRANT, governance_scope()),
        159,
    );
}

fn assert_denied(fixture: CapabilityFixture, action: EstateAction, seed: u8) {
    let denial = fixture
        .runtime
        .delegate_estate_capability_with_key(
            &fixture.authenticate(),
            action,
            &BankIdempotencyKey::new(format!("delegation-scope-{seed}")).unwrap(),
            &request_scope(),
        )
        .expect_err("a widened or substituted parent must mint no child authority");
    let BankEstateProgressionDenial::Authorization(denial) = denial else {
        panic!("scope narrowing must deny at Query delegation authorization: {denial:#?}");
    };
    assert!(
        matches!(
            denial.kind(),
            crate::BankAuthorizationDenialKind::CapabilityGrantMissing
                | crate::BankAuthorizationDenialKind::CapabilityAuthorizationMissing
                | crate::BankAuthorizationDenialKind::DelegationRejected
        ),
        "unexpected exact delegation denial: {:?}",
        denial.kind()
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
        .all(|capability| capability.id() != CHILD));
}

fn delegated_action(parent: CapabilityGrantId, scope: EstateCapabilityScope) -> EstateAction {
    EstateAction::DelegateCapability {
        estate: ESTATE,
        parent,
        child: EstateCapabilityDelegationRequest {
            id: CHILD,
            grantee: APPROVER,
            scope,
        },
    }
}

fn governance_scope() -> EstateCapabilityScope {
    EstateCapabilityScope {
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
    }
}
