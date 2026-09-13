use crate::observation::PhysicalIntegrityObservationOutcome;
use crate::validation::PhysicalArtifactScope;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalIntegrityScrubInspection {
    outcome: PhysicalIntegrityObservationOutcome,
    selector_identity: Option<worth_store_physical_format::RootSelectorIdentity>,
}

impl PhysicalIntegrityScrubInspection {
    pub const fn new(outcome: PhysicalIntegrityObservationOutcome) -> Self {
        Self {
            outcome,
            selector_identity: None,
        }
    }

    pub const fn scope(self) -> PhysicalArtifactScope {
        self.outcome.scope()
    }

    pub const fn outcome(self) -> PhysicalIntegrityObservationOutcome {
        self.outcome
    }

    pub(crate) const fn with_selector_identity(
        mut self,
        identity: worth_store_physical_format::RootSelectorIdentity,
    ) -> Self {
        self.selector_identity = Some(identity);
        self
    }

    pub const fn selector_identity(
        self,
    ) -> Option<worth_store_physical_format::RootSelectorIdentity> {
        self.selector_identity
    }
}
