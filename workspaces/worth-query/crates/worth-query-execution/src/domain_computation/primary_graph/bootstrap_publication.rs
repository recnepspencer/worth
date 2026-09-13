use std::collections::BTreeMap;

use worth_query_declaration::facade::authentication::{
    WorthQueryExternalPrincipalIdentityBinding, WorthQueryPrincipalMappingStatusBinding,
};
use worth_query_installation::facade::ApplicationScalarValueBinding;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::indexes::{DerivedIndexBuildRequest, DerivedIndexId};
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{
    AspectFieldPatch, CreateIntent, CreatedEntityRef, EntityReference, EntitySpec, MutationIntent,
    RelationSpec, WorkerIntentBatch,
};

use super::bootstrap::WorthQueryPrincipalBootstrapRow;
use super::typed_bootstrap::{
    WorthQueryTypedEntityBootstrapRow, WorthQueryTypedRelationBootstrapRow,
};
use super::{
    primary_relational_branch_id, WorthQueryPrimaryGraph, WorthQueryPrimaryGraphInstallationDenial,
    WorthQueryPrimaryGraphInstallationDenialKind,
};

pub(super) fn commit_bootstrap_rows(
    graph: &WorthQueryPrimaryGraph,
    rows: Vec<WorthQueryPrincipalBootstrapRow>,
    entity_rows: Vec<WorthQueryTypedEntityBootstrapRow>,
    relation_rows: Vec<WorthQueryTypedRelationBootstrapRow>,
) -> Result<worth_relational::facade::history::CommitId, WorthQueryPrimaryGraphInstallationDenial> {
    graph.integration_handle().with_runtime_mut(|runtime| {
        let main_identity = runtime.main_branch_identity();
        let options = runtime
            .admit_branch_basis(&main_identity)
            .map_err(map_bootstrap_basis_denial)?;
        let mut transaction = runtime
            .begin_branch_transaction(
                &options,
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            )
            .map_err(map_bootstrap_transaction_admission_denial)?;
        let mut batch = WorkerIntentBatch::new("application-principal-bootstrap");
        for (ordinal, row) in rows.into_iter().enumerate() {
            batch = append_principal_row(batch, ordinal, row);
        }
        for row in entity_rows {
            batch = append_typed_entity(batch, row);
        }
        for row in relation_rows {
            batch = append_typed_relation(batch, row);
        }
        transaction
            .push_batch(batch)
            .map_err(map_bootstrap_staging_denial)?;
        let committed = transaction
            .commit(runtime)
            .map_err(map_bootstrap_commit_denial)?;
        let commit_id = committed.commit.commit_id;
        crate::relational_snapshot_release::release_query_snapshot(runtime, &committed.snapshot);
        Ok(commit_id)
    })
}

fn map_bootstrap_basis_denial(
    denial: worth_relational::facade::branch::RelationalBranchBasisDenial,
) -> WorthQueryPrimaryGraphInstallationDenial {
    let kind = match denial {
        worth_relational::facade::branch::RelationalBranchBasisDenial::RetentionCapacityExhausted => {
            WorthQueryPrimaryGraphInstallationDenialKind::RetentionCapacityExhausted
        }
        worth_relational::facade::branch::RelationalBranchBasisDenial::RetentionIdentityExhausted => {
            WorthQueryPrimaryGraphInstallationDenialKind::RetentionIdentityExhausted
        }
        worth_relational::facade::branch::RelationalBranchBasisDenial::SnapshotIdentityExhausted => {
            WorthQueryPrimaryGraphInstallationDenialKind::SnapshotIdentityExhausted
        }
        _ => WorthQueryPrimaryGraphInstallationDenialKind::RelationalCommitRejected,
    };
    primary_graph_denial(
        kind,
        format!("Relational bootstrap basis denied: {denial:?}"),
    )
}

fn map_bootstrap_transaction_admission_denial(
    denial: worth_relational::facade::mvcc::RelationalBranchTransactionAdmissionDenial,
) -> WorthQueryPrimaryGraphInstallationDenial {
    use worth_relational::facade::mvcc::RelationalBranchTransactionAdmissionDenial as Denial;
    let kind = match denial {
        Denial::RetentionCapacityExhausted => {
            WorthQueryPrimaryGraphInstallationDenialKind::RetentionCapacityExhausted
        }
        Denial::RetentionIdentityExhausted => {
            WorthQueryPrimaryGraphInstallationDenialKind::RetentionIdentityExhausted
        }
        _ => WorthQueryPrimaryGraphInstallationDenialKind::RelationalCommitRejected,
    };
    primary_graph_denial(
        kind,
        format!("Relational bootstrap transaction admission denied: {denial:?}"),
    )
}

