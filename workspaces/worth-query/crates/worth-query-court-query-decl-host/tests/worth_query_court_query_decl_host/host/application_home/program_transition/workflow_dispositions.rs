//! Explicit absent workflow choices cannot disappear in an empty Home fast path.
use super::*;
use worth_query_host::facade::primary_graph::{
    WorthQueryWorkflowDefinitionDisposition, WorthQueryWorkflowDispositions,
};
use worth_query_testkit_document_retention::facade::document_retention_model as document;

#[test]
fn home_transition_refuses_a_foreign_workflow_choice_before_intent_and_preserves_a() {
    let memory = document::host::publish_workflow_on_first_program();
    let definition = match document::workflow::publish_definition(
        &memory, document::workflow::reviewed_document_definition("approved"),
        worth_query_host::facade::application_entry::WorkflowDefinitionExpectedPredecessor::Absent,
        0x717001,
    ).unwrap() {
        worth_query_host::facade::application_entry::WorkflowDefinitionPublicationOutcome::Published(published) => published.definition().clone(),
        other => panic!("the foreign definition is genuinely published: {other:?}"),
    };
    let location = fixture::location();
    let build = ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap();
    let contacts = Arc::new(AtomicUsize::new(0));
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        fixture::request(
            location.clone(),
            build,
            Arc::new(AtomicUsize::new(0)),
            contacts.clone(),
        )
        .open_in_advancement(&phase)
        .unwrap()
    })
    .unwrap();
    home.close(&execution).unwrap().reclaim();
    let before =
        fixture::request_with_initializer(location.clone(), build, contacts.clone(), |_, _, _| {
            panic!("existing A cannot initialize")
        });
    let request = successor(location.clone(), build, contacts.clone())
        .with_program_transition(before, resources(), |source, target| {
            let rows = source.entities("Account", 10_000)?;
            let [row] = rows.records() else {
                panic!("one actual A account");
            };
            let field = target
                .planned_field("Account", "AccountIdentity", "Balance")
                .unwrap();
            let mut step = body(row.entity_id, field, 8);
            step.batches.push(successor::body(target));
            Ok(Plan {
                target: 0,
                schema_transition: Some((successor::schema_change(source, target), None)),
                steps: vec![step],
            })
        })
        .with_workflow_dispositions(
            WorthQueryWorkflowDispositions::new()
                .definition(&definition, WorthQueryWorkflowDefinitionDisposition::Retire),
        );
    let refused = with_bootstrap_advancement_from_request(execution, |phase| {
        request.open_in_advancement(&phase)
    })
    .unwrap()
    .err()
    .expect("an absent foreign choice is not ignored");
    match refused.phase() {
        Refused::Transition {
            home,
            cause: Denial::Workflow(QueryHomeWorkflowDispositionDenial::Rejected),
            ..
        } => assert!(home.home_program_transition_intent().is_none()),
        other => panic!("typed workflow disposition refusal: {other:?}"),
    }
    drop(refused); // Abandon the original unserved opening, not a successful close.
    let current =
        fixture::request_with_initializer(location.clone(), build, contacts.clone(), |_, _, _| {
            panic!("A remains existing")
        })
        .with_program_transition(
            fixture::request_with_initializer(location, build, contacts.clone(), |_, _, _| {
                panic!("predecessor is not initialized")
            }),
            resources(),
            |_, _| panic!("current A skips its optional author and workflow choices"),
        )
        .with_workflow_dispositions(
            WorthQueryWorkflowDispositions::new()
                .definition(&definition, WorthQueryWorkflowDefinitionDisposition::Retire),
        );
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        current.open_in_advancement(&phase).unwrap()
    })
    .unwrap();
    let (_, scope) = super::super::pending_support::scope();
    assert_eq!(
        super::super::pending_support::balance(home.runtime(), &scope),
        7
    );
    assert_eq!(
        contacts.load(Ordering::SeqCst),
        0,
        "no supplier effect or cutover"
    );
    home.close(&execution).unwrap().reclaim();
    drop(memory);
}
