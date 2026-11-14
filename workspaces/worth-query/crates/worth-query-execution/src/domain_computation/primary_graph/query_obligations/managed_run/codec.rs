//! Version-two managed-work DATA, routed separately from cleanup obligations.
use super::super::{
    codec::Reader, managed_cleanup::codec as cleanup, QueryObligationDenial as Denial,
};
use super::*;
mod recovery;
mod release;
use worth_foundational::facade::HomeIdentity;
pub(in crate::domain_computation::primary_graph::query_obligations) fn encode(
    out: &mut Vec<u8>,
    record: &QueryManagedRunRecord,
) {
    let (tag, id) = match record {
        QueryManagedRunRecord::Retain(row) => (0, row.identity),
        QueryManagedRunRecord::Handoff { identity, .. } => (1, *identity),
        QueryManagedRunRecord::Discharge { identity } => (2, *identity),
        QueryManagedRunRecord::Original { identity, .. } => (3, *identity),
        QueryManagedRunRecord::Recovery { identity, .. } => (4, *identity),
    };
    out.push(tag);
    out.extend_from_slice(&id.acquire_position().get().to_le_bytes());
    match record {
        QueryManagedRunRecord::Retain(row) => {
            text(out, &row.logical_run);
            text(out, &row.original_attempt);
        }
        QueryManagedRunRecord::Handoff {
            posture, attempt, ..
        } => {
            text(out, attempt);
            match posture {
                QueryManagedRunPosture::Active => out.push(0),
                QueryManagedRunPosture::CleanupRequired { terminal, failure } => {
                    out.push(1);
                    out.push(cleanup::terminal(*terminal));
                    out.push(cleanup::failure(
                        crate::domain_computation::WorthQueryMutationCleanupFailureKind::Native(
                            *failure,
                        ),
                    ));
                }
                QueryManagedRunPosture::DirectCleanup {
                    terminal,
                    disposition,
                    resources,
                } => {
                    out.push(2);
                    out.push(cleanup::terminal(*terminal));
                    out.push(disposition_tag(*disposition));
                    release::encode(out, *resources);
                }
                QueryManagedRunPosture::YieldedCleanup {
                    disposition,
                    resources,
                } => {
                    out.push(3);
                    out.push(disposition_tag(*disposition));
                    release::encode(out, *resources);
                }
                QueryManagedRunPosture::ReadmissionCleanup {
                    disposition,
                    resources,
                } => {
                    out.push(5);
                    out.push(disposition_tag(*disposition));
                    release::encode(out, *resources);
                }
                QueryManagedRunPosture::WorkflowCleanup {
                    route,
                    disposition,
                    resources,
                    artifacts_released,
                } => {
                    out.push(6);
                    match route {
                        QueryManagedWorkflowCleanupRoute::Terminal(terminal) => {
                            out.push(0);
                            out.push(cleanup::terminal(*terminal));
                        }
                        QueryManagedWorkflowCleanupRoute::Yielded => out.push(1),
                        QueryManagedWorkflowCleanupRoute::Readmission => out.push(2),
                    }
                    out.push(disposition_tag(*disposition));
                    release::encode(out, *resources);
                    out.push(u8::from(*artifacts_released));
                }
                QueryManagedRunPosture::AdmissionReleased => out.push(4),
            }
        }
        QueryManagedRunRecord::Recovery { cause, .. } => recovery::encode(out, *cause),
        QueryManagedRunRecord::Discharge { .. } => {}
        QueryManagedRunRecord::Original { original, .. } => {
            super::super::codec::business_original::encode(out, Some(*original))
        }
    }
}
fn text(out: &mut Vec<u8>, text: &str) {
    out.extend_from_slice(&(text.len() as u32).to_le_bytes());
    out.extend_from_slice(text.as_bytes());
}
fn read_text(reader: &mut Reader<'_>) -> Result<String, Denial> {
    let size = u32::from_le_bytes(reader.take()?) as usize;
    let (value, tail) = reader
        .0
        .split_at_checked(size)
        .ok_or(Denial::MalformedFrame)?;
    let value = std::str::from_utf8(value)
        .map_err(|_| Denial::MalformedFrame)?
        .to_owned();
    reader.0 = tail;
    Ok(value)
}
fn disposition_tag(disposition: WorthQueryManagedRunCleanupDisposition) -> u8 {
    use WorthQueryManagedRunCleanupDisposition::*;
    match disposition {
        CleanupComplete => 0,
        CleanupPending => 1,
        RecoveryRequired => 2,
    }
}
fn read_disposition(
    reader: &mut Reader<'_>,
) -> Result<WorthQueryManagedRunCleanupDisposition, Denial> {
    use WorthQueryManagedRunCleanupDisposition::*;
    Ok(match reader.take::<1>()?[0] {
        0 => CleanupComplete,
        1 => CleanupPending,
        2 => RecoveryRequired,
        _ => return Err(Denial::MalformedFrame),
    })
}
pub(in crate::domain_computation::primary_graph::query_obligations) fn decode(
    reader: &mut Reader<'_>,
    home: HomeIdentity,
) -> Result<QueryManagedRunRecord, Denial> {
    let tag = reader.take::<1>()?[0];
    let position = std::num::NonZeroU64::new(u64::from_le_bytes(reader.take()?))
        .ok_or(Denial::MalformedFrame)?;
    let identity = ManagedRunObligationIdentity::from_recorded(home, position);
    Ok(match tag {
        0 => {
            let logical_run = read_text(reader)?;
            let attempt = read_text(reader)?;
            QueryManagedRunRecord::Retain(QueryManagedRunObligation {
                identity,
                logical_run,
                original_attempt: attempt.clone(),
                attempt,
                posture: QueryManagedRunPosture::Active,
                original: None,
                recovery: None,
            })
        }
        1 => QueryManagedRunRecord::Handoff {
            identity,
            attempt: read_text(reader)?,
            posture: match reader.take::<1>()?[0] {
                0 => QueryManagedRunPosture::Active,
                1 => {
                    let terminal = cleanup::read_terminal(reader.take::<1>()?[0])?;
                    let crate::domain_computation::WorthQueryMutationCleanupFailureKind::Native(
                        failure,
                    ) = cleanup::read_failure(reader.take::<1>()?[0])?
                    else {
                        return Err(Denial::MalformedFrame);
                    };
                    QueryManagedRunPosture::CleanupRequired { terminal, failure }
                }
                2 => QueryManagedRunPosture::DirectCleanup {
                    terminal: cleanup::read_terminal(reader.take::<1>()?[0])?,
                    disposition: read_disposition(reader)?,
                    resources: release::decode(reader)?,
                },
                3 => QueryManagedRunPosture::YieldedCleanup {
                    disposition: read_disposition(reader)?,
                    resources: release::decode(reader)?,
                },
                6 => QueryManagedRunPosture::WorkflowCleanup {
                    route: match reader.take::<1>()?[0] {
                        0 => QueryManagedWorkflowCleanupRoute::Terminal(cleanup::read_terminal(
                            reader.take::<1>()?[0],
                        )?),
                        1 => QueryManagedWorkflowCleanupRoute::Yielded,
                        2 => QueryManagedWorkflowCleanupRoute::Readmission,
                        _ => return Err(Denial::MalformedFrame),
                    },
                    disposition: read_disposition(reader)?,
                    resources: release::decode(reader)?,
                    artifacts_released: read_bool(reader)?,
                },
                4 => QueryManagedRunPosture::AdmissionReleased,
                5 => QueryManagedRunPosture::ReadmissionCleanup {
                    disposition: read_disposition(reader)?,
                    resources: release::decode(reader)?,
                },
                _ => return Err(Denial::MalformedFrame),
            },
        },
        2 => QueryManagedRunRecord::Discharge { identity },
        3 => QueryManagedRunRecord::Original {
            identity,
            original: super::super::codec::business_original::decode(reader)?
                .ok_or(Denial::MalformedFrame)?,
        },
        4 => QueryManagedRunRecord::Recovery {
            identity,
            cause: recovery::decode(reader)?,
        },
        _ => return Err(Denial::MalformedFrame),
    })
}

fn read_bool(reader: &mut Reader<'_>) -> Result<bool, Denial> {
    match reader.take::<1>()?[0] {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(Denial::MalformedFrame),
    }
}
