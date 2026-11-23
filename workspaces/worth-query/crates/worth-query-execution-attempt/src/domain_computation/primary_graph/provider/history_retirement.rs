//! History retirement behind a settled application publication.

use std::sync::Arc;

use worth_query_execution_runtime::facade::ports::ProductRuntimeInternalPort as _;
use worth_relational::facade::runtime::RelationalRuntime;

use super::WorthQueryPrimaryGraphProvider;
use crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime;

impl WorthQueryPrimaryGraphProvider {
    /// The provider reads one product World's history and follows every
    /// history release that product's branch and recovery cleanups make.
    pub fn install_world_history(self: &Arc<Self>, product: &WorthQueryProductRuntime) {
        assert!(
            self.world_history
                .set(product.owner_ref().recovery_port())
                .is_ok(),
            "one product World owns the provider's history"
        );
        let follower: std::sync::Weak<Self> = Arc::downgrade(self);
        product
            .owner_cleanup_ref()
            .follow_product_retention(follower);
    }

    /// World retires changed history interiors that nothing protects,
    /// splicing retired commits out from between kept ones, and the receipt
    /// bases of those commits go with it, so inspection of a retired commit
    /// is unavailable. Their completed evidence keeps only the
    /// publication's identities, so replay still answers. Relational then
    /// reclaims the roots the retired history no longer pins. A World that
    /// cannot retire keeps its history; nothing is lost but capacity.
    pub fn retire_changed_history(
        &self,
        runtime: &RelationalRuntime,
        committed: &worth_relational::facade::transactions::CommitResult,
    ) {
        let retired = acknowledged_retired_commits(
            self.world_history
                .get()
                .map(|world| world.retire_changed_history()),
        );
        self.apply_retired_world_commits(&retired);
        runtime.run_branch_root_reclamation_pass();
        let _ = runtime.run_index_generation_reclamation_after_publication(committed);
    }

    pub(crate) fn apply_retired_world_commits(
        &self,
        retired: &[worth_runtime_world::facade::CompositeCommitIdentity],
    ) {
        if retired.is_empty() {
            return;
        }
        let commits = self
            .completed_commit_evidence
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .retire(retired);
        let mut bases = self
            .receipt_basis_retention
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for commit in commits {
            bases.release_ordinary(commit);
        }
    }
}

fn acknowledged_retired_commits(
    result: Option<
        Result<
            Vec<worth_runtime_world::facade::CompositeCommitIdentity>,
            worth_runtime_world::facade::HistoryReclamationDenial,
        >,
    >,
) -> Vec<worth_runtime_world::facade::CompositeCommitIdentity> {
    acknowledged_from_retirement_result(result, |denial| match denial {
        worth_runtime_world::facade::HistoryReclamationDenial::PartialPublication {
            reclaimed,
            ..
        } => Some(reclaimed),
        _ => None,
    })
}

fn acknowledged_from_retirement_result<T, E>(
    result: Option<Result<Vec<T>, E>>,
    partial: impl FnOnce(E) -> Option<Vec<T>>,
) -> Vec<T> {
    match result {
        Some(Ok(retired)) => retired,
        Some(Err(denial)) => partial(denial).unwrap_or_default(),
        None => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_runtime_world::facade::HistoryReclamationDenial;

    #[test]
    fn ordinary_provider_consumes_acknowledged_prefix_on_partial_publication() {
        assert_eq!(
            acknowledged_from_retirement_result(Some(Ok(vec![1u64])), |_: ()| None),
            vec![1]
        );
        assert_eq!(
            acknowledged_from_retirement_result(Some(Err(Some(vec![1u64, 2u64]))), |p| p),
            vec![1, 2]
        );
        assert!(acknowledged_from_retirement_result::<u64, Option<Vec<u64>>>(
            Some(Err(None)),
            |p| p,
        )
        .is_empty());
        assert!(acknowledged_retired_commits(Some(Err(
            HistoryReclamationDenial::PartialPublication {
                reclaimed: Vec::new(),
                resume_from: None,
                indeterminate: true,
            },
        )))
        .is_empty());
        assert!(acknowledged_retired_commits(None).is_empty());
    }
}
