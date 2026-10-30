pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::application_graph::adoption::live_outputs::*;

#[test]
fn a_clean_output_settled_before_adoption_answers_under_the_adopted_program() {
    let (mut application, instance) = awaiting_assessment(98_000);
    let produced = settle_assessment(&application, instance.clone(), 98_010);
    assert_eq!(production(&produced), PRODUCED);
    let source_commit = produced
        .settlement()
        .application_commit_receipt()
        .expect("P0's producer committed the output");

    adopt_second_program(&mut application);

    // Adoption touched none of the output's facts, so its row is clean: the
    // demand under P1 is answered by the commit P0's producer performed, and
    // P1's producer is not contacted.
    let answered = settle_assessment(&application, instance.clone(), 98_020);
    assert_eq!(production(&answered), HELD);
    assert_eq!(
        answered.settlement().application_commit_receipt(),
        Some(source_commit),
        "a clean row answers with the output it holds"
    );
    assert!(matches!(
        accept_assessment(&application, instance, &answered, 98_030),
        Ok(WorkflowProgressOutcome::Completed(_))
    ));
}

#[test]
fn a_changed_source_is_produced_by_the_adopted_program() {
    let (mut application, instance) = awaiting_assessment(98_100);
    let produced = settle_assessment(&application, instance.clone(), 98_110);
    assert_eq!(production(&produced), PRODUCED);

    adopt_second_program(&mut application);

    // A source only P1's rule admits: the output over it is P1's alone.
    assert_eq!(
        settle(set_retention(
            application.program_runtime(),
            instance.branch(),
            P1_ONLY_RETENTION,
            98_120,
        )),
        RetentionVerdict::Performed(P1_ONLY_RETENTION)
    );
    let adopted = settle_assessment(&application, instance.clone(), 98_130);
    assert_eq!(production(&adopted), PRODUCED);
    let adopted_commit = adopted
        .settlement()
        .application_commit_receipt()
        .expect("P1's producer committed the output");
    assert_ne!(
        Some(adopted_commit),
        produced.settlement().application_commit_receipt()
    );
    assert_eq!(
        read_retention(application.runtime(), instance.branch()),
        P1_ONLY_RETENTION
    );

    // P1's own output is then held like any other.
    let held = settle_assessment(&application, instance.clone(), 98_140);
    assert_eq!(production(&held), HELD);
    assert_eq!(
        held.settlement().application_commit_receipt(),
        Some(adopted_commit)
    );
    // The output P0 settled is not this source's: the instance refuses it.
    assert!(accept_assessment(&application, instance.clone(), &produced, 98_150).is_err());
    assert!(matches!(
        accept_assessment(&application, instance, &held, 98_160),
        Ok(WorkflowProgressOutcome::Completed(_))
    ));
}
