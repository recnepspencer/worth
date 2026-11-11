//! Canonical versioned checksummed dispatch-custody frames; one concrete encoding site.
use super::*;
use sha2::{Digest, Sha256};
use std::num::NonZeroU64;
use worth_foundational::facade::BoundaryProtocolVersion;
const VERSION: BoundaryProtocolVersion = BoundaryProtocolVersion::new(1);
const MAGIC: &[u8; 8] = b"WQDSP001";
pub(super) const FRAME_ENVELOPE_BYTES: u64 = 8 + 4 + 16 + 8 + 32 + 8 + 32;
mod inbound;
mod source;
pub(super) use inbound::accepted_retained_bytes;

pub(super) fn encode(
    home: HomeIdentity,
    position: u64,
    predecessor: [u8; 32],
    generation: NonZeroU64,
    record: QueryDispatchRecord,
) -> QueryDispatchFrame {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&VERSION.get().to_le_bytes());
    bytes.extend_from_slice(&home.bytes());
    bytes.extend_from_slice(&position.to_le_bytes());
    bytes.extend_from_slice(&predecessor);
    bytes.extend_from_slice(&generation.get().to_le_bytes());
    match &record {
        QueryDispatchRecord::Inbound(value) => inbound::encode(&mut bytes, value),
        QueryDispatchRecord::SourceControl { source, posture } => {
            bytes.push(2);
            bytes.extend_from_slice(source.bytes());
            bytes.push(match posture {
                InboundSourceControl::Retired => 1,
                InboundSourceControl::Revoked => 2,
            });
        }
        QueryDispatchRecord::Attempt(attempt) => {
            bytes.push(0);
            bytes.extend_from_slice(&attempt.identity.attempt_position().get().to_le_bytes());
            bytes.extend_from_slice(attempt.correlation.bytes());
        }
        QueryDispatchRecord::Observation {
            attempt,
            observation,
        } => {
            bytes.push(1);
            bytes.extend_from_slice(&attempt.attempt_position().get().to_le_bytes());
            bytes.extend_from_slice(&observation_tag(*observation));
        }
    }
    let digest: [u8; 32] = Sha256::digest(&bytes).into();
    bytes.extend_from_slice(&digest);
    QueryDispatchFrame {
        home,
        position,
        predecessor,
        generation,
        record,
        bytes,
        digest,
    }
}

fn observation_tag(observation: QueryDispatchRecordedObservation) -> [u8; 2] {
    use QueryDispatchRecordedObservation as Observation;
    use QueryDispatchUnresolvedCause as Cause;
    match observation {
        Observation::Completed => [0, 0],
        Observation::Acknowledged => [1, 0],
        Observation::ProvenNoEffect(ExternalEffectNoEffectCause::PayloadRejected) => [2, 0],
        Observation::ProvenNoEffect(ExternalEffectNoEffectCause::ContactNotStarted) => [2, 1],
        Observation::Unresolved(cause) => [
            3,
            match cause {
                Cause::Timeout => 0,
                Cause::Disconnect => 1,
                Cause::LostResponse => 2,
                Cause::DuplicatedAcknowledgement => 3,
                Cause::UnsupportedProtocolVersion => 4,
                Cause::UnknownProviderOutcome => 5,
                Cause::ObservationDerivationDenied => 6,
            },
        ],
    }
}

fn read_observation(
    tag: [u8; 2],
) -> Result<QueryDispatchRecordedObservation, QueryDispatchJournalDenial> {
    use QueryDispatchRecordedObservation as Observation;
    use QueryDispatchUnresolvedCause as Cause;
    Ok(match tag {
        [0, 0] => Observation::Completed,
        [1, 0] => Observation::Acknowledged,
        [2, 0] => Observation::ProvenNoEffect(ExternalEffectNoEffectCause::PayloadRejected),
        [2, 1] => Observation::ProvenNoEffect(ExternalEffectNoEffectCause::ContactNotStarted),
        [3, 0] => Observation::Unresolved(Cause::Timeout),
        [3, 1] => Observation::Unresolved(Cause::Disconnect),
        [3, 2] => Observation::Unresolved(Cause::LostResponse),
        [3, 3] => Observation::Unresolved(Cause::DuplicatedAcknowledgement),
        [3, 4] => Observation::Unresolved(Cause::UnsupportedProtocolVersion),
        [3, 5] => Observation::Unresolved(Cause::UnknownProviderOutcome),
        [3, 6] => Observation::Unresolved(Cause::ObservationDerivationDenied),
        [_, _] => return Err(QueryDispatchJournalDenial::MalformedFrame),
    })
}

struct Reader<'a>(&'a [u8]);
impl Reader<'_> {
    fn take<const N: usize>(&mut self) -> Result<[u8; N], QueryDispatchJournalDenial> {
        let (head, rest) = self
            .0
            .split_at_checked(N)
            .ok_or(QueryDispatchJournalDenial::MalformedFrame)?;
        self.0 = rest;
        Ok(head.try_into().expect("exact checked width"))
    }
    fn attempt(
        &mut self,
        home: HomeIdentity,
    ) -> Result<DispatchAttemptIdentity, QueryDispatchJournalDenial> {
        NonZeroU64::new(u64::from_le_bytes(self.take()?))
            .map(|attempt| DispatchAttemptIdentity::from_recorded(home, attempt))
            .ok_or(QueryDispatchJournalDenial::MalformedFrame)
    }
}

pub(super) fn decode(
    bytes: &[u8],
    maximum_bytes: u64,
) -> Result<QueryDispatchFrame, QueryDispatchJournalDenial> {
    use QueryDispatchJournalDenial as Denial;
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
    // The size law belongs to this version, so an intact later version
    // reports itself instead of a malformed frame.
    if bytes.len() as u64 > maximum_bytes {
        return Err(Denial::MalformedFrame);
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
        0 => QueryDispatchRecord::Attempt(QueryDispatchAttempt {
            identity: reader.attempt(home)?,
            correlation: CanonicalDigestId::new(reader.take()?),
        }),
        1 => QueryDispatchRecord::Observation {
            attempt: reader.attempt(home)?,
            observation: read_observation(reader.take()?)?,
        },
        2 => QueryDispatchRecord::SourceControl {
            source: CanonicalDigestId::new(reader.take()?),
            posture: read_source_control(reader.take::<1>()?[0])?,
        },
        tag @ (3 | 4 | 5 | 6) => QueryDispatchRecord::Inbound(inbound::decode(tag, &mut reader)?),
        _ => return Err(Denial::MalformedFrame),
    };
    if !reader.0.is_empty() {
        return Err(Denial::MalformedFrame);
    }
    Ok(QueryDispatchFrame {
        home,
        position,
        predecessor,
        generation,
        record,
        bytes: bytes.to_vec(),
        digest,
    })
}

fn read_source_control(code: u8) -> Result<InboundSourceControl, QueryDispatchJournalDenial> {
    match code {
        1 => Ok(InboundSourceControl::Retired),
        2 => Ok(InboundSourceControl::Revoked),
        _ => Err(QueryDispatchJournalDenial::MalformedFrame),
    }
}
