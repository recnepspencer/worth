use worth_proof::{DenialTransitionOutcome, TransitionOutcome};
use worth_store_authority::StoreCurrentAuthorityWitness;

use crate::{
    AdmittedReplicationSource, ObservedReplicationProgress, ReplicationDeliveryKind,
    ReplicationPeerProgress,
};

#[derive(Debug)]
pub struct ReplicationPublicationOutcome {
    outcome: DenialTransitionOutcome<PublishedReplication, ReplicationPublicationDenial>,
}

#[derive(Debug, Clone, Copy)]
pub enum ReplicationPublicationOutcomeView<'a> {
    Published(&'a PublishedReplication),
    Denied(ReplicationPublicationDenial),
}

#[derive(Debug)]
pub struct ReplicationPublicationReadiness {
    progress: ObservedReplicationProgress,
}

#[derive(Debug)]
pub struct PublishedReplication {
    progress: ObservedReplicationProgress,
    peer_progress: ReplicationPeerProgress,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplicationPublicationDenial {
    CurrentAuthorityChanged,
    PeerProgressChanged,
    PeerCapacityExceeded,
    ProgressStoreIo,
}

pub(crate) fn publish_replication(
    readiness: ReplicationPublicationReadiness,
    current_authority: &StoreCurrentAuthorityWitness,
) -> ReplicationPublicationOutcome {
    if readiness.source().current_authority() != current_authority.authority_identity() {
        return ReplicationPublicationOutcome::denied(
            ReplicationPublicationDenial::CurrentAuthorityChanged,
        );
    }
    let peer_progress = ReplicationPeerProgress {
        peer_id: readiness.source().peer_id().clone(),
        source_epoch: readiness.source().source_epoch(),
        lineage: readiness.source().lineage().clone(),
        current_authority: readiness.source().current_authority(),
        security_scope: readiness.source().security_scope().admitted().identity(),
        replay: readiness.source().replay_identity().clone(),
    };
    ReplicationPublicationOutcome::published(PublishedReplication {
        progress: readiness.progress,
        peer_progress,
    })
}

impl ReplicationPublicationReadiness {
    pub(crate) const fn admit(progress: ObservedReplicationProgress) -> Self {
        Self { progress }
    }

    pub const fn source(&self) -> &AdmittedReplicationSource {
        self.progress.source()
    }

    pub const fn delivery_kind(&self) -> ReplicationDeliveryKind {
        self.progress.delivery_kind()
    }

    pub(crate) const fn prior_replay(&self) -> Option<&crate::DurabilityReplayIdentity> {
        self.progress.prior_replay()
    }
}

impl PublishedReplication {
    pub const fn source(&self) -> &AdmittedReplicationSource {
        self.progress.source()
    }

    pub const fn delivery_kind(&self) -> ReplicationDeliveryKind {
        self.progress.delivery_kind()
    }

    pub const fn peer_progress(&self) -> &ReplicationPeerProgress {
        &self.peer_progress
    }

    pub fn into_peer_progress(self) -> ReplicationPeerProgress {
        self.peer_progress
    }
}

impl ReplicationPublicationOutcome {
    fn published(published: PublishedReplication) -> Self {
        Self {
            outcome: TransitionOutcome::success(published),
        }
    }

    pub(crate) fn denied(denial: ReplicationPublicationDenial) -> Self {
        Self {
            outcome: TransitionOutcome::denied(denial),
        }
    }

    pub fn view(&self) -> ReplicationPublicationOutcomeView<'_> {
        match &self.outcome {
            TransitionOutcome::Success(published) => {
                ReplicationPublicationOutcomeView::Published(published)
            }
            TransitionOutcome::Denied(denial) => ReplicationPublicationOutcomeView::Denied(*denial),
            TransitionOutcome::Deferred(never)
            | TransitionOutcome::Stale(never)
            | TransitionOutcome::RebindRequired(never)
            | TransitionOutcome::Failed(never) => match *never {},
        }
    }

    pub fn into_result(self) -> Result<PublishedReplication, ReplicationPublicationDenial> {
        self.outcome.into_result()
    }
}
