use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryApplicationObservedFact as Fact,
    application_checkpoint::{decode_producer_facts, encode_producer_facts},
    output_reuse::{compare_retained_output_dependencies, OutputDependencySelection},
    tests::fixture::{publish_relational_mutation_on_application, restored_world, AccountStatus},
    WorthQueryPrimaryGraphApplicationRuntime,
};
use crate::facade::application_installation::WorthQueryCheckpointCapturePolicy as CapturePolicy;
use worth_query_declaration::facade::application_schema::{
    ApplicationScalarValueBinding, StringApplicationValueBinding,
};
use worth_relational::facade::transactions::{
    AspectFieldPatch, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent,
    WorkerIntentBatch,
};

#[test]
fn handler_absence_selection_competes_at_the_real_commit_boundary() {
    let world = installed_authorization_world(true);
    let (loser, loser_key) = {
        execute_with_key(
            &world,
            OptionalOutputPlan::RequiredAfterIndexedAbsence,
            "loser",
        )
    };
    let Ok(HandlerResult::Completed(loser)) = loser else {
        panic!("the installed handler must retain the absent predicate");
    };
    let (loser, _) = loser.into_parts();
    let (winner, winner_key) = {
        execute_with_key(
            &world,
            OptionalOutputPlan::RequiredAfterIndexedAbsence,
            "winner",
        )
    };
    let Ok(HandlerResult::Completed(winner)) = winner else {
        panic!("both decisions see the same absence");
    };
    let (winner, _) = winner.into_parts();
    let winner_outcome = world.application.compare_and_commit_application(
        winner,
        winner_key,
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    );
    assert!(
        matches!(
            winner_outcome,
            WorthQueryApplicationCommitOutcome::Committed(_)
        ),
        "winner: {winner_outcome:?}"
    );
    let committed_head = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request())
                .product()
                .selected_commit()
                .clone()
        })
        .expect("the fixture metadata read owns its host call");
    let outcome = world.application.compare_and_commit_application(
        loser,
        loser_key,
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    );
    super::super::assert_changed_decision(outcome, "a competing indexed absence");
    world
        .application
        .with_host_advancement(|phase| {
            assert_eq!(
                world
                    .selected_product(&(phase).test_host_request(),)
                    .product()
                    .selected_commit(),
                &committed_head
            )
        })
        .expect("the fixture metadata read owns its host call");
    let _published = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(
                &(phase).test_host_request(),
                &world,
                "pending-membership",
                &live_scope(),
            )
        })
        .expect("the fixture entity read owns its host call");
}

#[test]
fn handler_predicate_rebases_and_its_codec_compares_against_the_reopened_world() {
    let world = installed_authorization_world(true);
    let (execution, key) = { execute(&world, OptionalOutputPlan::RequiredAfterIndexedAbsence) };
    let Ok(HandlerResult::Completed(completed)) = execution else {
        panic!("the installed handler completes its indexed decision");
    };
    let (program, _) = completed.into_parts();
    let outcome = world.application.compare_and_commit_application(
        program,
        key,
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    );
    let WorthQueryApplicationCommitOutcome::Committed(receipt) = outcome else {
        panic!("the indexed handler publishes through the production commit lane: {outcome:?}");
    };
    let facts = world
        .application
        .primary_provider
        .graph
        .output_lineage
        .lock()
        .unwrap()
        .checkpoint_facts_for_receipt(&receipt)
        .map(|(facts, _)| facts)
        .unwrap();
    let subject = receipt
        .output_correspondence()
        .outputs_of::<OptionalOutputs>()
        .unwrap()
        .entity::<OptionalSubject>()
        .unwrap()
        .entity_id();
    assert!(facts.iter().any(|fact| matches!(fact,
        Fact::IndexedEntitySelection { value, candidates, .. }
        if value == &StringApplicationValueBinding::encode(&"pending-membership".to_owned()).unwrap()
            && candidates == &[subject]
    )), "the precommit absence must rebase to the member this candidate publishes: {facts:?}");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert!(matches!(
                selection(
                    &host_custody.borrowed_request_for_test(),
                    &world.application,
                    &facts
                ),
                OutputDependencySelection::Reuse
            ))
        })
        .expect("the fixture read owns its installed host call");

    let durable = encode_producer_facts(&facts).expect("all rebased handler reads are durable");
    let checkpoint = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .capture_application_checkpoint(&phase, CapturePolicy::SystemAllocation)
        })
        .expect("installed host call opens")
        .unwrap();
    drop(world);
    let reopened = restored_world(checkpoint).expect("the real native application reopens");
    let restored = decode_producer_facts(
        &durable,
        None,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .unwrap();
    reopened
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert!(matches!(
                selection(
                    &host_custody.borrowed_request_for_test(),
                    &reopened.application,
                    &restored
                ),
                OutputDependencySelection::Reuse
            ))
        })
        .expect("the fixture read owns its installed host call");
    reopened
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            change_status(
                &host_custody.borrowed_request_for_test(),
                &reopened.application,
                subject,
                "left-membership",
            )
        })
        .expect("the fixture read owns its installed host call");
    reopened
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert!(matches!(
                selection(
                    &host_custody.borrowed_request_for_test(),
                    &reopened.application,
                    &restored
                ),
                OutputDependencySelection::FreshRequired
            ))
        })
        .expect("the fixture read owns its installed host call");
}

fn selection(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    application: &WorthQueryPrimaryGraphApplicationRuntime<IdentityExecutionSchema>,
    facts: &[Fact],
) -> OutputDependencySelection {
    let selected = application
        .select_product_branch(execution, application.product_runtime().default_branch())
        .unwrap();
    application
        .primary_provider
        .graph
        .with_runtime(execution, |runtime| {
            compare_retained_output_dependencies(
                execution,
                runtime,
                selected.application_basis().snapshot_handle(),
                true,
                Some(facts),
                &mut application
                    .primary_provider
                    .graph
                    .source_owner
                    .invalidation_owner
                    .edit_admission_within(std::num::NonZeroUsize::new(32).unwrap()),
            )
            .unwrap()
        })
}

fn change_status(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    application: &WorthQueryPrimaryGraphApplicationRuntime<IdentityExecutionSchema>,
    subject: worth_relational::facade::identity::EntityId,
    value: &str,
) {
    let field = AccountStatus::reference();
    let locator = application
        .runtime
        .primary_graph()
        .unwrap()
        .layout
        .field_locator(field.entity(), field.aspect(), field.field())
        .unwrap()
        .clone();
    publish_relational_mutation_on_application(
        execution,
        application,
        WorkerIntentBatch::new("restored-indexed-selection").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: subject,
                fields: AspectFieldPatch::from(std::collections::BTreeMap::from([(
                    locator,
                    StringApplicationValueBinding::encode(&value.to_owned()).unwrap(),
                )])),
            }),
        )),
    );
}
