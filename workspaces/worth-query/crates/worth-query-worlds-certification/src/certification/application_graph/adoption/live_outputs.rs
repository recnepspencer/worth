//! Outputs settled under one program, demanded again after their branch
//! adopts another. Both programs supply the same assessment producer over
//! state the adoption validated, so a settled output's content is the one
//! either program produces; what differs is which program's commit it is.

pub use worth_query_host::facade::application_entry::{
    PublishedWorkflowInstanceRef, WorkflowDefinitionExpectedPredecessor,
    WorkflowDefinitionPublicationOutcome, WorkflowInstanceStartOutcome, WorkflowProgressOutcome,
    WorthQueryWorkflowAssessmentDemandSettlement,
};
pub use worth_query_host::facade::primary_graph::WorthQueryOutputSettlementPosture;

pub use crate::facade::certification::document_retention_model::{
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

pub type Settled = WorthQueryWorkflowAssessmentDemandSettlement<DocumentRetentionQuery>;

/// Retention only `document-retention-v2`, the rule P1 declares, admits.
pub const P1_ONLY_RETENTION: u64 = 15;

/// An instance on P0 whose head node waits for the retention assessment.
pub fn awaiting_assessment(key: u64) -> (DocumentWorkflowRuntime, PublishedWorkflowInstanceRef) {
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

pub fn adopt_second_program(application: &mut DocumentWorkflowRuntime) {
    support_workflow_program::<RetentionProgramP1>(application);
    let main = application.current_world();
    publish_adoption(
        application,
        prepare_second_program_adoption(
            application,
            main,
            Some(worth_query_host::facade::primary_graph::WorthQueryWorkflowDispositions::carry_compatible()),
        ),
    );
}

/// The producer runs this demand started, and what the settlement is.
pub fn production(settled: &Settled) -> (usize, WorthQueryOutputSettlementPosture) {
    let settlement = settled.settlement();
    (
        settlement.producer_contacts_in_this_demand(),
        settlement.posture(),
    )
}

pub const PRODUCED: (usize, WorthQueryOutputSettlementPosture) =
    (1, WorthQueryOutputSettlementPosture::Performed);
pub const HELD: (usize, WorthQueryOutputSettlementPosture) =
    (0, WorthQueryOutputSettlementPosture::Performed);
