use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::runtime::WorthQueryJournalReplayCounterSnapshot;

use super::evidence::{
    WorthQueryJournalIdentityInventoryEvidence, WorthQueryJournalIdentityScheduleEvidence,
    WorthQueryJournalReplaySurfaceEvidence,
};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryJournalIdentityBoundaryPosture {
    Open,
    Partial,
    Closed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryJournalIdentityCertification {
    inventory: WorthQueryJournalIdentityInventoryEvidence,
    schedule: WorthQueryJournalIdentityScheduleEvidence,
    replay: WorthQueryJournalReplaySurfaceEvidence,
    posture: WorthQueryJournalIdentityBoundaryPosture,
    replay_boundary: WorthQueryJournalReplayBoundaryCertification,
    artifact_digest: String,
    failure_digest: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryJournalReplayBoundaryCertification {
    journal_segment_identity_digest: String,
    journal_replay_outcome_digest: String,
    journal_replay_truth_digest: String,
    published_artifact_replay_digest: String,
    journal_identity_inventory_digest: String,
    journal_boundary_posture: WorthQueryJournalIdentityBoundaryPosture,
    failure_digest: String,
    counter_snapshot: WorthQueryJournalReplayCounterSnapshot,
}
impl WorthQueryJournalIdentityCertification {
    pub fn from_evidence(
        inventory: WorthQueryJournalIdentityInventoryEvidence,
        schedule: WorthQueryJournalIdentityScheduleEvidence,
        replay: WorthQueryJournalReplaySurfaceEvidence,
    ) -> Result<Self, IdentityDerivationDenial> {
        let posture = derive_journal_identity_posture(&inventory, &schedule, &replay);
        let failure_digest =
            worth_query_evidence_identity(WorthQueryEvidenceScope::ApplicationSupportReport)
                .field_usize(
                    WorthQueryEvidenceTag::new("journal_identity_failure_count"),
                    inventory.total_failure_count(),
                )
                .field_bool(
                    WorthQueryEvidenceTag::new("journal_identity_schedule_certified"),
                    schedule.certified(),
                )
                .field_bool(
                    WorthQueryEvidenceTag::new("journal_replay_surface_certified"),
                    replay.certified(),
                )
                .seal()?
                .as_str()
                .to_string();
        let artifact_digest = worth_query_evidence_identity(
            WorthQueryEvidenceScope::RuntimeHostileCertificationArtifact,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("journal_identity_inventory_digest"),
            inventory.inventory_digest(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("journal_identity_schedule_digest"),
            schedule.schedule_digest(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("journal_replay_outcome_digest"),
            replay.replay_outcome_digest(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("journal_replay_truth_digest"),
            replay.replay_truth_digest(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("published_artifact_replay_digest"),
            replay.published_artifact_digest(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("journal_replay_retained_entry_count"),
            replay.counter_snapshot().retained_entry_count(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("journal_replay_residue_count"),
            replay.counter_snapshot().replay_residue_count(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("journal_identity_failure_count"),
            inventory.total_failure_count(),
        )
        .field_bool(
            WorthQueryEvidenceTag::new("journal_identity_schedule_certified"),
            schedule.certified(),
        )
        .field_bool(
            WorthQueryEvidenceTag::new("journal_replay_surface_certified"),
            replay.certified(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("journal_identity_failure_digest"),
            &failure_digest,
        )
        .seal()?
        .as_str()
        .to_string();
        let replay_boundary = WorthQueryJournalReplayBoundaryCertification {
            journal_segment_identity_digest: replay.journal_segment_identity_digest().to_string(),
            journal_replay_outcome_digest: replay.replay_outcome_digest().to_string(),
            journal_replay_truth_digest: replay.replay_truth_digest().to_string(),
            published_artifact_replay_digest: replay.published_artifact_digest().to_string(),
            journal_identity_inventory_digest: inventory.inventory_digest().to_string(),
            journal_boundary_posture: posture,
            failure_digest: failure_digest.clone(),
            counter_snapshot: replay.counter_snapshot().clone(),
        };
        Ok(Self {
            inventory,
            schedule,
            replay,
            posture,
            replay_boundary,
            artifact_digest,
            failure_digest,
        })
    }

    pub fn closed(&self) -> bool {
        self.posture == WorthQueryJournalIdentityBoundaryPosture::Closed
    }
    pub fn posture(&self) -> WorthQueryJournalIdentityBoundaryPosture {
        self.posture
    }

    pub fn artifact_digest(&self) -> &str {
        &self.artifact_digest
    }

    pub fn failure_digest(&self) -> &str {
        &self.failure_digest
    }

    pub fn replay_boundary_certification(&self) -> &WorthQueryJournalReplayBoundaryCertification {
        &self.replay_boundary
    }
}
impl WorthQueryJournalReplayBoundaryCertification {
    pub fn journal_segment_identity_digest(&self) -> &str {
        &self.journal_segment_identity_digest
    }

    pub fn journal_replay_outcome_digest(&self) -> &str {
        &self.journal_replay_outcome_digest
    }

    pub fn journal_replay_truth_digest(&self) -> &str {
        &self.journal_replay_truth_digest
    }

    pub fn published_artifact_replay_digest(&self) -> &str {
        &self.published_artifact_replay_digest
    }

    pub fn journal_identity_inventory_digest(&self) -> &str {
        &self.journal_identity_inventory_digest
    }

    pub fn journal_boundary_posture(&self) -> WorthQueryJournalIdentityBoundaryPosture {
        self.journal_boundary_posture
    }

    pub fn failure_digest(&self) -> &str {
        &self.failure_digest
    }

    pub fn counter_snapshot(&self) -> &WorthQueryJournalReplayCounterSnapshot {
        &self.counter_snapshot
    }
}

fn derive_journal_identity_posture(
    inventory: &WorthQueryJournalIdentityInventoryEvidence,
    schedule: &WorthQueryJournalIdentityScheduleEvidence,
    replay: &WorthQueryJournalReplaySurfaceEvidence,
) -> WorthQueryJournalIdentityBoundaryPosture {
    let inventory_clean = inventory.total_failure_count() == 0;
    let schedule_certified = schedule.certified();
    let replay_certified = replay.certified();
    if inventory_clean && schedule_certified && replay_certified {
        return WorthQueryJournalIdentityBoundaryPosture::Closed;
    }
    if inventory_clean || schedule_certified || replay_certified {
        return WorthQueryJournalIdentityBoundaryPosture::Partial;
    }
    WorthQueryJournalIdentityBoundaryPosture::Open
}
