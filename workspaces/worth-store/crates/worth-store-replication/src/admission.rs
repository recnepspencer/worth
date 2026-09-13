use worth_proof::{DenialTransitionOutcome, TransitionOutcome};
use worth_store_authority::{StoreCurrentAuthorityIdentity, StoreCurrentAuthorityWitness};
use worth_store_security::StoreReadmittedSecurityScope;

use crate::{
    DurabilityReplayIdentity, ReplicationCapsuleId, ReplicationLineageIdentity, ReplicationPeerId,
    ReplicationSourceEpoch,
};

#[derive(Debug)]
pub struct ReplicationSourceAdmissionOutcome {
    outcome: DenialTransitionOutcome<AdmittedReplicationSource, ReplicationSourceAdmissionDenial>,
}

#[derive(Debug, Clone, Copy)]
pub enum ReplicationSourceAdmissionOutcomeView<'a> {
    Admitted(&'a AdmittedReplicationSource),
    Denied(ReplicationSourceAdmissionDenial),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplicationSourceDeclaration {
    capsule_id: ReplicationCapsuleId,
    peer_id: String,
    source_epoch: u64,
    lineage: String,
    replay_digest: String,
    first_lsn: u64,
    last_lsn: u64,
}

#[derive(Debug)]
pub struct AdmittedReplicationSource {
    capsule_id: ReplicationCapsuleId,
    peer_id: ReplicationPeerId,
    source_epoch: ReplicationSourceEpoch,
    lineage: ReplicationLineageIdentity,
    current_authority: StoreCurrentAuthorityIdentity,
    security_scope: StoreReadmittedSecurityScope,
    replay_identity: DurabilityReplayIdentity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplicationSourceAdmissionDenial {
    PeerIdentityRequired,
    SourceEpochRequired,
    LineageIdentityRequired,
    CurrentAuthorityMismatch,
    ReplayIdentityMismatch,
}

impl ReplicationSourceDeclaration {
    pub fn new(
        capsule_id: ReplicationCapsuleId,
        peer_id: impl Into<String>,
        source_epoch: u64,
        lineage: impl Into<String>,
        replay_digest: impl Into<String>,
        first_lsn: u64,
        last_lsn: u64,
    ) -> Self {
        Self {
            capsule_id,
            peer_id: peer_id.into(),
            source_epoch,
            lineage: lineage.into(),
            replay_digest: replay_digest.into(),
            first_lsn,
            last_lsn,
        }
    }
}

pub fn admit_replication_source(
    declaration: ReplicationSourceDeclaration,
    security_scope: StoreReadmittedSecurityScope,
    current_authority: &StoreCurrentAuthorityWitness,
    replay_identity: DurabilityReplayIdentity,
) -> ReplicationSourceAdmissionOutcome {
    if security_scope.current_authority().authority_identity()
        != current_authority.authority_identity()
    {
        return ReplicationSourceAdmissionOutcome::denied(
            ReplicationSourceAdmissionDenial::CurrentAuthorityMismatch,
        );
    }
    if !declaration_matches_replay(&declaration, &replay_identity) {
        return ReplicationSourceAdmissionOutcome::denied(
            ReplicationSourceAdmissionDenial::ReplayIdentityMismatch,
        );
    }
    let Some(peer_id) = ReplicationPeerId::admit(declaration.peer_id) else {
        return ReplicationSourceAdmissionOutcome::denied(
            ReplicationSourceAdmissionDenial::PeerIdentityRequired,
        );
    };
    let Some(source_epoch) = ReplicationSourceEpoch::admit(declaration.source_epoch) else {
        return ReplicationSourceAdmissionOutcome::denied(
            ReplicationSourceAdmissionDenial::SourceEpochRequired,
        );
    };
    let Some(lineage) = ReplicationLineageIdentity::admit(declaration.lineage) else {
        return ReplicationSourceAdmissionOutcome::denied(
            ReplicationSourceAdmissionDenial::LineageIdentityRequired,
        );
    };
    ReplicationSourceAdmissionOutcome::admitted(AdmittedReplicationSource {
        capsule_id: declaration.capsule_id,
        peer_id,
        source_epoch,
        lineage,
        current_authority: current_authority.authority_identity(),
        security_scope,
        replay_identity,
    })
}

impl ReplicationSourceAdmissionOutcome {
    fn admitted(source: AdmittedReplicationSource) -> Self {
        Self {
            outcome: TransitionOutcome::success(source),
        }
    }

    fn denied(denial: ReplicationSourceAdmissionDenial) -> Self {
        Self {
            outcome: TransitionOutcome::denied(denial),
        }
    }

    pub fn view(&self) -> ReplicationSourceAdmissionOutcomeView<'_> {
        match &self.outcome {
            TransitionOutcome::Success(source) => {
                ReplicationSourceAdmissionOutcomeView::Admitted(source)
            }
            TransitionOutcome::Denied(denial) => {
                ReplicationSourceAdmissionOutcomeView::Denied(*denial)
            }
            TransitionOutcome::Deferred(never)
            | TransitionOutcome::Stale(never)
            | TransitionOutcome::RebindRequired(never)
            | TransitionOutcome::Failed(never) => match *never {},
        }
    }

    pub fn into_result(
        self,
    ) -> Result<AdmittedReplicationSource, ReplicationSourceAdmissionDenial> {
        self.outcome.into_result()
    }
}

fn declaration_matches_replay(
    declaration: &ReplicationSourceDeclaration,
    replay: &DurabilityReplayIdentity,
) -> bool {
    declaration.replay_digest == replay.digest()
        && declaration.first_lsn == replay.first_lsn()
        && declaration.last_lsn == replay.last_lsn()
}

impl AdmittedReplicationSource {
    pub const fn capsule_id(&self) -> ReplicationCapsuleId {
        self.capsule_id
    }

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

    pub const fn security_scope(&self) -> &StoreReadmittedSecurityScope {
        &self.security_scope
    }

    pub const fn replay_identity(&self) -> &DurabilityReplayIdentity {
        &self.replay_identity
    }
}
