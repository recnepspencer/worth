use bank_domain::estate::{
    CapabilityGrantId, CapabilityGrantStatus, CapabilityValidity, DelegationLimit, EstateAction,
    EstateCapabilityDelegationRequest, EstateCapabilityOperation, EstateCapabilityPurpose,
    EstateCapabilityScope, EstateMoment, EstateWorkflowStage, RestrictedBankField,
};
use bank_domain::model::BankPrincipalId;
use bank_domain::queries::EstateGovernanceQuery;
use bank_domain::reads::{EstateCapabilityContext, EstateGovernanceContext};
use worth_query_host::facade::primary_graph::WorthQueryApplicationIdempotencyBinding;
use worth_query_host::facade::publication::domain_computation::WorthQueryPublishedApplicationResult;

use super::fixture::{
    delegation_world, delegation_world_without_command, request_scope, CapabilityFixture, APPROVER,
    BRANCH, ESTATE, GRANT, INSTITUTION, REVIEWER, UNRELATED_GOVERNANCE_GRANT,
};
use crate::{
    BankApplicationQueryDenial, BankAuthenticatedPrincipal, BankCommitDenialKind,
    BankCommitDenialStage, BankEstateProgressionDenial, BankMutationCommitOutcome,
    BankReadControls,
};

const CHILD: CapabilityGrantId = CapabilityGrantId::new(303).unwrap();
const DRIFTED_CHILD: CapabilityGrantId = CapabilityGrantId::new(304).unwrap();
const GRANDCHILD: CapabilityGrantId = CapabilityGrantId::new(305).unwrap();
const POST_CUTOFF_CHILD: CapabilityGrantId = CapabilityGrantId::new(306).unwrap();
const MISSING_PARENT: CapabilityGrantId = CapabilityGrantId::new(307).unwrap();
const MISSING_PARENT_CHILD: CapabilityGrantId = CapabilityGrantId::new(308).unwrap();

type GovernanceResult =
    WorthQueryPublishedApplicationResult<EstateGovernanceQuery, EstateGovernanceContext>;

#[test]
fn public_delegation_creates_the_exact_narrowed_child_and_retries_idempotently() {
    let fixture = delegation_world("capability-delegation");
    let principal = fixture.authenticate();
    let action = delegated_action(DelegationLimit::generations(1));
    let idempotency = WorthQueryApplicationIdempotencyBinding::new([91; 32], [92; 32]);

    let first = fixture
        .runtime
        .delegate_estate_capability(&principal, action, idempotency, &request_scope())
        .expect("Query must activate the exact narrowed child");
    let BankMutationCommitOutcome::Committed(receipt) = &first else {
        panic!("unexpected delegation outcome: {first:?}");
    };
    assert_eq!(receipt.decision_fact_count(), Some(5));
    let canonical = receipt.canonical_work();
    assert_eq!(canonical.admission().basis_preparations(), 0);
    assert_eq!(canonical.admission().digest_derivations(), 0);
    assert_eq!(canonical.admission().canonical_entries(), 0);
    assert_eq!(canonical.execution().basis_preparations(), 1);
    assert_eq!(canonical.execution().digest_derivations(), 1);
    assert_eq!(canonical.execution().canonical_entries(), 61);

    let retry = fixture
        .runtime
        .delegate_estate_capability(&principal, action, idempotency, &request_scope())
        .expect("the exact delegation retry must recover the first commit");
    assert!(matches!(
        retry,
        BankMutationCommitOutcome::AlreadyCommitted(_)
    ));

    let drift = fixture
        .runtime
        .delegate_estate_capability(
            &principal,
            delegated_action_for(DRIFTED_CHILD, DelegationLimit::generations(1)),
            idempotency,
            &request_scope(),
        )
        .expect("proposal drift is a typed commit outcome");
    assert!(matches!(
        drift,
        BankMutationCommitOutcome::Denied {
            kind: BankCommitDenialKind::IdempotencyIntentDrift,
            stage: BankCommitDenialStage::Idempotency,
        }
    ));

    let approver = fixture.authenticate_approver();
    let result = fixture
        .runtime
        .query(crate::queries::estate_governance_context(ESTATE))
        .as_principal(&approver)
        .controls(BankReadControls::current(request_scope(), 1, 20_000).unwrap())
        .execute()
        .expect("the delegated administration view must govern public readback");
    let child = result.rows()[0]
        .capabilities()
        .iter()
        .find(|capability| capability.id() == CHILD)
        .expect("the committed child must be publicly observable");
    assert_eq!(child.parent(), Some(GRANT));
    assert_eq!(child.grantor(), super::fixture::SPECIALIST);
    assert_eq!(child.grantee(), APPROVER);
    assert_eq!(child.delegation().remaining(), 1);
    assert_eq!(child.field(), Some(RestrictedBankField::GovernanceMetadata));
}

