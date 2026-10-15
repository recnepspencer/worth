//! Genuine proposal/approval/effect journey in the separate document schema.
//! Uses installed authentication, grants, both native assessment outputs and the
//! existing original-source owner. No actor value is supplied by the caller.
use super::super::document_retention_model::{
    operator_identity::{authenticate_reviewer, authenticate_ungranted_reviewer},
    workflow::{WorkflowAdvanceInput, WorkflowApprovalIntent},
};
use super::*;
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationRequestMutationDenial,
    primary_graph::{
        WorthQueryOperationAuthorizationDenialKind as Denial,
        WorthQueryOperationAuthorizationRefusalKind as Refusal,
    },
};

#[test]
fn workflow_actor_contract_multiactor_requires_positive_original_proposer_and_current_grant() {
    let (application, _, instance, proposal, required, _) =
        approval_journey("actor-review", 71_000);
    let runtime = application.runtime();
    let scope = request_scope();
    let author = authenticate_operator(runtime.installed_schema(), &scope);
    let ungranted = authenticate_ungranted_reviewer(runtime.installed_schema(), &scope);
    for (principal, expected, key) in [
        (
            &author,
            Refusal::WorkflowActorComparisonMismatch,
            71_010_u64,
        ),
        (&ungranted, Refusal::CapabilityGrantMissing, 71_011_u64),
    ] {
        let outcome = runtime
            .request(principal, &scope)
            .mutate(WorkflowApprovalIntent {
                input: WorkflowAdvanceInput {
                    document_identity: DOCUMENT_IDENTITY.to_owned(),
                },
            })
            .without_source()
            .idempotency(&key)
            .prepare_workflow_approval(
                &application,
                instance.clone(),
                &required,
                &proposal,
                WorkflowApprovalDecision::Approve,
            );
        match outcome {
            Err(WorthQueryWorkflowAdvancePreparationDenial::RequestAdmission(
                WorthQueryApplicationRequestMutationDenial::Authorization(cause),
            )) => assert_eq!(cause.kind(), Denial::Refused(expected)),
            _ => panic!("otherwise genuine approval must reach native actor/grant refusal"),
        }
        assert_eq!(
            read_retention(runtime, runtime.current_world()),
            SEED_RETENTION
        );
    }
    let performed = match approve_instance(
        &application,
        instance.clone(),
        &required,
        &proposal,
        WorkflowApprovalDecision::Approve,
        71_012,
    )
    .unwrap()
    {
        WorkflowProgressOutcome::Completed(performed) => performed,
        other => panic!("distinct currently admitted reviewer must approve: {other:?}"),
    };
    use worth_query_decl::facade::application_capability::ApplicationWorkflowRecordedRole as Role;
    let proposer = match propose_instance(&application, instance.clone(), 71_002).unwrap() {
        WorkflowProposalOutcome::Published(replay) => {
            assert!(replay.replayed());
            replay
                .receipt()
                .committed_changes()
                .workflow_actor_fields_for_test(proposal.entity_id(), Role::Proposer)
                .unwrap()
                .expect("original proposal has its admitted proposer")
        }
        other => panic!("original proposal replays without rediscovery: {other:?}"),
    };
    let recorded_advancer = performed
        .receipt()
        .committed_changes()
        .workflow_actor_fields_for_test(performed.transition(), Role::Advancer)
        .unwrap()
        .expect("the approval command has an original advancer");
    assert!(
        performed
            .receipt()
            .committed_changes()
            .workflow_actor_fields_for_test(performed.transition(), Role::Executor)
            .unwrap()
            .is_none(),
        "a control transition never fabricates a business executor"
    );
    let approval = performed.approval().expect("native approval was recorded");
    let principal = performed.receipt().principal_scope().principal();
    assert_eq!(
        approval.approver(),
        worth_relational::facade::identity::EntityId::new(
            worth_relational::facade::identity::PartitionId::new(principal.partition_id()),
            principal.local_slot(),
            principal.generation()
        )
    );
    assert_eq!(approval.proposal(), proposal.entity_id());
    assert_eq!(recorded_advancer, approval.approver());
    assert_ne!(
        proposer,
        approval.approver(),
        "the real native principals are distinct"
    );
    assert_eq!(
        performed
            .receipt()
            .committed_changes()
            .workflow_actor_fields_for_test(approval.entity(), Role::Approver)
            .unwrap(),
        Some(approval.approver())
    );

    super::instance_migration::perform_approved_effect(&application, &instance, 71_020);
    assert_eq!(read_retention(runtime, runtime.current_world()), 8);
}

