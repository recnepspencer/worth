use std::path::Path;

use worth_store_physical_integrity::{
    IndeterminatePhysicalIntegrityCause, PhysicalArtifactScope, PhysicalBlastRadius,
    PhysicalByteRange, PhysicalDamageLocalization, PhysicalIntegrityRejection,
    PhysicalIntegrityVersionAxis, UnknownPhysicalIntegrityCause,
};
use worth_store_recovery_runtime::{
    PhysicalRecoveryBlockKind, PhysicalRecoveryOutcome, PhysicalRecoveryRefusalKind,
    PhysicalRecoveryRootProtocolArtifact, PhysicalRecoveryRootProtocolCounters,
    PhysicalRecoveryRootProtocolDenial, PhysicalRecoverySourceDenial, WorthStoreRecovery,
};

use super::process_integrity_vocabulary::{
    project_artifact_family, project_damage_cause, project_format_field,
};
use super::process_recovery_observation::{
    ProcessBlastRadius, ProcessByteRange, ProcessDamageLocalization,
    ProcessIndeterminateIntegrityCause, ProcessIntegrityRejection, ProcessIntegrityScope,
    ProcessIntegrityVersionAxis, ProcessRecoveryBlockCause, ProcessRecoveryDiscoveryCounters,
    ProcessRecoveryObservation, ProcessRecoveryPosture, ProcessRecoveryRefusalCause,
    ProcessRecoveryRootProtocolCounters, ProcessRootProtocolArtifact, ProcessRootProtocolDenial,
    ProcessRootProtocolDenialKind, ProcessUnknownIntegrityCause,
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
        },
        PhysicalRecoveryOutcome::Refused(refusal) => ProcessRecoveryObservation {
            observed_store_identity: None,
            posture: ProcessRecoveryPosture::Refused(project_refusal_cause(refusal.kind)),
            recovery_effects: refusal.recovery_effects(),
            discovery: None,
            root_protocol: project_root_protocol_counters(refusal.root_protocol_counters()),
            root_protocol_denials: project_root_protocol_denials(refusal.root_protocol_denials()),
        },
        PhysicalRecoveryOutcome::Blocked(block) => {
            let evidence = block.evidence();
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

fn project_integrity_rejection(rejection: PhysicalIntegrityRejection) -> ProcessIntegrityRejection {
    match rejection {
        PhysicalIntegrityRejection::Damaged(localization) => {
            ProcessIntegrityRejection::Damaged(project_damage_localization(localization))
        }
        PhysicalIntegrityRejection::Unsupported(posture) => {
            ProcessIntegrityRejection::Unsupported {
                scope: project_integrity_scope(posture.scope()),
                axis: match posture.axis() {
                    PhysicalIntegrityVersionAxis::EnvelopeSchema => {
                        ProcessIntegrityVersionAxis::EnvelopeSchema
                    }
                    PhysicalIntegrityVersionAxis::PhysicalFormat => {
                        ProcessIntegrityVersionAxis::PhysicalFormat
                    }
                    PhysicalIntegrityVersionAxis::PhysicalWorkObligation => {
                        ProcessIntegrityVersionAxis::PhysicalWorkObligation
                    }
                    PhysicalIntegrityVersionAxis::WalFrame => ProcessIntegrityVersionAxis::WalFrame,
                    PhysicalIntegrityVersionAxis::CheckpointRecordSchema => {
                        ProcessIntegrityVersionAxis::CheckpointRecordSchema
                    }
                },
                observed: posture.observed(),
            }
        }
        PhysicalIntegrityRejection::Unknown(posture) => ProcessIntegrityRejection::Unknown {
            scope: project_integrity_scope(posture.scope()),
            cause: match posture.cause() {
                UnknownPhysicalIntegrityCause::ExpectedArtifactAbsent => {
                    ProcessUnknownIntegrityCause::ExpectedArtifactAbsent
                }
                UnknownPhysicalIntegrityCause::UnrecognizedArtifact => {
                    ProcessUnknownIntegrityCause::UnrecognizedArtifact
                }
                UnknownPhysicalIntegrityCause::ExpectedScopeUnavailable => {
                    ProcessUnknownIntegrityCause::ExpectedScopeUnavailable
                }
            },
        },
        PhysicalIntegrityRejection::Indeterminate(posture) => {
            ProcessIntegrityRejection::Indeterminate {
                scope: project_integrity_scope(posture.scope()),
                cause: match posture.cause() {
                    IndeterminatePhysicalIntegrityCause::SourceChangedDuringInspection => {
                        ProcessIndeterminateIntegrityCause::SourceChangedDuringInspection
                    }
                    IndeterminatePhysicalIntegrityCause::ObservationBoundExhausted => {
                        ProcessIndeterminateIntegrityCause::ObservationBoundExhausted
                    }
                    IndeterminatePhysicalIntegrityCause::StableRangeNotProven => {
                        ProcessIndeterminateIntegrityCause::StableRangeNotProven
                    }
                },
                observed_range: posture.observed_range().map(project_byte_range),
            }
        }
    }
}

fn project_damage_localization(
    localization: PhysicalDamageLocalization,
) -> ProcessDamageLocalization {
    ProcessDamageLocalization {
        scope: project_integrity_scope(localization.scope()),
        cause: project_damage_cause(localization.cause()),
        damaged_range: project_byte_range(localization.damaged_range()),
        field: localization.field().map(project_format_field),
        blast_radius: match localization.blast_radius() {
            PhysicalBlastRadius::DamagedRange => ProcessBlastRadius::DamagedRange,
            PhysicalBlastRadius::CanonicalFrame => ProcessBlastRadius::CanonicalFrame,
            PhysicalBlastRadius::CompleteArtifact => ProcessBlastRadius::CompleteArtifact,
            PhysicalBlastRadius::ReachableSubtree => ProcessBlastRadius::ReachableSubtree,
        },
    }
}

fn project_integrity_scope(scope: PhysicalArtifactScope) -> ProcessIntegrityScope {
    ProcessIntegrityScope {
        store_identity: scope.store_identity().bytes(),
        family: project_artifact_family(scope.artifact_family()),
        root_generation: scope.root_generation(),
        byte_range: project_byte_range(scope.byte_range()),
        record_format_identity: scope
            .durable_frame_record_format()
            .map(|format| format.canonical_identity_bytes()),
    }
}

fn project_byte_range(range: PhysicalByteRange) -> ProcessByteRange {
    ProcessByteRange {
        offset: range.offset(),
        length: range.length(),
    }
}