fn map_bootstrap_staging_denial(
    denial: worth_relational::facade::mvcc::RelationalTransactionStagingDenial,
) -> WorthQueryPrimaryGraphInstallationDenial {
    use worth_relational::facade::mvcc::RelationalTransactionStagingDenial as Denial;
    let kind = match denial {
        Denial::OverlayCapacityExhausted {
            maximum_bytes,
            required_bytes,
        } => WorthQueryPrimaryGraphInstallationDenialKind::TransactionOverlayCapacityExhausted {
            maximum_bytes,
            required_bytes,
        },
        Denial::FootprintCapacityExhausted {
            maximum_loci,
            required_loci,
        } => WorthQueryPrimaryGraphInstallationDenialKind::TransactionFootprintCapacityExhausted {
            maximum_loci,
            required_loci,
        },
        Denial::SavepointCapacityExhausted { maximum_savepoints } => {
            WorthQueryPrimaryGraphInstallationDenialKind::SavepointCapacityExhausted {
                maximum_savepoints,
            }
        }
        Denial::SavepointFootprintCapacityExhausted {
            maximum_loci,
            required_loci,
        } => WorthQueryPrimaryGraphInstallationDenialKind::SavepointFootprintCapacityExhausted {
            maximum_loci,
            required_loci,
        },
        Denial::SavepointIdentityExhausted => {
            WorthQueryPrimaryGraphInstallationDenialKind::SavepointIdentityExhausted
        }
    };
    primary_graph_denial(
        kind,
        format!("Relational bootstrap staging denied: {denial:?}"),
    )
}

fn map_bootstrap_commit_denial(
    error: worth_relational::facade::transactions::TransactionCommitError,
) -> WorthQueryPrimaryGraphInstallationDenial {
    use worth_relational::facade::mvcc::{
        RelationalPublicationDeferred as Deferred, RelationalPublicationFailureKind as Failure,
    };
    use worth_relational::facade::transactions::{
        CommitPreparationReason, TransactionCommitError as Error,
    };
    let kind = match &error {
        Error::PublicationDeferred { deferred, .. } => match deferred {
            Deferred::PatchPositionReservationContended => {
                WorthQueryPrimaryGraphInstallationDenialKind::PatchPositionReservationContended
            }
            Deferred::RetentionBackpressure => {
                WorthQueryPrimaryGraphInstallationDenialKind::RetentionCapacityExhausted
            }
            Deferred::CandidateCapacityExhausted { maximum_candidates } => {
                WorthQueryPrimaryGraphInstallationDenialKind::CandidateCapacityExhausted {
                    maximum_candidates: *maximum_candidates,
                }
            }
            Deferred::PublishedSnapshotCapacityExhausted { maximum_handles } => {
                WorthQueryPrimaryGraphInstallationDenialKind::PublishedSnapshotCapacityExhausted {
                    maximum_handles: *maximum_handles,
                }
            }
            Deferred::CandidateLifetimeExpired { .. } => {
                WorthQueryPrimaryGraphInstallationDenialKind::RelationalCommitRejected
            }
        },
        Error::PublicationFailed { failure, .. } => match failure.kind() {
            Failure::SnapshotIdentityExhausted => {
                WorthQueryPrimaryGraphInstallationDenialKind::SnapshotIdentityExhausted
            }
            Failure::CandidateIdentityExhausted => {
                WorthQueryPrimaryGraphInstallationDenialKind::CandidateIdentityExhausted
            }
            Failure::RetentionIdentityExhausted => {
                WorthQueryPrimaryGraphInstallationDenialKind::RetentionIdentityExhausted
            }
            Failure::PreparedRootBudgetExhausted {
                maximum_bytes,
                required_bytes,
            } => WorthQueryPrimaryGraphInstallationDenialKind::PreparedRootBudgetExhausted {
                maximum_bytes: *maximum_bytes,
                required_bytes: *required_bytes,
            },
            _ => WorthQueryPrimaryGraphInstallationDenialKind::RelationalCommitRejected,
        },
        Error::Preparation { error, .. }
            if error.reason() == CommitPreparationReason::ProposalIdentityOrdinalExhausted =>
        {
            WorthQueryPrimaryGraphInstallationDenialKind::ProposalIdentityExhausted
        }
        _ => WorthQueryPrimaryGraphInstallationDenialKind::RelationalCommitRejected,
    };
    primary_graph_denial(
        kind,
        format!(
            "Relational rejected the application principal bootstrap transaction: {}",
            error.detail()
        ),
    )
}