#[test]
fn workflow_actor_contract_foreign_instance_proposal_cannot_replace_original_source() {
    let (application, definition, instance, proposal, required, _) =
        approval_journey("actor-source", 72_000);
    let other_instance =
        super::instance_migration::started(start_instance(&application, definition, 72_100))
            .instance()
            .clone();
    let (other_proposal, _, _) =
        super::journey::approval_requirement(&application, other_instance, 72_110);
    assert_ne!(proposal.entity_id(), other_proposal.entity_id());
    let runtime = application.runtime();
    let scope = request_scope();
    let reviewer = authenticate_reviewer(runtime.installed_schema(), &scope);
    let outcome = runtime
        .request(&reviewer, &scope)
        .mutate(WorkflowApprovalIntent {
            input: WorkflowAdvanceInput {
                document_identity: DOCUMENT_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&72_150_u64)
        .prepare_workflow_approval(
            &application,
            instance.clone(),
            &required,
            &other_proposal,
            WorkflowApprovalDecision::Approve,
        );
    match outcome {
        Err(WorthQueryWorkflowAdvancePreparationDenial::RequestAdmission(
            WorthQueryApplicationRequestMutationDenial::Authorization(cause),
        )) => {
            assert_eq!(
                cause.kind(),
                Denial::Refused(Refusal::WorkflowActorSourceInvalid)
            );
            assert!(cause.workflow_actor_source_denial().is_some());
        }
        _ => panic!("foreign original occurrence must refuse before signing"),
    }
    assert!(matches!(
        approve_instance(
            &application,
            instance,
            &required,
            &proposal,
            WorkflowApprovalDecision::Approve,
            72_151
        ),
        Ok(WorkflowProgressOutcome::Completed(_))
    ));
    assert_eq!(
        read_retention(runtime, runtime.current_world()),
        SEED_RETENTION
    );
}

#[test]
fn workflow_actor_contract_proposal_replay_preserves_original_execution_principal() {
    use super::super::document_retention_model::workflow::{
        WorkflowDefinitionAuthoringInput, WorkflowDefinitionAuthoringIntent,
    };
    let (application, _, instance, proposal, _, _) = approval_journey("actor-replay", 73_000);
    match propose_instance(&application, instance.clone(), 73_002).unwrap() {
        WorkflowProposalOutcome::Published(replay) => {
            assert!(replay.replayed());
            assert_eq!(replay.proposal(), &proposal);
        }
        other => panic!("original admitted proposer must replay its exact occurrence: {other:?}"),
    }
    let runtime = application.runtime();
    let scope = request_scope();
    let reviewer = authenticate_reviewer(runtime.installed_schema(), &scope);
    let outcome = runtime
        .request(&reviewer, &scope)
        .mutate(WorkflowDefinitionAuthoringIntent {
            input: WorkflowDefinitionAuthoringInput {
                identity: DOCUMENT_IDENTITY.to_owned(),
                retention_days: 8,
            },
        })
        .without_source()
        .idempotency(&73_002_u64)
        .prepare_workflow_proposal(&application, instance)
        .unwrap()
        .execute();
    match outcome {
        WorkflowProposalOutcome::Application(WorthQueryApplicationUncommitted::Denied(cause)) => {
            assert_eq!(
                cause.kind(),
                WorthQueryApplicationCommitDenialKind::IdempotencyIntentDrift
            )
        }
        other => panic!(
            "freshly authorized different proposer must not inherit original intent: {other:?}"
        ),
    }
    assert_eq!(
        read_retention(runtime, runtime.current_world()),
        SEED_RETENTION
    );
}

mod generation;

mod allocation;
