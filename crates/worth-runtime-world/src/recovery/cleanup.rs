use crate::identity::{
    ProductBranchIdentity, ProductBranchIncarnation, ProductUnpublishedOwnerEffectsIdentity,
};

/// Exact cleanup facts released with one product-unpublished recovery record.
/// The history identity is descriptive input for the lifecycle owner's bounded
/// reclamation service; it grants no history or publication authority.
#[derive(Debug)]
pub struct ProductUnpublishedCleanup {
    unpublished_history_candidates: Vec<crate::identity::CompositeCommitIdentity>,
    owner_retirement_work: Vec<crate::branch::OwnerRetirementWork>,
}

impl ProductUnpublishedCleanup {
    pub(crate) fn new(
        unpublished_history_candidates: Vec<crate::identity::CompositeCommitIdentity>,
        owner_retirement_work: Vec<crate::branch::OwnerRetirementWork>,
    ) -> Self {
        Self {
            unpublished_history_candidates,
            owner_retirement_work,
        }
    }

    pub fn unpublished_history_candidates(&self) -> &[crate::identity::CompositeCommitIdentity] {
        &self.unpublished_history_candidates
    }

    pub fn owner_retirement_work(&self) -> &[crate::branch::OwnerRetirementWork] {
        &self.owner_retirement_work
    }

    pub fn into_parts(
        self,
    ) -> (
        Vec<crate::identity::CompositeCommitIdentity>,
        Vec<crate::branch::OwnerRetirementWork>,
    ) {
        (
            self.unpublished_history_candidates,
            self.owner_retirement_work,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RecoveryCleanupDenial {
    Missing,
    Busy,
    ForeignCatalog,
    CallerCapabilityLive,
    SettlementRequired,
    TooYoung,
    ClockRegressed,
}

/// What the catalog released: the record's identity and the product-branch
/// occurrence it named. The occurrence travels with the release because the
/// record was the only thing naming it, and the custody charged to it must be
/// drained by whoever released the record, on every release path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RecoveryCleanupOutcome {
    identity: ProductUnpublishedOwnerEffectsIdentity,
    destination: Option<(ProductBranchIdentity, ProductBranchIncarnation)>,
    unpublished_commit: Option<crate::identity::CompositeCommitIdentity>,
}

impl RecoveryCleanupOutcome {
    /// The occurrence whose custody the released record was answerable for,
    /// when the attempt created one.
    pub(crate) fn destination(&self) -> Option<(&ProductBranchIdentity, ProductBranchIncarnation)> {
        self.destination
            .as_ref()
            .map(|(branch, incarnation)| (branch, *incarnation))
    }

    pub(crate) fn unpublished_commit(&self) -> Option<&crate::identity::CompositeCommitIdentity> {
        self.unpublished_commit.as_ref()
    }
}

impl super::catalog::ProductUnpublishedRecoveryCatalog {
    pub(crate) fn cleanup_record(
        &self,
        handle: &super::ProductUnpublishedRecoveryHandle,
        age: Option<(crate::lifecycle::RuntimeWorldInstant, u64)>,
    ) -> Result<RecoveryCleanupOutcome, RecoveryCleanupDenial> {
        if handle.catalog_affinity() != self.affinity() {
            return Err(RecoveryCleanupDenial::ForeignCatalog);
        }
        let mut denial = RecoveryCleanupDenial::SettlementRequired;
        let removed = self.remove_record_if_exclusive(handle, |record| {
            if record.settlement_required() {
                return false;
            }
            if let Some((now, minimum)) = age {
                let Some(elapsed) = now.ticks().checked_sub(record.admitted_at().ticks()) else {
                    denial = RecoveryCleanupDenial::ClockRegressed;
                    return false;
                };
                if elapsed < minimum {
                    denial = RecoveryCleanupDenial::TooYoung;
                    return false;
                }
            }
            true
        });
        match removed {
            Ok(Some(record)) => {
                let identity = record.identity().clone();
                let destination = record
                    .destination()
                    .map(|(branch, incarnation)| (branch.clone(), incarnation));
                let unpublished_commit = record.successor_commit().cloned();
                drop(record);
                Ok(RecoveryCleanupOutcome {
                    identity,
                    destination,
                    unpublished_commit,
                })
            }
            Err(super::catalog::RecoveryRecordRemovalDenial::Busy) => {
                Err(RecoveryCleanupDenial::Busy)
            }
            Ok(None) => Err(RecoveryCleanupDenial::Missing),
            Err(super::catalog::RecoveryRecordRemovalDenial::CallerCapabilityLive) => {
                Err(RecoveryCleanupDenial::CallerCapabilityLive)
            }
            Err(super::catalog::RecoveryRecordRemovalDenial::NotEligible) => Err(denial),
        }
    }
}

impl From<RecoveryCleanupDenial> for super::RuntimeWorldRecoveryDenial {
    fn from(value: RecoveryCleanupDenial) -> Self {
        match value {
            RecoveryCleanupDenial::Missing => Self::MissingRecord,
            RecoveryCleanupDenial::Busy => Self::Busy,
            RecoveryCleanupDenial::ForeignCatalog => Self::ForeignHandle,
            RecoveryCleanupDenial::CallerCapabilityLive => Self::CallerCapabilityLive,
            RecoveryCleanupDenial::TooYoung => Self::TooYoung,
            RecoveryCleanupDenial::ClockRegressed => Self::ClockRegressed,
            RecoveryCleanupDenial::SettlementRequired => Self::SettlementRequired,
        }
    }
}