fn append_typed_entity(
    batch: WorkerIntentBatch,
    row: WorthQueryTypedEntityBootstrapRow,
) -> WorkerIntentBatch {
    batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
        partition_id: PartitionId::main(),
        kind_id: row.kind,
        client_key: ClientKey::raw(row.key),
        fields: AspectFieldPatch::from(row.fields),
    })))
}

fn append_typed_relation(
    batch: WorkerIntentBatch,
    row: WorthQueryTypedRelationBootstrapRow,
) -> WorkerIntentBatch {
    batch.push(MutationIntent::Create(CreateIntent::Relation(
        RelationSpec {
            partition_id: PartitionId::main(),
            kind_id: row.kind,
            client_key: ClientKey::raw(row.key),
            source: EntityReference::Created(CreatedEntityRef {
                partition_id: PartitionId::main(),
                kind_id: row.from_kind,
                client_key: ClientKey::raw(row.from_key),
            }),
            target: EntityReference::Created(CreatedEntityRef {
                partition_id: PartitionId::main(),
                kind_id: row.to_kind,
                client_key: ClientKey::raw(row.to_key),
            }),
            fields: AspectFieldPatch::default(),
        },
    )))
}

fn append_principal_row(
    batch: WorkerIntentBatch,
    ordinal: usize,
    row: WorthQueryPrincipalBootstrapRow,
) -> WorkerIntentBatch {
    let partition_id = PartitionId::main();
    let principal_client_key = ClientKey::raw(row.principal_key);
    let mapping_client_key = ClientKey::raw(format!("{}-mapping-{ordinal}", row.binding));
    let principal = CreatedEntityRef {
        partition_id,
        kind_id: row.layout.principal_kind,
        client_key: principal_client_key.clone(),
    };
    let mapping = CreatedEntityRef {
        partition_id,
        kind_id: row.layout.mapping_kind,
        client_key: mapping_client_key.clone(),
    };
    let mapping_fields = BTreeMap::from([
        (
            row.layout.identity_locator,
            WorthQueryExternalPrincipalIdentityBinding::encode(&row.identity)
                .expect("admitted external principal identity remains encodable"),
        ),
        (
            row.layout.status_locator,
            WorthQueryPrincipalMappingStatusBinding::encode(&row.status)
                .expect("declared principal mapping status remains encodable"),
        ),
    ]);
    batch
        .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id,
            kind_id: principal.kind_id,
            client_key: principal_client_key,
            fields: AspectFieldPatch::from(BTreeMap::from([(
                row.layout.principal_identity_locator.clone(),
                row.principal_identity,
            )])),
        })))
        .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id,
            kind_id: mapping.kind_id,
            client_key: mapping_client_key,
            fields: AspectFieldPatch::from(mapping_fields),
        })))
        .push(MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id,
                kind_id: row.layout.relation_kind,
                client_key: ClientKey::raw(format!("{}-target-{ordinal}", row.binding)),
                source: EntityReference::Created(mapping),
                target: EntityReference::Created(principal),
                fields: AspectFieldPatch::default(),
            },
        )))
}

pub(super) fn build_identity_indexes(
    graph: &WorthQueryPrimaryGraph,
    source_commit_id: worth_relational::facade::history::CommitId,
    index_ids: &[DerivedIndexId],
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    graph.integration_handle().with_runtime_mut(|runtime| {
        let build = runtime
            .index_authority()
            .build_for_commit(DerivedIndexBuildRequest {
                source_commit_id,
                branch_id: primary_relational_branch_id(),
                index_ids: index_ids.to_vec(),
            });
        if build.failed_indexes.is_empty() && build.generations.len() == index_ids.len() {
            Ok(())
        } else {
            Err(primary_graph_denial(
                WorthQueryPrimaryGraphInstallationDenialKind::IndexBuildRejected,
                format!(
                    "{} of {} identity indexes failed publication",
                    build.failed_indexes.len(),
                    index_ids.len()
                ),
            ))
        }
    })
}

fn primary_graph_denial(
    kind: WorthQueryPrimaryGraphInstallationDenialKind,
    subject: impl Into<String>,
) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(kind, subject)
}
