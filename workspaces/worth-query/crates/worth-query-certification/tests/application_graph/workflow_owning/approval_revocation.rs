//! The original constructed owner cannot restore approval after its actual grant is revoked.
use super::{host, model, program::*};
use model::{operator_identity::*, retention_entry::*, schema::*, workflow::*};
use worth_query_host::facade::{
    application_entry::*, primary_graph::WorthQueryApplicationAttemptDenialKind,
};
#[test]
fn workflow_owning_approval_revoked_after_construction_spends_no_business_effect() {
    let host = host::install();
    let runtime = host.workflow.runtime();
    let (instance, proposal) =
        host.begin_with_proposal(reviewed_document_definition("applied"), 8, 0x9176c400);
    let scope = request_scope();
    let actor = authenticate_operator(runtime.installed_schema(), &scope);
    let request = runtime.request(&actor, &scope).on_branch(instance.branch());
    for offset in [3, 5] {
        let (observer, settlement) = host.assessment(instance.clone(), 0x9176c400 + offset);
        assert!(matches!(
            host.accept_assessment(instance.clone(), &settlement, 0x9176c401 + offset),
            WorkflowProgressOutcome::Completed(_)
        ));
        observer
            .close(&request)
            .unwrap_or_else(|_| panic!("actual observer release"));
    }
    assert!(matches!(
        host.advance(instance.clone(), 0x9176c407),
        WorkflowProgressOutcome::Completed(_)
    ));
    let approval = match host.advance(instance.clone(), 0x9176c408) {
        WorkflowProgressOutcome::AwaitingApproval(required) => required,
        _ => panic!("real approval requirement"),
    };
    host.approve(instance.clone(), &approval, &proposal, 0x9176c409);
    let required = match host.advance(instance.clone(), 0x9176c410) {
        WorkflowProgressOutcome::AwaitingOperation(required) => required,
        _ => panic!("issued operation slot"),
    };
    let owner = request
        .mutate(ReviewedSetRetentionIntent {
            input: SetRetentionInput {
                identity: DOCUMENT_IDENTITY.into(),
                retention_days: 8,
            },
        })
        .without_source()
        .idempotency_owned(0x9176c411_u64)
        .for_workflow_operation(&host.workflow, &required)
        .unwrap_or_else(|_| panic!("owner BEFORE revocation"))
        .retain_operation::<WorkflowOwnerProgram, ReviewedAction>(host.workflow.program_runtime())
        .unwrap_or_else(|_| panic!("declared workflow owner"));
    let before = model::readback::read_retention(runtime, instance.branch());
    let change = request
        .mutate(WorkflowGrantStatusIntent {
            input: WorkflowGrantStatusInput {
                grant_identity: "workflow-approval-grant".into(),
                status: "revoked".into(),
            },
        })
        .without_source()
        .idempotency_owned(0x9176c412_u64)
        .retain_operation::<WorkflowOwnerProgram, GrantStatusAction>(
            host.workflow.program_runtime(),
        )
        .unwrap_or_else(|_| panic!("real grant domain action"));
    let WorthQuerySourceOnlyAdvance::Completed(changed) = change.advance(&request) else {
        panic!("revocation publishes before effect")
    };
    changed
        .close(&request)
        .unwrap_or_else(|_| panic!("grant change cleanup"));
    assert_eq!(
        model::host::read_owner_approval_grant(runtime, &host.projection),
        "revoked"
    );
    let fresh_scope = request_scope();
    let fresh_actor = authenticate_operator(runtime.installed_schema(), &fresh_scope);
    let fresh = runtime
        .request(&fresh_actor, &fresh_scope)
        .on_branch(instance.branch());
    let terminal = match owner.advance(&fresh) {
        WorthQueryWorkflowOperationAdvance::Terminal(terminal) => terminal,
        _ => panic!("revoked approval must stop original owner"),
    };
    assert!(
        matches!(terminal.denial(),Some(WorthQuerySourceOnlyDenial::Mutation(WorthQueryApplicationRequestMutationDenial::WorkflowTransitionCurrentness(cause))) if cause.kind()==WorthQueryApplicationAttemptDenialKind::WorkflowApprovalGrantUnavailable)
    );
    assert_eq!(
        model::readback::read_retention(runtime, instance.branch()),
        before
    );
    assert!(matches!(
        host.advance(instance.clone(), 0x9176c413),
        WorkflowProgressOutcome::AwaitingOperation(_)
    ));
    let repeat = fresh
        .mutate(ReviewedSetRetentionIntent {
            input: SetRetentionInput {
                identity: DOCUMENT_IDENTITY.into(),
                retention_days: 8,
            },
        })
        .without_source()
        .idempotency(&0x9176c414_u64)
        .for_workflow_operation(&host.workflow, &required)
        .unwrap()
        .advanced()
        .execute_in_program(
            host.workflow.program_runtime(),
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        );
    assert!(
        matches!(
            repeat,
            Err(WorthQueryApplicationRequestMutationDenial::WorkflowAuthoritySpent)
        ),
        "native post-take refusal must not restore the slot"
    );
    terminal
        .close(&fresh)
        .unwrap_or_else(|_| panic!("actual no-effect terminal cleanup"));
}
