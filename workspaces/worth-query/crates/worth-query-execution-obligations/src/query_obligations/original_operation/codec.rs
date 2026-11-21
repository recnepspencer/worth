//! Original-owner DATA uses family 20 of the current v2 envelope.
use super::super::{codec::Reader, QueryObligationDenial as D};
use super::*;
use worth_foundational::facade::HomeIdentity;
pub(in crate::query_obligations) fn encode(
    out: &mut Vec<u8>,
    record: &QueryOriginalOperationRecord,
) {
    let (tag, identity) = match record {
        QueryOriginalOperationRecord::Retain(row) => (0, row.identity),
        QueryOriginalOperationRecord::Advance { identity, .. } => (1, *identity),
        QueryOriginalOperationRecord::Discharge { identity } => (2, *identity),
    };
    out.push(tag);
    out.extend_from_slice(&identity.acquire_position().get().to_le_bytes());
    match record {
        QueryOriginalOperationRecord::Retain(row) => {
            let data = rmp_serde::to_vec(&row.data).expect("original DATA is serializable");
            out.extend_from_slice(&(data.len() as u64).to_le_bytes());
            out.extend_from_slice(&data);
        }
        QueryOriginalOperationRecord::Advance { position, .. } => out.push(match position {
            QueryOriginalOperationPosition::Initial => 0,
            QueryOriginalOperationPosition::Resolving => 1,
        }),
        QueryOriginalOperationRecord::Discharge { .. } => {}
    }
}
pub(in crate::query_obligations) fn decode(
    reader: &mut Reader<'_>,
    home: HomeIdentity,
) -> Result<QueryOriginalOperationRecord, D> {
    let tag = reader.take::<1>()?[0];
    let position =
        std::num::NonZeroU64::new(u64::from_le_bytes(reader.take()?)).ok_or(D::MalformedFrame)?;
    let identity = ProgramSupportObligationIdentity::from_recorded(home, position);
    Ok(match tag {
        0 => {
            let len = usize::try_from(u64::from_le_bytes(reader.take()?))
                .map_err(|_| D::MalformedFrame)?;
            let (data, rest) = reader.0.split_at_checked(len).ok_or(D::MalformedFrame)?;
            let data = rmp_serde::from_slice(data).map_err(|_| D::MalformedFrame)?;
            reader.0 = rest;
            QueryOriginalOperationRecord::Retain(QueryOriginalOperationObligation {
                identity,
                data,
                position: QueryOriginalOperationPosition::Initial,
            })
        }
        1 => QueryOriginalOperationRecord::Advance {
            identity,
            position: match reader.take::<1>()?[0] {
                0 => QueryOriginalOperationPosition::Initial,
                1 => QueryOriginalOperationPosition::Resolving,
                _ => return Err(D::MalformedFrame),
            },
        },
        2 => QueryOriginalOperationRecord::Discharge { identity },
        _ => return Err(D::MalformedFrame),
    })
}
