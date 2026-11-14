//! Canonical versioned checksummed support frames; one concrete encoding site.
use super::*;
use sha2::{Digest, Sha256};
use std::num::NonZeroU64;
use worth_foundational::facade::{
    BoundaryProtocolVersion, DurableWorldOwnerIdentity, WorldOwnerSlotIdentity,
};
pub(super) mod business_original;
const VERSION: BoundaryProtocolVersion = BoundaryProtocolVersion::new(2);
const MAGIC: &[u8; 8] = b"WQOBL001";

pub(super) fn encode(
    home: HomeIdentity,
    position: u64,
    predecessor: [u8; 32],
    generation: NonZeroU64,
    record: QueryObligationRecord,
) -> QueryObligationFrame {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&VERSION.get().to_le_bytes());
    bytes.extend_from_slice(&home.bytes());
    bytes.extend_from_slice(&position.to_le_bytes());
    bytes.extend_from_slice(&predecessor);
    bytes.extend_from_slice(&generation.get().to_le_bytes());
    match &record {
        QueryObligationRecord::OwnedAsync(record) => {
            bytes.push(18);
            owned_async::codec::encode(&mut bytes, record);
        }
        QueryObligationRecord::ManagedRun(record) => {
            bytes.push(17);
            managed_run::codec::encode(&mut bytes, record);
        }
        QueryObligationRecord::ConditionalDecision(record) => {
            bytes.push(16);
            conditional_decision::codec::encode(&mut bytes, record);
        }
        QueryObligationRecord::ProgramSupport(record) => {
            bytes.push(0);
            encode_support(&mut bytes, record);
        }
        QueryObligationRecord::ManagedCleanup(record) => {
            bytes.push(1);
            managed_cleanup::codec::encode(&mut bytes, record);
        }
    }
    let digest: [u8; 32] = Sha256::digest(&bytes).into();
    bytes.extend_from_slice(&digest);
    QueryObligationFrame {
        home,
        position,
        predecessor,
        generation,
        record,
        bytes,
        digest,
    }
}
fn encode_support(bytes: &mut Vec<u8>, record: &QueryProgramSupportRecord) {
    match record {
        QueryProgramSupportRecord::Install { revision } => {
            bytes.push(0);
            bytes.extend_from_slice(revision.bytes());
        }
        QueryProgramSupportRecord::Status {
            revision,
            before,
            after,
        } => {
            bytes.push(1);
            bytes.extend_from_slice(revision.bytes());
            bytes.push(status(*before));
            bytes.push(status(*after));
        }
        QueryProgramSupportRecord::Acquire(obligation) => {
            bytes.push(2);
            bytes.extend_from_slice(&obligation.identity.acquire_position().get().to_le_bytes());
            bytes.push(match obligation.kind {
                ProgramSupportObligationKind::OriginalInterpretation => 0,
                ProgramSupportObligationKind::BranchAdoption => 1,
            });
            bytes.extend_from_slice(obligation.source.bytes());
            bytes.extend_from_slice(obligation.target.bytes());
            bytes.extend_from_slice(&obligation.original.owner().home().bytes());
            bytes.extend_from_slice(&obligation.original.owner().slot().get().to_le_bytes());
            bytes.extend_from_slice(&obligation.original.ordinal().to_le_bytes());
        }
        QueryProgramSupportRecord::Release { identity } => {
            bytes.push(3);
            bytes.extend_from_slice(&identity.acquire_position().get().to_le_bytes());
        }
    }
}
fn status(value: ProgramSupportStatus) -> u8 {
    match value {
        ProgramSupportStatus::Active => 0,
        ProgramSupportStatus::Retiring => 1,
        ProgramSupportStatus::Retired => 2,
    }
}
fn read_status(value: u8) -> Result<ProgramSupportStatus, QueryObligationDenial> {
    match value {
        0 => Ok(ProgramSupportStatus::Active),
        1 => Ok(ProgramSupportStatus::Retiring),
        2 => Ok(ProgramSupportStatus::Retired),
        _ => Err(QueryObligationDenial::MalformedFrame),
    }
}
pub(super) struct Reader<'a>(pub(super) &'a [u8]);
impl Reader<'_> {
    pub(super) fn take<const N: usize>(&mut self) -> Result<[u8; N], QueryObligationDenial> {
        let (head, rest) = self
            .0
            .split_at_checked(N)
            .ok_or(QueryObligationDenial::MalformedFrame)?;
        self.0 = rest;
        Ok(head.try_into().expect("exact checked width"))
    }
}
pub(super) fn decode(bytes: &[u8]) -> Result<QueryObligationFrame, QueryObligationDenial> {
    use QueryObligationDenial as Denial;
    if bytes.len() as u64 > QUERY_OBLIGATION_FRAME_MAX_BYTES {
        return Err(Denial::MalformedFrame);
    }
    let (payload, checksum) = bytes
        .split_at_checked(bytes.len().checked_sub(32).ok_or(Denial::MalformedFrame)?)
        .ok_or(Denial::MalformedFrame)?;
    let digest: [u8; 32] = Sha256::digest(payload).into();
    if checksum != digest {
        return Err(Denial::Checksum);
    }
    let mut reader = Reader(payload);
    if &reader.take::<8>()? != MAGIC {
        return Err(Denial::MalformedFrame);
    }
    let version = u32::from_le_bytes(reader.take()?);
    if version != VERSION.get() {
        return Err(Denial::UnsupportedVersion(version));
    }
    let home =
        HomeIdentity::from_observed_bytes(reader.take()?).map_err(|_| Denial::MalformedFrame)?;
    let position = u64::from_le_bytes(reader.take()?);
    if position == 0 {
        return Err(Denial::MalformedFrame);
    }
    let predecessor = reader.take()?;
    let generation =
        NonZeroU64::new(u64::from_le_bytes(reader.take()?)).ok_or(Denial::MalformedFrame)?;
    let record = match reader.take::<1>()?[0] {
        0 => QueryObligationRecord::ProgramSupport(read_support(&mut reader, home)?),
        1 => QueryObligationRecord::ManagedCleanup(managed_cleanup::codec::decode(
            &mut reader,
            home,
        )?),
        18 => QueryObligationRecord::OwnedAsync(owned_async::codec::decode(&mut reader, home)?),
        17 => QueryObligationRecord::ManagedRun(managed_run::codec::decode(&mut reader, home)?),
        16 => QueryObligationRecord::ConditionalDecision(conditional_decision::codec::decode(
            &mut reader,
            home,
        )?),
        _ => return Err(Denial::MalformedFrame),
    };
    if !reader.0.is_empty() {
        return Err(Denial::MalformedFrame);
    }
    Ok(QueryObligationFrame {
        home,
        position,
        predecessor,
        generation,
        record,
        bytes: bytes.to_vec(),
        digest,
    })
}

