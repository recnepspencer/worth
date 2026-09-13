//! Bounded coverage conclusions retain their container and acquisition provenance.
use super::{
    BoundedMediaWalk, OfflineArtifactObservation, OfflineIndeterminatePhysicalReason,
    OfflineIntegrityOutcome as Outcome, OfflinePhysicalBlastRadius, OfflinePhysicalDamageCause,
    OfflinePhysicalDamageLocalization, OfflinePhysicalFormatField, OfflineUnknownPhysicalReason,
};
use worth_foundational::{
    PhysicalArtifactFamily, PhysicalArtifactGeneration, PhysicalArtifactIdentity,
};

#[derive(Clone, Copy)]
struct AdmittedWalRange {
    segment: u64,
    generation: u64,
    start: u64,
    end: u64,
}

pub(crate) struct WalCoverage {
    ranges: Vec<AdmittedWalRange>,
    uncertainty: Option<Outcome>,
}

impl WalCoverage {
    pub(crate) fn new(incomplete: Option<OfflineIndeterminatePhysicalReason>) -> Self {
        Self {
            ranges: Vec::new(),
            uncertainty: incomplete.map(Outcome::Indeterminate),
        }
    }

    pub(crate) fn admit(&mut self, segment: u64, generation: u64, start: u64, end: u64) {
        self.ranges.push(AdmittedWalRange {
            segment,
            generation,
            start,
            end,
        });
    }

    pub(crate) fn unresolved(&mut self, outcome: &Outcome) {
        // Rejected-present bytes are not proof that their LSN coverage is absent.
        // Preserve an actual acquisition failure over a merely unknown relation.
        if let Outcome::Indeterminate(reason) = outcome {
            self.uncertainty = Some(Outcome::Indeterminate(*reason));
        } else if self.uncertainty.is_none() {
            self.uncertainty = Some(Outcome::Unknown(
                OfflineUnknownPhysicalReason::WalCoverageUnavailable,
            ));
        }
    }

    pub(crate) fn finish(mut self, walk: &mut BoundedMediaWalk) -> Vec<OfflineArtifactObservation> {
        self.ranges.sort_unstable_by_key(|range| {
            (range.start, range.end, range.segment, range.generation)
        });
        let mut observations = Vec::new();
        let mut prior: Option<AdmittedWalRange> = None;
        for range in self.ranges {
            if let Some(previous) = prior {
                if range.start > previous.end {
                    let outcome = self.uncertainty.clone().unwrap_or_else(|| {
                        if range.generation != previous.generation
                            || range.segment <= previous.segment
                        {
                            return Outcome::Unknown(
                                OfflineUnknownPhysicalReason::WalCoverageUnavailable,
                            );
                        }
                        let missing_container = range.segment - previous.segment > 1;
                        let cause = if missing_container {
                            walk.counters_mut().missing_artifacts += 1;
                            OfflinePhysicalDamageCause::MissingArtifact
                        } else {
                            // Adjacent present containers disagree; do not invent a missing file.
                            OfflinePhysicalDamageCause::ScopeMismatch
                        };
                        Outcome::Damaged(OfflinePhysicalDamageLocalization::new(
                            cause,
                            None,
                            Some(OfflinePhysicalFormatField::WalLsn),
                            OfflinePhysicalBlastRadius::Artifact,
                        ))
                    });
                    walk.record_outcome(&outcome);
                    observations.push(OfflineArtifactObservation::new(
                        "families/wal",
                        PhysicalArtifactFamily::WalFrame.into(),
                        PhysicalArtifactIdentity::new(format!(
                            "wal-required-lsn:{}:{}",
                            previous.end, range.start,
                        ))
                        .unwrap(),
                        PhysicalArtifactGeneration::NotEncoded,
                        None,
                        outcome,
                    ));
                }
                if range.end <= previous.end {
                    continue;
                }
            }
            prior = Some(range);
        }
        observations
    }
}
