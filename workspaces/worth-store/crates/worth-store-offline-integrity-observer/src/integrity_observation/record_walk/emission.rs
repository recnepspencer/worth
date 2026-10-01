use std::collections::HashMap;

use super::super::{
    OfflineArtifactFamily, OfflineArtifactObservation, OfflineIndeterminatePhysicalReason,
    OfflineIntegrityOutcome,
};
use super::{damage, Blast, Cause, ChildExpectation};

type ScopeKey = (String, OfflineArtifactFamily, Option<u64>, Option<String>);

/// Root-specific validation and arena accounting run for every selected root,
/// but each canonical report scope is emitted only once.
#[derive(Default)]
pub(super) struct EmittedChildScopes {
    seen: HashMap<ScopeKey, EmittedScope>,
}

struct EmittedScope {
    expected: Option<ChildExpectation>,
    index: usize,
    conflicting: bool,
}

impl EmittedChildScopes {
    pub(super) fn seed(observations: &mut Vec<OfflineArtifactObservation>) -> Self {
        let mut emitted = Self::default();
        for observation in std::mem::take(observations) {
            emitted.push_inner(observations, None, observation);
        }
        emitted
    }

    pub(super) fn push(
        &mut self,
        observations: &mut Vec<OfflineArtifactObservation>,
        expected: &ChildExpectation,
        observation: OfflineArtifactObservation,
    ) {
        self.push_inner(observations, Some(expected), observation);
    }

    pub(super) fn push_untyped(
        &mut self,
        observations: &mut Vec<OfflineArtifactObservation>,
        observation: OfflineArtifactObservation,
    ) {
        self.push_inner(observations, None, observation);
    }

    fn push_inner(
        &mut self,
        observations: &mut Vec<OfflineArtifactObservation>,
        expected: Option<&ChildExpectation>,
        observation: OfflineArtifactObservation,
    ) {
        let offset = observation.range().map(|range| range.offset());
        let key = (
            observation.relative_path().to_owned(),
            observation.family(),
            offset,
            offset
                .is_none()
                .then(|| observation.identity().as_str().to_owned()),
        );
        if let Some(scope) = self.seen.get_mut(&key) {
            let conflicting_expectation = scope
                .expected
                .as_ref()
                .zip(expected)
                .is_some_and(|(prior, current)| prior != current);
            let prior = &observations[scope.index];
            let conflicting_identity = prior.identity() != observation.identity()
                || prior.generation() != observation.generation();
            scope.conflicting |= conflicting_expectation || conflicting_identity;
            let outcome = if scope.conflicting {
                damage(Cause::ScopeMismatch, None, Blast::Artifact)
            } else {
                repeated_observation_outcome(prior, &observation)
            };
            let mut merged = prior.clone().with_outcome(outcome);
            for duplicate in observation.duplicates() {
                if !merged.duplicates().contains(duplicate) {
                    merged = merged.with_duplicate(duplicate.clone());
                }
            }
            observations[scope.index] = merged;
            if scope.expected.is_none() {
                scope.expected = expected.cloned();
            }
        } else {
            self.seen.insert(
                key,
                EmittedScope {
                    expected: expected.cloned(),
                    index: observations.len(),
                    conflicting: false,
                },
            );
            observations.push(observation);
        }
    }
}

