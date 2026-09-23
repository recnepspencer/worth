use std::time::Duration;

use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{
    AspectFieldPatch, CreateIntent, DeleteRelationIntent, EntityReference, MutationIntent,
    RelationMutationIntent, RelationSpec, WorkerIntentBatch,
};

use super::current_controls;
use crate::domain_computation::primary_graph::{
    tests::fixture::{
        cross_root_definition, installed_authorization_world, AccountStatus, CrossRootQuery,
        ScopedAccountSummaryQuery,
    },
    WorthQueryApplicationObservedFact, WorthQueryApplicationQueryAccessContext,
    WorthQueryPrincipalResolutionMode,
};

#[test]
fn root_path_guard_literal_changes_canonical_query_identity() {
    let open = cross_root_definition("open").into_erased();
    let closed = cross_root_definition("closed").into_erased();

    assert_ne!(open.canonical_basis(), closed.canonical_basis());
}

#[test]
fn exact_scope_root_executes_without_an_invented_predicate() {
    let world = installed_authorization_world(true);
    let request = super::super::fixture::live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_authenticated_principal(
            &world.binding,
            &external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let account = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_string(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let query = world
        .application
        .installed_schema()
        .certification_query(ScopedAccountSummaryQuery::reference())
        .unwrap();
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
    let plan = world
        .selected_product()
        .admit_application_query(
            &query,
            &access,
            ApplicationQueryParameterSet::new(),
            current_controls(&request),
        )
        .unwrap();

    assert_eq!(
        plan.graph_read_plan()
            .requirements()
            .counters()
            .predicate_support_count(),
        0
    );
    let result = world
        .application
        .execute_application_query_one_shot(plan)
        .unwrap();
    assert_eq!(result.rows().len(), 1);
    assert_eq!(result.rows()[0].status(), "open");
    assert_eq!(result.receipt().examined_candidate_count(), 1);
}

#[test]
fn declared_root_paths_retain_per_row_native_witnesses() {
    let world = installed_authorization_world(true);
    let request = super::super::fixture::live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_authenticated_principal(
            &world.binding,
            &external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let account = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_string(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let query = world
        .application
        .installed_schema()
        .certification_query(CrossRootQuery::reference())
        .unwrap();
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
    let plan = world
        .selected_product()
        .admit_application_query(
            &query,
            &access,
            ApplicationQueryParameterSet::new(),
            current_controls(&request),
        )
        .unwrap();
    assert_eq!(
        plan.graph_read_plan()
            .requirements()
            .counters()
            .predicate_support_count(),
        1
    );
    let predicate_fields = plan
        .graph_read_plan()
        .requirements()
        .rows()
        .iter()
        .flat_map(|row| row.predicate_field_authorities())
        .map(|field| field.native_field_key().as_str())
        .collect::<Vec<_>>();
    assert_eq!(predicate_fields, ["AccountStatus"]);
    let result = world
        .application
        .execute_application_query_one_shot(plan)
        .unwrap();

    assert_eq!(
        result
            .rows()
            .iter()
            .map(|row| row.sequence())
            .collect::<Vec<_>>(),
        [11, 22]
    );
    assert_eq!(result.receipt().result_count(), 2);
    assert_eq!(result.receipt().adjacency_list_read_count(), 3);
    assert_eq!(result.receipt().edge_scan_count(), 4);
    assert_eq!(result.receipt().examined_candidate_count(), 3);
    assert_eq!(result.receipt().work().predicate_work_units(), 6);
    assert_eq!(result.receipt().fallback_count(), 0);
    assert_eq!(result.receipt().per_result_neighbor_lookup_count(), 0);
    let first = result.observed_sources()[0]
        .footprint_for_test()
        .root_selection
        .as_ref()
        .expect("path-selected rows retain native selection evidence");
    let second = result.observed_sources()[1]
        .footprint_for_test()
        .root_selection
        .as_ref()
        .expect("every row retains its own path witness");
    assert!(!std::sync::Arc::ptr_eq(first, second));
    assert_eq!(first.aspects.len(), 1);
    assert_eq!(first.adjacencies.len(), 1);
    assert_eq!(second.adjacencies.len(), 1);
    assert!(!first
        .entities
        .contains(&result.observed_sources()[1].footprint_for_test().root));
    assert!(!second
        .entities
        .contains(&result.observed_sources()[0].footprint_for_test().root));
    assert_eq!(
        first.adjacencies[0].relation_kind,
        second.adjacencies[0].relation_kind
    );

    let graph = world.application.runtime.primary_graph().unwrap();
    let observed_source = &result.observed_sources()[0];
    let crate::domain_computation::primary_graph::application_query::resource_lifecycle::WorthQueryApplicationBasisSelectionIdentity::Product(product) = &observed_source.selection else {
        panic!("the product query must retain its selected branch identity");
    };
    let source_facts = observed_source
        .clone()
        .validate_and_into_facts(
            observed_source.runtime_authority,
            &observed_source.schema_binding,
            &observed_source.branch,
            observed_source.model_root,
            product,
            &observed_source.query_identifier,
            &observed_source.query_identity,
            &graph.layout,
        )
        .expect("ordinary source binding retains root-path native facts");
    assert_eq!(
        source_facts
            .iter()
            .filter(|fact| matches!(
                fact,
                WorthQueryApplicationObservedFact::SourceAdjacencyRevision { .. }
            ))
            .count(),
        1
    );
    assert!(source_facts.iter().any(|fact| matches!(fact,
        WorthQueryApplicationObservedFact::SourceAspectRevision { entity_id, .. }
            if *entity_id == account.entity_id()
    )));
    let kind = first.adjacencies[0].relation_kind;
    let selected = world.selected_product();
    let (relation_id, target) = graph.integration_handle().with_runtime(|runtime| {
        runtime
            .read_truth()
            .visible_relations_of_kind(
                kind,
                selected.application_basis().snapshot_handle().version_id(),
            )
            .into_iter()
            .find(|relation| relation.source == account.entity_id())
            .map(|relation| (relation.relation_id, relation.target))
            .unwrap()
    });
    drop(selected);
    super::super::fixture::publish_relational_mutation(
        &world,
        WorkerIntentBatch::new("root-path-relation-aba")
            .push(MutationIntent::Relation(RelationMutationIntent::Delete(
                DeleteRelationIntent { relation_id },
            )))
            .push(MutationIntent::Create(CreateIntent::Relation(
                RelationSpec {
                    partition_id: PartitionId::main(),
                    kind_id: kind,
                    client_key: ClientKey::raw("root-path-relation-aba-replacement"),
                    source: EntityReference::Existing(account.entity_id()),
                    target: EntityReference::Existing(target),
                    fields: AspectFieldPatch::default(),
                },
            ))),
    );
    let observed = first
        .adjacencies
        .iter()
        .find(|adjacency| adjacency.relation_kind == kind)
        .unwrap();
    let fact = WorthQueryApplicationObservedFact::SourceAdjacencyRevision {
        relation_kind: observed.relation_kind,
        anchor: observed.anchor,
        direction: observed.direction,
        native_revision: observed.native_revision,
        comparison_work_limit: observed.comparison_work_limit,
        endpoints: vec![target],
    };
    let current = world.selected_product();
    assert!(
        !graph.integration_handle().with_runtime(|runtime| {
            fact.source_currentness_in(runtime, current.application_basis().snapshot_handle(), 1)
                .unwrap()
                .0
        }),
        "same endpoints after relation ABA must stale even a merged adjacency fact"
    );
    let refreshed = world
        .application
        .execute_application_query_one_shot(
            current
                .admit_application_query(
                    &query,
                    &access,
                    ApplicationQueryParameterSet::new(),
                    current_controls(&request),
                )
                .unwrap(),
        )
        .unwrap();
    assert_eq!(
        refreshed
            .rows()
            .iter()
            .map(|row| row.sequence())
            .collect::<Vec<_>>(),
        [11, 22],
        "ABA keeps the public result set unchanged"
    );
    assert_ne!(
        result.observed_sources()[0].idempotency_identity(),
        refreshed.observed_sources()[0].idempotency_identity(),
        "native path revisions must participate in source identity"
    );
    assert_ne!(
        result.observed_sources()[1].idempotency_identity(),
        refreshed.observed_sources()[1].idempotency_identity(),
        "rows reached through the same native adjacency granule both become stale"
    );
}
