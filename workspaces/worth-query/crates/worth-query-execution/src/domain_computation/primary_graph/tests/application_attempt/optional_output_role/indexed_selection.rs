use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryApplicationObservedFact as Fact,
    application_checkpoint::{decode_producer_facts, encode_producer_facts},
    output_reuse::{compare_retained_output_dependencies, OutputDependencySelection},
    tests::fixture::{publish_relational_mutation_on_application, restored_world, AccountStatus},
    WorthQueryPrimaryGraphApplicationRuntime,
};
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
    let (loser, loser_key) = execute_with_key(
        &world,
        OptionalOutputPlan::RequiredAfterIndexedAbsence,
        "loser",
    );
    let Ok(HandlerResult::Completed(loser)) = loser else {
        panic!("the installed handler must retain the absent predicate");
    };
    let (loser, _) = loser.into_parts();
    let (winner, winner_key) = execute_with_key(
        &world,
        OptionalOutputPlan::RequiredAfterIndexedAbsence,
        "winner",
    );
    let Ok(HandlerResult::Completed(winner)) = winner else {
        panic!("both decisions see the same absence");
    };
    let (winner, _) = winner.into_parts();
    let winner_outcome = world
        .application
        .compare_and_commit_application(winner, winner_key);
    assert!(
        matches!(
            winner_outcome,
            WorthQueryApplicationCommitOutcome::Committed(_)
        ),
        "winner: {winner_outcome:?}"
    );
    let committed_head = world.selected_product().product().selected_commit().clone();
    let outcome = world
        .application
        .compare_and_commit_application(loser, loser_key);
    super::super::assert_changed_decision(outcome, "a competing indexed absence");
    assert_eq!(
        world.selected_product().product().selected_commit(),
        &committed_head
    );
    let _published = resolved_account(&world, "pending-membership", &live_scope());
}

#[test]
fn handler_predicate_rebases_and_its_codec_compares_against_the_reopened_world() {
    let world = installed_authorization_world(true);
    let (execution, key) = execute(&world, OptionalOutputPlan::RequiredAfterIndexedAbsence);
    let Ok(HandlerResult::Completed(completed)) = execution else {
        panic!("the installed handler completes its indexed decision");
    };
    let (program, _) = completed.into_parts();
    let outcome = world
        .application
        .compare_and_commit_application(program, key);
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
        .producer_facts_for_receipt(&receipt)
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
    assert!(matches!(
        selection(&world.application, &facts),
        OutputDependencySelection::Reuse
    ));

    let durable = encode_producer_facts(&facts).expect("all rebased handler reads are durable");
    let checkpoint = world.application.capture_application_checkpoint().unwrap();
    drop(world);
    let reopened = restored_world(checkpoint).expect("the real native application reopens");
    let restored = decode_producer_facts(&durable).unwrap();
    assert!(matches!(
        selection(&reopened.application, &restored),
        OutputDependencySelection::Reuse
    ));
    change_status(&reopened.application, subject, "left-membership");
    assert!(matches!(
        selection(&reopened.application, &restored),
        OutputDependencySelection::FreshRequired
    ));
}

fn selection(
    application: &WorthQueryPrimaryGraphApplicationRuntime<IdentityExecutionSchema>,
    facts: &[Fact],
) -> OutputDependencySelection {
    let selected = application
        .select_product_branch(application.product_runtime().default_branch())
        .unwrap();
    application.primary_provider.graph.with_runtime(|runtime| {
        compare_retained_output_dependencies(
            runtime,
            selected.application_basis().snapshot_handle(),
            true,
            Some(facts),
            &mut 32,
        )
        .unwrap()
    })
}

fn change_status(
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
