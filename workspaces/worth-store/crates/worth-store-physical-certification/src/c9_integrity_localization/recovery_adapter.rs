use std::path::Path;

use worth_store_recovery_runtime::{
    PhysicalRecoveryBlockKind, PhysicalRecoveryOutcome, PhysicalRecoveryRefusalKind,
    PhysicalRecoveryRootProtocolArtifact, PhysicalRecoveryRootProtocolCounters,
    PhysicalRecoveryRootProtocolDenial, PhysicalRecoverySourceDenial, WorthStoreRecovery,
};

use super::process_ingress_observation::{project_counters, project_ingress, project_wal};
use super::process_integrity_projection::project_integrity_rejection;
use super::process_recovery_observation::{
    ProcessRecoveryBlockCause, ProcessRecoveryDiscoveryCounters, ProcessRecoveryObservation,
    ProcessRecoveryPosture, ProcessRecoveryRefusalCause, ProcessRecoveryRootProtocolCounters,
    ProcessRootProtocolArtifact, ProcessRootProtocolDenial, ProcessRootProtocolDenialKind,
};
use super::recovery_request::open_request;

pub(crate) fn recover(root: &Path) -> Result<ProcessRecoveryObservation, String> {
    let request = open_request(root)?;
    Ok(project_outcome(WorthStoreRecovery::recover(request)))
}

fn project_outcome(outcome: PhysicalRecoveryOutcome) -> ProcessRecoveryObservation {
    match outcome {
        PhysicalRecoveryOutcome::Recovered(handoff) => ProcessRecoveryObservation {
            observed_store_identity: Some(handoff.core().store_identity().bytes()),
            posture: ProcessRecoveryPosture::Recovered,
            recovery_effects: handoff.core().recovery_effect_count(),
            discovery: Some(project_discovery(handoff.discovery_counters())),
            root_protocol: project_root_protocol_counters(handoff.root_protocol_counters()),
            root_protocol_denials: project_root_protocol_denials(handoff.root_protocol_denials()),
            ingress: project_ingress(handoff.integrity_observations()),
            wal: project_wal(handoff.wal_integrity_observations()),
            integrity_counters: project_counters(handoff.integrity_counters()),
        },
        PhysicalRecoveryOutcome::Refused(refusal) => ProcessRecoveryObservation {
            observed_store_identity: None,
            posture: ProcessRecoveryPosture::Refused(project_refusal_cause(refusal.kind)),
            recovery_effects: refusal.recovery_effects(),
            discovery: None,
            root_protocol: project_root_protocol_counters(refusal.root_protocol_counters()),
            root_protocol_denials: project_root_protocol_denials(refusal.root_protocol_denials()),
            ingress: project_ingress(refusal.integrity_observations()),
            wal: project_wal(refusal.wal_integrity_observations().wal()),
            integrity_counters: project_counters(refusal.integrity_counters()),
        },
        PhysicalRecoveryOutcome::Blocked(block) => {
            let evidence = block.evidence();
            println!(
                "C9 recovery block kind={:?} planning={:?} sources={:?}",
                block.kind, evidence.planning_denial, evidence.source_denials
            );
            ProcessRecoveryObservation {
                observed_store_identity: Some(block.store_identity().bytes()),
                posture: ProcessRecoveryPosture::Blocked(project_block_cause(block.kind)),
                recovery_effects: block.recovery_effects(),
                discovery: Some(project_discovery(evidence.counters)),
                root_protocol: evidence
                    .root_protocol_counters
                    .map(project_root_protocol_counters)
                    .unwrap_or_default(),
                root_protocol_denials: project_root_protocol_denials(&evidence.source_denials),
                ingress: project_ingress(evidence.integrity_observations()),
                wal: project_wal(evidence.integrity_observations.wal()),
                integrity_counters: project_counters(evidence.integrity_counters()),
            }
        }
        PhysicalRecoveryOutcome::PublicationIndeterminate(indeterminate) => {
            ProcessRecoveryObservation {
                observed_store_identity: Some(indeterminate.store_identity().bytes()),
                posture: ProcessRecoveryPosture::PublicationIndeterminate,
                recovery_effects: indeterminate.recovery_effects(),
                discovery: None,
                root_protocol: project_root_protocol_counters(
                    indeterminate.root_protocol_counters(),
                ),
                root_protocol_denials: project_root_protocol_denials(
                    indeterminate.root_protocol_denials(),
                ),
                ingress: project_ingress(indeterminate.integrity_observations()),
                wal: project_wal(indeterminate.wal_integrity_observations().wal()),
                integrity_counters: project_counters(indeterminate.integrity_counters()),
            }
        }
    }
}

