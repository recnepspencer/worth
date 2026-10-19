//! Native schema installation and ordinary independent reference authoring.
use super::super::super::fixture::installed_with;
use super::*;
use crate::schema::data::*;
pub(in super::super) fn installed_references(
    version: u8,
) -> (
    crate::runtime::RelationalRuntime,
    crate::runtime::RelationalInitialSchemaInstallationReceipt,
) {
    let mut schema = registry(version);
    let relation = schema.relation_kinds.get_mut(&KindId(4)).unwrap();
    relation.relation_integrity = RelationIntegrityDeclarations::new(
        vec![EndpointKindContractDeclaration {
            contract_id: ContractId::new("native.reference.endpoints"),
            allowed_source_kinds: vec![KindId(1), KindId(3)],
            allowed_target_kinds: vec![KindId(1), KindId(3)],
            self_edges_allowed: false,
            cross_context_policy: crate::config::data::CrossContextPolicy::Forbid,
        }],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    installed_with(schema, Vec::new())
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
        client_key: crate::symbols::data::ClientKey::raw(key),
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
    assert!(committed
        .publication()
        .envelope
        .required_output_origin
        .is_none());
    let id = committed.created_relation(&edge).unwrap();
    world
        .runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();
    id
}
