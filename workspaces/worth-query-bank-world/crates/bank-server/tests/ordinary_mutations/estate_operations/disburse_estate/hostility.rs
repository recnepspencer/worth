use super::{assert_no_disbursement_effects, disburse, fixture::*, idempotency};
use bank_domain::{proposals::BankProposalDenial, schema::AccountStatus};
use bank_server::{
    BankEstateDisbursementProjectionDenial, BankEstateProgressionDenial, BankMutationCommitOutcome,
};

#[test]
fn exact_beneficiary_and_destination_joint_ownership_are_required() {
    for (ordinal, posture) in [
        BeneficiaryPosture::Missing,
        BeneficiaryPosture::WrongEstate,
        BeneficiaryPosture::JointOwnerMissing,
        BeneficiaryPosture::JointOwnerWrongAccount,
    ]
    .into_iter()
    .enumerate()
    {
        let fixture = hostile_world(
            &format!("estate-disbursement-beneficiary-{posture:?}"),
            DisbursementWorldSpec {
                beneficiary: posture,
                ..DisbursementWorldSpec::ready()
            },
        );
        let specialist = fixture.authenticate_actor();
        let denial = disburse(
            &fixture,
            &specialist,
            fixture.action(250),
            idempotency(61 + ordinal as u8),
        )
        .expect_err("missing or misdirected beneficiary truth must deny");
        match posture {
            BeneficiaryPosture::Missing | BeneficiaryPosture::WrongEstate => assert!(matches!(
                denial,
                BankEstateProgressionDenial::EstateDisbursementProjection(
                    BankEstateDisbursementProjectionDenial::EstateBeneficiaryRelationMissing
                )
            )),
            BeneficiaryPosture::JointOwnerMissing | BeneficiaryPosture::JointOwnerWrongAccount => {
                assert!(matches!(
                    denial,
                    BankEstateProgressionDenial::EstateDisbursementProjection(
                        BankEstateDisbursementProjectionDenial::EstateJointOwnerRelationMissing
                    )
                ))
            }
            BeneficiaryPosture::Ready => unreachable!(),
        }
        if ordinal == 0 {
            assert_no_disbursement_effects(&fixture);
        }
    }
}

#[test]
fn a_recognized_exact_executor_authority_is_required_but_not_artificially_unique() {
    for (ordinal, posture) in [
        ExecutorPosture::Missing,
        ExecutorPosture::Unrecognized,
        ExecutorPosture::WrongHolder,
    ]
    .into_iter()
    .enumerate()
    {
        let fixture = hostile_world(
            &format!("estate-disbursement-executor-{posture:?}"),
            DisbursementWorldSpec {
                executor: posture,
                ..DisbursementWorldSpec::ready()
            },
        );
        let specialist = fixture.authenticate_actor();
        let denial = disburse(
            &fixture,
            &specialist,
            fixture.action(250),
            idempotency(71 + ordinal as u8),
        )
        .expect_err("missing exact recognized executor authority must deny");
        assert!(matches!(
            denial,
            BankEstateProgressionDenial::EstateDisbursementProjection(
                BankEstateDisbursementProjectionDenial::RecognizedExecutorAuthorityMissing
            )
        ));
    }

    let fixture = hostile_world(
        "estate-disbursement-multiple-executors",
        DisbursementWorldSpec {
            executor: ExecutorPosture::MultipleLawful,
            ..DisbursementWorldSpec::ready()
        },
    );
    let specialist = fixture.authenticate_actor();
    let outcome = disburse(&fixture, &specialist, fixture.action(250), idempotency(74))
        .expect("multiple lawful recognized executor authorities must remain admissible");
    let BankMutationCommitOutcome::Committed(receipt) = outcome else {
        panic!("multiple lawful executors must commit: {outcome:?}");
    };
    assert_eq!(receipt.decision_fact_count(), Some(41));
}

#[test]
fn source_and_destination_must_both_be_open() {
    for (ordinal, source_status, destination_status, expected_account) in [
        (0, AccountStatus::Frozen, AccountStatus::Open, 0),
        (1, AccountStatus::Open, AccountStatus::Closed, 1),
    ] {
        let fixture = hostile_world(
            &format!("estate-disbursement-status-{ordinal}"),
            DisbursementWorldSpec {
                source_status,
                destination_status,
                ..DisbursementWorldSpec::ready()
            },
        );
        let specialist = fixture.authenticate_actor();
        let denial = disburse(
            &fixture,
            &specialist,
            fixture.action(250),
            idempotency(81 + ordinal),
        )
        .expect_err("a non-open movement endpoint must deny");
        let expected = if expected_account == 0 {
            fixture.source
        } else {
            fixture.destination
        };
        let status = if expected_account == 0 {
            source_status
        } else {
            destination_status
        };
        assert!(matches!(
            denial,
            BankEstateProgressionDenial::Proposal(BankProposalDenial::AccountStatus {
                account,
                status: observed,
            }) if account == expected && observed == status
        ));
    }
}

#[test]
fn beneficiary_and_executor_actors_deny_at_capability_composition() {
    for (ordinal, actor_conflict) in [ActorConflict::Beneficiary, ActorConflict::Executor]
        .into_iter()
        .enumerate()
    {
        let fixture = hostile_world(
            &format!("estate-disbursement-conflict-{actor_conflict:?}"),
            DisbursementWorldSpec {
                actor_conflict,
                ..DisbursementWorldSpec::ready()
            },
        );
        let specialist = fixture.authenticate_actor();
        let denial = disburse(
            &fixture,
            &specialist,
            fixture.action(250),
            idempotency(91 + ordinal as u8),
        )
        .expect_err("conflicted authority must deny before invariant projection");
        let BankEstateProgressionDenial::ApplicationEntry(
            worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial::Authorization(denial),
        ) = denial else {
            panic!("actor conflict must deny at Query authorization")
        };
        let kind = match actor_conflict {
            ActorConflict::Beneficiary => {
                worth_query_host::facade::primary_graph::WorthQueryOperationAuthorizationDenialKind::ConflictRuleMatched
            }
            ActorConflict::Executor => {
                worth_query_host::facade::primary_graph::WorthQueryOperationAuthorizationDenialKind::SeparationOfDutyRuleMatched
            }
            ActorConflict::None => unreachable!(),
        };
        assert_eq!(denial.kind(), kind);
        assert_eq!(denial.causes().len(), 1);
        assert_no_disbursement_effects(&fixture);
    }
}

#[test]
fn approved_emergency_graph_state_is_not_ordinary_disbursement_authority() {
    let fixture = hostile_world(
        "estate-disbursement-emergency-escalation",
        DisbursementWorldSpec {
            grant: GrantPosture::ApprovedEmergencyOnly,
            ..DisbursementWorldSpec::ready()
        },
    );
    let specialist = fixture.authenticate_actor();
    let denial = disburse(&fixture, &specialist, fixture.action(250), idempotency(101))
        .expect_err("an approved graph record does not replace ordinary command authority");
    assert!(matches!(
        denial,
        BankEstateProgressionDenial::ApplicationEntry(
            worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial::Authorization(_)
        )
    ));
}
