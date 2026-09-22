use crate::transactions::data::{
    CommitConflict, ConflictClass, RollbackOutcome, RollbackSummary, SavepointId,
};

#[derive(Clone, Debug)]
pub(crate) struct RelationalTransactionSavepoint {
    id: SavepointId,
    retained_batch_count: usize,
    footprint: super::RelationalTransactionFootprint,
}

impl RelationalTransactionSavepoint {
    pub(crate) fn new(
        id: SavepointId,
        retained_batch_count: usize,
        footprint: super::RelationalTransactionFootprint,
    ) -> Self {
        Self {
            id,
            retained_batch_count,
            footprint,
        }
    }

    pub(crate) const fn id(&self) -> SavepointId {
        self.id
    }

    pub(crate) const fn retained_batch_count(&self) -> usize {
        self.retained_batch_count
    }

    pub(crate) fn footprint(&self) -> &super::RelationalTransactionFootprint {
        &self.footprint
    }

    pub(crate) fn footprint_loci(&self) -> usize {
        self.footprint.total_locus_count()
    }
}

impl super::BranchBoundRelationalTransaction {
    pub fn create_savepoint(
        &mut self,
    ) -> Result<SavepointId, super::RelationalTransactionStagingDenial> {
        assert!(
            self.intent.allow_nested_savepoints(),
            "nested savepoints are disabled for this transaction"
        );
        if self.savepoints.len() >= self.maximum_savepoints {
            return Err(
                super::RelationalTransactionStagingDenial::SavepointCapacityExhausted {
                    maximum_savepoints: self.maximum_savepoints,
                },
            );
        }
        let savepoint_id = SavepointId(self.next_savepoint_ordinal);
        let next_savepoint_ordinal = self
            .next_savepoint_ordinal
            .checked_add(1)
            .ok_or(super::RelationalTransactionStagingDenial::SavepointIdentityExhausted)?;
        let footprint_loci = self.footprint.total_locus_count();
        let required_loci = self
            .savepoint_footprint_loci
            .checked_add(footprint_loci)
            .ok_or(
                super::RelationalTransactionStagingDenial::SavepointFootprintCapacityExhausted {
                    maximum_loci: self.maximum_footprint_loci,
                    required_loci: usize::MAX,
                },
            )?;
        if required_loci > self.maximum_footprint_loci {
            return Err(
                super::RelationalTransactionStagingDenial::SavepointFootprintCapacityExhausted {
                    maximum_loci: self.maximum_footprint_loci,
                    required_loci,
                },
            );
        }
        self.savepoints.push(RelationalTransactionSavepoint::new(
            savepoint_id,
            self.batches().len(),
            self.footprint.clone(),
        ));
        self.savepoint_footprint_loci = required_loci;
        self.next_savepoint_ordinal = next_savepoint_ordinal;
        Ok(savepoint_id)
    }

    pub fn rollback_to_savepoint(
        &mut self,
        savepoint_id: SavepointId,
    ) -> Result<RollbackOutcome, CommitConflict> {
        let Some(index) = self
            .savepoints
            .iter()
            .position(|candidate| candidate.id() == savepoint_id)
        else {
            return Err(CommitConflict::new(ConflictClass::InvalidSavepoint {
                savepoint_id,
            }));
        };
        let batch_len = self.savepoints[index].retained_batch_count();
        let restored_footprint = self.savepoints[index].footprint().clone();
        let released_savepoint_loci = self.savepoints[index..]
            .iter()
            .map(RelationalTransactionSavepoint::footprint_loci)
            .sum::<usize>();
        let drained = self
            .overlay
            .truncate_batches(batch_len, &mut self.footprint, &self.basis);
        let released_bytes = drained
            .iter()
            .map(crate::transactions::data::WorkerIntentBatch::resident_capacity_bytes)
            .sum::<u64>();
        self.overlay_bytes = self.overlay_bytes.saturating_sub(released_bytes);
        self.footprint = restored_footprint;
        self.last_merged_plan = None;
        self.savepoints.truncate(index);
        self.savepoint_footprint_loci = self
            .savepoint_footprint_loci
            .saturating_sub(released_savepoint_loci);
        let effects = drained
            .into_iter()
            .flat_map(|batch| batch.intents.into_iter())
            .filter_map(|intent| intent.rollback_effect())
            .collect::<Vec<_>>();
        let summary = RollbackSummary::from_effects(&effects);
        Ok(RollbackOutcome {
            transaction_id: self.transaction_id,
            summary,
            effects,
        })
    }
}
