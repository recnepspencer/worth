//! Immutable provenance of a computation's inputs across publication and recovery.
use crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphCommittedApplication;

/// Rebased source facts cannot replace what the computation actually read.
/// This evidence has no default and derived runtime records must copy it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct ComputationSourceEvidence(Origin);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Origin {
    Performed {
        superseded_by_own_effect: bool,
    },
    /// Current-version checkpoint facts exclude computations with own-write
    /// staleness or consumed outputs. Restore carries that wire promise and
    /// must still compare the original producer facts and native output.
    CheckpointFacts,
    Unavailable,
}

/// Every output-current proof and retained consumption requires this value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct CurrentComputation {
    source: ComputationSourceEvidence,
}

impl ComputationSourceEvidence {
    /// Only actual completed Native computation evidence originates a
    /// performed record; every derived record must carry the source evidence.
    pub(super) fn from_completed(application: &WorthQueryPrimaryGraphCommittedApplication) -> Self {
        Self(Origin::Performed {
            superseded_by_own_effect: !application
                .commit_evidence()
                .source_facts_superseded_by_own_effect()
                .is_empty(),
        })
    }

    /// Current-version producer facts were checkpointed only after excluding
    /// flagged computations. This is wire provenance, not a fresh computation.
    pub(in crate::domain_computation::primary_graph) fn from_checkpoint_facts(
        _facts: &crate::domain_computation::primary_graph::application_checkpoint::CheckpointProducerFacts,
    ) -> Self {
        Self(Origin::CheckpointFacts)
    }

    pub(super) fn unavailable() -> Self {
        Self(Origin::Unavailable)
    }

    /// The one certification door. Own-effect staleness survives every
    /// verification route and record derivation until a computation runs again.
    pub(in crate::domain_computation::primary_graph) fn certify_current(
        self,
    ) -> Option<CurrentComputation> {
        match self.0 {
            Origin::Performed {
                superseded_by_own_effect: true,
            }
            | Origin::Unavailable => None,
            Origin::Performed {
                superseded_by_own_effect: false,
            }
            | Origin::CheckpointFacts => Some(CurrentComputation { source: self }),
        }
    }

    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn for_test(
        superseded_by_own_effect: bool,
    ) -> Self {
        Self(Origin::Performed {
            superseded_by_own_effect,
        })
    }
}

impl CurrentComputation {
    pub(super) fn source(self) -> ComputationSourceEvidence {
        self.source
    }
}

impl ComputationSourceEvidence {
    /// Bind explicitly carried provenance to admitted wire or Native facts.
    /// This does not certify currentness or expose a stale record's facts.
    pub(in crate::domain_computation::primary_graph) fn retain_facts(
        self,
        facts: std::sync::Arc<
            [crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact],
        >,
    ) -> super::RetainedSourceFacts {
        super::RetainedSourceFacts::retain(self, facts)
    }
}
