//! Owner DATA only: native ports, queue bindings and clocks are never encoded.
use super::super::{codec::Reader, QueryObligationDenial as Denial};
use super::*;
use std::num::NonZeroU64;
use worth_foundational::facade::HomeIdentity;
pub(in crate::domain_computation::primary_graph::query_obligations) fn encode(
    out: &mut Vec<u8>,
    record: &QueryOwnedAsyncRecord,
) {
    let (tag, identity) = match record {
        QueryOwnedAsyncRecord::Retain(row) => (0, row.identity),
        QueryOwnedAsyncRecord::Progress { identity, .. } => (1, *identity),
        QueryOwnedAsyncRecord::ProcessLocalReleased { identity, .. } => (2, *identity),
        QueryOwnedAsyncRecord::Discharge { identity } => (3, *identity),
        QueryOwnedAsyncRecord::QueueDrained { identity } => (4, *identity),
    };
    out.push(tag);
    number(out, identity.acquire_position().get());
    match record {
        QueryOwnedAsyncRecord::Retain(row) => {
            definition(out, &row.declaration);
            blob(out, &row.payload);
            request(out, row.original);
            number(out, row.queue_depth);
        }
        QueryOwnedAsyncRecord::Progress {
            current,
            lineage,
            posture,
            queue_depth,
            completion,
            ..
        } => {
            request(out, *current);
            number(out, lineage.len() as u64);
            for prior in lineage {
                request(out, *prior);
            }
            write_posture(out, *posture);
            number(out, *queue_depth);
            out.push(u8::from(completion.is_some()));
            if let Some(value) = completion {
                blob(out, value);
            }
        }
        QueryOwnedAsyncRecord::ProcessLocalReleased { terminal, .. } => {
            out.push(terminal_tag(*terminal))
        }
        QueryOwnedAsyncRecord::Discharge { .. } | QueryOwnedAsyncRecord::QueueDrained { .. } => {}
    }
}
fn number(out: &mut Vec<u8>, number: u64) {
    out.extend_from_slice(&number.to_le_bytes());
}
fn blob(out: &mut Vec<u8>, bytes: &[u8]) {
    number(out, bytes.len() as u64);
    out.extend_from_slice(bytes);
}
fn request(out: &mut Vec<u8>, request: QueryOwnedAsyncRequestGeneration) {
    number(out, request.request);
    number(out, request.generation);
}
fn terminal_tag(terminal: QueryOwnedAsyncTerminal) -> u8 {
    match terminal {
        QueryOwnedAsyncTerminal::Fulfilled => 0,
        QueryOwnedAsyncTerminal::Rejected => 1,
        QueryOwnedAsyncTerminal::Cancelled => 2,
        QueryOwnedAsyncTerminal::TimedOut => 3,
        QueryOwnedAsyncTerminal::Superseded => 4,
    }
}
fn write_posture(out: &mut Vec<u8>, posture: QueryOwnedAsyncPosture) {
    match posture {
        QueryOwnedAsyncPosture::InFlight => out.push(0),
        QueryOwnedAsyncPosture::Retryable(reason) => {
            out.push(1);
            out.push(match reason {
                QueryOwnedAsyncRetryCause::TimedOut => 0,
                QueryOwnedAsyncRetryCause::HostRequested => 1,
            });
        }
        QueryOwnedAsyncPosture::CompletionPending => out.push(2),
        QueryOwnedAsyncPosture::Terminal(terminal) => {
            out.push(3);
            out.push(terminal_tag(terminal));
        }
        QueryOwnedAsyncPosture::ProcessLocalReleased(terminal) => {
            out.push(4);
            out.push(terminal_tag(terminal));
        }
    }
}
pub(in crate::domain_computation::primary_graph::query_obligations) fn decode(
    reader: &mut Reader<'_>,
    home: HomeIdentity,
) -> Result<QueryOwnedAsyncRecord, Denial> {
    let tag = reader.take::<1>()?[0];
    let identity = OwnedAsyncObligationIdentity::from_recorded(
        home,
        NonZeroU64::new(read_number(reader)?).ok_or(Denial::MalformedFrame)?,
    );
    Ok(match tag {
        0 => {
            let declaration = read_definition(reader)?;
            let payload = read_blob(reader)?;
            let original = read_request(reader)?;
            let queue_depth = read_number(reader)?;
            QueryOwnedAsyncRecord::Retain(QueryOwnedAsyncObligation {
                identity,
                declaration,
                payload,
                original,
                current: original,
                lineage: Vec::new(),
                posture: QueryOwnedAsyncPosture::InFlight,
                queue_depth,
                completion: None,
            })
        }
        1 => {
            let current = read_request(reader)?;
            let count =
                usize::try_from(read_number(reader)?).map_err(|_| Denial::MalformedFrame)?;
            if count > reader.0.len() / 16 {
                return Err(Denial::MalformedFrame);
            }
            let mut lineage = Vec::with_capacity(count);
            for _ in 0..count {
                lineage.push(read_request(reader)?);
            }
            let posture = read_posture(reader)?;
            let queue_depth = read_number(reader)?;
            let completion = match reader.take::<1>()?[0] {
                0 => None,
                1 => Some(read_blob(reader)?),
                _ => return Err(Denial::MalformedFrame),
            };
            QueryOwnedAsyncRecord::Progress {
                identity,
                current,
                lineage,
                posture,
                queue_depth,
                completion,
            }
        }
        2 => QueryOwnedAsyncRecord::ProcessLocalReleased {
            identity,
            terminal: read_terminal(reader)?,
        },
        3 => QueryOwnedAsyncRecord::Discharge { identity },
        4 => QueryOwnedAsyncRecord::QueueDrained { identity },
        _ => return Err(Denial::MalformedFrame),
    })
}
fn read_number(reader: &mut Reader<'_>) -> Result<u64, Denial> {
    Ok(u64::from_le_bytes(reader.take()?))
}
fn read_blob(reader: &mut Reader<'_>) -> Result<Vec<u8>, Denial> {
    let length = usize::try_from(read_number(reader)?).map_err(|_| Denial::MalformedFrame)?;
    let (value, tail) = reader
        .0
        .split_at_checked(length)
        .ok_or(Denial::MalformedFrame)?;
    reader.0 = tail;
    Ok(value.to_vec())
}
fn read_request(reader: &mut Reader<'_>) -> Result<QueryOwnedAsyncRequestGeneration, Denial> {
    Ok(QueryOwnedAsyncRequestGeneration {
        request: read_number(reader)?,
        generation: read_number(reader)?,
    })
}
fn read_terminal(reader: &mut Reader<'_>) -> Result<QueryOwnedAsyncTerminal, Denial> {
    Ok(match reader.take::<1>()?[0] {
        0 => QueryOwnedAsyncTerminal::Fulfilled,
        1 => QueryOwnedAsyncTerminal::Rejected,
        2 => QueryOwnedAsyncTerminal::Cancelled,
        3 => QueryOwnedAsyncTerminal::TimedOut,
        4 => QueryOwnedAsyncTerminal::Superseded,
        _ => return Err(Denial::MalformedFrame),
    })
}
fn read_posture(reader: &mut Reader<'_>) -> Result<QueryOwnedAsyncPosture, Denial> {
    Ok(match reader.take::<1>()?[0] {
        0 => QueryOwnedAsyncPosture::InFlight,
        1 => QueryOwnedAsyncPosture::Retryable(match reader.take::<1>()?[0] {
            0 => QueryOwnedAsyncRetryCause::TimedOut,
            1 => QueryOwnedAsyncRetryCause::HostRequested,
            _ => return Err(Denial::MalformedFrame),
        }),
        2 => QueryOwnedAsyncPosture::CompletionPending,
        3 => QueryOwnedAsyncPosture::Terminal(read_terminal(reader)?),
        4 => QueryOwnedAsyncPosture::ProcessLocalReleased(read_terminal(reader)?),
        _ => return Err(Denial::MalformedFrame),
    })
}

