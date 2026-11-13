//! Source and demand payloads inside the one Query obligation envelope.
use super::*;
use crate::domain_computation::primary_graph::query_obligations::{
    codec::Reader, QueryObligationDenial as Denial,
};
use std::num::NonZeroU64;
use worth_foundational::facade::{DurableWorldOwnerIdentity, HomeIdentity, WorldOwnerSlotIdentity};

pub(in crate::domain_computation::primary_graph::query_obligations) fn encode_source(
    bytes: &mut Vec<u8>,
    record: &QueryOutputSourceRecord,
) {
    match record {
        QueryOutputSourceRecord::Bind {
            commit: original,
            sources,
        } => {
            bytes.push(2);
            commit(bytes, *original);
            encode_bindings(bytes, Some(sources));
        }
        QueryOutputSourceRecord::Retain(row) => {
            bytes.push(0);
            commit(bytes, row.commit);
            bytes.extend_from_slice(row.revision.bytes());
            blob(bytes, row.root.as_bytes());
            blob(bytes, row.branch.as_bytes());
            bytes.extend_from_slice(&row.lifecycle.to_le_bytes());
            bytes.extend_from_slice(&row.generation.to_le_bytes());
            bytes.extend_from_slice(&row.scope.0.to_le_bytes());
            bytes.extend_from_slice(&row.scope.1.to_le_bytes());
            bytes.extend_from_slice(&row.scope.2.to_le_bytes());
            bytes.extend_from_slice(&row.key);
            blob(bytes, row.intent.as_bytes());
            bytes.push(u8::from(row.discovery_recipe.is_some()));
            if let Some(recipe) = &row.discovery_recipe {
                blob(bytes, recipe);
            }
            encode_bindings(bytes, row.bindings.as_deref());
        }
        QueryOutputSourceRecord::Discharge { commit: identity } => {
            bytes.push(1);
            commit(bytes, *identity);
        }
    }
}
pub(in crate::domain_computation::primary_graph::query_obligations) fn decode_source(
    reader: &mut Reader<'_>,
) -> Result<QueryOutputSourceRecord, Denial> {
    Ok(match reader.take::<1>()?[0] {
        0 => QueryOutputSourceRecord::Retain(QueryOutputSourceObligation {
            commit: read_commit(reader)?,
            revision: CanonicalDigestId::new(reader.take()?),
            root: text(reader)?,
            branch: text(reader)?,
            lifecycle: u64::from_le_bytes(reader.take()?),
            generation: u64::from_le_bytes(reader.take()?),
            scope: (
                u32::from_le_bytes(reader.take()?),
                u64::from_le_bytes(reader.take()?),
                u32::from_le_bytes(reader.take()?),
            ),
            key: reader.take()?,
            intent: text(reader)?,
            discovery_recipe: match reader.take::<1>()?[0] {
                0 => None,
                1 => Some(read_blob(reader)?.to_vec()),
                _ => return Err(Denial::MalformedFrame),
            },
            bindings: decode_bindings(reader)?,
        }),
        2 => QueryOutputSourceRecord::Bind {
            commit: read_commit(reader)?,
            sources: decode_bindings(reader)?.ok_or(Denial::MalformedFrame)?,
        },
        1 => QueryOutputSourceRecord::Discharge {
            commit: read_commit(reader)?,
        },
        _ => return Err(Denial::MalformedFrame),
    })
}
pub(in crate::domain_computation::primary_graph::query_obligations) fn encode_demand(
    bytes: &mut Vec<u8>,
    record: &QueryOutputDemandRecord,
) {
    match record {
        QueryOutputDemandRecord::DomainDenied {
            acquire_position,
            failure,
        } => {
            bytes.push(3);
            bytes.extend_from_slice(&acquire_position.get().to_le_bytes());
            blob(bytes, failure);
        }
        QueryOutputDemandRecord::Retain(row) => {
            bytes.push(0);
            bytes.extend_from_slice(&row.acquire_position.get().to_le_bytes());
            bytes.extend_from_slice(row.revision.bytes());
            for value in [&row.family, &row.producer, &row.branch] {
                blob(bytes, value.as_bytes());
            }
            bytes.extend_from_slice(&row.query);
            commit(bytes, row.basis);
            bytes.extend_from_slice(&row.root);
            blob(bytes, &row.parameters);
            blob(bytes, &row.scope);
            bytes.extend_from_slice(&(row.performed_sources.len() as u64).to_le_bytes());
            for source in &row.performed_sources {
                commit(bytes, *source);
            }
            bytes.extend_from_slice(&(row.prerequisites.len() as u64).to_le_bytes());
            for prerequisite in &row.prerequisites {
                bytes.extend_from_slice(&prerequisite.get().to_le_bytes());
            }
            encode_checkpoints(bytes, &row.checkpoint_prerequisites);
        }
        QueryOutputDemandRecord::Links {
            acquire_position,
            performed_sources,
            prerequisites,
            checkpoint_prerequisites,
        } => {
            bytes.push(2);
            bytes.extend_from_slice(&acquire_position.get().to_le_bytes());
            bytes.extend_from_slice(&(performed_sources.len() as u64).to_le_bytes());
            for source in performed_sources {
                commit(bytes, *source);
            }
            bytes.extend_from_slice(&(prerequisites.len() as u64).to_le_bytes());
            for prerequisite in prerequisites {
                bytes.extend_from_slice(&prerequisite.get().to_le_bytes());
            }
            encode_checkpoints(bytes, checkpoint_prerequisites);
        }
        QueryOutputDemandRecord::Discharge { acquire_position } => {
            bytes.push(1);
            bytes.extend_from_slice(&acquire_position.get().to_le_bytes());
        }
    }
}
pub(in crate::domain_computation::primary_graph::query_obligations) fn decode_demand(
    reader: &mut Reader<'_>,
) -> Result<QueryOutputDemandRecord, Denial> {
    let tag = reader.take::<1>()?[0];
    let acquire_position = position(reader)?;
    Ok(match tag {
        3 => QueryOutputDemandRecord::DomainDenied {
            acquire_position,
            failure: read_blob(reader)?.to_vec(),
        },
        0 => {
            let revision = CanonicalDigestId::new(reader.take()?);
            let family = text(reader)?;
            let producer = text(reader)?;
            let branch = text(reader)?;
            let query = reader.take()?;
            let basis = read_commit(reader)?;
            let root = reader.take()?;
            let parameters = read_blob(reader)?.to_vec();
            let scope = read_blob(reader)?.to_vec();
            let source_count = count(reader, 32)?;
            let performed_sources = (0..source_count)
                .map(|_| read_commit(reader))
                .collect::<Result<_, _>>()?;
            let prerequisite_count = count(reader, 8)?;
            let prerequisites = (0..prerequisite_count)
                .map(|_| position(reader))
                .collect::<Result<_, _>>()?;
            QueryOutputDemandRecord::Retain(QueryOutputDemandObligation {
                acquire_position,
                revision,
                family,
                producer,
                query,
                branch,
                basis,
                root,
                parameters,
                scope,
                performed_sources,
                prerequisites,
                checkpoint_prerequisites: decode_checkpoints(reader)?,
            })
        }
        1 => QueryOutputDemandRecord::Discharge { acquire_position },
        2 => {
            let source_count = count(reader, 32)?;
            let performed_sources = (0..source_count)
                .map(|_| read_commit(reader))
                .collect::<Result<_, _>>()?;
            let prerequisite_count = count(reader, 8)?;
            let prerequisites = (0..prerequisite_count)
                .map(|_| position(reader))
                .collect::<Result<_, _>>()?;
            QueryOutputDemandRecord::Links {
                acquire_position,
                performed_sources,
                prerequisites,
                checkpoint_prerequisites: decode_checkpoints(reader)?,
            }
        }
        _ => return Err(Denial::MalformedFrame),
    })
}
fn commit(bytes: &mut Vec<u8>, value: DurableCompositeCommitIdentity) {
    bytes.extend_from_slice(&value.owner().home().bytes());
    bytes.extend_from_slice(&value.owner().slot().get().to_le_bytes());
    bytes.extend_from_slice(&value.ordinal().to_le_bytes());
}
fn read_commit(reader: &mut Reader<'_>) -> Result<DurableCompositeCommitIdentity, Denial> {
    let home =
        HomeIdentity::from_observed_bytes(reader.take()?).map_err(|_| Denial::MalformedFrame)?;
    let slot = WorldOwnerSlotIdentity::from_recorded(u64::from_le_bytes(reader.take()?));
    Ok(DurableCompositeCommitIdentity::from_recorded(
        DurableWorldOwnerIdentity::from_recorded(home, slot),
        u64::from_le_bytes(reader.take()?),
    ))
}
fn blob(bytes: &mut Vec<u8>, value: &[u8]) {
    bytes.extend_from_slice(&(value.len() as u64).to_le_bytes());
    bytes.extend_from_slice(value);
}
fn read_blob<'a>(reader: &mut Reader<'a>) -> Result<&'a [u8], Denial> {
    let size = count(reader, 1)?;
    let (value, rest) = reader
        .0
        .split_at_checked(size)
        .ok_or(Denial::MalformedFrame)?;
    reader.0 = rest;
    Ok(value)
}
fn text(reader: &mut Reader<'_>) -> Result<String, Denial> {
    String::from_utf8(read_blob(reader)?.to_vec()).map_err(|_| Denial::MalformedFrame)
}
fn count(reader: &mut Reader<'_>, width: usize) -> Result<usize, Denial> {
    let count =
        usize::try_from(u64::from_le_bytes(reader.take()?)).map_err(|_| Denial::MalformedFrame)?;
    if count > reader.0.len() / width {
        return Err(Denial::MalformedFrame);
    }
    Ok(count)
}
fn position(reader: &mut Reader<'_>) -> Result<NonZeroU64, Denial> {
    NonZeroU64::new(u64::from_le_bytes(reader.take()?)).ok_or(Denial::MalformedFrame)
}

