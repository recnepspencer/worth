//! Restore authoring input before an abandoned preparation round can load.
use super::{
    BranchBoundRelationalTransaction, DetachedRelationalTransactionOverlay,
    RelationalTransactionFootprint,
};
use crate::authority::mutation::MutationApplyDenial;
use crate::runtime::PendingSymbolMeanings;

struct PreparationOverlayRound<'transaction> {
    transaction: &'transaction mut BranchBoundRelationalTransaction,
    overlay: DetachedRelationalTransactionOverlay,
    footprint: RelationalTransactionFootprint,
    cached_plan: Option<crate::transactions::data::MergedCommitPlan>,
}

impl Drop for PreparationOverlayRound<'_> {
    fn drop(&mut self) {
        std::mem::swap(&mut self.transaction.overlay, &mut self.overlay);
        std::mem::swap(&mut self.transaction.footprint, &mut self.footprint);
        std::mem::swap(
            &mut self.transaction.last_merged_plan,
            &mut self.cached_plan,
        );
    }
}

impl BranchBoundRelationalTransaction {
    pub(crate) fn with_preparation_overlay<T>(&mut self, round: impl FnOnce(&mut Self) -> T) -> T {
        let guard = PreparationOverlayRound {
            overlay: self.overlay.clone(),
            footprint: self.footprint.clone(),
            cached_plan: self.last_merged_plan.clone(),
            transaction: self,
        };
        round(&mut *guard.transaction)
    }

    pub(crate) fn reserve_preparation_client_keys(
        &self,
        symbols: &mut PendingSymbolMeanings,
        policy: crate::symbols::data::ClientKeySymbolPolicy,
        allocation: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), MutationApplyDenial> {
        if !policy.interns_requested_strings() {
            return Ok(());
        }
        let mut raw = std::collections::BTreeSet::new();
        for intent in self.batches().iter().flat_map(|batch| &batch.intents) {
            intent
                .collect_raw_client_keys(&mut raw, allocation)
                .map_err(|denial| {
                    crate::transactions::data::TransactionCommitError::conflict(
                        super::RelationalTransactionStagingDenial::from(denial).into_conflict(),
                    )
                })?;
        }
        self.footprint
            .collect_raw_client_keys(&mut raw, allocation)
            .map_err(|denial| {
                crate::transactions::data::TransactionCommitError::conflict(denial.into_conflict())
            })?;
        for text in raw {
            symbols.reserve(&text)?;
        }
        Ok(())
    }
}