#[test]
fn equal_or_wider_delegation_limit_is_denied_before_child_creation() {
    let fixture = delegation_world("capability-delegation-widening");
    let principal = fixture.authenticate();
    let denial = fixture
        .runtime
        .delegate_estate_capability(
            &principal,
            delegated_action(DelegationLimit::generations(2)),
            WorthQueryApplicationIdempotencyBinding::new([93; 32], [94; 32]),
            &request_scope(),
        )
        .expect_err("a child must strictly reduce its downstream delegation limit");
    assert!(matches!(
        denial,
        crate::estate_progression::BankEstateProgressionDenial::Authorization(_)
    ));
}

#[test]
fn revoking_the_exact_root_immediately_cuts_active_children_and_grandchildren() {
    let fixture = delegation_world("capability-delegation-descendant-cutoff");
    let specialist = fixture.authenticate();
    let approver = fixture.authenticate_approver();
    let reviewer = fixture.authenticate_reviewer();
    let executor = fixture.authenticate_executor();

    assert_committed(
        delegate(
            &fixture,
            &specialist,
            delegated_action(DelegationLimit::generations(1)),
            idempotency(101),
        )
        .expect("the root must activate its exact narrowed child"),
    );
    assert_committed(
        delegate(
            &fixture,
            &approver,
            delegated_action_from(CHILD, GRANDCHILD, REVIEWER, DelegationLimit::generations(0)),
            idempotency(103),
        )
        .expect("the child must activate its exact narrowed grandchild"),
    );

    governance_readback(&fixture, &specialist);
    governance_readback(&fixture, &approver);
    governance_readback(&fixture, &reviewer);

    let revoked = fixture
        .runtime
        .revoke_estate_capability(
            &specialist,
            EstateAction::RevokeCapability {
                estate: ESTATE,
                grant: GRANT,
            },
            idempotency(105),
            &request_scope(),
        )
        .expect("the exact active root must revoke through the public Bank command");
    assert_committed(revoked);

    assert_governance_denied(&fixture, &specialist);
    assert_governance_denied(&fixture, &approver);
    assert_governance_denied(&fixture, &reviewer);

    let readback = governance_readback(&fixture, &executor);
    assert_eq!(
        capability(&readback, GRANT).status(),
        CapabilityGrantStatus::Revoked
    );
    assert_eq!(
        capability(&readback, CHILD).status(),
        CapabilityGrantStatus::Active
    );
    assert_eq!(
        capability(&readback, GRANDCHILD).status(),
        CapabilityGrantStatus::Active,
        "descendants may retain Active storage status but revoked lineage must make them unusable"
    );
    assert_eq!(
        capability(&readback, UNRELATED_GOVERNANCE_GRANT).status(),
        CapabilityGrantStatus::Active
    );

    let denial = delegate(
        &fixture,
        &specialist,
        delegated_action_from(
            GRANT,
            POST_CUTOFF_CHILD,
            APPROVER,
            DelegationLimit::generations(1),
        ),
        idempotency(107),
    )
    .expect_err("a revoked parent must not activate another child");
    assert!(matches!(
        denial,
        BankEstateProgressionDenial::Authorization(_)
    ));
    assert!(
        capability_if_present(&governance_readback(&fixture, &executor), POST_CUTOFF_CHILD)
            .is_none()
    );
}

