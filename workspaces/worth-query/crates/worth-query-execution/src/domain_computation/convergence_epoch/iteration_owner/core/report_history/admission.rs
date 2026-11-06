use std::sync::Arc;

use worth_query_installation::facade::WorthQueryConvergenceOscillationPosture;

use super::super::super::WorthQueryConvergenceEpochCore;
use super::incumbent_transition::prepare_incumbent_transition;
use crate::domain_computation::convergence_epoch::{
    WorthQueryBoundConvergenceReport, WorthQueryConvergenceEpochDenial,
    WorthQueryConvergenceEpochDenialKind,
};
use crate::domain_computation::provider_session::graph_provider::convergence_port::WorthQueryConvergenceDomainAssessmentOutcome;
use crate::execution_digest::hash_parts;
use {
    crate::domain_computation::provider_session::graph_provider::convergence_port::WorthQueryConvergenceDisposition,
    crate::domain_computation::provider_session::graph_provider::convergence_port::WorthQueryConvergenceDomainDecision,
    crate::domain_computation::provider_session::graph_provider::convergence_port::WorthQueryConvergenceFeasibility,
    crate::domain_computation::provider_session::graph_provider::convergence_port::WorthQueryConvergenceIncumbentUpdate,
    crate::domain_computation::provider_session::graph_provider::convergence_port::WorthQueryConvergenceProgress,
    crate::domain_computation::provider_session::graph_provider::convergence_port::WorthQueryConvergenceRepeatedState,
    crate::domain_computation::WorthQueryBoundGraphExecutionReceipt,
    crate::domain_computation::WorthQueryConvergenceDomainEvidenceBinding,
};

pub(in crate::domain_computation::convergence_epoch::iteration_owner) enum WorthQueryConvergenceReportAdmissionFailure
{
    Epoch(WorthQueryConvergenceEpochDenial),
}

pub(in crate::domain_computation::convergence_epoch::iteration_owner) fn admit_assessed_domain_report(
    core: &mut WorthQueryConvergenceEpochCore,
    receipt: &WorthQueryBoundGraphExecutionReceipt,
    outcome: WorthQueryConvergenceDomainAssessmentOutcome,
    domain_evidence: WorthQueryConvergenceDomainEvidenceBinding,
) -> Result<WorthQueryConvergenceDisposition, WorthQueryConvergenceReportAdmissionFailure> {
    let (decision, domain_work) = outcome.into_parts();
    validate_domain_evidence(core, receipt, &decision, &domain_evidence)?;
    validate_domain_decision(core, &decision)?;

    let candidate_occurrence_identity =
        Arc::<str>::from(domain_evidence.candidate_occurrence_identity());
    let iteration_ordinal = core.counters().iteration_count();
    let report_identity = bound_report_identity(
        core,
        receipt,
        &decision,
        &candidate_occurrence_identity,
        domain_work,
    );
    let disposition = decision.disposition();
    let report = WorthQueryBoundConvergenceReport::new(
        report_identity,
        receipt.provider_receipt(),
        receipt.evidence_identity(),
        iteration_ordinal,
        decision,
        domain_work,
        receipt.work_report(),
    );
    let prepared = prepare_incumbent_transition(
        core.contract().incumbent_posture(),
        core.incumbents(),
        candidate_occurrence_identity,
        report,
    )
    .map_err(|detail| epoch_failure(core, detail))?;
    core.commit_prepared_report(prepared);
    Ok(disposition)
}

fn validate_domain_evidence(
    core: &WorthQueryConvergenceEpochCore,
    receipt: &WorthQueryBoundGraphExecutionReceipt,
    decision: &WorthQueryConvergenceDomainDecision,
    evidence: &WorthQueryConvergenceDomainEvidenceBinding,
) -> Result<(), WorthQueryConvergenceReportAdmissionFailure> {
    if !evidence
        .contract()
        .is_some_and(|authority| core.contract().admits_artifact_authority(authority))
        || !receipt.admits_domain_evidence(evidence)
        || evidence.candidate_selection_key() != decision.candidate_selection_key()
    {
        return Err(domain_report_failure(
            core,
            "candidate evidence does not bind the exact installed contract and graph execution",
        ));
    }
    Ok(())
}

fn validate_domain_decision(
    core: &WorthQueryConvergenceEpochCore,
    decision: &WorthQueryConvergenceDomainDecision,
) -> Result<(), WorthQueryConvergenceReportAdmissionFailure> {
    if !oscillation_policy_is_coherent(core, decision) {
        return Err(domain_report_failure(
            core,
            "domain oscillation evidence contradicts the installed oscillation posture",
        ));
    }
    let coherent = match decision.disposition() {
        WorthQueryConvergenceDisposition::StableWithoutProof => {
            decision.progress() == WorthQueryConvergenceProgress::Stable
        }
        WorthQueryConvergenceDisposition::FeasibleIncumbent => {
            decision.feasibility() == WorthQueryConvergenceFeasibility::Feasible
                && update_leaves_incumbent(core, decision.incumbent_update())
        }
        WorthQueryConvergenceDisposition::Oscillating => {
            decision.repeated_state() == WorthQueryConvergenceRepeatedState::Repeated
                || core.contract().oscillation_posture()
                    == WorthQueryConvergenceOscillationPosture::DomainClassified
        }
        _ => true,
    };
    if !coherent {
        return Err(domain_report_failure(
            core,
            "domain terminal disposition contradicts its accompanying semantic evidence",
        ));
    }
    Ok(())
}