pub(in crate::domain_computation::primary_graph::query_obligations) fn encode_accepted(
    bytes: &mut Vec<u8>,
    row: &QueryAcceptedOutputRecord,
) {
    commit(bytes, row.commit);
    blob(bytes, row.branch.as_bytes());
    blob(bytes, row.producer.as_bytes());
    bytes.extend_from_slice(&row.scope.0.to_le_bytes());
    bytes.extend_from_slice(&row.scope.1.to_le_bytes());
    bytes.extend_from_slice(&row.scope.2.to_le_bytes());
    bytes.extend_from_slice(&row.partition);
    blob(bytes, &row.payload);
}
pub(in crate::domain_computation::primary_graph::query_obligations) fn decode_accepted(
    reader: &mut Reader<'_>,
) -> Result<QueryAcceptedOutputRecord, Denial> {
    Ok(QueryAcceptedOutputRecord {
        commit: read_commit(reader)?,
        branch: text(reader)?,
        producer: text(reader)?,
        scope: (
            u32::from_le_bytes(reader.take()?),
            u64::from_le_bytes(reader.take()?),
            u32::from_le_bytes(reader.take()?),
        ),
        partition: reader.take()?,
        payload: read_blob(reader)?.to_vec(),
    })
}

fn encode_bindings(bytes: &mut Vec<u8>, bindings: Option<&[([u8; 32], (u32, u64, u32))]>) {
    bytes.push(u8::from(bindings.is_some()));
    let Some(bindings) = bindings else {
        return;
    };
    bytes.extend_from_slice(&(bindings.len() as u64).to_le_bytes());
    for (identity, scope) in bindings {
        bytes.extend_from_slice(identity);
        bytes.extend_from_slice(&scope.0.to_le_bytes());
        bytes.extend_from_slice(&scope.1.to_le_bytes());
        bytes.extend_from_slice(&scope.2.to_le_bytes());
    }
}
fn decode_bindings(
    reader: &mut Reader<'_>,
) -> Result<Option<Vec<([u8; 32], (u32, u64, u32))>>, Denial> {
    match reader.take::<1>()?[0] {
        0 => Ok(None),
        1 => {
            let count = count(reader, 48)?;
            (0..count)
                .map(|_| {
                    Ok((
                        reader.take()?,
                        (
                            u32::from_le_bytes(reader.take()?),
                            u64::from_le_bytes(reader.take()?),
                            u32::from_le_bytes(reader.take()?),
                        ),
                    ))
                })
                .collect::<Result<Vec<_>, _>>()
                .map(Some)
        }
        _ => Err(Denial::MalformedFrame),
    }
}

fn encode_checkpoints(bytes: &mut Vec<u8>, records: &[Vec<u8>]) {
    bytes.extend_from_slice(&(records.len() as u64).to_le_bytes());
    for record in records {
        blob(bytes, record);
    }
}
fn decode_checkpoints(reader: &mut Reader<'_>) -> Result<Vec<Vec<u8>>, Denial> {
    let count = count(reader, 8)?;
    (0..count)
        .map(|_| Ok(read_blob(reader)?.to_vec()))
        .collect()
}
