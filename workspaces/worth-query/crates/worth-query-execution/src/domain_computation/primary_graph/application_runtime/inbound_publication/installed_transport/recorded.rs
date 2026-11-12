//! Live dispatch proof and admitted recorded descriptions remain distinct evidence.
use super::*;
use crate::domain_computation::application_aftermath::dispatch_journal::{
    QueryInboundDispatchSource, WorthQueryDispatchJournal,
};
use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;
use worth_foundational::facade::CanonicalDigestId;

impl InstalledTransportCompletion {
    pub(in crate::domain_computation::primary_graph) fn from_recorded_observation(
        journal: &WorthQueryDispatchJournal,
        runtime: WorthQueryRuntimeAuthorityIdentity,
        committed: WorthQueryCommittedDispatchOutboxObservation,
    ) -> Result<Self, InstalledTransportCompletionDenial> {
        use InstalledTransportCompletionDenial as Denial;
        if committed.record().inbound().is_none() {
            return Err(Denial::NoInstalledInboundContract);
        }
        let causality = journal
            .recorded_transport_completion(runtime, &committed)
            .map_err(|_| Denial::RecordedObservationUnavailable)?;
        let source = QueryInboundDispatchSource::from_owner(&committed)
            .map_err(|_| Denial::RecordedObservationUnavailable)?;
        Ok(Self {
            committed,
            observation: TransportCompletionObservation::Recorded { causality, source },
        })
    }

    pub(in crate::domain_computation::primary_graph) fn matches_committed_owner(
        &self,
        runtime: WorthQueryRuntimeAuthorityIdentity,
        owner: &WorthQueryCommittedDispatchOutboxObservation,
    ) -> bool {
        if self.committed != *owner {
            return false;
        }
        match &self.observation {
            TransportCompletionObservation::Live(dispatch) => {
                dispatch.matches_committed_owner(runtime, owner)
            }
            TransportCompletionObservation::Recorded { source, .. } => {
                QueryInboundDispatchSource::from_owner(owner).as_ref() == Ok(source)
            }
        }
    }

    pub(in crate::domain_computation::primary_graph) fn attempt_identity(
        &self,
    ) -> &CanonicalDigestId {
        match &self.observation {
            TransportCompletionObservation::Live(dispatch) => {
                dispatch.causal_ladder().attempt().identity().digest()
            }
            TransportCompletionObservation::Recorded { causality, .. } => causality.attempt(),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn observation_identity(
        &self,
    ) -> Option<&CanonicalDigestId> {
        match &self.observation {
            TransportCompletionObservation::Live(dispatch) => dispatch
                .causal_ladder()
                .observation()
                .map(|observed| observed.identity().digest()),
            TransportCompletionObservation::Recorded { causality, .. } => {
                Some(causality.observation())
            }
        }
    }
}
