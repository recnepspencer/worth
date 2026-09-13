use core::convert::Infallible;

use worth_proof::TransitionOutcome;
use worth_store_authority::StoreCurrentAuthorityIdentity;
use worth_store_security::StoreSecurityScopeIdentity;

use crate::progress_store::StoredReplicationPeerProgress;
use crate::{
    AdmittedReplicationSource, DurabilityReplayIdentity, ReplicationLineageIdentity,
    ReplicationPeerId, ReplicationPublicationReadiness, ReplicationSourceEpoch,
};

#[derive(Debug)]
pub struct ObservedReplicationProgress {
    source: AdmittedReplicationSource,
    delivery_kind: ReplicationDeliveryKind,
    prior_replay: Option<DurabilityReplayIdentity>,
}

#[derive(Debug)]
pub struct ReplicationProgressOutcome {
    outcome: TransitionOutcome<
        ObservedReplicationProgress,
        ReplicationProgressDenial,
        Infallible,
        ReplicationDuplicateDelivery,
    >,
}

#[derive(Debug, Clone, Copy)]
pub enum ReplicationProgressOutcomeView<'a> {
    Observed(&'a ObservedReplicationProgress),
    Denied(ReplicationProgressDenial),
    Duplicate(&'a ReplicationDuplicateDelivery),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplicationDeliveryKind {
    Fresh,
    Resumed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplicationProgressDenial {
    CurrentAuthorityMismatch,
    SourceEpochMismatch,
    LineageDivergence,
    DivergentReplayOverlap,
    ReplayProgressGap,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplicationDuplicateDelivery {
    peer_id: ReplicationPeerId,
    source_epoch: ReplicationSourceEpoch,
    lineage: ReplicationLineageIdentity,
    current_authority: StoreCurrentAuthorityIdentity,
    security_scope: StoreSecurityScopeIdentity,
    replay: DurabilityReplayIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplicationPeerProgress {
    pub(crate) peer_id: ReplicationPeerId,
    pub(crate) source_epoch: ReplicationSourceEpoch,
    pub(crate) lineage: ReplicationLineageIdentity,
    pub(crate) current_authority: StoreCurrentAuthorityIdentity,
    pub(crate) security_scope: StoreSecurityScopeIdentity,
    pub(crate) replay: DurabilityReplayIdentity,
}

pub(crate) fn observe_replication_progress(
    source: AdmittedReplicationSource,
    prior: Option<&StoredReplicationPeerProgress>,
) -> ReplicationProgressOutcome {
    let Some(prior) = prior else {
        return ReplicationProgressOutcome::observed(source, ReplicationDeliveryKind::Fresh, None);
    };
    if source.current_authority() != prior.current_authority
        || source
            .security_scope()
            .admitted()
            .identity()
            .stable_fingerprint()
            != prior.security_scope_fingerprint
    {
        return ReplicationProgressOutcome::denied(
            ReplicationProgressDenial::CurrentAuthorityMismatch,
        );
    }
    if source.source_epoch() != prior.source_epoch {
        return ReplicationProgressOutcome::denied(ReplicationProgressDenial::SourceEpochMismatch);
    }
    if source.lineage() != &prior.lineage {
        return ReplicationProgressOutcome::denied(ReplicationProgressDenial::LineageDivergence);
    }
    let replay = source.replay_identity();
    if replay == &prior.replay {
        return ReplicationProgressOutcome::duplicate(ReplicationDuplicateDelivery {
            peer_id: prior.peer_id.clone(),
            source_epoch: prior.source_epoch,
            lineage: prior.lineage.clone(),
            current_authority: source.current_authority(),
            security_scope: source.security_scope().admitted().identity(),
            replay: prior.replay.clone(),
        });
    }
    if replay.first_lsn() < prior.replay.last_lsn() {
        return ReplicationProgressOutcome::denied(
            ReplicationProgressDenial::DivergentReplayOverlap,
        );
    }
    if replay.first_lsn() > prior.replay.last_lsn() {
        return ReplicationProgressOutcome::denied(ReplicationProgressDenial::ReplayProgressGap);
    }
    ReplicationProgressOutcome::observed(
        source,
        ReplicationDeliveryKind::Resumed,
        Some(prior.replay.clone()),
    )
}

pub fn admit_replication_publication_readiness(
    progress: ObservedReplicationProgress,
) -> ReplicationPublicationReadiness {
    ReplicationPublicationReadiness::admit(progress)
}

impl ObservedReplicationProgress {
    pub const fn source(&self) -> &AdmittedReplicationSource {
        &self.source
    }

    pub const fn delivery_kind(&self) -> ReplicationDeliveryKind {
        self.delivery_kind
    }

    pub(crate) const fn prior_replay(&self) -> Option<&DurabilityReplayIdentity> {
        self.prior_replay.as_ref()
    }
}

impl ReplicationProgressOutcome {
    fn observed(
        source: AdmittedReplicationSource,
        delivery_kind: ReplicationDeliveryKind,
        prior_replay: Option<DurabilityReplayIdentity>,
    ) -> Self {
        Self {
            outcome: TransitionOutcome::success(ObservedReplicationProgress {
                source,
                delivery_kind,
                prior_replay,
            }),
        }
    }

    pub(crate) fn denied(denial: ReplicationProgressDenial) -> Self {
        Self {
            outcome: TransitionOutcome::denied(denial),
        }
    }

    fn duplicate(duplicate: ReplicationDuplicateDelivery) -> Self {
        Self {
            outcome: TransitionOutcome::stale(duplicate),
        }
    }

    pub fn view(&self) -> ReplicationProgressOutcomeView<'_> {
        match &self.outcome {
            TransitionOutcome::Success(progress) => {
                ReplicationProgressOutcomeView::Observed(progress)
            }
            TransitionOutcome::Denied(denial) => ReplicationProgressOutcomeView::Denied(*denial),
            TransitionOutcome::Stale(duplicate) => {
                ReplicationProgressOutcomeView::Duplicate(duplicate)
            }
            TransitionOutcome::Deferred(never)
            | TransitionOutcome::RebindRequired(never)
            | TransitionOutcome::Failed(never) => match *never {},
        }
    }

    pub fn into_observed_progress(
        self,
    ) -> Result<ObservedReplicationProgress, ReplicationProgressInterruption> {
        match self.outcome {
            TransitionOutcome::Success(progress) => Ok(progress),
            TransitionOutcome::Denied(denial) => {
                Err(ReplicationProgressInterruption::Denied(denial))
            }
            TransitionOutcome::Stale(duplicate) => {
                Err(ReplicationProgressInterruption::Duplicate(duplicate))
            }
            TransitionOutcome::Deferred(never)
            | TransitionOutcome::RebindRequired(never)
            | TransitionOutcome::Failed(never) => match never {},
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplicationProgressInterruption {
    Denied(ReplicationProgressDenial),
    Duplicate(ReplicationDuplicateDelivery),
}

impl ReplicationDuplicateDelivery {
    pub const fn peer_id(&self) -> &ReplicationPeerId {
        &self.peer_id
    }

    pub const fn source_epoch(&self) -> ReplicationSourceEpoch {
        self.source_epoch
    }

    pub const fn lineage(&self) -> &ReplicationLineageIdentity {
        &self.lineage
    }

    pub const fn replay_identity(&self) -> &DurabilityReplayIdentity {
        &self.replay
    }

    pub const fn current_authority(&self) -> StoreCurrentAuthorityIdentity {
        self.current_authority
    }

    pub const fn security_scope(&self) -> StoreSecurityScopeIdentity {
        self.security_scope
    }
}

impl ReplicationPeerProgress {
    pub const fn peer_id(&self) -> &ReplicationPeerId {
        &self.peer_id
    }

    pub const fn source_epoch(&self) -> ReplicationSourceEpoch {
        self.source_epoch
    }

    pub const fn lineage(&self) -> &ReplicationLineageIdentity {
        &self.lineage
    }

    pub const fn current_authority(&self) -> StoreCurrentAuthorityIdentity {
        self.current_authority
    }

    pub const fn security_scope(&self) -> StoreSecurityScopeIdentity {
        self.security_scope
    }

    pub const fn replay_identity(&self) -> &DurabilityReplayIdentity {
        &self.replay
    }
}
