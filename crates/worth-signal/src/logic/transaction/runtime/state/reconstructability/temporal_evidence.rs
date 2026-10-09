use serde::{Deserialize, Serialize};

use super::super::super::transaction::TemporalTransactionEvidence;
use super::super::merge::canonical_digest;
use super::super::retention_omission::RetentionOmission;
use super::super::temporal::TemporalRuntimeState;
use crate::data::temporal::{RuntimeClockBasis, TemporalWakeSummary};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemporalReconstructabilityArtifact {
    pub clock_basis: RuntimeClockBasis,
    pub wake_summary: TemporalWakeSummary,
    pub eligibility_fact_count: u64,
    pub scheduled_wake_count: u64,
    pub ready_wake_count: u64,
    pub retired_wake_count: u64,
    #[serde(default, skip_serializing_if = "is_zero_u64")]
    pub expired_retired_wake_count: u64,
    #[serde(default, skip_serializing_if = "is_zero_u64")]
    pub expired_retired_wake_high_water: u64,
    #[serde(default, skip_serializing_if = "is_zero_digest")]
    pub expired_retired_wake_digest: [u8; 32],
    pub rescheduled_wake_count: u64,
    pub reused_wake_count: u64,
    pub interval_regeneration_count: u64,
    pub previous_value_reference_count: u64,
    pub clock_checkpoint_digest: String,
    pub scheduled_wake_digest: String,
    pub ready_wake_digest: String,
    pub retired_wake_digest: String,
    pub rescheduled_wake_digest: String,
    pub reused_wake_digest: String,
    pub interval_regeneration_digest: String,
    pub temporal_eligibility_digest: String,
    pub previous_value_reference_digest: String,
    pub certification_digest: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TemporalReplayMismatchClass {
    ClockCheckpointDigestMismatch,
    ScheduledWakeDigestMismatch,
    ReadyWakeDigestMismatch,
    RetiredWakeDigestMismatch,
    RescheduledWakeDigestMismatch,
    ReusedWakeDigestMismatch,
    IntervalRegenerationDigestMismatch,
    TemporalEligibilityDigestMismatch,
    PreviousValueReferenceDigestMismatch,
    CertificationDigestMismatch,
    WakeSummaryMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemporalReplayParityReport {
    pub proof_schema_version: String,
    pub expected: TemporalReconstructabilityArtifact,
    pub replayed: TemporalReconstructabilityArtifact,
    pub parity: bool,
    pub mismatch_classes: Vec<TemporalReplayMismatchClass>,
}

pub const TEMPORAL_REPLAY_PARITY_SCHEMA_VERSION: &str = "worth-signal-temporal-replay-parity-v1";
const TEMPORAL_REPLAY_PARITY_WITH_OMISSION_SCHEMA_VERSION: &str =
    "worth-signal-temporal-replay-parity-v2";

fn is_zero_u64(value: &u64) -> bool {
    *value == 0
}

fn is_zero_digest(value: &[u8; 32]) -> bool {
    *value == [0; 32]
}

impl Default for TemporalReconstructabilityArtifact {
    fn default() -> Self {
        Self::from_evidence(
            TemporalWakeSummary::default(),
            &TemporalTransactionEvidence::default(),
        )
    }
}

pub fn temporal_replay_parity_report(
    expected: &TemporalReconstructabilityArtifact,
    replayed: &TemporalReconstructabilityArtifact,
) -> TemporalReplayParityReport {
    let mut mismatch_classes = Vec::new();
    if expected.clock_checkpoint_digest != replayed.clock_checkpoint_digest {
        mismatch_classes.push(TemporalReplayMismatchClass::ClockCheckpointDigestMismatch);
    }
    if expected.scheduled_wake_digest != replayed.scheduled_wake_digest {
        mismatch_classes.push(TemporalReplayMismatchClass::ScheduledWakeDigestMismatch);
    }
    if expected.ready_wake_digest != replayed.ready_wake_digest {
        mismatch_classes.push(TemporalReplayMismatchClass::ReadyWakeDigestMismatch);
    }
    if expected.retired_wake_digest != replayed.retired_wake_digest
        || expected.expired_retired_wake_count != replayed.expired_retired_wake_count
        || expected.expired_retired_wake_high_water != replayed.expired_retired_wake_high_water
        || expected.expired_retired_wake_digest != replayed.expired_retired_wake_digest
    {
        mismatch_classes.push(TemporalReplayMismatchClass::RetiredWakeDigestMismatch);
    }
    if expected.rescheduled_wake_digest != replayed.rescheduled_wake_digest {
        mismatch_classes.push(TemporalReplayMismatchClass::RescheduledWakeDigestMismatch);
    }
    if expected.reused_wake_digest != replayed.reused_wake_digest {
        mismatch_classes.push(TemporalReplayMismatchClass::ReusedWakeDigestMismatch);
    }
    if expected.interval_regeneration_digest != replayed.interval_regeneration_digest {
        mismatch_classes.push(TemporalReplayMismatchClass::IntervalRegenerationDigestMismatch);
    }
    if expected.temporal_eligibility_digest != replayed.temporal_eligibility_digest {
        mismatch_classes.push(TemporalReplayMismatchClass::TemporalEligibilityDigestMismatch);
    }
    if expected.previous_value_reference_digest != replayed.previous_value_reference_digest {
        mismatch_classes.push(TemporalReplayMismatchClass::PreviousValueReferenceDigestMismatch);
    }
    if expected.certification_digest != replayed.certification_digest {
        mismatch_classes.push(TemporalReplayMismatchClass::CertificationDigestMismatch);
    }
    if expected.wake_summary != replayed.wake_summary {
        mismatch_classes.push(TemporalReplayMismatchClass::WakeSummaryMismatch);
    }
    TemporalReplayParityReport {
        proof_schema_version: if expected.expired_retired_wake_count == 0
            && replayed.expired_retired_wake_count == 0
        {
            TEMPORAL_REPLAY_PARITY_SCHEMA_VERSION
        } else {
            TEMPORAL_REPLAY_PARITY_WITH_OMISSION_SCHEMA_VERSION
        }
        .to_owned(),
        expected: expected.clone(),
        replayed: replayed.clone(),
        parity: mismatch_classes.is_empty(),
        mismatch_classes,
    }
}

impl TemporalReconstructabilityArtifact {
    pub fn from_evidence(
        wake_summary: TemporalWakeSummary,
        evidence: &TemporalTransactionEvidence,
    ) -> Self {
        Self::from_evidence_with_omission(wake_summary, evidence, RetentionOmission::default())
    }

    fn from_evidence_with_omission(
        wake_summary: TemporalWakeSummary,
        evidence: &TemporalTransactionEvidence,
        omitted: RetentionOmission,
    ) -> Self {
        let clock_checkpoint_digest = canonical_digest(&evidence.clock_basis);
        let scheduled_wake_digest = canonical_digest(&evidence.scheduled_wakes);
        let ready_wake_digest = canonical_digest(&evidence.ready_wakes);
        let retired_wake_digest = canonical_digest(&evidence.retired_wakes);
        let rescheduled_wake_digest = canonical_digest(&evidence.rescheduled_wakes);
        let reused_wake_digest = canonical_digest(&evidence.reused_wakes);
        let interval_regeneration_digest = canonical_digest(&evidence.interval_regenerations);
        let temporal_eligibility_digest = canonical_digest(&evidence.eligibility_facts);
        let previous_value_reference_digest = canonical_digest(&evidence.previous_value_references);
        let certification_digest = canonical_digest(&TemporalCertificationDigestBasis {
            clock_checkpoint_digest: &clock_checkpoint_digest,
            scheduled_wake_digest: &scheduled_wake_digest,
            ready_wake_digest: &ready_wake_digest,
            retired_wake_digest: &retired_wake_digest,
            expired_retired_wake_count: omitted.count(),
            expired_retired_wake_digest: omitted.digest(),
            rescheduled_wake_digest: &rescheduled_wake_digest,
            reused_wake_digest: &reused_wake_digest,
            interval_regeneration_digest: &interval_regeneration_digest,
            temporal_eligibility_digest: &temporal_eligibility_digest,
            previous_value_reference_digest: &previous_value_reference_digest,
        });
        Self {
            clock_basis: evidence.clock_basis,
            wake_summary,
            eligibility_fact_count: evidence.eligibility_facts.len() as u64,
            scheduled_wake_count: evidence.scheduled_wakes.len() as u64,
            ready_wake_count: evidence.ready_wakes.len() as u64,
            retired_wake_count: evidence.retired_wakes.len() as u64,
            expired_retired_wake_count: omitted.count(),
            expired_retired_wake_high_water: omitted.high_water(),
            expired_retired_wake_digest: omitted.digest(),
            rescheduled_wake_count: evidence.rescheduled_wakes.len() as u64,
            reused_wake_count: evidence.reused_wakes.len() as u64,
            interval_regeneration_count: evidence.interval_regenerations.len() as u64,
            previous_value_reference_count: evidence.previous_value_references.len() as u64,
            clock_checkpoint_digest,
            scheduled_wake_digest,
            ready_wake_digest,
            retired_wake_digest,
            rescheduled_wake_digest,
            reused_wake_digest,
            interval_regeneration_digest,
            temporal_eligibility_digest,
            previous_value_reference_digest,
            certification_digest,
        }
    }

    pub(in crate::logic::transaction::runtime) fn from_temporal_state(
        temporal: &TemporalRuntimeState,
    ) -> Self {
        let evidence = TemporalTransactionEvidence {
            clock_basis: temporal.clock_basis(),
            eligibility_facts: Vec::new(),
            scheduled_wakes: temporal.scheduled_wake_evidence(),
            ready_wakes: temporal.ready_wake_evidence(),
            retired_wakes: temporal.retired_wake_evidence(),
            rescheduled_wakes: Vec::new(),
            reused_wakes: Vec::new(),
            interval_regenerations: Vec::new(),
            previous_value_references: Vec::new(),
        };
        Self::from_evidence_with_omission(
            temporal.wake_summary(),
            &evidence,
            temporal.expired_retired_wakes,
        )
    }
}

#[derive(Debug, Serialize)]
struct TemporalCertificationDigestBasis<'a> {
    clock_checkpoint_digest: &'a str,
    scheduled_wake_digest: &'a str,
    ready_wake_digest: &'a str,
    retired_wake_digest: &'a str,
    #[serde(skip_serializing_if = "is_zero_u64")]
    expired_retired_wake_count: u64,
    #[serde(skip_serializing_if = "is_zero_digest")]
    expired_retired_wake_digest: [u8; 32],
    rescheduled_wake_digest: &'a str,
    reused_wake_digest: &'a str,
    interval_regeneration_digest: &'a str,
    temporal_eligibility_digest: &'a str,
    previous_value_reference_digest: &'a str,
}

#[cfg(test)]
mod compatibility_tests {
    use serde::Serialize;

    use super::{
        canonical_digest, temporal_replay_parity_report, TemporalReconstructabilityArtifact,
        TEMPORAL_REPLAY_PARITY_SCHEMA_VERSION,
    };

    #[derive(Serialize)]
    struct LegacyCertificationBasis<'a> {
        clock_checkpoint_digest: &'a str,
        scheduled_wake_digest: &'a str,
        ready_wake_digest: &'a str,
        retired_wake_digest: &'a str,
        rescheduled_wake_digest: &'a str,
        reused_wake_digest: &'a str,
        interval_regeneration_digest: &'a str,
        temporal_eligibility_digest: &'a str,
        previous_value_reference_digest: &'a str,
    }

    #[test]
    fn no_omission_temporal_artifact_keeps_v1_serialization_and_certification_digest() {
        let artifact = TemporalReconstructabilityArtifact::default();
        let legacy_digest = canonical_digest(&LegacyCertificationBasis {
            clock_checkpoint_digest: &artifact.clock_checkpoint_digest,
            scheduled_wake_digest: &artifact.scheduled_wake_digest,
            ready_wake_digest: &artifact.ready_wake_digest,
            retired_wake_digest: &artifact.retired_wake_digest,
            rescheduled_wake_digest: &artifact.rescheduled_wake_digest,
            reused_wake_digest: &artifact.reused_wake_digest,
            interval_regeneration_digest: &artifact.interval_regeneration_digest,
            temporal_eligibility_digest: &artifact.temporal_eligibility_digest,
            previous_value_reference_digest: &artifact.previous_value_reference_digest,
        });
        assert_eq!(artifact.certification_digest, legacy_digest);
        let serialized = serde_json::to_value(&artifact).unwrap();
        for field in [
            "expired_retired_wake_count",
            "expired_retired_wake_high_water",
            "expired_retired_wake_digest",
        ] {
            assert!(serialized.get(field).is_none(), "v1 omitted {field}");
        }
        let decoded: TemporalReconstructabilityArtifact =
            serde_json::from_value(serialized).unwrap();
        assert_eq!(decoded, artifact);
        let parity = temporal_replay_parity_report(&artifact, &decoded);
        assert!(parity.parity);
        assert_eq!(
            parity.proof_schema_version,
            TEMPORAL_REPLAY_PARITY_SCHEMA_VERSION
        );
    }
}
