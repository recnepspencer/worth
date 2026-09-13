use std::collections::BTreeMap;
use std::path::Path;

use worth_store_authority::{StoreCurrentAuthorityIdentity, StoreCurrentAuthorityWitness};
use worth_store_physical_backend::{
    ProductionStorageBoundaryControl, UninterruptedStorageBoundaryControl,
};

use crate::progress::observe_replication_progress;
use crate::progress_store::{
    ReplicationPeerCapacity, ReplicationProgressStore, ReplicationProgressStoreError,
    StoredReplicationPeerProgress,
};
use crate::publication::publish_replication;
use crate::{
    AdmittedReplicationSource, ReplicationDeliveryKind, ReplicationPeerId,
    ReplicationProgressDenial, ReplicationProgressOutcome, ReplicationPublicationDenial,
    ReplicationPublicationOutcome, ReplicationPublicationOutcomeView,
    ReplicationPublicationReadiness,
};

#[derive(Debug)]
pub struct ReplicationAdmissionRuntime {
    current_authority: StoreCurrentAuthorityIdentity,
    peer_progress: BTreeMap<ReplicationPeerId, StoredReplicationPeerProgress>,
    progress_store: ReplicationProgressStore,
}

impl ReplicationAdmissionRuntime {
    pub fn open(
        progress_directory: &Path,
        current_authority: &StoreCurrentAuthorityWitness,
        capacity: ReplicationPeerCapacity,
    ) -> Result<Self, ReplicationPublicationDenial> {
        let (progress_store, peer_progress) = ReplicationProgressStore::open(
            progress_directory,
            capacity,
            current_authority.authority_identity(),
        )
        .map_err(map_store_error)?;
        Ok(Self {
            current_authority: current_authority.authority_identity(),
            peer_progress,
            progress_store,
        })
    }

    #[cfg(any(test, feature = "certification-test-authority"))]
    pub fn bind(
        progress_directory: &Path,
        current_authority: &StoreCurrentAuthorityWitness,
    ) -> Self {
        Self::open(
            progress_directory,
            current_authority,
            ReplicationPeerCapacity::new(1024).expect("nonzero test capacity"),
        )
        .expect("isolated replication progress store must open")
    }

    pub fn observe_progress(
        &self,
        source: AdmittedReplicationSource,
    ) -> ReplicationProgressOutcome {
        if source.current_authority() != self.current_authority {
            return ReplicationProgressOutcome::denied(
                ReplicationProgressDenial::CurrentAuthorityMismatch,
            );
        }
        let prior = self.peer_progress.get(source.peer_id());
        observe_replication_progress(source, prior)
    }

    pub fn publish(
        &mut self,
        readiness: ReplicationPublicationReadiness,
        current_authority: &StoreCurrentAuthorityWitness,
    ) -> ReplicationPublicationOutcome {
        self.publish_controlled(
            readiness,
            current_authority,
            &UninterruptedStorageBoundaryControl,
        )
    }

    fn publish_controlled(
        &mut self,
        readiness: ReplicationPublicationReadiness,
        current_authority: &StoreCurrentAuthorityWitness,
        control: &impl ProductionStorageBoundaryControl,
    ) -> ReplicationPublicationOutcome {
        if !self.observation_basis_is_current(&readiness) {
            return ReplicationPublicationOutcome::denied(
                ReplicationPublicationDenial::PeerProgressChanged,
            );
        }
        let outcome = publish_replication(readiness, current_authority);
        let ReplicationPublicationOutcomeView::Published(published) = outcome.view() else {
            return outcome;
        };
        let peer_id = published.peer_progress().peer_id().clone();
        let stored = match self
            .progress_store
            .persist_controlled(published.peer_progress(), control)
        {
            Ok(stored) => stored,
            Err(error) => return ReplicationPublicationOutcome::denied(map_store_error(error)),
        };
        self.peer_progress.insert(peer_id, stored);
        outcome
    }

    #[cfg(any(test, feature = "certification-test-authority"))]
    pub fn publish_with_boundary_control(
        &mut self,
        readiness: ReplicationPublicationReadiness,
        current_authority: &StoreCurrentAuthorityWitness,
        control: &impl ProductionStorageBoundaryControl,
    ) -> ReplicationPublicationOutcome {
        self.publish_controlled(readiness, current_authority, control)
    }

    pub fn peer_progress(
        &self,
        peer_id: &ReplicationPeerId,
    ) -> Option<&crate::DurabilityReplayIdentity> {
        self.peer_progress
            .get(peer_id)
            .map(|progress| &progress.replay)
    }

    pub const fn current_authority(&self) -> StoreCurrentAuthorityIdentity {
        self.current_authority
    }

    fn observation_basis_is_current(&self, readiness: &ReplicationPublicationReadiness) -> bool {
        let prior = self.peer_progress.get(readiness.source().peer_id());
        match (readiness.delivery_kind(), readiness.prior_replay(), prior) {
            (ReplicationDeliveryKind::Fresh, None, None) => true,
            (ReplicationDeliveryKind::Resumed, Some(expected), Some(actual)) => {
                expected == &actual.replay
            }
            _ => false,
        }
    }
}

const fn map_store_error(error: ReplicationProgressStoreError) -> ReplicationPublicationDenial {
    match error {
        ReplicationProgressStoreError::AuthorityMismatch => {
            ReplicationPublicationDenial::CurrentAuthorityChanged
        }
        ReplicationProgressStoreError::CapacityExceeded => {
            ReplicationPublicationDenial::PeerCapacityExceeded
        }
        ReplicationProgressStoreError::Io => ReplicationPublicationDenial::ProgressStoreIo,
    }
}
