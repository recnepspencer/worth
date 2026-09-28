use crate::evidence::WorthQueryCertificationCounters;
use crate::scenario::{
    WorthQueryCertificationJourneyCheckpoint, WorthQueryCertificationScenarioKind,
};
use std::collections::BTreeSet;

/// The certified result of one scenario in a provider-pair run: its identity,
/// kind, required journey checkpoints, and the counters both providers
/// observed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryCertificationScenarioReport {
    scenario_identity: String,
    kind: WorthQueryCertificationScenarioKind,
    required_journey_checkpoints: BTreeSet<WorthQueryCertificationJourneyCheckpoint>,
    counters: WorthQueryCertificationCounters,
}

impl WorthQueryCertificationScenarioReport {
    pub(crate) fn new(
        scenario_identity: String,
        kind: WorthQueryCertificationScenarioKind,
        required_journey_checkpoints: BTreeSet<WorthQueryCertificationJourneyCheckpoint>,
        counters: WorthQueryCertificationCounters,
    ) -> Self {
        Self {
            scenario_identity,
            kind,
            required_journey_checkpoints,
            counters,
        }
    }

    pub fn scenario_identity(&self) -> &str {
        &self.scenario_identity
    }

    pub fn kind(&self) -> WorthQueryCertificationScenarioKind {
        self.kind
    }

    pub fn required_journey_checkpoints(
        &self,
    ) -> &BTreeSet<WorthQueryCertificationJourneyCheckpoint> {
        &self.required_journey_checkpoints
    }

    pub fn counters(&self) -> &WorthQueryCertificationCounters {
        &self.counters
    }
}

/// The report from a successful `certify_provider_pair` run: the two
/// provider identities and one scenario report per suite scenario, in suite
/// order. It is evidence of parity, not authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryCertificationReport {
    provider_identities: [String; 2],
    scenarios: Vec<WorthQueryCertificationScenarioReport>,
}

impl WorthQueryCertificationReport {
    pub(crate) fn new(
        provider_identities: [String; 2],
        scenarios: Vec<WorthQueryCertificationScenarioReport>,
    ) -> Self {
        Self {
            provider_identities,
            scenarios,
        }
    }

    pub fn provider_identities(&self) -> &[String; 2] {
        &self.provider_identities
    }

    pub fn scenarios(&self) -> &[WorthQueryCertificationScenarioReport] {
        &self.scenarios
    }
}

/// The report from a successful `certify_hostile_provider` run: the provider
/// identity and how many hostile cases it answered with the expected denial
/// evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryHostileCertificationReport {
    provider_identity: String,
    hostile_case_count: usize,
}

impl WorthQueryHostileCertificationReport {
    pub(crate) fn new(provider_identity: String, hostile_case_count: usize) -> Self {
        Self {
            provider_identity,
            hostile_case_count,
        }
    }

    pub fn provider_identity(&self) -> &str {
        &self.provider_identity
    }

    pub fn hostile_case_count(&self) -> usize {
        self.hostile_case_count
    }
}
