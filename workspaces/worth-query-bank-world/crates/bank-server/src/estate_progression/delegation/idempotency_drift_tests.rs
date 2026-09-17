use bank_domain::{
    estate::{
        CapabilityValidity, DelegationLimit, EstateAction, EstateCapabilityDelegationRequest,
        EstateCapabilityOperation, EstateCapabilityPurpose, EstateCapabilityScope, EstateMoment,
        EstateWorkflowStage,
    },
    model::{Money, USD},
    proposals::BankIdempotencyKey,
};

use crate::estate_capability_admission::fixture::{
    delegation_world, delegation_world_with_parent_spec, request_scope, GrantSpec, ACCOUNT,
    APPROVER, BRANCH, ESTATE, GRANT, INSTITUTION, UNRELATED_GOVERNANCE_GRANT,
};
use crate::{BankCommitDenialKind, BankCommitDenialStage, BankMutationCommitOutcome};

#[test]
fn delegation_retry_rejects_each_changed_optional_scope_and_validity_field() {
    let fixture = delegation_world_with_parent_spec(
        "delegation-idempotency-scope-drift",
        GrantSpec::disburse(50_000),
    );
    let specialist = fixture.authenticate();
    let key = idempotency("delegate-scope-drift");
    let action = delegation_action(40_000, 0, u64::MAX - 1);
    let first = fixture
        .runtime
        .delegate_estate_capability_with_key(&specialist, action, &key, &request_scope())
        .expect("the first exact delegation should execute");
    assert!(matches!(first, BankMutationCommitOutcome::Committed(_)));

    for (axis, retry) in [
        ("amount ceiling", delegation_action(30_000, 0, u64::MAX - 1)),
        ("not-before", delegation_action(40_000, 1, u64::MAX - 1)),
        ("not-after", delegation_action(40_000, 0, u64::MAX - 2)),
    ] {
        assert_intent_drift(
            fixture.runtime.delegate_estate_capability_with_key(
                &specialist,
                retry,
                &key,
                &request_scope(),
            ),
            axis,
        );
    }
}

#[test]
fn revocation_retry_rejects_a_different_grant_target() {
    let fixture = delegation_world("revocation-idempotency-target-drift");
    let specialist = fixture.authenticate();
    let key = idempotency("revoke-target-drift");
    let first = fixture
        .runtime
        .revoke_estate_capability_with_key(
            &specialist,
            EstateAction::RevokeCapability {
                estate: ESTATE,
                grant: UNRELATED_GOVERNANCE_GRANT,
            },
            &key,
            &request_scope(),
        )
        .expect("the first exact revocation should execute");
    assert!(matches!(first, BankMutationCommitOutcome::Committed(_)));

    let drift = fixture.runtime.revoke_estate_capability_with_key(
        &specialist,
        EstateAction::RevokeCapability {
            estate: ESTATE,
            grant: GRANT,
        },
        &key,
        &request_scope(),
    );
    assert_intent_drift(drift, "grant target");
}

fn assert_intent_drift(
    outcome: Result<BankMutationCommitOutcome, crate::BankEstateProgressionDenial>,
    axis: &str,
) {
    assert!(
        matches!(
            &outcome,
            Ok(BankMutationCommitOutcome::Denied {
                kind: BankCommitDenialKind::IdempotencyIntentDrift,
                stage: BankCommitDenialStage::Idempotency,
            })
        ),
        "changing {axis} produced an unexpected retry result: {outcome:?}"
    );
}

fn delegation_action(amount: i64, not_before: u64, not_after: u64) -> EstateAction {
    EstateAction::DelegateCapability {
        estate: ESTATE,
        parent: GRANT,
        child: EstateCapabilityDelegationRequest {
            id: bank_domain::estate::CapabilityGrantId::new(451).unwrap(),
            grantee: APPROVER,
            scope: EstateCapabilityScope {
                account: Some(ACCOUNT),
                estate: ESTATE,
                institution: INSTITUTION,
                branch: BRANCH,
                operation: EstateCapabilityOperation::DisburseEstate,
                purpose: EstateCapabilityPurpose::EstateDisbursement,
                field: None,
                amount_ceiling: Some(Money::<USD>::from_minor(amount).unwrap()),
                validity: CapabilityValidity::new(
                    EstateMoment::from_epoch_seconds(not_before),
                    EstateMoment::from_epoch_seconds(not_after),
                )
                .unwrap(),
                delegation: DelegationLimit::generations(1),
                workflow_stage: EstateWorkflowStage::Administration,
            },
        },
    }
}

fn idempotency(value: &str) -> BankIdempotencyKey {
    BankIdempotencyKey::new(value).expect("the test idempotency key should be valid")
}
