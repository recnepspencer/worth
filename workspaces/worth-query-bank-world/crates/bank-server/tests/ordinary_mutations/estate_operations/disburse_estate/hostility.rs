use super::{assert_no_disbursement_effects, disburse, fixture::*, idempotency};
use bank_domain::{proposals::BankProposalDenial, schema::AccountStatus};
use bank_server::BankEstateDisbursementProjectionDenial;
use worth_query_host::facade::primary_graph::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;

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
        let (_denial_source, denial) = {
            let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Refused { operation: original_source, denial: worth_query_host::facade::application_entry::WorthQuerySourceOnlyDenial::Mutation(worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial::Handler(worth_query_host::facade::primary_graph::MutationHandlerExecutionDenial::Handler(cause))) } = disburse(
            &fixture,
            &specialist,
            fixture.action(250),
            idempotency(61 + ordinal as u8),
        ) else { panic!("native pre-effect refusal") };
            (
                original_source,
                cause
                    .downcast::<bank_server::BankEstateDisbursementProjectionDenial>()
                    .expect("native projection cause"),
            )
        };
        match posture {
            BeneficiaryPosture::Missing | BeneficiaryPosture::WrongEstate => assert!(matches!(
                denial,
                BankEstateDisbursementProjectionDenial::EstateBeneficiaryRelationMissing
            )),
            BeneficiaryPosture::JointOwnerMissing | BeneficiaryPosture::JointOwnerWrongAccount => {
                assert!(matches!(
                    denial,
                    BankEstateDisbursementProjectionDenial::EstateJointOwnerRelationMissing
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
        let (_denial_source, denial) = {
            let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Refused { operation: original_source, denial: worth_query_host::facade::application_entry::WorthQuerySourceOnlyDenial::Mutation(worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial::Handler(worth_query_host::facade::primary_graph::MutationHandlerExecutionDenial::Handler(cause))) } = disburse(
            &fixture,
            &specialist,
            fixture.action(250),
            idempotency(71 + ordinal as u8),
        ) else { panic!("native pre-effect refusal") };
            (
                original_source,
                cause
                    .downcast::<bank_server::BankEstateDisbursementProjectionDenial>()
                    .expect("native projection cause"),
            )
        };
        assert!(matches!(
            denial,
            BankEstateDisbursementProjectionDenial::RecognizedExecutorAuthorityMissing
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
    let outcome = {
        let closing_actor = &specialist;
        let closing_scope = crate::support::request_scope();
        let closing_request = fixture.world.runtime.request(closing_actor, &closing_scope);
        let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Completed(
            done,
        ) = disburse(&fixture, &specialist, fixture.action(250), idempotency(74))
        else {
            panic!("native source completion")
        };
        assert!(matches!(
            done.evidence(),
            worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Fresh { .. }
        ));
        bank_server::BankCommitReceipt::from_closed_source(
            done.close_with_evidence(&closing_request)
                .unwrap_or_else(|(_, cause)| panic!("source close: {cause:?}")),
        )
    };
    let receipt = outcome;
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
        let denial = {
            let closing_actor = &specialist;
            let closing_scope = crate::support::request_scope();
            let closing_request = fixture.world.runtime.request(closing_actor, &closing_scope);
            let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Terminal(
                done,
            ) = disburse(
                &fixture,
                &specialist,
                fixture.action(250),
                idempotency(81 + ordinal),
            )
            else {
                panic!("native domain no-effect denial")
            };
            let Some(worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome::DomainDenied(bank_domain::schema::EstateMutationDenial::Proposal(cause))) = done.native_outcome() else { panic!("original proposal denial") };
            let cause = cause.clone();
            done.close(&closing_request)
                .unwrap_or_else(|(_, denial)| panic!("terminal close: {denial:?}"));
            cause
        };
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
            BankProposalDenial::AccountStatus {
                account,
                status: observed,
            } if account == expected && observed == status
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
        let (_denial_source, denial) = {
            let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Refused {
                operation: original_source,
                denial:
                    worth_query_host::facade::application_entry::WorthQuerySourceOnlyDenial::Mutation(
                        cause,
                    ),
            } = disburse(
                &fixture,
                &specialist,
                fixture.action(250),
                idempotency(91 + ordinal as u8),
            )
            else {
                panic!("native pre-effect refusal")
            };
            (original_source, cause)
        };
        let worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial::Authorization(denial) = denial else {
            panic!("actor conflict must deny at Query authorization")
        };
        let kind = match actor_conflict {
            ActorConflict::Beneficiary => {
                worth_query_host::facade::primary_graph::WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::ConflictRuleMatched)
            }
            ActorConflict::Executor => {
                worth_query_host::facade::primary_graph::WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::SeparationOfDutyRuleMatched)
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
    let (_denial_source, denial) = {
        let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Refused {
            operation: original_source,
            denial:
                worth_query_host::facade::application_entry::WorthQuerySourceOnlyDenial::Mutation(cause),
        } = disburse(&fixture, &specialist, fixture.action(250), idempotency(101))
        else {
            panic!("native pre-effect refusal")
        };
        (original_source, cause)
    };
    assert!(matches!(
        denial,
        worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial::Authorization(_)
    ));
}
