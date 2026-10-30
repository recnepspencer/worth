//! The public request keeps an early write refusal distinct from commit lowering.
use super::{
    fixture, host,
    readback::{observe_head, read_retention},
    schema::*,
};
use worth_query_host::facade::{
    application_entry::*,
    primary_graph::{
        MutationHandlerExecutionDenial, WorthQueryApplicationAttemptDenial,
        WorthQueryApplicationAttemptDenialKind, WorthQueryPrincipalResolutionMode,
    },
};
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};

#[test]
fn public_unobserved_write_keeps_exact_target_and_observed_twin_commits() {
    let host = host::install_without_retention_observation();
    let runtime = host.runtime();
    let scope = request_scope();
    let actor = authenticate_operator(host.installed_schema(), &scope);
    let before = observe_head(runtime, runtime.current_world());
    let target = runtime
        .with_host_advancement(|phase| {
            before
                .select_on(&phase, runtime)
                .unwrap()
                .resolve_entity(
                    &phase,
                    DocumentIdentityField::reference(),
                    DOCUMENT_IDENTITY.to_owned(),
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
                .relational_record_identity_parts()
        })
        .unwrap();
    let request = runtime.request(&actor, &scope);
    let (operation, denial) =
        match fixture::operation(&host, &request, 28, 0x9181_1401).advance(&request) {
            WorthQuerySourceOnlyAdvance::Refused {
                operation,
                denial: WorthQuerySourceOnlyDenial::Mutation(denial),
            } => (operation, denial),
            _ => panic!("the actual candidate writer must refuse without its field observation"),
        };
    operation
        .close(&request)
        .unwrap_or_else(|(_, cause)| panic!("the refused owner remains closeable: {cause:?}"));
    let WorthQueryApplicationRequestMutationDenial::Handler(
        MutationHandlerExecutionDenial::Handler(denial),
    ) = denial
    else {
        panic!("the refusal must remain at the handler boundary: {denial:?}");
    };
    let denial = denial
        .downcast::<WorthQueryApplicationAttemptDenial>()
        .expect("the original typed candidate refusal survives the public wrapper");
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationAttemptDenialKind::UnobservedWrite
    );
    let cause = denial.unobserved_write().expect("exact write coordinates");
    assert_eq!(cause.entity_id().partition_value(), target.partition_id());
    assert_eq!(cause.entity_id().local_slot_value(), target.local_slot());
    assert_eq!(cause.entity_id().generation_value(), target.generation());
    assert_eq!(
        cause.locator().aspect().aspect_key().as_str(),
        "DocumentFacts"
    );
    assert_eq!(
        cause.locator().field_path().fields()[0].as_str(),
        "DocumentRetentionField"
    );
    assert_eq!(
        host.contacts.counts(),
        (1, 1),
        "the disputed write was reached"
    );
    assert_eq!(
        observe_head(runtime, runtime.current_world()).selected_commit(),
        before.selected_commit()
    );
    assert_eq!(read_retention(runtime, runtime.current_world()), 7);

    // The same actual installed contract with its ordinary decision read commits.
    let twin = fixture::install();
    let actor = authenticate_operator(twin.installed_schema(), &scope);
    let request = twin.runtime().request(&actor, &scope);
    let completed = fixture::completed(
        fixture::operation(&twin, &request, 28, 0x9181_1401),
        &request,
    );
    assert!(matches!(
        completed.evidence(),
        WorthQuerySourceCompletion::Fresh { .. }
    ));
    completed
        .close(&request)
        .unwrap_or_else(|(_, cause)| panic!("the completed owner remains closeable: {cause:?}"));
    assert_eq!(read_retention(twin.runtime(), twin.current_world()), 28);
    assert_eq!(twin.contacts.counts(), (1, 1));
}
