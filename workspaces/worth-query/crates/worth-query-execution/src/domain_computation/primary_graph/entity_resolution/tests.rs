use worth_query_declaration::facade::application_schema::{
    ApplicationScalarValueBinding, StringApplicationValueBinding,
};
use worth_relational::facade::indexes::DerivedIndexBuildRequest;

use super::super::tests::fixture::{
    installed_authorization_world, live_scope, AccountStatus, IdentityExecutionSchema,
};
use super::super::{
    primary_relational_branch_id, WorthQueryEntityResolutionDenialKind,
    WorthQueryPrincipalResolutionMode,
};

#[test]
fn equal_version_snapshot_from_another_relational_runtime_is_rejected() {
    let first = installed_authorization_world(true);
    let second = installed_authorization_world(true);
    let first_graph = first.application.runtime.primary_graph().unwrap();
    let second_graph = second.application.runtime.primary_graph().unwrap();
    let installed = first_graph.retain_entity_resolution_context();
    let first_product = first.selected_product();
    let first_version = first_graph.integration_handle().with_runtime_mut(|_| {
        first_product
            .application_basis()
            .snapshot_handle()
            .version_id()
    });

    let second_product = second.selected_product();
    second_graph
        .integration_handle()
        .with_runtime_mut(|runtime| {
            let snapshot = second_product.application_basis().snapshot_handle();
            assert_eq!(snapshot.version_id(), first_version);
            let denial = match installed.at_snapshot(
                runtime,
                snapshot,
                WorthQueryPrincipalResolutionMode::Ordinary,
            ) {
                Ok(_) => panic!("foreign runtime truth entered entity resolution"),
                Err(denial) => denial,
            };
            assert_eq!(
                denial.kind(),
                WorthQueryEntityResolutionDenialKind::ForeignResolutionTruth
            );
        });
}

#[test]
fn rebuilt_index_generation_preserves_stable_entity_meaning() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let identity = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_owned(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let graph = world.application.runtime.primary_graph().unwrap();
    let installed = graph.retain_entity_resolution_context();
    let selected = world.selected_product();
    let index_id = graph
        .layout()
        .equality_field("Account", "AccountPolicy", "AccountStatus")
        .and_then(|field| field.equality_index_id)
        .unwrap();

    graph.integration_handle().with_runtime_mut(|runtime| {
        let head = runtime
            .history()
            .historical_latest_commit()
            .unwrap()
            .clone();
        let build = runtime
            .index_authority()
            .build_for_commit(DerivedIndexBuildRequest {
                source_commit_id: head.commit_id,
                branch_id: primary_relational_branch_id(),
                index_ids: vec![index_id],
            });
        assert!(build.failed_indexes.is_empty());
        assert_ne!(
            build.generations[0].generation_id,
            identity.identity_index_generation()
        );
        let snapshot = selected.application_basis().snapshot_handle();
        let truth = installed
            .at_snapshot(
                runtime,
                snapshot,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap();
        truth.validate_entity_freshness(&identity).unwrap();
    });
}

#[test]
fn installed_context_derives_binding_layout_and_index_from_its_graph() {
    let world = installed_authorization_world(true);
    let graph = world.application.runtime.primary_graph().unwrap();
    let installed = graph.retain_entity_resolution_context();
    let selected = world.selected_product();
    graph.integration_handle().with_runtime_mut(|runtime| {
        let snapshot = selected.application_basis().snapshot_handle();
        let truth = installed
            .at_snapshot(runtime, snapshot, WorthQueryPrincipalResolutionMode::Ordinary)
            .unwrap();
        let resolved = truth
            .resolve(
                "Account",
                "AccountPolicy",
                "AccountStatus",
                StringApplicationValueBinding::encode(&"open".to_owned()).unwrap(),
            )
            .unwrap();
        let typed = resolved.into_application_identity::<IdentityExecutionSchema, super::super::tests::fixture::Account>();
        assert_eq!(typed.binding_identity(), graph.binding_identity());
        assert_eq!(typed.identity_index_id(), graph.layout().equality_field("Account", "AccountPolicy", "AccountStatus").unwrap().equality_index_id.unwrap());
    });
}
