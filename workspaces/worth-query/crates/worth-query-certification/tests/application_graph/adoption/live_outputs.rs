//! Outputs settled under one program, demanded again after their branch
//! adopts another. Both programs supply the same assessment producer over
//! state the adoption validated, so a settled output's content is the one
//! either program produces; what differs is which program's commit it is.

use worth_query_host::facade::application_entry::{
    PublishedWorkflowInstanceRef, WorkflowDefinitionExpectedPredecessor,
    WorkflowDefinitionPublicationOutcome, WorkflowInstanceStartOutcome, WorkflowProgressOutcome,
    WorthQueryWorkflowAssessmentDemandSettlement,
};
use worth_query_host::facade::primary_graph::WorthQueryOutputSettlementPosture;

use crate::document_retention_model::{
    host::{publish_workflow_on_first_program, DocumentWorkflowRuntime},
    presented_request::set_retention,
    programs::RetentionProgramP1,
    readback::read_retention,
    schema::DocumentRetentionQuery,
    settled_verdict::{settle, RetentionVerdict},
    workflow::{
        accept_assessment, prepare_second_program_adoption, propose_instance, publish_adoption,
        publish_definition, reviewed_document_definition, settle_assessment, start_instance,
        support_workflow_program,
    },
};

type Settled = WorthQueryWorkflowAssessmentDemandSettlement<DocumentRetentionQuery>;

/// Retention only `document-retention-v2`, the rule P1 declares, admits.
const P1_ONLY_RETENTION: u64 = 15;

/// An instance on P0 whose head node waits for the retention assessment.
fn awaiting_assessment(key: u64) -> (DocumentWorkflowRuntime, PublishedWorkflowInstanceRef) {
    let application = publish_workflow_on_first_program();
    let definition = match publish_definition(
        &application,
        reviewed_document_definition("completed"),
        WorkflowDefinitionExpectedPredecessor::Absent,
        key,
    )
    .expect("the assessment definition prepares")
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => {
            performed.definition().clone()
        }
        other => panic!("the definition did not publish: {other:?}"),
    };
    let instance = match start_instance(&application, definition, key + 1)
        .expect("the assessment instance prepares")
    {
        WorkflowInstanceStartOutcome::Started(started) => started.instance().clone(),
        other => panic!("the instance did not start: {other:?}"),
    };
    propose_instance(&application, instance.clone(), key + 2).expect("the proposal settles");
    (application, instance)
}

fn adopt_second_program(application: &mut DocumentWorkflowRuntime) {
    support_workflow_program::<RetentionProgramP1>(application);
    let main = application.current_world();
    publish_adoption(prepare_second_program_adoption(
        application,
        main,
        Some(&|inventory| inventory.carry_compatible().unwrap()),
    ));
}

/// The producer runs this demand started, and what the settlement is.
fn production(settled: &Settled) -> (usize, WorthQueryOutputSettlementPosture) {
    let settlement = settled.settlement();
    (
        settlement.producer_contacts_in_this_demand(),
        settlement.posture(),
    )
}

const PRODUCED: (usize, WorthQueryOutputSettlementPosture) =
    (1, WorthQueryOutputSettlementPosture::Performed);
const HELD: (usize, WorthQueryOutputSettlementPosture) =
    (0, WorthQueryOutputSettlementPosture::Performed);

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
