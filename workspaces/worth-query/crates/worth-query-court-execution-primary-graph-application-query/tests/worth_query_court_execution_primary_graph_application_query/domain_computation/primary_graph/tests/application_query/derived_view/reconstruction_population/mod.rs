//! A third declared root makes first, middle and last failure ranks distinct.
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountIdentity;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountLabel;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountMembershipTag;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
use std::collections::BTreeMap;
use worth_query_declaration::facade::application_schema::StringApplicationValueBinding;
use worth_query_installation::facade::ApplicationScalarValueBinding;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::AspectFieldPatch;
use worth_relational::facade::transactions::CreateIntent;
use worth_relational::facade::transactions::EntitySpec;
use worth_relational::facade::transactions::MutationIntent;
use worth_relational::facade::transactions::WorkerIntentBatch;
pub fn add_third_root(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
) {
    let graph = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap();
    macro_rules! declared_field {
        ($marker:ty, $value:expr) => {{
            let field = <$marker>::reference();
            (
                (*graph.layout_for_test())
                    .field_locator_for_test(field.entity(), field.aspect(), field.field())
                    .unwrap()
                    .clone(),
                StringApplicationValueBinding::encode(&$value.to_owned()).unwrap(),
            )
        }};
    }
    let fields = AspectFieldPatch::from(BTreeMap::from([
        declared_field!(AccountIdentity, "account-3"),
        declared_field!(AccountStatus, "third"),
        declared_field!(AccountLabel, "third-label"),
        declared_field!(AccountMembershipTag, "open"),
    ]));
    publish_relational_mutation(
        execution,
        world,
        WorkerIntentBatch::new("third-reconstruction-root").push(MutationIntent::Create(
            CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: (*graph.layout_for_test())
                    .entity_kind_for_test(Account::reference().name())
                    .unwrap(),
                client_key: ClientKey::raw("account-3"),
                fields,
            }),
        )),
    );
}

/// Other roots have one primary child and no collections. The original root
/// retains two collections and an optional child, giving unequal read costs.
pub fn add_primary_reads(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    third: bool,
) {
    use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
    use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountPrimaryActivity;
    use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;
    use worth_relational::facade::transactions::EntityReference;
    use worth_relational::facade::transactions::RelationSpec;
    let scope = live_scope();
    let selected = world.selected_product(&(phase_custody).test_host_request_for_test());
    let root = selected
        .resolve_entity(
            phase_custody,
            AccountStatus::reference(),
            "open".to_owned(),
            &scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let graph = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap();
    let kind = (*(*graph.layout_for_test())
        .relation(AccountPrimaryActivity::reference().name())
        .unwrap()
        .kind_for_test());
    let target = graph
        .integration_handle_for_test()
        .with_runtime(execution, |runtime| {
            runtime
                .read_truth()
                .visible_relations_of_kind(
                    execution,
                    kind,
                    selected
                        .application_basis_for_test()
                        .snapshot_handle()
                        .version_id(),
                )
                .expect("fixture read request remains live")
                .into_iter()
                .find(|relation| relation.source == root.entity_id_for_test())
                .unwrap()
                .target
        });
    let sources = ["unrelated", "third"]
        .into_iter()
        .take(if third { 2 } else { 1 })
        .map(|status| {
            selected
                .resolve_entity(
                    phase_custody,
                    AccountStatus::reference(),
                    status.to_owned(),
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
                .entity_id_for_test()
        })
        .collect::<Vec<_>>();
    drop(selected);
    let mut batch = WorkerIntentBatch::new("unequal-reconstruction-reads");
    for (index, source) in sources.into_iter().enumerate() {
        batch = batch.push(MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: kind,
                client_key: ClientKey::raw(format!("other-primary-{index}")),
                source: EntityReference::Existing(source),
                target: EntityReference::Existing(target),
                fields: AspectFieldPatch::default(),
            },
        )));
    }
    publish_relational_mutation(execution, world, batch);
}
