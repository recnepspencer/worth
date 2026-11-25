use worth_runtime_bridge::facade::{
    BridgeCorrespondenceDeliveryReceipt, BridgeGranularInvalidationDelivery,
};

use crate::domain_computation::primary_graph::WorthQueryGranularInvalidationCoverage;

/// Granular invalidations one evaluation binding owes its next accepted clock
/// observation.
///
/// A Query-performed product change waits as the direct delivery until an
/// observation reconsiders it. Invalidations consumed from the subscription
/// stay owed until an accepted or duplicate reading emits them: the commit
/// cursor has already moved past their commits, so a refused or failed
/// observation must never drop them.
///
/// Owed invalidations stay bounded while readings keep failing and commits
/// continue. Deliveries from distinct commits with an identical admitted
/// impact keep only the newest commit. A dependency carried past a reading that did not accept it
/// takes no further distinct invalidation; one escalates the binding to
/// `RefreshAll`, which absorbs everything owed after it.
pub(in crate::domain_computation::primary_graph::conditional_operation) struct WorthQueryPendingGranularInvalidations
{
    direct: Vec<BridgeGranularInvalidationDelivery>,
    owed: Vec<BridgeGranularInvalidationDelivery>,
    /// `owed[..carried]` was already owed before the current observation.
    carried: usize,
    coverage: WorthQueryGranularInvalidationCoverage,
}

/// The owed invalidations one accepted observation emits.
pub(super) struct WorthQueryReleasedGranularInvalidations {
    pub(super) deliveries: Vec<BridgeGranularInvalidationDelivery>,
    pub(super) coverage: WorthQueryGranularInvalidationCoverage,
}

impl WorthQueryPendingGranularInvalidations {
    pub(in crate::domain_computation::primary_graph::conditional_operation) const fn empty() -> Self
    {
        Self {
            direct: Vec::new(),
            owed: Vec::new(),
            carried: 0,
            coverage: WorthQueryGranularInvalidationCoverage::Exact,
        }
    }

    pub(super) fn retain_direct(&mut self, receipt: &BridgeCorrespondenceDeliveryReceipt) {
        let incoming = receipt.change_set();
        assert!(
            !self.direct.iter().any(|existing| {
                let change = existing.correspondence_receipt().change_set();
                change.commit_identity() == incoming.commit_identity()
                    && change.dependency().dependency_ordinal()
                        == incoming.dependency().dependency_ordinal()
            }),
            "one exact product occurrence can retain only its unique performed change"
        );
        self.direct
            .push(BridgeGranularInvalidationDelivery::direct(receipt));
    }

    /// The Query-performed deliveries the next observation must not deliver
    /// again and must reconsider retained wakes against.
    pub(in crate::domain_computation::primary_graph::conditional_operation) fn direct(
        &self,
    ) -> &[BridgeGranularInvalidationDelivery] {
        &self.direct
    }

    /// Starts an observation: everything still owed was carried past a
    /// reading that did not accept it.
    pub(super) fn carry_owed(&mut self) {
        self.carried = self.owed.len();
    }

    /// Records invalidations consumed from the subscription.
    pub(in crate::domain_computation::primary_graph::conditional_operation) fn owe(
        &mut self,
        deliveries: impl IntoIterator<Item = BridgeGranularInvalidationDelivery>,
    ) {
        for delivery in deliveries {
            self.owe_one(delivery);
        }
    }

    /// Removes and returns only the direct deliveries whose exact commit has
    /// just been passed by the observation cursor, leaving later preperformed
    /// commits retained as deduplication evidence across bounded partial scans.
    pub(in crate::domain_computation::primary_graph::conditional_operation) fn take_direct_for_commit(
        &mut self,
        commit: worth_relational::facade::history::CommitId,
    ) -> Vec<BridgeGranularInvalidationDelivery> {
        let target =
            worth_runtime_bridge::facade::TruthCommitIdentity::from_relational_commit_id(commit.0);
        let mut matched = Vec::new();
        let mut remaining = Vec::new();
        for delivery in std::mem::take(&mut self.direct) {
            if delivery
                .correspondence_receipt()
                .change_set()
                .commit_identity()
                == &target
            {
                matched.push(delivery);
            } else {
                remaining.push(delivery);
            }
        }
        self.direct = remaining;
        matched
    }

    /// Owes the reconsidered direct deliveries alongside consumed ones.
    pub(super) fn owe_direct(&mut self) {
        for delivery in std::mem::take(&mut self.direct) {
            self.owe_one(delivery);
        }
    }