fn project_refusal_cause(kind: PhysicalRecoveryRefusalKind) -> ProcessRecoveryRefusalCause {
    match kind {
        PhysicalRecoveryRefusalKind::CancelledBeforeDiscovery => {
            ProcessRecoveryRefusalCause::CancelledBeforeDiscovery
        }
        PhysicalRecoveryRefusalKind::CancelledBeforeReconstruction => {
            ProcessRecoveryRefusalCause::CancelledBeforeReconstruction
        }
        PhysicalRecoveryRefusalKind::CancelledBeforeExecution => {
            ProcessRecoveryRefusalCause::CancelledBeforeExecution
        }
        PhysicalRecoveryRefusalKind::EntryBindingDrift(_) => {
            ProcessRecoveryRefusalCause::EntryBindingDrift
        }
        PhysicalRecoveryRefusalKind::PersistedStoreAdmission(_) => {
            ProcessRecoveryRefusalCause::PersistedStoreAdmission
        }
        PhysicalRecoveryRefusalKind::CoordinationUnavailable => {
            ProcessRecoveryRefusalCause::CoordinationUnavailable
        }
    }
}

fn project_block_cause(kind: PhysicalRecoveryBlockKind) -> ProcessRecoveryBlockCause {
    match kind {
        PhysicalRecoveryBlockKind::DiscoveryLimit => ProcessRecoveryBlockCause::DiscoveryLimit,
        PhysicalRecoveryBlockKind::MediaObservation => ProcessRecoveryBlockCause::MediaObservation,
        PhysicalRecoveryBlockKind::RootProtocol => ProcessRecoveryBlockCause::RootProtocol,
        PhysicalRecoveryBlockKind::Checkpoint => ProcessRecoveryBlockCause::Checkpoint,
        PhysicalRecoveryBlockKind::WalInventory => ProcessRecoveryBlockCause::WalInventory,
        PhysicalRecoveryBlockKind::SourceSelection => ProcessRecoveryBlockCause::SourceSelection,
        PhysicalRecoveryBlockKind::BindingFreshness => ProcessRecoveryBlockCause::BindingFreshness,
        PhysicalRecoveryBlockKind::PageAdmission => ProcessRecoveryBlockCause::PageAdmission,
        PhysicalRecoveryBlockKind::OperationReconciliation => {
            ProcessRecoveryBlockCause::OperationReconciliation
        }
        PhysicalRecoveryBlockKind::RedoPlanning => ProcessRecoveryBlockCause::RedoPlanning,
        PhysicalRecoveryBlockKind::Staging => ProcessRecoveryBlockCause::Staging,
        PhysicalRecoveryBlockKind::Publication => ProcessRecoveryBlockCause::Publication,
    }
}

fn project_discovery(
    counters: worth_store_recovery_runtime::PhysicalRecoveryDiscoveryCounters,
) -> ProcessRecoveryDiscoveryCounters {
    ProcessRecoveryDiscoveryCounters {
        current_selector_integrity_admissions: counters.current_selector_integrity_admissions,
        previous_selector_integrity_admissions: counters.previous_selector_integrity_admissions,
        current_selector_interpretations: counters.current_selector_interpretations,
        previous_selector_interpretations: counters.previous_selector_interpretations,
        current_root_integrity_admissions: counters.current_root_integrity_admissions,
        previous_root_integrity_admissions: counters.previous_root_integrity_admissions,
        current_root_candidate_interpretations: counters.current_root_candidate_interpretations,
        previous_root_candidate_interpretations: counters.previous_root_candidate_interpretations,
    }
}

