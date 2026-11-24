//! Closed Native ordinary-write preflights, starting from Query's real publication.
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::identity::data::{EntityId, KindId};
use crate::storage::overlay::CurrentPartitionAccess;
use crate::transactions::data::intents::mutation_intent::{
    CreateIntent, DeleteEntityIntent, DeleteRelationIntent, EntityMutationIntent, MutationIntent,
    RelationMutationIntent, UpdateEntityFieldsIntent, UpdateRelationEndpointsIntent,
};
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::transactions::data::primitives::{
    EntityReference, EntitySpec, RecordRef, RelationSpec, WorkerIntentBatch,
};
use worth_execution::ExecutionRequest;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasisFieldsPort as _;
use worth_relational_contracts::internal_port::schema::data::current_publication_storage::CurrentPublicationStorageOwnerPort as _;
use worth_relational_contracts::internal_port::schema::data::RelationalSchemaRegistryFieldsPort as _;
pub(crate) fn inspect_metadata_writes(
    execution: &ExecutionRequest<'_, '_>,
    basis: &AdmittedRelationalBranchBasis,
    owner: EntityId,
    mut prepare: impl FnMut(
        WorkerIntentBatch,
    ) -> Result<Result<(), TransactionCommitError>, TransactionCommitError>,
) -> Result<Vec<(KindId, u8, Result<(), TransactionCommitError>)>, TransactionCommitError> {
    let current = super::select_current(execution, basis, &[RecordRef::Entity(owner)])?;
    let storage = (*basis
        .owner_inner()
        .root
        .schema_authority()
        .registry()
        .owner_current_publication_storage())
    .as_ref()
    .ok_or_else(|| {
        crate::transactions::data::outcomes::transaction_commit_error::origin_denial(
            "real owned publication has its installed storage",
        )
    })?;
    let mut rows = Vec::new();
    rows.extend(
        current
            .catalogs
            .iter()
            .map(|(id, _)| (storage.catalog_kind, RecordRef::Entity(*id))),
    );
    rows.extend(
        current
            .publications
            .iter()
            .map(|r| (storage.publication_kind, RecordRef::Entity(r.entity))),
    );
    rows.extend(
        current
            .edges
            .iter()
            .map(|(id, _)| (storage.edge_kind, RecordRef::Entity(*id))),
    );
    rows.extend(
        current
            .positions
            .iter()
            .map(|(id, _)| (storage.position_kind, RecordRef::Entity(*id))),
    );
    rows.extend(
        current
            .links
            .iter()
            .map(|r| (r.kind, RecordRef::Relation(r.id))),
    );
    for publication in &current.publications {
        let claims = execution
            .run_native_with_checkpoint(|context| {
                super::read::publication_claims(
                    execution,
                    basis,
                    publication.entity,
                    publication.value.origin.commit.commit_id,
                    context,
                )
            })
            .map_err(super::request)?
            .0?;
        for link in &claims.links {
            rows.push((link.kind, RecordRef::Relation(link.id)));
            if link.kind == storage.edge_link {
                rows.push((storage.edge_kind, RecordRef::Entity(link.source)));
            }
        }
    }
    let mut answer = Vec::new();
    for kind in storage.metadata_kinds() {
        let realized = rows
            .iter()
            .find(|(k, _)| *k == kind)
            .map(|(_, record)| record);
        // An input link exists only where a producer consumed a parent
        // output. Without one, creation is still probed over the real
        // endpoints of another private link.
        let (record, verbs) = match realized {
            Some(record) => (record, 0..3),
            None if kind == storage.input_link => (
                rows.iter()
                    .map(|(_, record)| record)
                    .find(|record| matches!(record, RecordRef::Relation(_)))
                    .ok_or_else(|| {
                        crate::transactions::data::outcomes::transaction_commit_error::origin_denial("court producer must realize a private link")
                    })?,
                0..1,
            ),
            None => {
                return Err(crate::transactions::data::outcomes::transaction_commit_error::origin_denial(
                    "court producer must realize every private kind",
                ))
            }
        };
        for verb in verbs {
            let intent = match record {
                RecordRef::Entity(id) => match verb {
                    0 => MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                        partition_id: id.partition_id,
                        kind_id: kind,
                        client_key: crate::symbols::data::ClientKey::raw("ordinary-private-create"),
                        fields: Default::default(),
                    })),
                    1 => MutationIntent::Entity(EntityMutationIntent::UpdateFields(
                        UpdateEntityFieldsIntent {
                            entity_id: *id,
                            fields: Default::default(),
                        },
                    )),
                    _ => MutationIntent::Entity(EntityMutationIntent::Delete(DeleteEntityIntent {
                        entity_id: *id,
                    })),
                },
                RecordRef::Relation(id) => {
                    let endpoints = basis
                        .owner_inner()
                        .root
                        .get_current_partition(id.partition_id)
                        .and_then(|p| p.relation_arena().get(id))
                        .and_then(|r| r.extra().endpoints.clone())
                        .ok_or_else(|| {
                            crate::transactions::data::outcomes::transaction_commit_error::origin_denial("actual metadata link has endpoints")
                        })?;
                    match verb {
                        0 => MutationIntent::Create(CreateIntent::Relation(RelationSpec {
                            partition_id: id.partition_id,
                            kind_id: kind,
                            client_key: crate::symbols::data::ClientKey::raw(
                                "ordinary-private-create",
                            ),
                            source: EntityReference::Existing(endpoints.source),
                            target: EntityReference::Existing(endpoints.target),
                            fields: Default::default(),
                        })),
                        1 => MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(
                            UpdateRelationEndpointsIntent {
                                relation_id: *id,
                                kind_id: kind,
                                source: EntityReference::Existing(endpoints.source),
                                target: EntityReference::Existing(endpoints.target),
                            },
                        )),
                        _ => MutationIntent::Relation(RelationMutationIntent::Delete(
                            DeleteRelationIntent { relation_id: *id },
                        )),
                    }
                }
            };
            let batch = WorkerIntentBatch::new("ordinary-private-write").push(intent);
            answer.push((kind, verb, prepare(batch)?));
        }
    }
    Ok(answer)
}