#[test]
fn delegation_denies_missing_command_authority_and_missing_parent_independently() {
    let without_command = delegation_world_without_command("delegation-missing-command");
    let specialist = without_command.authenticate();
    let denial = delegate(
        &without_command,
        &specialist,
        delegated_action(DelegationLimit::generations(1)),
        idempotency(109),
    )
    .expect_err("a valid parent cannot substitute for delegate-command authority");
    assert!(matches!(
        denial,
        BankEstateProgressionDenial::Authorization(_)
    ));
    assert!(capability_if_present(
        &governance_readback(&without_command, &without_command.authenticate_executor()),
        CHILD,
    )
    .is_none());

    let missing_parent = delegation_world("delegation-missing-parent");
    let denial = delegate(
        &missing_parent,
        &missing_parent.authenticate(),
        delegated_action_from(
            MISSING_PARENT,
            MISSING_PARENT_CHILD,
            APPROVER,
            DelegationLimit::generations(1),
        ),
        idempotency(111),
    )
    .expect_err("delegate-command authority cannot substitute for a real active parent");
    assert!(matches!(
        denial,
        BankEstateProgressionDenial::Authorization(_)
    ));
    assert!(capability_if_present(
        &governance_readback(&missing_parent, &missing_parent.authenticate_executor()),
        MISSING_PARENT_CHILD,
    )
    .is_none());
}

fn delegated_action(delegation: DelegationLimit) -> EstateAction {
    delegated_action_for(CHILD, delegation)
}

fn delegated_action_for(child_id: CapabilityGrantId, delegation: DelegationLimit) -> EstateAction {
    delegated_action_from(GRANT, child_id, APPROVER, delegation)
}

fn delegated_action_from(
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

fn delegate(
    fixture: &CapabilityFixture,
    principal: &BankAuthenticatedPrincipal,
    action: EstateAction,
    idempotency: WorthQueryApplicationIdempotencyBinding,
) -> Result<BankMutationCommitOutcome, BankEstateProgressionDenial> {
    fixture
        .runtime
        .delegate_estate_capability(principal, action, idempotency, &request_scope())
}

fn governance_readback(
    fixture: &CapabilityFixture,
    principal: &BankAuthenticatedPrincipal,
) -> GovernanceResult {
    fixture
        .runtime
        .query(crate::queries::estate_governance_context(ESTATE))
        .as_principal(principal)
        .controls(BankReadControls::current(request_scope(), 1, 20_000).unwrap())
        .execute()
        .expect("the principal's current governance grant must admit public readback")
}

fn assert_governance_denied(fixture: &CapabilityFixture, principal: &BankAuthenticatedPrincipal) {
    let denial = match fixture
        .runtime
        .query(crate::queries::estate_governance_context(ESTATE))
        .as_principal(principal)
        .controls(BankReadControls::current(request_scope(), 1, 20_000).unwrap())
        .execute()
    {
        Ok(_) => panic!("revoked lineage must deny ordinary public Query admission"),
        Err(denial) => denial,
    };
    let BankApplicationQueryDenial::CapabilityAdmission(denial) = denial else {
        panic!("descendant cutoff must fail at capability admission: {denial:#?}")
    };
    assert!(matches!(
        denial.kind(),
        crate::BankAuthorizationDenialKind::CapabilityAuthorizationMissing
            | crate::BankAuthorizationDenialKind::DelegationRejected
    ));
}

fn capability(result: &GovernanceResult, grant: CapabilityGrantId) -> &EstateCapabilityContext {
    capability_if_present(result, grant).expect("the exact capability must exist in readback")
}

fn capability_if_present(
    result: &GovernanceResult,
    grant: CapabilityGrantId,
) -> Option<&EstateCapabilityContext> {
    result.rows()[0]
        .capabilities()
        .iter()
        .find(|capability| capability.id() == grant)
}

fn assert_committed(outcome: BankMutationCommitOutcome) {
    assert!(
        matches!(outcome, BankMutationCommitOutcome::Committed(_)),
        "the public mutation must authoritatively commit: {outcome:?}"
    );
}

fn idempotency(seed: u8) -> WorthQueryApplicationIdempotencyBinding {
    WorthQueryApplicationIdempotencyBinding::new([seed; 32], [seed + 1; 32])
}
