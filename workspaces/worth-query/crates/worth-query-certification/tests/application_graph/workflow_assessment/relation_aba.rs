//! A newly required review cannot reuse evidence across native relation ABA.

use worth_query_host::facade::application_entry::{
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestExt,
};

use super::super::bounded_dimension_model::{
    dimension_entry::{PartDimensionConditionQueryBinding, PartDimensionConditionRead},
    operator_identity::{authenticate_operator, request_scope},
    workflow::{
        conditionally_required_related_assessment_definition, link_review_requirement,
        unlink_review_requirement, WorkflowAdvanceInput, WorkflowAdvanceIntent,
    },
};
use super::*;

#[test]
fn newly_required_related_review_stales_after_relation_away_and_back() {
    let application = publish_workflow_on_first_program();
    assert_eq!(
        settle(set_dimension(
            application.program_runtime(),
            application.program_runtime().current_world(),
            0,
            760,
        )),
        DimensionVerdict::Performed(0),
    );
    let definition = match publish_definition(
        &application,
        conditionally_required_related_assessment_definition(),
        WorkflowDefinitionExpectedPredecessor::Absent,
        761,
    )
    .expect("conditional definition prepares")
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => {
            performed.definition().clone()
        }
        other => panic!("expected published definition: {other:?}"),
    };
    let instance = match start_instance(&application, definition, 762)
        .expect("conditional instance prepares")
    {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("expected started instance: {other:?}"),
    };
    propose_instance(&application, instance.clone(), 763).expect("proposal publishes");
    let condition =
        match advance_instance(&application, instance.clone(), 764).expect("condition prepares") {
            WorkflowProgressOutcome::AwaitingCondition(required) => required,
            other => panic!("expected condition: {other:?}"),
        };
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let result = runtime
        .request(&principal, &scope)
        .query(PartDimensionConditionRead {
            identity: PART_IDENTITY.to_owned(),
        })
        .execute()
        .expect("condition query reads the source");
    assert_eq!(result.rows(), &[false]);
    assert!(matches!(
        runtime
            .request(&principal, &scope)
            .mutate(WorkflowAdvanceIntent {
                input: WorkflowAdvanceInput {
                    part_identity: PART_IDENTITY.to_owned(),
                },
            })
            .without_source()
            .idempotency(&765_u64)
            .prepare_workflow_advance(&application, instance.clone())
            .expect("condition acceptance prepares")
            .accept_condition::<PartDimensionConditionQueryBinding>(&condition, result)
            .expect("false condition settles"),
        WorkflowProgressOutcome::Completed(_),
    ));
    for (demand, acceptance, path) in [(766, 767, "checks/first"), (768, 769, "checks/second")] {
        let settled = settle_assessment(&application, instance.clone(), demand);
        assert_eq!(settled.required().node_path(), path);
        match accept_assessment(&application, instance.clone(), &settled, acceptance) {
            Ok(WorkflowProgressOutcome::Completed(_)) => {}
            other => panic!("{path} assessment must publish: {other:?}"),
        }
    }
    assert!(matches!(
        link_review_requirement(&application, 770).expect("new required relation publishes"),
        WorthQueryApplicationMutationOutcome::Committed { .. },
    ));
    let join = advance_instance(&application, instance.clone(), 771)
        .expect("join observes the independent authored requirement inventory");
    let WorkflowProgressOutcome::AwaitingEvidence(required) = join else {
        panic!("old two-review coverage must not complete the join: {join:?}");
    };
    assert_eq!(required.required_assessments(), 3);
    assert_eq!(required.completed_assessments(), 2);

    let related = settle_early_assessment_for(
        &application,
        instance.clone(),
        "checks/related",
        772,
        RELATED_PART_IDENTITY,
    );
    match accept_early_assessment(
        &application,
        instance.clone(),
        "checks/related",
        &related,
        773,
    ) {
        Ok(WorkflowProgressOutcome::Completed(_)) => {}
        other => panic!("new related assessment must publish: {other:?}"),
    }
    assert!(matches!(
        unlink_review_requirement(&application, 774).expect("relation removal publishes"),
        WorthQueryApplicationMutationOutcome::Committed { .. },
    ));
    assert!(matches!(
        link_review_requirement(&application, 775).expect("same relation is reinserted"),
        WorthQueryApplicationMutationOutcome::Committed { .. },
    ));
    let warm = advance_instance(&application, instance.clone(), 776)
        .expect("warm join checks the native relation revision");
    let WorkflowProgressOutcome::AwaitingEvidence(required) = warm else {
        panic!("warm join reused pre-ABA related evidence: {warm:?}");
    };
    assert_eq!(required.required_assessments(), 3);
    assert_eq!(required.completed_assessments(), 2);
    application
        .runtime()
        .release_workflow_instance_progress_for_test();
    let join = advance_instance(&application, instance.clone(), 780)
        .expect("cold join checks native relation revision, not only current endpoints");
    let WorkflowProgressOutcome::AwaitingEvidence(required) = join else {
        panic!("pre-ABA related evidence must be stale: {join:?}");
    };
    assert_eq!(required.required_assessments(), 3);
    assert_eq!(required.completed_assessments(), 2);

    let refreshed = settle_early_assessment_for(
        &application,
        instance.clone(),
        "checks/related",
        781,
        RELATED_PART_IDENTITY,
    );
    assert!(matches!(
        accept_early_assessment(
            &application,
            instance.clone(),
            "checks/related",
            &refreshed,
        782,
        ),
        Ok(WorkflowProgressOutcome::Completed(_)),
    ));
    let settled = advance_instance(&application, instance, 783)
        .expect("fresh related evidence lets the join settle");
    assert!(matches!(settled, WorkflowProgressOutcome::Completed(_)));
}
