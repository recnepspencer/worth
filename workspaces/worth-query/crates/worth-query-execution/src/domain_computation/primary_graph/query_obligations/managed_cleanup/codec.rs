//! Version-two cleanup payloads; no live owner or capability is encoded.
use super::super::{codec::Reader, QueryObligationDenial as Denial};
use super::*;
use std::num::NonZeroU64;
use worth_foundational::facade::{DurableWorldOwnerIdentity, HomeIdentity, WorldOwnerSlotIdentity};

pub(in crate::domain_computation::primary_graph::query_obligations) fn encode(
    bytes: &mut Vec<u8>,
    record: &QueryManagedCleanupRecord,
) {
    match record {
        QueryManagedCleanupRecord::Retain(row) => {
            bytes.push(0);
            identity(bytes, row.identity);
            bytes.extend_from_slice(&(row.run.len() as u32).to_le_bytes());
            bytes.extend_from_slice(row.run.as_bytes());
            bytes.push(failure(row.failure));
            match row.original {
                None => bytes.push(0),
                Some(original) => {
                    let (tag, owner, ordinal) = match original {
                        QueryManagedCleanupOriginal::Attempt(original) => {
                            (1, original.owner(), original.ordinal())
                        }
                        QueryManagedCleanupOriginal::Unpublished(original) => {
                            (2, original.owner(), original.ordinal())
                        }
                    };
                    bytes.push(tag);
                    bytes.extend_from_slice(&owner.home().bytes());
                    bytes.extend_from_slice(&owner.slot().get().to_le_bytes());
                    bytes.extend_from_slice(&ordinal.to_le_bytes());
                }
            }
        }
        QueryManagedCleanupRecord::Released {
            identity: id,
            release,
        } => {
            bytes.push(1);
            identity(bytes, *id);
            bytes.push(terminal(release.terminal));
            bytes.push(match release.disposition {
                WorthQueryManagedRunCleanupDisposition::CleanupComplete => 0,
                WorthQueryManagedRunCleanupDisposition::CleanupPending => 1,
                WorthQueryManagedRunCleanupDisposition::RecoveryRequired => 2,
            });
        }
        QueryManagedCleanupRecord::Failed {
            identity: id,
            failure: kind,
        } => {
            bytes.push(3);
            identity(bytes, *id);
            bytes.push(failure(*kind));
        }
        QueryManagedCleanupRecord::Discharge { identity: id } => {
            bytes.push(2);
            identity(bytes, *id);
        }
    }
}
fn identity(bytes: &mut Vec<u8>, identity: ManagedCleanupObligationIdentity) {
    bytes.extend_from_slice(&identity.acquire_position().get().to_le_bytes());
}
pub(in crate::domain_computation::primary_graph::query_obligations) fn decode(
    reader: &mut Reader<'_>,
    home: HomeIdentity,
) -> Result<QueryManagedCleanupRecord, Denial> {
    let tag = reader.take::<1>()?[0];
    let position =
        NonZeroU64::new(u64::from_le_bytes(reader.take()?)).ok_or(Denial::MalformedFrame)?;
    let identity = ManagedCleanupObligationIdentity::from_recorded(home, position);
    Ok(match tag {
        0 => {
            let length = u32::from_le_bytes(reader.take()?) as usize;
            let (run, tail) = reader
                .0
                .split_at_checked(length)
                .ok_or(Denial::MalformedFrame)?;
            let run = std::str::from_utf8(run)
                .map_err(|_| Denial::MalformedFrame)?
                .to_owned();
            reader.0 = tail;
            let failure = read_failure(reader.take::<1>()?[0])?;
            let original = match reader.take::<1>()?[0] {
                0 => None,
                tag @ (1 | 2) => {
                    let owner = DurableWorldOwnerIdentity::from_recorded(
                        HomeIdentity::from_observed_bytes(reader.take()?)
                            .map_err(|_| Denial::MalformedFrame)?,
                        WorldOwnerSlotIdentity::from_recorded(u64::from_le_bytes(reader.take()?)),
                    );
                    let ordinal = u64::from_le_bytes(reader.take()?);
                    Some(match tag {
                        1 => QueryManagedCleanupOriginal::Attempt(DurableCompositePublicationAttemptIdentity::from_recorded(owner, ordinal)),
                        2 => QueryManagedCleanupOriginal::Unpublished(worth_foundational::facade::DurableProductUnpublishedOwnerEffectsIdentity::from_recorded(owner, ordinal)),
                        _ => unreachable!("matched association tag"),
                    })
                }
                _ => return Err(Denial::MalformedFrame),
            };
            QueryManagedCleanupRecord::Retain(QueryManagedCleanupObligation {
                identity,
                run,
                failure,
                original,
                release: None,
            })
        }
        1 => QueryManagedCleanupRecord::Released {
            identity,
            release: QueryManagedCleanupRelease {
                terminal: read_terminal(reader.take::<1>()?[0])?,
                disposition: match reader.take::<1>()?[0] {
                    0 => WorthQueryManagedRunCleanupDisposition::CleanupComplete,
                    1 => WorthQueryManagedRunCleanupDisposition::CleanupPending,
                    2 => WorthQueryManagedRunCleanupDisposition::RecoveryRequired,
                    _ => return Err(Denial::MalformedFrame),
                },
            },
        },
        2 => QueryManagedCleanupRecord::Discharge { identity },
        3 => QueryManagedCleanupRecord::Failed {
            identity,
            failure: read_failure(reader.take::<1>()?[0])?,
        },
        _ => return Err(Denial::MalformedFrame),
    })
}
fn terminal(terminal: WorthQueryManagedRunTerminalKind) -> u8 {
    use WorthQueryManagedRunTerminalKind::*;
    match terminal {
        Completed => 0,
        Cancelled => 1,
        TimedOut => 2,
        Exhausted => 3,
        Degraded => 4,
        Failed => 5,
    }
}
fn read_terminal(tag: u8) -> Result<WorthQueryManagedRunTerminalKind, Denial> {
    use WorthQueryManagedRunTerminalKind::*;
    Ok(match tag {
        0 => Completed,
        1 => Cancelled,
        2 => TimedOut,
        3 => Exhausted,
        4 => Degraded,
        5 => Failed,
        _ => return Err(Denial::MalformedFrame),
    })
}
fn failure(kind: WorthQueryMutationCleanupFailureKind) -> u8 {
    use crate::domain_computation::WorthQueryManagedRunCleanupFailureKind as Native;
    use worth_runtime_bridge::facade::{
        BridgeExecutionBasisFinalizationFailureKind as Bridge,
        BridgeManagedQueueFailureKind as Queue,
    };
    match kind {
        WorthQueryMutationCleanupFailureKind::AffinityInconsistent => 0,
        WorthQueryMutationCleanupFailureKind::CompletionInconsistent => 1,
        WorthQueryMutationCleanupFailureKind::Native(native) => match native {
            Native::ProviderMemoryRetained => 2,
            Native::QueueRelease(queue) => match queue {
                Queue::AtomicExecutionUnsupported => 3,
                Queue::SignalRuntimeThreadAffinityViolation => 4,
                Queue::SignalQueueMutationDenied => 5,
                Queue::SignalRequestMismatch => 6,
            },
            Native::BridgeFinalization(bridge) => match bridge {
                Bridge::AtomicExecutionUnsupported => 7,
                Bridge::ManagedQueueOccupied => 8,
                Bridge::SignalRuntimeThreadAffinityViolation => 9,
                Bridge::SignalCompletionDenied => 10,
                Bridge::SignalCompletionStagingFailed => 11,
                Bridge::SignalCompletionCommitFailed => 12,
                Bridge::SignalCancellationDenied => 13,
                Bridge::SignalCancellationFailed => 14,
            },
        },
    }
}
fn read_failure(tag: u8) -> Result<WorthQueryMutationCleanupFailureKind, Denial> {
    use crate::domain_computation::WorthQueryManagedRunCleanupFailureKind as Native;
    use worth_runtime_bridge::facade::{
        BridgeExecutionBasisFinalizationFailureKind as Bridge,
        BridgeManagedQueueFailureKind as Queue,
    };
    use WorthQueryMutationCleanupFailureKind::{
        AffinityInconsistent, CompletionInconsistent, Native as Failure,
    };
    Ok(match tag {
        0 => AffinityInconsistent,
        1 => CompletionInconsistent,
        2 => Failure(Native::ProviderMemoryRetained),
        3 => Failure(Native::QueueRelease(Queue::AtomicExecutionUnsupported)),
        4 => Failure(Native::QueueRelease(
            Queue::SignalRuntimeThreadAffinityViolation,
        )),
        5 => Failure(Native::QueueRelease(Queue::SignalQueueMutationDenied)),
        6 => Failure(Native::QueueRelease(Queue::SignalRequestMismatch)),
        7 => Failure(Native::BridgeFinalization(
            Bridge::AtomicExecutionUnsupported,
        )),
        8 => Failure(Native::BridgeFinalization(Bridge::ManagedQueueOccupied)),
        9 => Failure(Native::BridgeFinalization(
            Bridge::SignalRuntimeThreadAffinityViolation,
        )),
        10 => Failure(Native::BridgeFinalization(Bridge::SignalCompletionDenied)),
        11 => Failure(Native::BridgeFinalization(
            Bridge::SignalCompletionStagingFailed,
        )),
        12 => Failure(Native::BridgeFinalization(
            Bridge::SignalCompletionCommitFailed,
        )),
        13 => Failure(Native::BridgeFinalization(Bridge::SignalCancellationDenied)),
        14 => Failure(Native::BridgeFinalization(Bridge::SignalCancellationFailed)),
        _ => return Err(Denial::MalformedFrame),
    })
}