fn repeated_observation_outcome(
    prior: &OfflineArtifactObservation,
    current: &OfflineArtifactObservation,
) -> OfflineIntegrityOutcome {
    use OfflineIndeterminatePhysicalReason::SourceChanged;
    use OfflineIntegrityOutcome::{Indeterminate, Unknown};
    match (prior.outcome(), current.outcome()) {
        (Indeterminate(SourceChanged), _) | (_, Indeterminate(SourceChanged)) => {
            Indeterminate(SourceChanged)
        }
        (Indeterminate(reason), _) | (_, Indeterminate(reason)) => Indeterminate(*reason),
        (Unknown(reason), _) | (_, Unknown(reason)) => Unknown(*reason),
        (first, second) if first == second && prior.range() == current.range() => first.clone(),
        // Different conclusions or measured lengths for the same declared scope
        // cannot establish a stable source. They do not prove a bad route.
        _ => Indeterminate(SourceChanged),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrity_observation::{child_expectation::ChildScope, OfflineIntegrityOutcome};
    use worth_foundational::PhysicalArtifactFamily;

    fn page(generation: u64) -> ChildExpectation {
        ChildExpectation {
            path: "families/records/segments/segment-1-1.pages".to_owned(),
            family: PhysicalArtifactFamily::PageFrame,
            generation,
            format: [0; 10],
            offset: 0,
            length: Some(4096),
            checksum: None,
            scope: ChildScope::Page {
                segment: 1,
                page: 1,
                pages: 1,
            },
        }
    }

    #[test]
    fn shared_selected_scope_is_reported_once_after_both_root_walks() {
        let expected = page(1);
        let mut emitted = EmittedChildScopes::default();
        let mut observations = Vec::new();
        for _ in 0..2 {
            emitted.push(
                &mut observations,
                &expected,
                super::super::project(&expected, 4096, OfflineIntegrityOutcome::Intact),
            );
        }
        assert_eq!(observations.len(), 1);
        assert_eq!(observations[0].outcome(), &OfflineIntegrityOutcome::Intact);
    }

    #[test]
    fn historical_graph_scope_is_seeded_before_selected_root_walk() {
        let expected = page(1);
        let historical = super::super::project(&expected, 4096, OfflineIntegrityOutcome::Intact);
        let mut observations = vec![historical.clone(), historical.clone()];
        let mut emitted = EmittedChildScopes::seed(&mut observations);
        emitted.push(&mut observations, &expected, historical);
        emitted.push_untyped(
            &mut observations,
            super::super::project(&expected, 4096, OfflineIntegrityOutcome::Intact),
        );
        assert_eq!(observations.len(), 1);
        assert_eq!(observations[0].outcome(), &OfflineIntegrityOutcome::Intact);
    }

    #[test]
    fn repeated_scope_preserves_acquisition_uncertainty_in_either_order() {
        use OfflineIndeterminatePhysicalReason::{ElapsedBoundExceeded, SourceChanged};
        for reason in [ElapsedBoundExceeded, SourceChanged] {
            for uncertain_first in [false, true] {
                let mut expected = page(1);
                expected.length = None;
                let intact =
                    super::super::project(&expected, 4096, OfflineIntegrityOutcome::Intact);
                let uncertain = super::super::project(
                    &expected,
                    8192,
                    OfflineIntegrityOutcome::Indeterminate(reason),
                );
                let sequence = if uncertain_first {
                    [uncertain, intact]
                } else {
                    [intact, uncertain]
                };
                let mut observations = Vec::new();
                let mut emitted = EmittedChildScopes::default();
                for observation in sequence {
                    emitted.push(&mut observations, &expected, observation);
                }
                assert_eq!(observations.len(), 1);
                assert_eq!(
                    observations[0].outcome(),
                    &OfflineIntegrityOutcome::Indeterminate(reason)
                );
            }
        }
    }

    #[test]
    fn conclusive_source_disagreement_makes_the_final_report_indeterminate() {
        use crate::integrity_observation::{
            OfflineIntegrityObservationCounters, OfflineIntegrityObservationLimits,
            OfflineIntegrityProtocolContext, OfflineIntegrityReport,
            OfflineIntegrityReportCompleteness,
        };
        let mut expected = page(1);
        expected.length = None;
        let mut observations = Vec::new();
        let mut emitted = EmittedChildScopes::default();
        for length in [4096, 8192] {
            emitted.push(
                &mut observations,
                &expected,
                super::super::project(&expected, length, OfflineIntegrityOutcome::Intact),
            );
        }
        let report = OfflineIntegrityReport::new(
            OfflineIntegrityProtocolContext::new("observer", "process", "run", "scenario").unwrap(),
            None,
            OfflineIntegrityObservationLimits::new(8, 8192, 5, 4, 0, 1000, 8192).unwrap(),
            OfflineIntegrityObservationCounters::default(),
            OfflineIntegrityReportCompleteness::Complete,
            observations,
            None,
            Default::default(),
        );
        assert_eq!(report.artifacts().len(), 1);
        assert_eq!(
            report.artifacts()[0].outcome(),
            &OfflineIntegrityOutcome::Indeterminate(
                OfflineIndeterminatePhysicalReason::SourceChanged
            )
        );
        assert_eq!(
            report.completeness(),
            OfflineIntegrityReportCompleteness::Indeterminate
        );
        assert_eq!(report.counters().indeterminate_reads(), 0);
    }

    #[test]
    fn root_scoped_accounting_preserves_each_root_outcome() {
        use worth_foundational::{PhysicalArtifactGeneration, PhysicalArtifactIdentity};
        let mut emitted = EmittedChildScopes::default();
        let mut observations = Vec::new();
        for generation in [4, 5] {
            emitted.push_untyped(
                &mut observations,
                OfflineArtifactObservation::new(
                    "families/records/arenas/arena-1.data",
                    PhysicalArtifactFamily::ExtentArenaFrame.into(),
                    PhysicalArtifactIdentity::new(format!("arena-accounting:{generation}:arena-1"))
                        .unwrap(),
                    PhysicalArtifactGeneration::encoded(generation).unwrap(),
                    None,
                    OfflineIntegrityOutcome::Intact,
                ),
            );
        }
        assert_eq!(observations.len(), 2);
        assert!(observations
            .iter()
            .all(|row| row.outcome() == &OfflineIntegrityOutcome::Intact));
    }

    #[test]
    fn conflicting_root_expectations_damage_the_one_report_scope() {
        let first = page(1);
        let second = page(2);
        let mut emitted = EmittedChildScopes::default();
        let mut observations = Vec::new();
        for expected in [&first, &second, &first] {
            emitted.push(
                &mut observations,
                expected,
                super::super::project(expected, 4096, OfflineIntegrityOutcome::Intact),
            );
        }
        assert_eq!(observations.len(), 1);
        assert!(matches!(
            observations[0].outcome(),
            OfflineIntegrityOutcome::Damaged(value) if value.cause() == Cause::ScopeMismatch
        ));
    }
}
