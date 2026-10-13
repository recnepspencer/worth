//! The public conditional-definition admission preserves a cancelled input.
use super::{adapters::ReplacementPredicate, primary_graph as graph, world::CourtroomWorld};
use std::sync::Arc;
#[test]
fn conditional_definition_publication_classifies_its_cancelled_input_as_execution() {
    let world = CourtroomWorld::publish("ready");
    let selected = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .on_branch(world.application.current_world())
                .select(&phase)
                .unwrap()
        })
        .unwrap();
    let (replacement, _) = ReplacementPredicate::controlled(world.contacts.clone());
    let source = worth_query_host::facade::runtime::RuntimeWorldCancellationSource::new();
    let cancellation = source.token();
    // Admission-answer proof: conditional-only Signal publication has no SDK
    // consult, so this does not claim an in-flight interruption.
    source.cancel();
    let answer =
        selected.publish_conditional_definition(&world.clock, Arc::new(replacement), &cancellation);
    let Err(graph::WorthQueryConditionalDefinitionPublicationDenial::ExecutionRequest(cause)) =
        answer
    else {
        panic!("conditional publication stop must be the execution answer")
    };
    use worth_query_execution::facade::application_contribution::{
        WorthQueryAdvancementDenial as Denial, WorthQueryManagedComputationInterruption as Control,
    };
    assert_eq!(cause, Denial::Interrupted(Control::Cancelled));
}
