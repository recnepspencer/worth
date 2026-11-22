//! Close-custody correlation set for installed transport completion.
use crate::domain_computation::application_aftermath::ExternalEffectCorrelationIdentity;
use std::collections::BTreeSet;

#[derive(Default)]
pub struct InstalledTransportCloseCustody {
    correlations: BTreeSet<ExternalEffectCorrelationIdentity>,
}

impl InstalledTransportCloseCustody {
    pub fn insert(&mut self, correlation: ExternalEffectCorrelationIdentity) {
        self.correlations.insert(correlation);
    }

    pub fn covers(&self, correlation: &ExternalEffectCorrelationIdentity) -> bool {
        self.correlations.contains(correlation)
    }
}
