//! A third declared root makes first, middle and last failure ranks distinct.
use crate::domain_computation::primary_graph::tests::fixture::{
    publish_relational_mutation, Account, AccountIdentity, AccountLabel, AccountMembershipTag,
    AccountStatus, AuthorizationWorld,
};
use std::collections::BTreeMap;
use worth_query_declaration::facade::application_schema::StringApplicationValueBinding;
use worth_query_installation::facade::ApplicationScalarValueBinding;
use worth_relational::facade::{
    identity::PartitionId,
    symbols::ClientKey,
    transactions::{AspectFieldPatch, CreateIntent, EntitySpec, MutationIntent, WorkerIntentBatch},
};
pub(super) fn add_third_root(world: &AuthorizationWorld) {
    let graph = world.application.runtime.primary_graph().unwrap();
    macro_rules! declared_field {
        ($marker:ty, $value:expr) => {{
            let field = <$marker>::reference();
            (
                graph
                    .layout
                    .field_locator(field.entity(), field.aspect(), field.field())
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
        world,
        WorkerIntentBatch::new("third-reconstruction-root").push(MutationIntent::Create(
            CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: graph
                    .layout
                    .entity_kind(Account::reference().name())
                    .unwrap(),
                client_key: ClientKey::raw("account-3"),
                fields,
            }),
        )),
    );
}

/// Other roots have one primary child and no collections. The original root
/// retains two collections and an optional child, giving unequal read costs.
pub(super) fn add_primary_reads(world: &AuthorizationWorld, third: bool) {
    use crate::domain_computation::primary_graph::tests::fixture::{
        live_scope, AccountPrimaryActivity,
    };
    use crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionMode;
    use worth_relational::facade::transactions::{EntityReference, RelationSpec};
    let scope = live_scope();
    let selected = world.selected_product();
    let root = selected
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_owned(),
            &scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let graph = world.application.runtime.primary_graph().unwrap();
    let kind = graph
        .layout
        .relation(AccountPrimaryActivity::reference().name())
        .unwrap()
        .kind;
    let target = graph.integration_handle().with_runtime(|runtime| {
        runtime
            .read_truth()
            .visible_relations_of_kind(
                kind,
                selected.application_basis().snapshot_handle().version_id(),
            )
            .into_iter()
            .find(|relation| relation.source == root.entity_id())
            .unwrap()
            .target
    });
    let sources = ["unrelated", "third"]
        .into_iter()
        .take(if third { 2 } else { 1 })
        .map(|status| {
            selected
                .resolve_entity(
                    AccountStatus::reference(),
                    status.to_owned(),
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
                .entity_id()
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
    publish_relational_mutation(world, batch);
}
