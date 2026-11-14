//! Typed resource family inside the existing checksummed v2 envelope.
use super::super::{codec::Reader, QueryObligationDenial as D};
use super::*;
use worth_foundational::facade::HomeIdentity;

pub(in crate::domain_computation::primary_graph::query_obligations) fn encode(
    out: &mut Vec<u8>,
    record: &QueryResourceRecord,
) {
    let (tag, identity) = match record {
        QueryResourceRecord::Retain(row) => (0, row.identity),
        QueryResourceRecord::Handoff { identity, .. } => (1, *identity),
        QueryResourceRecord::Discharge { identity } => (2, *identity),
    };
    out.push(tag);
    out.extend_from_slice(&identity.acquire_position().get().to_le_bytes());
    match record {
        QueryResourceRecord::Retain(row) => blob(out, &row.original),
        QueryResourceRecord::Handoff { handoff, .. } => blob(out, handoff),
        QueryResourceRecord::Discharge { .. } => {}
    }
}
fn blob(out: &mut Vec<u8>, value: &impl serde::Serialize) {
    let bytes = serde_json::to_vec(value).expect("resource DATA contains supported values");
    out.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    out.extend_from_slice(&bytes);
}
fn read<T: serde::de::DeserializeOwned>(reader: &mut Reader<'_>) -> Result<T, D> {
    let length =
        usize::try_from(u64::from_le_bytes(reader.take()?)).map_err(|_| D::MalformedFrame)?;
    let (value, rest) = reader.0.split_at_checked(length).ok_or(D::MalformedFrame)?;
    let value = serde_json::from_slice(value).map_err(|_| D::MalformedFrame)?;
    reader.0 = rest;
    Ok(value)
}
pub(in crate::domain_computation::primary_graph::query_obligations) fn decode(
    reader: &mut Reader<'_>,
    home: HomeIdentity,
) -> Result<QueryResourceRecord, D> {
    let tag = reader.take::<1>()?[0];
    let position =
        std::num::NonZeroU64::new(u64::from_le_bytes(reader.take()?)).ok_or(D::MalformedFrame)?;
    let identity = OwnedAsyncObligationIdentity::from_recorded(home, position);
    Ok(match tag {
        0 => QueryResourceRecord::Retain(QueryResourceObligation {
            identity,
            original: read(reader)?,
            handoff: QueryResourceHandoff::default(),
        }),
        1 => QueryResourceRecord::Handoff {
            identity,
            handoff: read(reader)?,
        },
        2 => QueryResourceRecord::Discharge { identity },
        _ => return Err(D::MalformedFrame),
    })
}
