use worth_store::physical_runtime::{
    CompletedPhysicalRecoveryPublicationCandidate, CompletedPhysicalRecoveryPublicationCommand,
    PerformedRecoveryPhysicalEffect, PhysicalRecoveryPublicationCandidateMaterialization,
    PhysicalRecoveryPublicationCommandDenial, PhysicalRecoveryPublicationCommandIndeterminate,
    RecoveryRootProtocolReplacementAction,
};

use crate::entry::PhysicalRecoveryPublicationCounters;

pub(super) fn completed_counters(
    planned_effects: u64,
    completed: &CompletedPhysicalRecoveryPublicationCommand,
) -> PhysicalRecoveryPublicationCounters {
    let mut counters = prefix_counters(planned_effects, completed.candidates(), None, None);
    counters.root_protocol_replacements_performed = 1;
    counters.namespace_synchronizations_performed = 1;
    counters
}

pub(super) fn denial_counters(
    planned_effects: u64,
    denial: &PhysicalRecoveryPublicationCommandDenial,
) -> PhysicalRecoveryPublicationCounters {
    prefix_counters(
        planned_effects,
        denial.candidates(),
        denial.candidate_materialization(),
        denial.root_protocol(),
    )
}

pub(super) fn indeterminate_counters(
    planned_effects: u64,
    outcome: &PhysicalRecoveryPublicationCommandIndeterminate,
) -> PhysicalRecoveryPublicationCounters {
    use PhysicalRecoveryPublicationCommandIndeterminate as Indeterminate;

    match outcome {
        Indeterminate::CandidateMaterialization { completed, .. }
        | Indeterminate::CandidateMaterializationSettlement { completed, .. }
        | Indeterminate::CandidateMaterializationYieldpoint { completed, .. } => {
            prefix_counters(planned_effects, completed, None, None)
        }
        Indeterminate::CandidateSynchronization {
            materialization,
            completed,
            ..
        }
        | Indeterminate::CandidateSynchronizationSettlement {
            materialization,
            completed,
            ..
        } => prefix_counters(planned_effects, completed, Some(materialization), None),
        Indeterminate::CandidateSynchronizationYieldpoint {
            materialization,
            completed,
            ..
        } => prefix_counters(planned_effects, completed, Some(materialization), None),
        Indeterminate::Media {
            candidates,
            root_protocol,
            ..
        }
        | Indeterminate::Scheduler {
            candidates,
            root_protocol,
            ..
        }
        | Indeterminate::Signal {
            candidates,
            root_protocol,
            ..
        }
        | Indeterminate::Yieldpoint {
            candidates,
            root_protocol,
            ..
        } => prefix_counters(planned_effects, candidates, None, root_protocol.as_ref()),
    }
}

fn prefix_counters(
    planned_effects: u64,
    candidates: &[CompletedPhysicalRecoveryPublicationCandidate],
    current_materialization: Option<&PhysicalRecoveryPublicationCandidateMaterialization>,
    root_protocol: Option<&PerformedRecoveryPhysicalEffect<RecoveryRootProtocolReplacementAction>>,
) -> PhysicalRecoveryPublicationCounters {
    PhysicalRecoveryPublicationCounters {
        planned_effects,
        candidate_artifacts_settled: candidates.len() as u64,
        candidate_materializations_performed: candidates
            .iter()
            .map(CompletedPhysicalRecoveryPublicationCandidate::materialization)
            .chain(current_materialization)
                .filter(|materialization| {
                    matches!(
                        materialization,
                    PhysicalRecoveryPublicationCandidateMaterialization::Created(_)
                        | PhysicalRecoveryPublicationCandidateMaterialization::CompletedFromExactPrefix(_)
                    )
                })
            .count() as u64,
        candidate_synchronizations_performed: candidates.len() as u64,
        root_protocol_replacements_performed: u64::from(root_protocol.is_some()),
        namespace_synchronizations_performed: 0,
    }
}
