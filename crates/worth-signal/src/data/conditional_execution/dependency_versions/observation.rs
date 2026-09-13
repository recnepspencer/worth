use crate::data::aspect::{AspectMask, AspectVersion};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

/// Cached coordinates and values, never source or installation authority.
/// Unobserved slots in `versions` cannot satisfy a different requested mask.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SignalConditionalVersionObservation {
    aspects: AspectMask,
    versions: AspectVersion,
}

impl SignalConditionalVersionObservation {
    pub(crate) fn new(aspects: AspectMask, versions: AspectVersion) -> Self {
        Self { aspects, versions }
    }

    pub(super) fn for_aspects(&self, aspects: AspectMask) -> Option<AspectVersion> {
        (self.aspects == aspects).then_some(self.versions)
    }
}

impl RetainedStorageMeasurement for SignalConditionalVersionObservation {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            aspects: _,
            versions,
        } = self;
        versions.retained_heap_charge(work)
    }
}
