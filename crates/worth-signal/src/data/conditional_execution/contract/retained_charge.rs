use super::{InstalledSignalConditionalContract, SignalConditionalContractDefinition};
use crate::data::retained_storage::{RetainedStorageMeasurement, RetainedStoragePreparation};

impl InstalledSignalConditionalContract {
    pub(crate) fn service_retained_bytes(&self) -> usize {
        self.service_retained_bytes
    }
}

impl SignalConditionalContractDefinition {
    pub(crate) fn installation_retained_bytes(&self) -> usize {
        let dynamic = self
            .retained_heap_charge(&mut RetainedStoragePreparation::new(None))
            .ok()
            .and_then(|charge| usize::try_from(charge.bytes()).ok())
            .unwrap_or(usize::MAX);
        std::mem::size_of::<Self>()
            .saturating_add(dynamic)
            .saturating_add(4 * std::mem::size_of::<usize>())
    }
}
