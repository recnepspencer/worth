pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::application_graph::workflow_approval::custody::*;

#[test]
fn native_writer_cannot_revise_a_published_approval_or_delete_its_evidence() {
    let (application, _, instance, proposal, required, _) = approval_journey("applied", 1_880);
    let approval = match approve_instance(
        &application,
        instance.clone(),
        &required,
        &proposal,
        WorkflowApprovalDecision::Approve,
        1_890,
    ) {
        Ok(WorkflowProgressOutcome::Completed(performed)) => performed
            .approval()
            .expect("approval publication must expose its durable fact")
            .entity(),
        other => panic!("expected a published approval, got {other:?}"),
    };
    let error = application
        .with_host_advancement(|host_phase| {
            application
                .runtime()
                .attempt_workflow_approval_field_update_for_test(&host_phase, &instance, approval)
                .expect("native control enters its admitted phase")
                .expect_err("a native writer cannot revise an issued decision")
        })
        .expect("installed host call opens");
    assert_fact_custody_denial(error);
    let error = application
        .with_host_advancement(|host_phase| {
            application
                .runtime()
                .attempt_workflow_approval_evidence_delete_for_test(
                    &host_phase,
                    &instance,
                    approval,
                )
                .expect("native control enters its admitted phase")
                .expect_err("a native writer cannot delete issued approval evidence")
        })
        .expect("installed host call opens");
    assert_fact_custody_denial(error);
}