    fn owe_one(&mut self, delivery: BridgeGranularInvalidationDelivery) {
        if self.coverage == WorthQueryGranularInvalidationCoverage::RefreshAll {
            return;
        }
        if let Some(owed) = self
            .owed
            .iter_mut()
            .find(|owed| same_invalidation(owed, &delivery))
        {
            if snapshot_version(&delivery) >= snapshot_version(owed) {
                *owed = delivery;
            }
            return;
        }
        let dependency = delivery.truth().change_set().dependency();
        if self.owed[..self.carried]
            .iter()
            .any(|owed| owed.truth().change_set().dependency() == dependency)
        {
            self.refresh_all();
            return;
        }
        self.owed.push(delivery);
    }

    /// Every consumer refreshes its full scope, which covers each owed
    /// invalidation, so nothing further is owed exactly. The binding was
    /// rebuilt after losing subscription continuity, or a carried dependency
    /// kept changing while readings were refused.
    pub(super) fn refresh_all(&mut self) {
        self.coverage = WorthQueryGranularInvalidationCoverage::RefreshAll;
        self.owed.clear();
        self.carried = 0;
    }

    pub(super) fn release(&mut self) -> WorthQueryReleasedGranularInvalidations {
        self.carried = 0;
        WorthQueryReleasedGranularInvalidations {
            deliveries: std::mem::take(&mut self.owed),
            coverage: std::mem::replace(
                &mut self.coverage,
                WorthQueryGranularInvalidationCoverage::Exact,
            ),
        }
    }

    /// Restores released invalidations when an accepted observation cannot
    /// complete persistence and therefore cannot return a receipt to the caller.
    pub(super) fn restore_released(&mut self, released: WorthQueryReleasedGranularInvalidations) {
        if released.coverage == WorthQueryGranularInvalidationCoverage::RefreshAll {
            self.refresh_all();
            return;
        }
        self.owe(released.deliveries);
    }

    /// Transfers all owed and direct invalidation deliveries from a superseded
    /// same-branch predecessor into this successor binding so no restored or
    /// cursor-advanced delivery is ever stranded on branch reselection, while
    /// preserving direct preperformed-delivery evidence in `self.direct` so
    /// `authoritative_reconsideration` skips duplicate Bridge delivery until
    /// the inherited cursor passes those commits.
    pub(super) fn absorb_predecessor(&mut self, predecessor: &mut Self) {
        for direct in std::mem::take(&mut predecessor.direct) {
            let incoming = direct.correspondence_receipt().change_set();
            let duplicate = self.direct.iter().any(|existing| {
                let change = existing.correspondence_receipt().change_set();
                change.commit_identity() == incoming.commit_identity()
                    && change.dependency().dependency_ordinal()
                        == incoming.dependency().dependency_ordinal()
            });
            if !duplicate {
                self.direct.push(direct);
            }
        }
        let released = predecessor.release();
        if released.coverage == WorthQueryGranularInvalidationCoverage::RefreshAll {
            self.refresh_all();
        } else {
            self.owe(released.deliveries);
        }
    }

    pub(super) fn has_owed(&self) -> bool {
        self.retained_delivery_count() > 0
            || self.coverage == WorthQueryGranularInvalidationCoverage::RefreshAll
    }

    pub(super) fn retained_delivery_count(&self) -> usize {
        self.direct.len() + self.owed.len()
    }
}

/// Query converges these as one invalidation: the same basis, dependency,
/// value-free changes and Signal consequence.
fn same_invalidation(
    left: &BridgeGranularInvalidationDelivery,
    right: &BridgeGranularInvalidationDelivery,
) -> bool {
    let (left_set, right_set) = (left.truth().change_set(), right.truth().change_set());
    left_set.basis() == right_set.basis()
        && left_set.dependency() == right_set.dependency()
        && left_set.changes() == right_set.changes()
        && left.performed_signal().is_some() == right.performed_signal().is_some()
}

/// The newer snapshot is the one Query must not treat as already settled.
fn snapshot_version(delivery: &BridgeGranularInvalidationDelivery) -> Option<u64> {
    delivery
        .truth()
        .change_set()
        .snapshot_identity()
        .relational_snapshot_parts()
        .map(|parts| parts.version_id())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absorb_predecessor_propagates_refresh_all_and_clears_predecessor() {
        let mut predecessor = WorthQueryPendingGranularInvalidations::empty();
        assert!(!predecessor.has_owed());
        predecessor.refresh_all();
        assert!(predecessor.has_owed());

        let mut successor = WorthQueryPendingGranularInvalidations::empty();
        successor.absorb_predecessor(&mut predecessor);
        assert!(!predecessor.has_owed());
        assert!(successor.has_owed());

        let released = successor.release();
        assert_eq!(
            released.coverage,
            WorthQueryGranularInvalidationCoverage::RefreshAll
        );
        assert!(!successor.has_owed());
    }
}