fn definition(out: &mut Vec<u8>, declaration: &QueryOwnedAsyncDefinition) {
    blob(out, declaration.identity.as_bytes());
    number(out, declaration.payload_contract);
    number(out, declaration.max_payload_bytes);
    out.extend_from_slice(&declaration.retry_max_attempts.to_le_bytes());
    number(out, declaration.retry_delay_ticks);
    number(out, declaration.timeout_ticks);
    number(
        out,
        declaration
            .managed_queue_capacity
            .map_or(0, NonZeroU64::get),
    );
    out.push(match declaration.retention {
        QueryOwnedAsyncRetention::Mandatory => 0,
        QueryOwnedAsyncRetention::ProcessLocal => 1,
    });
}
fn read_definition(reader: &mut Reader<'_>) -> Result<QueryOwnedAsyncDefinition, Denial> {
    Ok(QueryOwnedAsyncDefinition {
        identity: String::from_utf8(read_blob(reader)?).map_err(|_| Denial::MalformedFrame)?,
        payload_contract: read_number(reader)?,
        max_payload_bytes: read_number(reader)?,
        retry_max_attempts: u32::from_le_bytes(reader.take()?),
        retry_delay_ticks: read_number(reader)?,
        timeout_ticks: read_number(reader)?,
        managed_queue_capacity: NonZeroU64::new(read_number(reader)?),
        retention: match reader.take::<1>()?[0] {
            0 => QueryOwnedAsyncRetention::Mandatory,
            1 => QueryOwnedAsyncRetention::ProcessLocal,
            _ => return Err(Denial::MalformedFrame),
        },
    })
}
