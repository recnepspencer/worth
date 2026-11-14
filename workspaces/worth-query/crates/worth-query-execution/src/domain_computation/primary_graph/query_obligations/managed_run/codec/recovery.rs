//! Original typed failures survive successful subsequent resource cleanup.
use super::{cleanup, Denial, Reader};
use crate::domain_computation::artifact_owner::WorthQueryArtifactDenialKind as Artifact;
use crate::domain_computation::primary_graph::query_obligations::QueryManagedRunRecovery as Recovery;
use crate::domain_computation::{
    WorthQueryDirectReadmissionRecoveryKind as Readmission,
    WorthQueryManagedRunCleanupFailureKind as Cleanup,
    WorthQueryMutationCleanupFailureKind as Mutation,
    WorthQueryProviderCheckpointRetentionFailureKind as Retention,
    WorthQueryProviderCheckpointSuspensionFailureKind as Suspension,
    WorthQueryYieldRecoveryKind as Yield,
};
use worth_runtime_bridge::facade::BridgeExecutionBasisSignalTerminal as Signal;
pub(super) fn encode(out: &mut Vec<u8>, cause: Recovery) {
    match cause {
        Recovery::Cleanup(cause) => {
            out.push(0);
            out.push(cleanup::failure(Mutation::Native(cause)));
        }
        Recovery::Yield { cause, state } => {
            out.push(1);
            out.push(match state {
                super::super::QueryManagedYieldRecoveryState::Running => 0,
                super::super::QueryManagedYieldRecoveryState::Terminalized => 1,
            });
            match cause {
                Yield::ArtifactProductionFreeze(cause) => {
                    out.push(0);
                    out.push(artifact(cause));
                }
                Yield::BridgeTerminalization(cause) => {
                    out.push(1);
                    out.push(cleanup::failure(Mutation::Native(
                        Cleanup::BridgeFinalization(cause),
                    )));
                }
                Yield::SignalAttemptAlreadyTerminal(cause) => {
                    out.push(2);
                    out.push(match cause {
                        Signal::Fulfilled => 0,
                        Signal::Cancelled => 1,
                        Signal::TimedOut => 2,
                        Signal::Rejected => 3,
                        Signal::Superseded => 4,
                    });
                }
                Yield::ProviderCheckpointSuspension(cause) => {
                    out.push(3);
                    out.push(match cause {
                        Suspension::ProviderRejected => 0,
                        Suspension::ProviderPanicked => 1,
                        Suspension::ProviderExecutionReleaseRecoveryRequired => 2,
                        Suspension::CheckpointMemoryMismatch => 3,
                        Suspension::CheckpointRetention(Retention::RetainedByteProbePanicked) => 4,
                    });
                }
                Yield::RetainedBytesExceeded => out.push(4),
            }
        }
        Recovery::Readmission(cause) => {
            out.push(2);
            out.push(match cause {
                Readmission::BridgeCleanupFailed => 0,
                Readmission::ProviderRestorePanicked => 1,
                Readmission::ProviderRestoreRejectedAfterExecutionAdmission => 2,
                Readmission::RestoredExecutionReleaseRecoveryRequired => 3,
                Readmission::CheckpointReleasePanicked => 4,
            });
        }
    }
}
pub(super) fn decode(reader: &mut Reader<'_>) -> Result<Recovery, Denial> {
    Ok(match reader.take::<1>()?[0] {
        0 => {
            let Mutation::Native(cause) = cleanup::read_failure(reader.take::<1>()?[0])? else {
                return Err(Denial::MalformedFrame);
            };
            Recovery::Cleanup(cause)
        }
        1 => Recovery::Yield {
            state: match reader.take::<1>()?[0] {
                0 => super::super::QueryManagedYieldRecoveryState::Running,
                1 => super::super::QueryManagedYieldRecoveryState::Terminalized,
                _ => return Err(Denial::MalformedFrame),
            },
            cause: match reader.take::<1>()?[0] {
                0 => Yield::ArtifactProductionFreeze(read_artifact(reader.take::<1>()?[0])?),
                1 => {
                    let Mutation::Native(Cleanup::BridgeFinalization(cause)) =
                        cleanup::read_failure(reader.take::<1>()?[0])?
                    else {
                        return Err(Denial::MalformedFrame);
                    };
                    Yield::BridgeTerminalization(cause)
                }
                2 => Yield::SignalAttemptAlreadyTerminal(match reader.take::<1>()?[0] {
                    0 => Signal::Fulfilled,
                    1 => Signal::Cancelled,
                    2 => Signal::TimedOut,
                    3 => Signal::Rejected,
                    4 => Signal::Superseded,
                    _ => return Err(Denial::MalformedFrame),
                }),
                3 => Yield::ProviderCheckpointSuspension(match reader.take::<1>()?[0] {
                    0 => Suspension::ProviderRejected,
                    1 => Suspension::ProviderPanicked,
                    2 => Suspension::ProviderExecutionReleaseRecoveryRequired,
                    3 => Suspension::CheckpointMemoryMismatch,
                    4 => Suspension::CheckpointRetention(Retention::RetainedByteProbePanicked),
                    _ => return Err(Denial::MalformedFrame),
                }),
                4 => Yield::RetainedBytesExceeded,
                _ => return Err(Denial::MalformedFrame),
            },
        },
        2 => Recovery::Readmission(match reader.take::<1>()?[0] {
            0 => Readmission::BridgeCleanupFailed,
            1 => Readmission::ProviderRestorePanicked,
            2 => Readmission::ProviderRestoreRejectedAfterExecutionAdmission,
            3 => Readmission::RestoredExecutionReleaseRecoveryRequired,
            4 => Readmission::CheckpointReleasePanicked,
            _ => return Err(Denial::MalformedFrame),
        }),
        _ => return Err(Denial::MalformedFrame),
    })
}
fn artifact(cause: Artifact) -> u8 {
    match cause {
        Artifact::ArtifactContractNotInstalled => 0,
        Artifact::ProducerRoleNotInstalled => 1,
        Artifact::ConsumerRoleNotInstalled => 2,
        Artifact::ProviderFamilyMismatch => 3,
        Artifact::EmptySemanticProjection => 4,
        Artifact::InvalidProductionEvidence => 5,
        Artifact::ForeignRuntime => 6,
        Artifact::StaleInstallationGeneration => 7,
        Artifact::OperationMismatch => 8,
        Artifact::RunMismatch => 9,
        Artifact::StageMismatch => 10,
        Artifact::BasisMismatch => 11,
        Artifact::ArtifactContractMismatch => 12,
        Artifact::PayloadOwnerMismatch => 13,
        Artifact::StageExecutionMismatch => 14,
        Artifact::ActiveWorkflowRun => 15,
        Artifact::ProductionClosed => 16,
        Artifact::MovementForbidden => 17,
        Artifact::BorrowForbidden => 18,
        Artifact::LeaseForbidden => 19,
        Artifact::StaleLifecycleGeneration => 20,
        Artifact::ActiveBorrow => 21,
        Artifact::ActiveLease => 22,
        Artifact::AlreadyDisposed => 23,
    }
}
fn read_artifact(tag: u8) -> Result<Artifact, Denial> {
    Ok(match tag {
        0 => Artifact::ArtifactContractNotInstalled,
        1 => Artifact::ProducerRoleNotInstalled,
        2 => Artifact::ConsumerRoleNotInstalled,
        3 => Artifact::ProviderFamilyMismatch,
        4 => Artifact::EmptySemanticProjection,
        5 => Artifact::InvalidProductionEvidence,
        6 => Artifact::ForeignRuntime,
        7 => Artifact::StaleInstallationGeneration,
        8 => Artifact::OperationMismatch,
        9 => Artifact::RunMismatch,
        10 => Artifact::StageMismatch,
        11 => Artifact::BasisMismatch,
        12 => Artifact::ArtifactContractMismatch,
        13 => Artifact::PayloadOwnerMismatch,
        14 => Artifact::StageExecutionMismatch,
        15 => Artifact::ActiveWorkflowRun,
        16 => Artifact::ProductionClosed,
        17 => Artifact::MovementForbidden,
        18 => Artifact::BorrowForbidden,
        19 => Artifact::LeaseForbidden,
        20 => Artifact::StaleLifecycleGeneration,
        21 => Artifact::ActiveBorrow,
        22 => Artifact::ActiveLease,
        23 => Artifact::AlreadyDisposed,
        _ => return Err(Denial::MalformedFrame),
    })
}
