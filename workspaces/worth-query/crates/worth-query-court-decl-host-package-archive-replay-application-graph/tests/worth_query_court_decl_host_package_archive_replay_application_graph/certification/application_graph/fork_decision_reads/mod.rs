pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::application_graph::fork_decision_reads::*;

#[test]
fn a_fork_does_not_observe_a_relation_its_parent_removed_after_the_fork() {
    let application = publish_workflow_on_first_program();
    let main = application.runtime().current_world();
    assert!(matches!(
        link_review_requirement_on(&application, main, 93_000).expect("main links"),
        WorthQueryApplicationMutationOutcome::Committed { .. },
    ));
    let fork = fork_of(&application, main);
    assert!(matches!(
        unlink_review_requirement_on(&application, main, 93_001).expect("main unlinks"),
        WorthQueryApplicationMutationOutcome::Committed { .. },
    ));
    advance_past_sibling(&application, fork, 93_002);

    let outcome = unlink_review_requirement_on(&application, fork, 93_003).expect("fork decides");
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "the fork still holds the relation it was forked with: {outcome:?}",
    );
}

#[test]
fn a_fork_does_not_observe_a_relation_its_parent_added_after_the_fork() {
    let application = publish_workflow_on_first_program();
    let main = application.runtime().current_world();
    let fork = fork_of(&application, main);
    assert!(matches!(
        link_review_requirement_on(&application, main, 93_100).expect("main links"),
        WorthQueryApplicationMutationOutcome::Committed { .. },
    ));
    advance_past_sibling(&application, fork, 93_101);

    let outcome = unlink_review_requirement_on(&application, fork, 93_102).expect("fork decides");
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::DomainDenied(
                ReviewRequirementDenial::RelationMismatch
            ),
        ),
        "the fork never held the parent's later relation: {outcome:?}",
    );
}

#[test]
fn a_parent_does_not_observe_a_relation_its_fork_added() {
    let application = publish_workflow_on_first_program();
    let main = application.runtime().current_world();
    let fork = fork_of(&application, main);
    assert!(matches!(
        link_review_requirement_on(&application, fork, 93_200).expect("fork links"),
        WorthQueryApplicationMutationOutcome::Committed { .. },
    ));
    advance_past_sibling(&application, main, 93_201);

    let outcome = unlink_review_requirement_on(&application, main, 93_202).expect("main decides");
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::DomainDenied(
                ReviewRequirementDenial::RelationMismatch
            ),
        ),
        "the parent never held its fork's relation: {outcome:?}",
    );
}
