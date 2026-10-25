//! Declared record and adjacency demand before preparation reads substrate.
use crate::storage::residency::RelationalResidencyDemandFailure;
use crate::transactions::data::{EntityMutationIntent, ExistingRecordTarget, MutationIntent};
use std::collections::BTreeSet;

impl super::BranchBoundRelationalTransaction {
    pub(crate) fn demand_preparation_regions(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        symbols: &crate::runtime::RuntimeSymbolTable,
        indexes: &crate::runtime::IndexDefinitionReadBinding,
        mirror: &crate::runtime::MirrorResidencyDemandBinding,
    ) -> Result<(), crate::transactions::data::TransactionCommitError> {
        let mut raw = BTreeSet::new();
        if symbols.has_cold_pages() && self.client_key_symbol_policy.interns_requested_strings() {
            for intent in self.batches().iter().flat_map(|batch| &batch.intents) {
                intent
                    .collect_raw_client_keys(&mut raw, execution.allocation_policy())
                    .map_err(|denial| {
                        crate::transactions::data::TransactionCommitError::conflict(
                            super::RelationalTransactionStagingDenial::from(denial).into_conflict(),
                        )
                    })?;
            }
            self.footprint
                .collect_raw_client_keys(&mut raw, execution.allocation_policy())
                .map_err(|denial| {
                    crate::transactions::data::TransactionCommitError::conflict(
                        denial.into_conflict(),
                    )
                })?;
        }
        let admitted = (|| {
            indexes.demand_checkpoint_image(execution)?;
            for value in raw {
                symbols.demand_text(execution, &value)?;
            }
            self.demand_declared_regions(execution, symbols, mirror)
        })();
        admitted.map_err(|failure| match failure {
            RelationalResidencyDemandFailure::StorageUnavailable(failure) => {
                crate::transactions::data::TransactionCommitError::preparation(
                    crate::transactions::data::CommitPreparationError::storage_unavailable(
                        self.basis.identity().branch_id().clone(),
                        self.basis.observation().commit_id().map(|id| id.0),
                        self.basis.observation().version_id(),
                        failure,
                    ),
                )
            }
            RelationalResidencyDemandFailure::Request(cause) => {
                crate::transactions::data::TransactionCommitError::execution(
                    crate::transactions::data::CommitExecutionDenial {
                        kind: crate::transactions::data::CommitExecutionDenialKind::Cause(cause),
                        partition_identity: None,
                    },
                )
            }
        })
    }

    fn demand_declared_regions(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        symbols: &crate::runtime::RuntimeSymbolTable,
        mirror: &crate::runtime::MirrorResidencyDemandBinding,
    ) -> Result<(), RelationalResidencyDemandFailure> {
        let root = &self.basis.inner.root;
        let mut partitions = self.footprint.validation_partitions();
        // A new schema carrier reframes every retained region, including
        // unchanged records. Admit those contents during the operation;
        // capture itself must never reconstruct a missing member.
        if self.schema_authority_input.as_ref().is_some_and(|input| {
            crate::schema::data::schema_authority_snapshot_digest_bytes(
                input.target_schema_authority(),
            ) != root.schema_authority().authority_digest()
                || input.descriptor_semantics_version()
                    != root.schema_authority().descriptor_semantics_version()
        }) {
            partitions.extend(root.partition_ids_iter());
        }
        let mut relations = BTreeSet::new();
        for intent in self.batches().iter().flat_map(|batch| &batch.intents) {
            intent.seed_touched_partitions(&mut partitions);
        }
        for intent in self.batches().iter().flat_map(|batch| &batch.intents) {
            match intent.existing_record_target() {
                Some(ExistingRecordTarget::Relation(id)) => {
                    relations.insert(id);
                }
                Some(ExistingRecordTarget::Entity(id))
                    if matches!(
                        intent,
                        MutationIntent::Entity(
                            EntityMutationIntent::Delete(_) | EntityMutationIntent::Replace(_)
                        )
                    ) =>
                {
                    if let Some(lease) = root.demand_region_head(execution, id.partition_id)? {
                        relations.extend(lease.head_view().outgoing(id).iter().copied());
                        relations.extend(lease.head_view().incoming(id).iter().copied());
                    }
                }
                _ => {}
            }
        }
        for id in relations {
            partitions.insert(id.partition_id);
            if let Some(lease) = root.demand_region_head(execution, id.partition_id)?
                && let Some(endpoints) = lease
                    .head_view()
                    .relation(id)
                    .and_then(|slot| slot.extra().endpoints.clone())
            {
                partitions.insert(endpoints.source.partition_id);
                partitions.insert(endpoints.target.partition_id);
            }
        }
        mirror.demand(
            execution,
            self.basis.identity().branch_id(),
            partitions.iter().copied(),
        )?;
        root.demand_regions_full(execution, symbols, partitions)
    }
}
