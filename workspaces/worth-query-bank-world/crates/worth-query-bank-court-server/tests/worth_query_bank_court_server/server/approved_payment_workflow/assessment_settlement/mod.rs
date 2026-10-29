pub(crate) use worth_query_bank_testkit_server::facade::approved_payment_workflow::assessment_settlement::*;

#[test]
fn progressed_stages_settle_and_a_later_call_answers_the_same_demand() {
    let fixture = world();
    let approver = fixture.authenticate(APPROVER);
    let scope = request_scope();
    let workflow = fixture
        .world
        .runtime
        .approved_business_payment(&approver, &scope);
    let authority = authority(&fixture);
    let instance = awaiting_payment_assessment(&workflow, &authority);
    let mut demand = workflow
        .retain_payment_assessment(
            instance.clone(),
            authority.clone(),
            policy(1, 1),
            &key("assessment:settle"),
        )
        .expect("the payment assessment demand starts");
    let assessment = settle(&fixture, &mut demand);
    let repeated = match advance(&fixture, &mut demand) {
        BankApprovedPaymentAssessmentProgress::Settled(repeated) => repeated,
        BankApprovedPaymentAssessmentProgress::Progressed => {
            panic!("a settled demand cannot restart native progress");
        }
        BankApprovedPaymentAssessmentProgress::Pending => {
            panic!("a settled demand cannot become an external wait");
        }
    };
    assert_eq!(repeated.required(), assessment.required());
    require_completed(
        workflow
            .accept_assessment(instance, authority, &assessment, &key("assessment:accept"))
            .expect("the settled assessment is accepted"),
        "review/payment",
    );
}

#[test]
fn an_undelivered_readiness_reports_pending_and_a_later_call_resumes_the_same_demand() {
    let fixture = world();
    let approver = fixture.authenticate(APPROVER);
    let scope = request_scope();
    let workflow = fixture
        .world
        .runtime
        .approved_business_payment(&approver, &scope);
    let authority = authority(&fixture);
    let instance = awaiting_payment_assessment(&workflow, &authority);
    let mut demand = workflow
        .retain_payment_assessment(
            instance.clone(),
            authority.clone(),
            policy(1, 1),
            &key("assessment:settle"),
        )
        .expect("the payment assessment demand starts");
    fixture
        .world
        .runtime
        .delay_next_output_readiness_delivery_for_test();
    loop {
        match advance(&fixture, &mut demand) {
            BankApprovedPaymentAssessmentProgress::Progressed => continue,
            BankApprovedPaymentAssessmentProgress::Pending => break,
            BankApprovedPaymentAssessmentProgress::Settled(_) => {
                panic!("the delayed readiness must expose its actual Pending hold");
            }
        }
    }
    let assessment = settle(&fixture, &mut demand);
    require_completed(
        workflow
            .accept_assessment(instance, authority, &assessment, &key("assessment:accept"))
            .expect("the resumed assessment is accepted"),
        "review/payment",
    );
}

#[test]
fn notified_rounds_preserve_stage_yields_until_settlement() {
    let fixture = world();
    let approver = fixture.authenticate(APPROVER);
    let scope = request_scope();
    let workflow = fixture
        .world
        .runtime
        .approved_business_payment(&approver, &scope);
    let authority = authority(&fixture);
    let instance = awaiting_payment_assessment(&workflow, &authority);
    let mut demand = workflow
        .retain_payment_assessment(
            instance,
            authority,
            policy(1, 16),
            &key("assessment:settle"),
        )
        .expect("the payment assessment demand starts");
    fixture
        .world
        .runtime
        .delay_next_output_readiness_delivery_for_test();
    let settled = settle(&fixture, &mut demand);
    assert_eq!(settled.required().node_path(), "review/payment");
}
