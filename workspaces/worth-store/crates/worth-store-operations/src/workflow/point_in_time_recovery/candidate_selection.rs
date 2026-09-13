//! Deterministic point-in-time candidate ordering for Operations workflows.

use sha2::{Digest, Sha256};
use worth_proof::{CanonicalVec, NonEmpty};

use super::exact_frontier::{
    ExactRecoveryFrontier, RecoveryTimelineObservation, RecoveryTimelineOwner,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PitrCandidatePosture {
    Available,
    Degraded,
    Unavailable,
    Indeterminate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PitrRoundingPolicy {
    ExactOnly,
    PreviousAcknowledged,
    NextAcknowledged,
    NearestAcknowledged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum CandidateTimeRelation {
    Before,
    ContainsRequestedTime,
    After,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PointInTimeCandidate {
    relation: CandidateTimeRelation,
    distance: u64,
    observed_time: i64,
    clock_interval_start: i64,
    clock_interval_end: i64,
    frontier: ExactRecoveryFrontier,
    source_identity: [u8; 32],
    posture: PitrCandidatePosture,
    identity: [u8; 32],
}

impl PointInTimeCandidate {
    fn from_observation(
        requested_time: i64,
        rounding: PitrRoundingPolicy,
        observation: RecoveryTimelineObservation,
    ) -> Self {
        let lower =
            saturating_sub_uncertainty(observation.observed_time, observation.uncertainty_before);
        let upper =
            saturating_add_uncertainty(observation.observed_time, observation.uncertainty_after);
        let (relation, distance) = if requested_time < lower {
            (CandidateTimeRelation::After, lower.abs_diff(requested_time))
        } else if requested_time > upper {
            (
                CandidateTimeRelation::Before,
                requested_time.abs_diff(upper),
            )
        } else {
            (CandidateTimeRelation::ContainsRequestedTime, 0)
        };
        let mut digest = Sha256::new();
        digest.update(b"worth-store-point-in-time-candidate-v1");
        digest.update(requested_time.to_be_bytes());
        digest.update([
            rounding_tag(rounding),
            relation_tag(relation),
            posture_tag(observation.posture),
        ]);
        digest.update(distance.to_be_bytes());
        digest.update(observation.observed_time.to_be_bytes());
        digest.update(lower.to_be_bytes());
        digest.update(upper.to_be_bytes());
        digest.update(observation.frontier.identity());
        digest.update(observation.source_identity);
        Self {
            relation,
            distance,
            observed_time: observation.observed_time,
            clock_interval_start: lower,
            clock_interval_end: upper,
            frontier: observation.frontier,
            source_identity: observation.source_identity,
            posture: observation.posture,
            identity: digest.finalize().into(),
        }
    }
    pub const fn exact_frontier(self) -> ExactRecoveryFrontier {
        self.frontier
    }
    pub const fn source_identity(self) -> [u8; 32] {
        self.source_identity
    }
    pub const fn posture(self) -> PitrCandidatePosture {
        self.posture
    }
    pub const fn clock_distance(self) -> u64 {
        self.distance
    }
    pub const fn observed_time(self) -> i64 {
        self.observed_time
    }
    pub const fn clock_uncertainty_interval(self) -> (i64, i64) {
        (self.clock_interval_start, self.clock_interval_end)
    }
    pub const fn identity(self) -> [u8; 32] {
        self.identity
    }
}

fn saturating_sub_uncertainty(observed: i64, uncertainty: u64) -> i64 {
    i64::try_from(uncertainty).map_or(i64::MIN, |value| observed.saturating_sub(value))
}

fn saturating_add_uncertainty(observed: i64, uncertainty: u64) -> i64 {
    i64::try_from(uncertainty).map_or(i64::MAX, |value| observed.saturating_add(value))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PointInTimeCandidateSet {
    requested_time: i64,
    rounding: PitrRoundingPolicy,
    candidates: CanonicalVec<PointInTimeCandidate>,
    evidence_count: usize,
}

impl PointInTimeCandidateSet {
    pub const fn requested_time(&self) -> i64 {
        self.requested_time
    }
    pub const fn rounding(&self) -> PitrRoundingPolicy {
        self.rounding
    }
    pub fn candidates(&self) -> &[PointInTimeCandidate] {
        self.candidates.as_slice()
    }
    pub const fn evidence_count(&self) -> usize {
        self.evidence_count
    }
    pub fn select(self) -> Result<PointInTimeCandidate, PitrCandidateSelectionDenial> {
        self.candidates
            .as_slice()
            .iter()
            .copied()
            .filter(|candidate| {
                matches!(
                    candidate.posture,
                    PitrCandidatePosture::Available | PitrCandidatePosture::Degraded
                ) && policy_allows(self.rounding, candidate.relation)
            })
            .min_by_key(|candidate| {
                (
                    candidate.distance,
                    candidate.observed_time,
                    candidate.frontier.identity(),
                )
            })
            .ok_or(PitrCandidateSelectionDenial::NoAdmissibleCandidate)
    }
}

impl RecoveryTimelineOwner {
    pub fn resolve_candidates(
        requested_time: i64,
        rounding: PitrRoundingPolicy,
        mut observations: Vec<RecoveryTimelineObservation>,
    ) -> Result<PointInTimeCandidateSet, PitrCandidateSelectionDenial> {
        observations.sort_by(|left, right| {
            left.observed_time
                .cmp(&right.observed_time)
                .then_with(|| left.frontier.identity().cmp(&right.frontier.identity()))
                .then_with(|| left.source_identity.cmp(&right.source_identity))
        });
        observations.dedup();
        let non_empty = NonEmpty::try_from_vec(observations.clone())
            .map_err(|_| PitrCandidateSelectionDenial::NoTimelineEvidence)?;
        let mut candidates = observations
            .into_iter()
            .map(|observation| {
                PointInTimeCandidate::from_observation(requested_time, rounding, observation)
            })
            .collect::<Vec<_>>();
        candidates.sort();
        let candidates = CanonicalVec::try_from_sorted(candidates)
            .map_err(|_| PitrCandidateSelectionDenial::AllocationFailed)?;
        Ok(PointInTimeCandidateSet {
            requested_time,
            rounding,
            candidates,
            evidence_count: non_empty.len(),
        })
    }
}

const fn rounding_tag(rounding: PitrRoundingPolicy) -> u8 {
    match rounding {
        PitrRoundingPolicy::ExactOnly => 1,
        PitrRoundingPolicy::PreviousAcknowledged => 2,
        PitrRoundingPolicy::NextAcknowledged => 3,
        PitrRoundingPolicy::NearestAcknowledged => 4,
    }
}

const fn relation_tag(relation: CandidateTimeRelation) -> u8 {
    match relation {
        CandidateTimeRelation::Before => 1,
        CandidateTimeRelation::ContainsRequestedTime => 2,
        CandidateTimeRelation::After => 3,
    }
}

const fn posture_tag(posture: PitrCandidatePosture) -> u8 {
    match posture {
        PitrCandidatePosture::Available => 1,
        PitrCandidatePosture::Degraded => 2,
        PitrCandidatePosture::Unavailable => 3,
        PitrCandidatePosture::Indeterminate => 4,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PitrCandidateSelectionDenial {
    NoTimelineEvidence,
    NoAdmissibleCandidate,
    AllocationFailed,
}

const fn policy_allows(policy: PitrRoundingPolicy, relation: CandidateTimeRelation) -> bool {
    match policy {
        PitrRoundingPolicy::ExactOnly => {
            matches!(relation, CandidateTimeRelation::ContainsRequestedTime)
        }
        PitrRoundingPolicy::PreviousAcknowledged => {
            !matches!(relation, CandidateTimeRelation::After)
        }
        PitrRoundingPolicy::NextAcknowledged => !matches!(relation, CandidateTimeRelation::Before),
        PitrRoundingPolicy::NearestAcknowledged => true,
    }
}
