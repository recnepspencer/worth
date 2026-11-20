//! One correction journal envelope grammar, independent of its body interpreter.
use super::*;
use sha2::{Digest, Sha256};
const MAGIC: &[u8; 8] = b"WQCOR001";
pub(super) const HEADER: usize = 85;
pub(super) fn encode(
    home: HomeIdentity,
    position: NonZeroU64,
    predecessor: [u8; 32],
    generation: NonZeroU64,
    body_retention: RetentionClass,
    body: &[u8],
) -> Result<QueryCorrectionMaterialFrame, QueryCorrectionMaterialDenial> {
    use QueryCorrectionMaterialDenial as Denial;
    let length = HEADER
        .checked_add(body.len())
        .and_then(|length| length.checked_add(32))
        .ok_or(Denial::MalformedFrame)?;
    let body_length = u64::try_from(body.len()).map_err(|_| Denial::MalformedFrame)?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(length)
        .map_err(|_| Denial::Allocation)?;
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&QUERY_CORRECTION_JOURNAL_VERSION.get().to_le_bytes());
    bytes.extend_from_slice(&home.bytes());
    bytes.extend_from_slice(&position.get().to_le_bytes());
    bytes.extend_from_slice(&predecessor);
    bytes.extend_from_slice(&generation.get().to_le_bytes());
    bytes.push(match body_retention {
        RetentionClass::Authoritative => 0,
        RetentionClass::MandatoryObligation => 1,
        RetentionClass::OptionalRetainedMaterial => 2,
        RetentionClass::Derived => 3,
    });
    bytes.extend_from_slice(&body_length.to_le_bytes());
    bytes.extend_from_slice(body);
    let digest: [u8; 32] = Sha256::digest(&bytes).into();
    bytes.extend_from_slice(&digest);
    Ok(QueryCorrectionMaterialFrame {
        home,
        position,
        predecessor,
        generation,
        body_retention,
        bytes,
        digest,
    })
}
struct Reader<'a>(&'a [u8]);
impl Reader<'_> {
    fn take<const N: usize>(&mut self) -> Result<[u8; N], QueryCorrectionMaterialDenial> {
        let (value, rest) = self
            .0
            .split_at_checked(N)
            .ok_or(QueryCorrectionMaterialDenial::MalformedFrame)?;
        self.0 = rest;
        Ok(value.try_into().expect("exact checked width"))
    }
}
pub(super) fn decode(
    bytes: Vec<u8>,
) -> Result<QueryCorrectionMaterialFrame, QueryCorrectionMaterialDenial> {
    use QueryCorrectionMaterialDenial as Denial;
    if bytes.len() < HEADER + 32 {
        return Err(Denial::MalformedFrame);
    }
    let (payload, checksum) = bytes.split_at(bytes.len() - 32);
    let digest: [u8; 32] = Sha256::digest(payload).into();
    if checksum != digest {
        return Err(Denial::Checksum);
    }
    let mut reader = Reader(payload);
    if &reader.take::<8>()? != MAGIC {
        return Err(Denial::MalformedFrame);
    }
    let version = u32::from_le_bytes(reader.take()?);
    if version != QUERY_CORRECTION_JOURNAL_VERSION.get() {
        return Err(Denial::UnsupportedVersion(version));
    }
    let home =
        HomeIdentity::from_observed_bytes(reader.take()?).map_err(|_| Denial::MalformedFrame)?;
    let position =
        NonZeroU64::new(u64::from_le_bytes(reader.take()?)).ok_or(Denial::MalformedFrame)?;
    let predecessor = reader.take()?;
    let generation =
        NonZeroU64::new(u64::from_le_bytes(reader.take()?)).ok_or(Denial::MalformedFrame)?;
    let body_retention = match reader.take::<1>()?[0] {
        0 => RetentionClass::Authoritative,
        1 => RetentionClass::MandatoryObligation,
        2 => RetentionClass::OptionalRetainedMaterial,
        3 => RetentionClass::Derived,
        _ => return Err(Denial::MalformedFrame),
    };
    let length =
        usize::try_from(u64::from_le_bytes(reader.take()?)).map_err(|_| Denial::MalformedFrame)?;
    if length != reader.0.len() {
        return Err(Denial::MalformedFrame);
    }
    Ok(QueryCorrectionMaterialFrame {
        home,
        position,
        predecessor,
        generation,
        body_retention,
        bytes,
        digest,
    })
}
