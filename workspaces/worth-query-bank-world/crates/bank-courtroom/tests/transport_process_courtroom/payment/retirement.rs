//! Bank P2 adoption after the original payment workflow has settled.

use bank_domain::schema::ApprovePayment;
use bank_server::{BankApplicationP2, BankAuthenticatedPrincipal, BankIdentityRuntime};
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;
use worth_query_host::facade::application_entry::{
    WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
    WorthQueryApplicationRequestMutationDenial, WorthQueryBranchAdoptionPublicationOutcome,
};
use worth_query_host::facade::product::WorthQueryProductBranch;

use super::key;

pub(super) fn adopt_p2_after_completed_payment(
    runtime: &BankIdentityRuntime,
    approver: &BankAuthenticatedPrincipal,
    scope: &WorthQueryRequestScope,
    branch: WorthQueryProductBranch,
    authority: ApprovePayment,
) {
    let workflow = runtime.approved_business_payment(approver, scope);
    let definition = match workflow
        .publish_definition(
            authority.clone(),
            WorkflowDefinitionExpectedPredecessor::Absent,
            &key("approved-payment:definition"),
        )
        .expect("definition replay remains inspectable")
    {
        WorkflowDefinitionPublicationOutcome::Published(published) => {
            assert!(published.replayed());
            published.definition().clone()
        }
        other => panic!("expected original definition replay, got {other:?}"),
    };
    let target = runtime
        .supported_program_revision::<BankApplicationP2>()
        .expect("Bank P2 is explicitly rostered");
    let prepared = runtime
        .prepare_branch_program_adoption_retiring_definition::<BankApplicationP2>(
            approver,
            scope,
            branch,
            &definition,
            4_096,
        )
        .expect("settled payment operation permits program adoption");
    assert!(matches!(
        prepared.publish(),
        WorthQueryBranchAdoptionPublicationOutcome::Performed(_)
    ));
    assert_eq!(
        runtime
            .inspect_branch_program(approver, scope, branch)
            .expect("the selected program remains inspectable")
            .revision(),
        &target
    );
    let ordinary = runtime
        .request(approver, scope)
        .on_branch(branch)
        .mutate(authority)
        .idempotency(&key("p2:ordinary-payment-after-completion"))
        .execute();
    assert!(matches!(
        ordinary,
        Err(WorthQueryApplicationRequestMutationDenial::RequiresWorkflowTransition)
    ));
}
