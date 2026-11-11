//! Unrelated accounts that grow the graph without touching the measured mutation.
use super::*;

pub fn grow_unrelated_accounts(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
    count: usize,
) {
    if count == 0 {
        return;
    }
    let graph = (*(*(*world.application.runtime_for_test()).primary_provider_for_test())
        .graph_for_test())
    .clone();
    let kind = (*graph.layout_for_test())
        .entity_kind_for_test(Account::reference().name())
        .expect("account kind is installed");
    let locator = |entity: &str, aspect: &str, field: &str| {
        (*graph.layout_for_test())
            .field_locator_for_test(entity, aspect, field)
            .expect("scale field is installed")
            .clone()
    };
    let identity_ref = AccountIdentity::at("AccountIdentity");
    let status_ref = AccountStatus::at("AccountStatus");
    let label_ref = AccountLabel::at("AccountLabel");
    let identity = locator(
        identity_ref.entity(),
        identity_ref.aspect(),
        identity_ref.field(),
    );
    let status = locator(status_ref.entity(), status_ref.aspect(), status_ref.field());
    let label = locator(label_ref.entity(), label_ref.aspect(), label_ref.field());
    for start in (0..count).step_by(4_000) {
        let batch = (start..(start + 4_000).min(count)).fold(
            WorkerIntentBatch::new("unrelated-mutation-scale-population"),
            |batch, ordinal| {
                let key = format!("unrelated-scale-{ordinal}");
                let fields = AspectFieldPatch::from(BTreeMap::from([
                    (
                        identity.clone(),
                        StringApplicationValueBinding::encode(&key.clone()).unwrap(),
                    ),
                    (
                        status.clone(),
                        StringApplicationValueBinding::encode(&"population".to_owned()).unwrap(),
                    ),
                    (
                        label.clone(),
                        StringApplicationValueBinding::encode(&"population".to_owned()).unwrap(),
                    ),
                ]));
                batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                    partition_id: worth_relational::facade::identity::PartitionId::main(),
                    kind_id: kind,
                    client_key: worth_relational::facade::symbols::ClientKey::raw(key),
                    fields,
                })))
            },
        );
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation(execution, world, batch);
    }
}
