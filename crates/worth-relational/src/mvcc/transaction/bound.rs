use crate::transactions::data::{MergedCommitPlan, TransactionId, WorkerIntentBatch};

use super::{
    DetachedRelationalTransactionOverlay, RelationalTransactionFootprint,
    RelationalTransactionIntent,
};

/// Move-only detached transaction bound to one owner-admitted exact basis.
/// It contains no reference to `RelationalRuntime`.
#[derive(Debug)]
pub struct BranchBoundRelationalTransaction {
    pub(crate) basis: crate::branch::AdmittedRelationalBranchBasis,
    pub(crate) mutation_authority: crate::branch::RelationalBranchMutationAuthority,
    pub(crate) transaction_id: TransactionId,
    pub(crate) intent: RelationalTransactionIntent,
    pub(crate) merge_parent_bases: Vec<crate::branch::AdmittedRelationalBranchBasis>,
    pub(crate) schema_authority_input: Option<crate::schema::SchemaContinuityAuthorityInput>,
    pub(crate) schema_authority: std::sync::Arc<crate::branch::RelationalBranchRootSchemaAuthority>,
    pub(crate) overlay: DetachedRelationalTransactionOverlay,
    pub(crate) overlay_bytes: u64,
    pub(crate) maximum_overlay_bytes: u64,
    pub(crate) maximum_footprint_loci: usize,
    pub(crate) maximum_savepoints: usize,
    pub(crate) savepoint_footprint_loci: usize,
    pub(crate) footprint: RelationalTransactionFootprint,
    pub(crate) savepoints: Vec<super::RelationalTransactionSavepoint>,
    pub(crate) next_savepoint_ordinal: u64,
    pub(crate) last_merged_plan: Option<MergedCommitPlan>,
    pub(crate) client_key_symbol_policy: crate::symbols::data::ClientKeySymbolPolicy,
    pub(crate) retention:
        std::sync::Arc<crate::history::retention::RelationalTransactionRetentionObligation>,
    pub(crate) control: crate::mvcc::RelationalOperationControl,
}

impl BranchBoundRelationalTransaction {
    pub fn transaction_id(&self) -> TransactionId {
        self.transaction_id
    }

    pub fn basis(&self) -> &crate::branch::AdmittedRelationalBranchBasis {
        &self.basis
    }

    pub fn footprint(&self) -> &RelationalTransactionFootprint {
        &self.footprint
    }

    /// Stages one owned input batch under an explicit physical allocation policy.
    /// Index/footprint backing is admitted before semantic publication; nested
    /// input/key heaps and the existing input directory remain separate owners.
    pub fn push_batch(
        &mut self,
        batch: WorkerIntentBatch,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), super::RelationalTransactionStagingDenial> {
        policy.check_live()?;
        self.admit_materialization_batch(&batch)?;
        let required_bytes = self
            .overlay_bytes
            .saturating_add(batch.resident_capacity_bytes());
        if required_bytes > self.maximum_overlay_bytes {
            return Err(
                super::RelationalTransactionStagingDenial::OverlayCapacityExhausted {
                    maximum_bytes: self.maximum_overlay_bytes,
                    required_bytes,
                },
            );
        }
        let (index, footprint) = self.overlay.prepare_stage(
            &batch,
            &self.footprint,
            self.maximum_footprint_loci,
            policy,
        )?;
        self.overlay.reserve_input_directory()?;
        policy.check_live()?;
        self.overlay.stage(batch, index);
        self.footprint = footprint;
        self.overlay_bytes = required_bytes;
        self.last_merged_plan = None;
        Ok(())
    }

    fn admit_materialization_batch(
        &self,
        batch: &WorkerIntentBatch,
    ) -> Result<(), super::RelationalTransactionStagingDenial> {
        let mode = self.intent.materialization_mode();
        for intent in &batch.intents {
            let crate::transactions::data::MutationIntent::Materialization(intent) = intent else {
                if mode.is_some() {
                    return Err(
                        super::RelationalTransactionStagingDenial::MaterializationModeMismatch,
                    );
                }
                continue;
            };
            let Some(mode) = mode else {
                return Err(
                    super::RelationalTransactionStagingDenial::MaterializationAuthorityRequired,
                );
            };
            let matches_mode = match mode {
                super::RelationalMaterializationTransactionMode::Suspend => intent.is_suspension(),
                super::RelationalMaterializationTransactionMode::Rematerialize => {
                    intent.is_rematerialization()
                }
            };
            if !matches_mode {
                return Err(super::RelationalTransactionStagingDenial::MaterializationModeMismatch);
            }
        }
        Ok(())
    }

    pub(crate) fn batches(&self) -> &[WorkerIntentBatch] {
        self.overlay.batches()
    }

    pub fn commit(
        self,
        runtime: &crate::runtime::RelationalRuntime,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        crate::transactions::data::CommitResult,
        crate::transactions::data::TransactionCommitError,
    > {
        runtime.commit_branch_transaction(self, allocation_policy)
    }

    pub fn validate(
        self,
        runtime: &crate::runtime::RelationalRuntime,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        crate::mvcc::ValidatedRelationalProposal,
        crate::transactions::data::TransactionCommitError,
    > {
        runtime.validate_branch_transaction(self, allocation_policy)
    }
}