fn read_support(
    reader: &mut Reader<'_>,
    home: HomeIdentity,
) -> Result<QueryProgramSupportRecord, QueryObligationDenial> {
    use QueryObligationDenial as Denial;
    Ok(match reader.take::<1>()?[0] {
        0 => QueryProgramSupportRecord::Install {
            revision: CanonicalDigestId::new(reader.take()?),
        },
        1 => QueryProgramSupportRecord::Status {
            revision: CanonicalDigestId::new(reader.take()?),
            before: read_status(reader.take::<1>()?[0])?,
            after: read_status(reader.take::<1>()?[0])?,
        },
        2 => {
            let acquire = NonZeroU64::new(u64::from_le_bytes(reader.take()?))
                .ok_or(Denial::MalformedFrame)?;
            let kind = match reader.take::<1>()?[0] {
                0 => ProgramSupportObligationKind::OriginalInterpretation,
                1 => ProgramSupportObligationKind::BranchAdoption,
                _ => return Err(Denial::MalformedFrame),
            };
            QueryProgramSupportRecord::Acquire(QueryProgramSupportObligation {
                identity: ProgramSupportObligationIdentity::from_recorded(home, acquire),
                kind,
                source: CanonicalDigestId::new(reader.take()?),
                target: CanonicalDigestId::new(reader.take()?),
                original: {
                    let home = HomeIdentity::from_observed_bytes(reader.take()?)
                        .map_err(|_| Denial::MalformedFrame)?;
                    let slot =
                        WorldOwnerSlotIdentity::from_recorded(u64::from_le_bytes(reader.take()?));
                    DurableCompositePublicationAttemptIdentity::from_recorded(
                        DurableWorldOwnerIdentity::from_recorded(home, slot),
                        u64::from_le_bytes(reader.take()?),
                    )
                },
            })
        }
        3 => QueryProgramSupportRecord::Release {
            identity: ProgramSupportObligationIdentity::from_recorded(
                home,
                NonZeroU64::new(u64::from_le_bytes(reader.take()?))
                    .ok_or(Denial::MalformedFrame)?,
            ),
        },
        _ => return Err(Denial::MalformedFrame),
    })
}
