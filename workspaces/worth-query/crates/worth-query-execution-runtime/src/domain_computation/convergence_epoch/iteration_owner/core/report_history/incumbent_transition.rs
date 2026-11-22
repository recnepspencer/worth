use std::sync::Arc;

use worth_query_installation::facade::WorthQueryConvergenceIncumbentPosture;

use super::WorthQueryPreparedConvergenceReportCommit;
use crate::domain_computation::{
    WorthQueryBoundConvergenceReport, WorthQueryConvergenceDomainDecision,
    WorthQueryConvergenceFeasibility, WorthQueryConvergenceIncumbentUpdate,
    WorthQueryRetainedConvergenceCandidateEvidence,
};

pub(super) fn prepare_incumbent_transition(
    posture: WorthQueryConvergenceIncumbentPosture,
    incumbents: &[WorthQueryRetainedConvergenceCandidateEvidence],
    candidate_occurrence_identity: Arc<str>,
    report: WorthQueryBoundConvergenceReport,
) -> Result<WorthQueryPreparedConvergenceReportCommit, &'static str> {
    validate_incumbent_update(
        posture,
        incumbents,
        report.decision(),
        &candidate_occurrence_identity,
    )?;
    let prepared = match report.decision().incumbent_update() {
        WorthQueryConvergenceIncumbentUpdate::Retain => {
            WorthQueryPreparedConvergenceReportCommit::retain(report)
        }
        WorthQueryConvergenceIncumbentUpdate::ReplaceWithCandidate => {
            let candidate = retained_candidate(&candidate_occurrence_identity, &report);
            WorthQueryPreparedConvergenceReportCommit::replace_with_candidate(report, candidate)
        }
        WorthQueryConvergenceIncumbentUpdate::AddCandidate => {
            let candidate = retained_candidate(&candidate_occurrence_identity, &report);
            WorthQueryPreparedConvergenceReportCommit::add_candidate(report, candidate)
        }
        WorthQueryConvergenceIncumbentUpdate::RemoveCandidatesAndAdd { .. } => {
            let candidate = retained_candidate(&candidate_occurrence_identity, &report);
            WorthQueryPreparedConvergenceReportCommit::remove_candidates_and_add(report, candidate)
        }
        WorthQueryConvergenceIncumbentUpdate::Clear => {
            WorthQueryPreparedConvergenceReportCommit::clear(report)
        }
    };
    Ok(prepared)
}

fn validate_incumbent_update(
    posture: WorthQueryConvergenceIncumbentPosture,
    incumbents: &[WorthQueryRetainedConvergenceCandidateEvidence],
    decision: &WorthQueryConvergenceDomainDecision,
    candidate_occurrence_identity: &str,
) -> Result<(), &'static str> {
    let valid = match posture {
        WorthQueryConvergenceIncumbentPosture::NoIncumbent => {
            no_incumbent_transition_is_valid(decision.incumbent_update())
        }
        WorthQueryConvergenceIncumbentPosture::FirstFeasible => {
            first_feasible_transition_is_valid(incumbents, decision)
        }
        WorthQueryConvergenceIncumbentPosture::BestObserved => {
            best_observed_transition_is_valid(decision.incumbent_update())
        }
        WorthQueryConvergenceIncumbentPosture::ParetoFrontier => pareto_transition_is_valid(
            incumbents,
            decision.incumbent_update(),
            candidate_occurrence_identity,
        ),
    };
    valid
        .then_some(())
        .ok_or("domain incumbent transition contradicts the installed incumbent posture")
}

fn no_incumbent_transition_is_valid(update: &WorthQueryConvergenceIncumbentUpdate) -> bool {
    matches!(
        update,
        WorthQueryConvergenceIncumbentUpdate::Retain | WorthQueryConvergenceIncumbentUpdate::Clear
    )
}

fn first_feasible_transition_is_valid(
    incumbents: &[WorthQueryRetainedConvergenceCandidateEvidence],
    decision: &WorthQueryConvergenceDomainDecision,
) -> bool {
    match decision.incumbent_update() {
        WorthQueryConvergenceIncumbentUpdate::Retain => true,
        WorthQueryConvergenceIncumbentUpdate::ReplaceWithCandidate => {
            incumbents.is_empty()
                && decision.feasibility() == WorthQueryConvergenceFeasibility::Feasible
        }
        _ => false,
    }
}

fn best_observed_transition_is_valid(update: &WorthQueryConvergenceIncumbentUpdate) -> bool {
    matches!(
        update,
        WorthQueryConvergenceIncumbentUpdate::Retain
            | WorthQueryConvergenceIncumbentUpdate::ReplaceWithCandidate
            | WorthQueryConvergenceIncumbentUpdate::Clear
    )
}

fn pareto_transition_is_valid(
    incumbents: &[WorthQueryRetainedConvergenceCandidateEvidence],
    update: &WorthQueryConvergenceIncumbentUpdate,
    candidate_occurrence_identity: &str,
) -> bool {
    match update {
        WorthQueryConvergenceIncumbentUpdate::Retain
        | WorthQueryConvergenceIncumbentUpdate::Clear => true,
        WorthQueryConvergenceIncumbentUpdate::AddCandidate => {
            !contains_incumbent(incumbents, candidate_occurrence_identity)
        }
        WorthQueryConvergenceIncumbentUpdate::RemoveCandidatesAndAdd {
            removed_occurrence_identities,
        } => {
            let every_removal_exists = removed_occurrence_identities
                .iter()
                .all(|removed| contains_incumbent(incumbents, removed));
            let candidate_collision = incumbents.iter().any(|incumbent| {
                incumbent.occurrence_identity() == candidate_occurrence_identity
                    && !removed_occurrence_identities
                        .iter()
                        .any(|removed| removed.as_ref() == candidate_occurrence_identity)
            });
            every_removal_exists && !candidate_collision
        }
        WorthQueryConvergenceIncumbentUpdate::ReplaceWithCandidate => false,
    }
}

fn retained_candidate(
    candidate_occurrence_identity: &Arc<str>,
    report: &WorthQueryBoundConvergenceReport,
) -> WorthQueryRetainedConvergenceCandidateEvidence {
    WorthQueryRetainedConvergenceCandidateEvidence::new(
        Arc::clone(candidate_occurrence_identity),
        report.decision().state_identity(),
        report.evidence_identity(),
    )
}

fn contains_incumbent(
    incumbents: &[WorthQueryRetainedConvergenceCandidateEvidence],
    occurrence_identity: &str,
) -> bool {
    incumbents
        .iter()
        .any(|incumbent| incumbent.occurrence_identity() == occurrence_identity)
}
