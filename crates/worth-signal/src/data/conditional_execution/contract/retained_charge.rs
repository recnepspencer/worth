use super::{
    InstalledSignalConditionalContract, SignalConditionalCondition,
    SignalConditionalContractDefinition,
};

impl InstalledSignalConditionalContract {
    pub(crate) fn service_retained_bytes(&self) -> usize {
        self.service_retained_bytes
    }
}

impl SignalConditionalContractDefinition {
    pub(crate) fn installation_retained_bytes(&self) -> usize {
        let dynamic = match &self.condition {
            SignalConditionalCondition::DeltaThreshold(threshold) => threshold.unit_identity.len(),
            _ => 0,
        };
        std::mem::size_of::<Self>()
            .saturating_add(dynamic)
            .saturating_add(4 * std::mem::size_of::<usize>())
    }
}