fn project_root_protocol_counters(
    counters: PhysicalRecoveryRootProtocolCounters,
) -> ProcessRecoveryRootProtocolCounters {
    ProcessRecoveryRootProtocolCounters {
        successor_root_integrity_admissions: counters.successor_root_integrity_admissions(),
        successor_root_interpretations: counters.successor_root_interpretations(),
        staged_selector_integrity_admissions: counters.staged_selector_integrity_admissions(),
        closeout_selector_interpretations: counters.closeout_selector_interpretations(),
    }
}

fn project_root_protocol_denials(
    denials: &[PhysicalRecoverySourceDenial],
) -> Vec<ProcessRootProtocolDenial> {
    denials
        .iter()
        .filter_map(|denial| match denial {
            PhysicalRecoverySourceDenial::RootProtocol { artifact, denial } => {
                Some(ProcessRootProtocolDenial {
                    artifact: project_root_protocol_artifact(*artifact),
                    denial: project_root_protocol_denial(*denial),
                })
            }
            _ => None,
        })
        .collect()
}

fn project_root_protocol_artifact(
    artifact: PhysicalRecoveryRootProtocolArtifact,
) -> ProcessRootProtocolArtifact {
    match artifact {
        PhysicalRecoveryRootProtocolArtifact::BootstrapCatalog => {
            ProcessRootProtocolArtifact::BootstrapCatalog
        }
        PhysicalRecoveryRootProtocolArtifact::CheckpointSourceRoot { generation } => {
            ProcessRootProtocolArtifact::CheckpointSourceRoot { generation }
        }
        PhysicalRecoveryRootProtocolArtifact::CurrentSelector => {
            ProcessRootProtocolArtifact::CurrentSelector
        }
        PhysicalRecoveryRootProtocolArtifact::PreviousSelector => {
            ProcessRootProtocolArtifact::PreviousSelector
        }
        PhysicalRecoveryRootProtocolArtifact::StagedCurrentSelector { publication } => {
            ProcessRootProtocolArtifact::StagedCurrentSelector { publication }
        }
        PhysicalRecoveryRootProtocolArtifact::CurrentRoot { generation } => {
            ProcessRootProtocolArtifact::CurrentRoot { generation }
        }
        PhysicalRecoveryRootProtocolArtifact::PreviousRoot { generation } => {
            ProcessRootProtocolArtifact::PreviousRoot { generation }
        }
    }
}

fn project_root_protocol_denial(
    denial: PhysicalRecoveryRootProtocolDenial,
) -> ProcessRootProtocolDenialKind {
    match denial {
        PhysicalRecoveryRootProtocolDenial::Absent => ProcessRootProtocolDenialKind::Absent,
        PhysicalRecoveryRootProtocolDenial::ConflictingDuplication { observed_sources } => {
            ProcessRootProtocolDenialKind::ConflictingDuplication { observed_sources }
        }
        PhysicalRecoveryRootProtocolDenial::Integrity(rejection) => {
            ProcessRootProtocolDenialKind::Integrity(project_integrity_rejection(rejection))
        }
        PhysicalRecoveryRootProtocolDenial::NonCanonicalEncoding => {
            ProcessRootProtocolDenialKind::NonCanonicalEncoding
        }
        PhysicalRecoveryRootProtocolDenial::ScopeMismatch => {
            ProcessRootProtocolDenialKind::ScopeMismatch
        }
        PhysicalRecoveryRootProtocolDenial::SourceIncarnationMismatch => {
            ProcessRootProtocolDenialKind::SourceIncarnationMismatch
        }
    }
}
