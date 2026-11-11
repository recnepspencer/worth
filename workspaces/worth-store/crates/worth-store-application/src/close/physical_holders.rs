//! Read-only observations name actual retained physical adapter shares.
use std::sync::{Arc, Weak};
use worth_foundational::facade::DurableSignalOwnerIdentity;
use worth_store::physical_runtime::ServingPhysicalRuntime;

#[derive(Debug)]
pub struct ApplicationHomePhysicalCustodyHolders {
    pub serving_shares: usize,
    pub signal: HomeSignalPhysicalHolder,
    pub unidentified_shares: usize,
}
#[derive(Debug)]
pub enum HomeSignalPhysicalHolder {
    Absent,
    Released(DurableSignalOwnerIdentity),
    Retaining(DurableSignalOwnerIdentity),
    InspectionBusy(DurableSignalOwnerIdentity),
}
pub(super) fn observe(
    actual: &Arc<ServingPhysicalRuntime>,
    signal: Option<&Weak<crate::signal_medium::StoreSignalMedium>>,
) -> ApplicationHomePhysicalCustodyHolders {
    let serving_shares = Arc::strong_count(actual);
    let signal = signal
        .and_then(Weak::upgrade)
        .map_or(HomeSignalPhysicalHolder::Absent, |medium| {
            medium.physical_close_holder(actual)
        });
    let named = match signal {
        HomeSignalPhysicalHolder::Retaining(_) => 1,
        HomeSignalPhysicalHolder::Absent
        | HomeSignalPhysicalHolder::Released(_)
        | HomeSignalPhysicalHolder::InspectionBusy(_) => 0,
    };
    ApplicationHomePhysicalCustodyHolders {
        serving_shares,
        signal,
        unidentified_shares: serving_shares.saturating_sub(1 + named),
    }
}