fn oscillation_policy_is_coherent(
    core: &WorthQueryConvergenceEpochCore,
    decision: &WorthQueryConvergenceDomainDecision,
) -> bool {
    let repeated = decision.repeated_state() == WorthQueryConvergenceRepeatedState::Repeated;
    let oscillating = decision.disposition() == WorthQueryConvergenceDisposition::Oscillating;
    match core.contract().oscillation_posture() {
        WorthQueryConvergenceOscillationPosture::Impossible => !repeated && !oscillating,
        WorthQueryConvergenceOscillationPosture::DetectAndDeny => {
            repeated == oscillating
                && (!oscillating || update_does_not_select_candidate(decision.incumbent_update()))
        }
        WorthQueryConvergenceOscillationPosture::DetectAndSelectIncumbent => {
            repeated == oscillating
                && (!oscillating || update_leaves_incumbent(core, decision.incumbent_update()))
        }
        WorthQueryConvergenceOscillationPosture::DomainClassified => true,
    }
}

fn update_does_not_select_candidate(update: &WorthQueryConvergenceIncumbentUpdate) -> bool {
    matches!(
        update,
        WorthQueryConvergenceIncumbentUpdate::Retain | WorthQueryConvergenceIncumbentUpdate::Clear
    )
}

fn update_leaves_incumbent(
    core: &WorthQueryConvergenceEpochCore,
    update: &WorthQueryConvergenceIncumbentUpdate,
) -> bool {
    match update {
        WorthQueryConvergenceIncumbentUpdate::Retain => !core.incumbents().is_empty(),
        WorthQueryConvergenceIncumbentUpdate::ReplaceWithCandidate
        | WorthQueryConvergenceIncumbentUpdate::AddCandidate
        | WorthQueryConvergenceIncumbentUpdate::RemoveCandidatesAndAdd { .. } => true,
        WorthQueryConvergenceIncumbentUpdate::Clear => false,
    }
}

fn bound_report_identity(
    core: &WorthQueryConvergenceEpochCore,
    receipt: &WorthQueryBoundGraphExecutionReceipt,
    decision: &WorthQueryConvergenceDomainDecision,
    candidate_occurrence_identity: &str,
    domain_work: crate::domain_computation::provider_session::graph_provider::convergence_port::WorthQueryConvergenceDomainWorkEvidence,
) -> String {
    hash_parts(&[
        "worth_query_bound_convergence_report_v1".into(),
        format!("epoch:{}", core.identity()),
        format!("iteration:{}", core.counters().iteration_count()),
        format!("graph-evidence:{}", receipt.evidence_identity()),
        format!("provider-receipt:{}", receipt.provider_receipt()),
        format!("candidate-selection:{}", decision.candidate_selection_key()),
        format!("candidate-occurrence:{candidate_occurrence_identity}"),
        format!("state:{}", decision.state_identity()),
        format!("disposition:{}", decision.disposition().canonical_name()),
        format!("feasibility:{}", decision.feasibility().canonical_name()),
        format!("progress:{}", decision.progress().canonical_name()),
        format!(
            "repeated-state:{}",
            decision.repeated_state().canonical_name()
        ),
        format!(
            "incumbent:{}",
            decision.incumbent_update().canonical_identity()
        ),
        format!(
            "domain-work:{}:{}:{}",
            domain_work.comparator_call_count(),
            domain_work.progress_check_count(),
            domain_work.repeated_state_probe_count()
        ),
    ])
}

fn epoch_failure(
    core: &WorthQueryConvergenceEpochCore,
    detail: &'static str,
) -> WorthQueryConvergenceReportAdmissionFailure {
    WorthQueryConvergenceReportAdmissionFailure::Epoch(WorthQueryConvergenceEpochDenial::new(
        WorthQueryConvergenceEpochDenialKind::InvalidIncumbentTransition,
        Arc::<str>::from(detail),
        core.counters().clone(),
    ))
}

fn domain_report_failure(
    core: &WorthQueryConvergenceEpochCore,
    detail: &'static str,
) -> WorthQueryConvergenceReportAdmissionFailure {
    WorthQueryConvergenceReportAdmissionFailure::Epoch(WorthQueryConvergenceEpochDenial::new(
        WorthQueryConvergenceEpochDenialKind::InvalidDomainReport,
        Arc::<str>::from(detail),
        core.counters().clone(),
    ))
}
