use super::{RetainedStorageCharge, RetainedStoragePreparation, RetainedStoragePreparationDenial};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RetainedStorageForkGrowthDenial {
    PreparationRequired,
    Accounting(RetainedStoragePreparationDenial),
}
impl From<RetainedStoragePreparationDenial> for RetainedStorageForkGrowthDenial {
    fn from(value: RetainedStoragePreparationDenial) -> Self {
        Self::Accounting(value)
    }
}

/// New allocations for a persistent fork, with bounded construction work.
/// Existing payload custody is retained by the caller and shared backings.
/// Implementations must not walk payloads or repair missing accounting facts.
pub(crate) trait RetainedStorageForkGrowth {
    fn prepare_fork_growth(
        &mut self,
        work: &mut RetainedStoragePreparation,
    ) -> Result<RetainedStorageCharge, RetainedStorageForkGrowthDenial>;
}
