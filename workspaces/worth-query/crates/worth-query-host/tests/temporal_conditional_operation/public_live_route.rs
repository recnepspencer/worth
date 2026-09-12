use worth_query_host::facade::{
    declaration::application_query::ApplicationQueryParameterSet, primary_graph,
};

use super::{courtroom_support, product_query_support, schema, world, CourtroomWorld};

pub fn world_no_effect_retains_conditional_provenance() {
    let world = CourtroomWorld::publish_with_world_history_limit("ready", 1);
    let request = world::request_scope();
    let principal = product_query_support::principal(&world, &request);
    let branch = world.application.current_world();
    let scope = world
        .application
        .on_branch(branch)
        .select()
        .unwrap()
        .resolve_entity(
            schema::IntentIdentityField::reference(),
            "intent-1".to_owned(),
            &request,
            primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let query = world
        .application
        .installed_schema()
        .certification_query(schema::TemporalIntentLiveQuery::reference())
        .unwrap();
    let live = world
        .application
        .on_branch(branch)
        .select()
        .unwrap()
        .open_application_query_live::<
            schema::TemporalIntentLiveQuery,
            schema::IntentLiveQueryParameters,
            schema::IntentLiveQueryResult,
            _,
            _,
            schema::TemporalIntent,
            schema::TemporalIntent,
            schema::TemporalIntentLiveCause,
        >(
            query,
            &principal,
            scope,
            ApplicationQueryParameterSet::new(),
            primary_graph::WorthQueryApplicationLiveControls::bounded(
                request.clone(),
                4,
                16,
                2_048,
            )
            .unwrap(),
        )
        .unwrap();

    let transaction = world.change_input_on_branch(branch, "history-pressure");
    let Err(transaction) = transaction.require_committed() else {
        panic!("an exhausted World history budget cannot satisfy require_committed");
    };
    let primary_graph::WorthQueryApplicationCommitOutcome::NoEffect(transaction) = transaction
    else {
        panic!("the public transaction must preserve its typed NoEffect terminal");
    };
    assert_eq!(
        transaction.cause(),
        worth_query_host::facade::product::WorthQueryApplicationNoEffectCause::CapacityExhausted,
    );

    let no_effect = courtroom_support::observe(&world);
    assert_eq!(no_effect.committed_operation_count(), 0);
    let [provenance] = no_effect.execution_provenance() else {
        panic!("one due operation must retain its exact NoEffect provenance");
    };
    assert_eq!(
        provenance.terminal(),
        primary_graph::WorthQueryConditionalExecutionTerminal::NoEffect
    );
    assert_eq!(
        provenance.cause(),
        Some(primary_graph::WorthQueryConditionalExecutionCause::NoEffect(
            worth_query_host::facade::product::WorthQueryApplicationNoEffectCause::CapacityExhausted,
        ))
    );
    courtroom_support::assert_authoritative_value(
        &world,
        schema::IntentEffectField::reference(),
        "pending".to_owned(),
    );
    assert!(matches!(
        live.close(),
        primary_graph::WorthQueryApplicationLiveCloseOutcome::Completed(_)
    ));
}

pub fn live_query_receives_conditional_world_publication() {
    let world = CourtroomWorld::publish("ready");
    let request = world::request_scope();
    let principal = product_query_support::principal(&world, &request);
    let branch = world.application.current_world();
    let scope = world
        .application
        .on_branch(branch)
        .select()
        .unwrap()
        .resolve_entity(
            schema::IntentIdentityField::reference(),
            "intent-1".to_owned(),
            &request,
            primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let query = world
        .application
        .installed_schema()
        .certification_query(schema::TemporalIntentLiveQuery::reference())
        .unwrap();
    let mut live = world
        .application
        .on_branch(branch)
        .select()
        .unwrap()
        .open_application_query_live::<
            schema::TemporalIntentLiveQuery,
            schema::IntentLiveQueryParameters,
            schema::IntentLiveQueryResult,
            _,
            _,
            schema::TemporalIntent,
            schema::TemporalIntent,
            schema::TemporalIntentLiveCause,
        >(
            query,
            &principal,
            scope,
            ApplicationQueryParameterSet::new(),
            primary_graph::WorthQueryApplicationLiveControls::bounded(
                request.clone(),
                4,
                16,
                2_048,
            )
            .unwrap(),
        )
        .unwrap();

    let committed = courtroom_support::observe(&world);
    assert_eq!(committed.committed_operation_count(), 1);
    assert!(matches!(
        live.poll(),
        primary_graph::WorthQueryApplicationLiveOutcome::Delivered(_)
    ));
    assert!(matches!(
        live.close(),
        primary_graph::WorthQueryApplicationLiveCloseOutcome::Completed(_)
    ));
}
