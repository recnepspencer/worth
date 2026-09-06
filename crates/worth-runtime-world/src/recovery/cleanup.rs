use crate::identity::{
    ProductBranchIdentity, ProductBranchIncarnation, ProductUnpublishedOwnerEffectsIdentity,
};

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
}

impl RecoveryCleanupOutcome {
    /// The occurrence whose custody the released record was answerable for,
    /// when the attempt created one.
    pub(crate) fn destination(&self) -> Option<(&ProductBranchIdentity, ProductBranchIncarnation)> {
        self.destination
            .as_ref()
            .map(|(branch, incarnation)| (branch, *incarnation))
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
                drop(record);
                Ok(RecoveryCleanupOutcome {
                    identity,
                    destination,
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
