//! Native schema installation and ordinary independent reference authoring.
use super::super::super::fixture::installed_with;
use super::*;
use worth_relational::facade::schema::*;
pub(in super::super) fn installed_references(
    version: u8,
) -> (
    worth_relational::facade::runtime::RelationalRuntime,
    worth_relational::facade::runtime::RelationalInitialSchemaInstallationReceipt,
) {
    let integrity = RelationIntegrityDeclarations::new(
        vec![EndpointKindContractDeclaration {
            contract_id: ContractId::new("native.reference.endpoints"),
            allowed_source_kinds: vec![KindId(1), KindId(3)],
            allowed_target_kinds: vec![KindId(1), KindId(3)],
            self_edges_allowed: false,
            cross_context_policy: worth_relational::facade::config::CrossContextPolicy::Forbid,
        }],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    installed_with(
        registry_with_relation_integrity(version, integrity),
        Vec::new(),
    )
}

pub(super) fn add_reference(
    execution: &ExecutionRequest<'_, '_>,
    world: &RelatedWorld,
    source: EntityId,
    target: EntityId,
    key: &str,
) -> RelationId {
    let edge = CreatedRelationRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(4),
        client_key: worth_relational::facade::symbols::ClientKey::raw(key),
        source: EntityReference::Existing(source),
        target: EntityReference::Existing(target),
    };
    let mut transaction = world
        .runtime
        .begin_branch_transaction(
            &basis(&world.runtime),
            RelationalTransactionIntent::ordinary(),
        )
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new("ordinary authored reference").push(MutationIntent::Create(
                CreateIntent::Relation(RelationSpec {
                    partition_id: edge.partition_id,
                    kind_id: edge.kind_id,
                    client_key: edge.client_key.clone(),
                    source: edge.source.clone(),
                    target: edge.target.clone(),
                    fields: AspectFieldPatch::from_locator(
                        locator("edge"),
                        worth_foundational::facade::AspectValue::String(
                            worth_foundational::facade::InternedString::from(
                                "independent incoming placement",
                            ),
                        ),
                    ),
                }),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
    let committed = transaction.commit(execution, &world.runtime).unwrap();
    assert!(super::super::origin::single_origin(committed.envelope()).is_none());
    let id = committed.created_relation(&edge).unwrap();
    world
        .runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();
    id
}
