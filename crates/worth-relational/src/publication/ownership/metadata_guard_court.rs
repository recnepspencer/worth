//! Closed Native ordinary-write preflights, starting from Query's real publication.
use crate::{
    branch::AdmittedRelationalBranchBasis, identity::data::*, runtime::RelationalRuntime,
    storage::overlay::CurrentPartitionAccess, transactions::data::*,
};
use worth_execution::ExecutionRequest;
impl RelationalRuntime {
    #[doc(hidden)]
    pub fn probe_current_metadata_writes_for_test(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        owner: EntityId,
    ) -> Result<Vec<(KindId, u8, Result<(), TransactionCommitError>)>, TransactionCommitError> {
        let current = super::select_current(execution, basis, &[RecordRef::Entity(owner)])?;
        let storage = basis
            .inner
            .root
            .schema_authority()
            .registry()
            .current_publication_storage
            .as_ref()
            .ok_or_else(|| {
                crate::mvcc::origin_denial("real owned publication has its installed storage")
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
            let (_, record) = rows.iter().find(|(k, _)| *k == kind).ok_or_else(|| {
                crate::mvcc::origin_denial("court producer must realize every private kind")
            })?;
            for verb in 0..3 {
                let intent = match record {
                    RecordRef::Entity(id) => match verb {
                        0 => MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                            partition_id: id.partition_id,
                            kind_id: kind,
                            client_key: crate::symbols::data::ClientKey::raw(
                                "ordinary-private-create",
                            ),
                            fields: Default::default(),
                        })),
                        1 => MutationIntent::Entity(EntityMutationIntent::UpdateFields(
                            UpdateEntityFieldsIntent {
                                entity_id: *id,
                                fields: Default::default(),
                            },
                        )),
                        _ => MutationIntent::Entity(EntityMutationIntent::Delete(
                            DeleteEntityIntent { entity_id: *id },
                        )),
                    },
                    RecordRef::Relation(id) => {
                        let endpoints = basis
                            .inner
                            .root
                            .get_current_partition(id.partition_id)
                            .and_then(|p| p.relation_arena().get(id))
                            .and_then(|r| r.extra().endpoints.clone())
                            .ok_or_else(|| {
                                crate::mvcc::origin_denial("actual metadata link has endpoints")
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
                let mut tx = self
                    .begin_branch_transaction(
                        basis,
                        crate::mvcc::RelationalTransactionIntent::ordinary(),
                    )
                    .map_err(|e| crate::mvcc::origin_denial(format!("court transaction: {e:?}")))?;
                tx.push_batch(
                    WorkerIntentBatch::new("ordinary-private-write").push(intent),
                    execution.allocation_policy(),
                )
                .map_err(|e| TransactionCommitError::conflict(e.into_conflict()))?;
                answer.push((
                    kind,
                    verb,
                    self.prepare_branch_transaction(execution, tx).map(|_| ()),
                ));
            }
        }
        Ok(answer)
    }
}
