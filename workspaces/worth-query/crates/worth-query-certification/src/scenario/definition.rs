use super::WorthQueryCertificationJourneyCheckpoint as Checkpoint;
use crate::evidence::{
    WorthQueryCertificationCounters, WorthQueryCertificationObservation,
    WorthQueryCertificationObservationDenial,
};
use std::collections::BTreeSet;

/// Narrow semantic scenario families supplied by a downstream domain.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WorthQueryCertificationScenarioKind {
    Workflow,
    Replay,
    ConditionalNode,
    SemanticAspectCorrespondence,
    Reversal,
    Lineage,
    DependencyImpact,
    CounterContract,
}

impl WorthQueryCertificationScenarioKind {
    pub const ALL: [Self; 8] = [
        Self::Workflow,
        Self::Replay,
        Self::ConditionalNode,
        Self::SemanticAspectCorrespondence,
        Self::Reversal,
        Self::Lineage,
        Self::DependencyImpact,
        Self::CounterContract,
    ];
}

/// Why `WorthQueryCertificationScenario::with_oracle` refused a scenario.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryCertificationScenarioDenial {
    /// The scenario identity was empty or contained characters other than
    /// ASCII letters, digits, `-`, `.`, and `_`.
    InvalidIdentity,
    /// The oracle observation was invalid.
    InvalidOracle(WorthQueryCertificationObservationDenial),
}

/// One certification scenario supplied by a downstream domain: an identity, a
/// semantic family, and the oracle observation every provider must reproduce.
///
/// Build it with `with_oracle`; the required journey checkpoints follow from
/// the scenario kind.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryCertificationScenario {
    id: String,
    kind: WorthQueryCertificationScenarioKind,
    required_journey_checkpoints: BTreeSet<Checkpoint>,
    oracle: WorthQueryCertificationObservation,
}

impl WorthQueryCertificationScenario {
    pub fn with_oracle(
        id: impl Into<String>,
        kind: WorthQueryCertificationScenarioKind,
        semantic_facts: impl IntoIterator<Item = (String, String)>,
        counters: WorthQueryCertificationCounters,
    ) -> Result<Self, WorthQueryCertificationScenarioDenial> {
        let id = id.into();
        if id.is_empty()
            || !id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"-._".contains(&byte))
        {
            return Err(WorthQueryCertificationScenarioDenial::InvalidIdentity);
        }
        let oracle = WorthQueryCertificationObservation::new(semantic_facts, counters)
            .map_err(WorthQueryCertificationScenarioDenial::InvalidOracle)?;
        Ok(Self {
            id,
            kind,
            required_journey_checkpoints: required_checkpoints(kind),
            oracle,
        })
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn kind(&self) -> WorthQueryCertificationScenarioKind {
        self.kind
    }

    /// Query-owned journey requirements associated with this semantic family.
    ///
    /// These are requirements for the real facade certification suite, not
    /// evidence that a semantic provider executed each checkpoint.
    pub fn required_journey_checkpoints(&self) -> &BTreeSet<Checkpoint> {
        &self.required_journey_checkpoints
    }

    pub fn oracle(&self) -> &WorthQueryCertificationObservation {
        &self.oracle
    }
}

fn required_checkpoints(kind: WorthQueryCertificationScenarioKind) -> BTreeSet<Checkpoint> {
    use Checkpoint::*;
    let values: &[Checkpoint] = match kind {
        WorthQueryCertificationScenarioKind::Workflow => &[
            OperationResolution,
            Installation,
            SingleRootEntry,
            GraphParticipation,
            MultiDomainBinding,
            MultiGraphBinding,
            WorkflowProgression,
            Execution,
            Publication,
            Consumption,
        ],
        WorthQueryCertificationScenarioKind::Replay => &[Reexecution, Replay],
        WorthQueryCertificationScenarioKind::ConditionalNode => &[Support],
        WorthQueryCertificationScenarioKind::SemanticAspectCorrespondence => &[Compatibility],
        WorthQueryCertificationScenarioKind::Reversal => &[Reversal],
        WorthQueryCertificationScenarioKind::Lineage => &[Lineage, Promotion],
        WorthQueryCertificationScenarioKind::DependencyImpact => &[DependencyImpact, Invalidation],
        WorthQueryCertificationScenarioKind::CounterContract => &[
            NativeAccess,
            Sharing,
            Lease,
            CollectionWindow,
            CollectionPatch,
        ],
    };
    values.iter().copied().collect()
}
