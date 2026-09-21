//! Separately versioned record-preserving rewrite redo.
//!
//! This payload is not a reinterpretation of `store.physical.wal.canonical-redo.v3`.

pub const REWRITE_REDO_DOMAIN: &[u8] = b"store.physical.rewrite-redo.v1";

const BODY_BYTES: usize = 216;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalRewriteRedo {
    operation: [u8; 32],
    group: [u8; 32],
    source_root_generation: u64,
    source_generation: u64,
    source_offset: u64,
    source_length: u32,
    source_digest: [u8; 32],
    destination_generation: u64,
    destination_offset: u64,
    destination_length: u32,
    page_lsn: u64,
    record_identity: [u8; 32],
    source_placement: u64,
    destination_placement: u64,
    candidate_bytes: u64,
    resulting_root_generation: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRewriteRedoDenial {
    WrongDomain,
    Truncated,
    TrailingBytes,
    LengthMismatch,
    CandidateLimit,
}

impl PhysicalRewriteRedo {
    pub fn new(
        operation: [u8; 32],
        group: [u8; 32],
        source_root_generation: u64,
        source_generation: u64,
        source_offset: u64,
        source_length: u32,
        source_digest: [u8; 32],
        destination_generation: u64,
        destination_offset: u64,
        page_lsn: u64,
        record_identity: [u8; 32],
        source_placement: u64,
        destination_placement: u64,
        resulting_root_generation: u64,
    ) -> Option<Self> {
        if source_length == 0 {
            return None;
        }
        Some(Self {
            operation,
            group,
            source_root_generation,
            source_generation,
            source_offset,
            source_length,
            source_digest,
            destination_generation,
            destination_offset,
            destination_length: source_length,
            page_lsn,
            record_identity,
            source_placement,
            destination_placement,
            candidate_bytes: u64::from(source_length),
            resulting_root_generation,
        })
    }

    pub const fn operation(self) -> [u8; 32] {
        self.operation
    }

    pub const fn group(self) -> [u8; 32] {
        self.group
    }

    pub const fn source_root_generation(self) -> u64 {
        self.source_root_generation
    }

    pub const fn source_generation(self) -> u64 {
        self.source_generation
    }

    pub const fn source_offset(self) -> u64 {
        self.source_offset
    }

    pub const fn source_length(self) -> u32 {
        self.source_length
    }

    pub const fn source_digest(self) -> [u8; 32] {
        self.source_digest
    }

    pub const fn destination_generation(self) -> u64 {
        self.destination_generation
    }

    pub const fn destination_offset(self) -> u64 {
        self.destination_offset
    }

    pub const fn destination_length(self) -> u32 {
        self.destination_length
    }

    pub const fn page_lsn(self) -> u64 {
        self.page_lsn
    }

    pub const fn record_identity(self) -> [u8; 32] {
        self.record_identity
    }

    pub const fn source_placement(self) -> u64 {
        self.source_placement
    }

    pub const fn destination_placement(self) -> u64 {
        self.destination_placement
    }

    pub const fn candidate_bytes(self) -> u64 {
        self.candidate_bytes
    }

    pub const fn with_page_lsn(mut self, page_lsn: u64) -> Self {
        self.page_lsn = page_lsn;
        self
    }

    pub const fn with_group(mut self, group: [u8; 32]) -> Self {
        self.group = group;
        self
    }

    pub const fn resulting_root_generation(self) -> u64 {
        self.resulting_root_generation
    }

    pub fn encode(self) -> Vec<u8> {
        let mut encoded = Vec::with_capacity(8 + REWRITE_REDO_DOMAIN.len() + BODY_BYTES);
        write_field(&mut encoded, REWRITE_REDO_DOMAIN);
        encoded.extend_from_slice(&self.operation);
        encoded.extend_from_slice(&self.group);
        encoded.extend_from_slice(&self.source_root_generation.to_le_bytes());
        encoded.extend_from_slice(&self.source_generation.to_le_bytes());
        encoded.extend_from_slice(&self.source_offset.to_le_bytes());
        encoded.extend_from_slice(&self.source_length.to_le_bytes());
        encoded.extend_from_slice(&self.source_digest);
        encoded.extend_from_slice(&self.destination_generation.to_le_bytes());
        encoded.extend_from_slice(&self.destination_offset.to_le_bytes());
        encoded.extend_from_slice(&self.destination_length.to_le_bytes());
        encoded.extend_from_slice(&self.page_lsn.to_le_bytes());
        encoded.extend_from_slice(&self.record_identity);
        encoded.extend_from_slice(&self.source_placement.to_le_bytes());
        encoded.extend_from_slice(&self.destination_placement.to_le_bytes());
        encoded.extend_from_slice(&self.candidate_bytes.to_le_bytes());
        encoded.extend_from_slice(&self.resulting_root_generation.to_le_bytes());
        encoded
    }

    pub fn decode(bytes: &[u8], maximum_candidate_bytes: u64) -> Result<Self, PhysicalRewriteRedoDenial> {
        let mut cursor = bytes;
        let domain = take_field(&mut cursor)?;
        if domain != REWRITE_REDO_DOMAIN {
            return Err(PhysicalRewriteRedoDenial::WrongDomain);
        }
        if cursor.len() != BODY_BYTES {
            return Err(if cursor.len() < BODY_BYTES {
                PhysicalRewriteRedoDenial::Truncated
            } else {
                PhysicalRewriteRedoDenial::TrailingBytes
            });
        }
        let redo = Self {
            operation: take_array(&mut cursor)?,
            group: take_array(&mut cursor)?,
            source_root_generation: take_u64(&mut cursor)?,
            source_generation: take_u64(&mut cursor)?,
            source_offset: take_u64(&mut cursor)?,
            source_length: take_u32(&mut cursor)?,
            source_digest: take_array(&mut cursor)?,
            destination_generation: take_u64(&mut cursor)?,
            destination_offset: take_u64(&mut cursor)?,
            destination_length: take_u32(&mut cursor)?,
            page_lsn: take_u64(&mut cursor)?,
            record_identity: take_array(&mut cursor)?,
            source_placement: take_u64(&mut cursor)?,
            destination_placement: take_u64(&mut cursor)?,
            candidate_bytes: take_u64(&mut cursor)?,
            resulting_root_generation: take_u64(&mut cursor)?,
        };
        if !cursor.is_empty() {
            return Err(PhysicalRewriteRedoDenial::TrailingBytes);
        }
        if redo.source_length == 0
            || redo.source_length != redo.destination_length
            || u64::from(redo.destination_length) != redo.candidate_bytes
        {
            return Err(PhysicalRewriteRedoDenial::LengthMismatch);
        }
        if redo.candidate_bytes > maximum_candidate_bytes {
            return Err(PhysicalRewriteRedoDenial::CandidateLimit);
        }
        Ok(redo)
    }
}

fn write_field(target: &mut Vec<u8>, bytes: &[u8]) {
    target.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    target.extend_from_slice(bytes);
}

fn take_field<'a>(cursor: &mut &'a [u8]) -> Result<&'a [u8], PhysicalRewriteRedoDenial> {
    let length = usize::try_from(take_u64(cursor)?).map_err(|_| PhysicalRewriteRedoDenial::Truncated)?;
    if cursor.len() < length {
        return Err(PhysicalRewriteRedoDenial::Truncated);
    }
    let (field, rest) = cursor.split_at(length);
    *cursor = rest;
    Ok(field)
}

fn take_array<const N: usize>(cursor: &mut &[u8]) -> Result<[u8; N], PhysicalRewriteRedoDenial> {
    if cursor.len() < N {
        return Err(PhysicalRewriteRedoDenial::Truncated);
    }
    let mut array = [0; N];
    array.copy_from_slice(&cursor[..N]);
    *cursor = &cursor[N..];
    Ok(array)
}

fn take_u64(cursor: &mut &[u8]) -> Result<u64, PhysicalRewriteRedoDenial> {
    Ok(u64::from_le_bytes(take_array(cursor)?))
}

fn take_u32(cursor: &mut &[u8]) -> Result<u32, PhysicalRewriteRedoDenial> {
    Ok(u32::from_le_bytes(take_array(cursor)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> PhysicalRewriteRedo {
        PhysicalRewriteRedo::new(
            [1; 32],
            [2; 32],
            4,
            7,
            128,
            32,
            [9; 32],
            8,
            256,
            11,
            [3; 32],
            128,
            256,
            5,
        )
        .unwrap()
    }

    #[test]
    fn a_rewrite_payload_round_trips_without_becoming_canonical_redo() {
        let encoded = sample().encode();
        assert!(encoded.windows(REWRITE_REDO_DOMAIN.len()).any(|window| window == REWRITE_REDO_DOMAIN));
        assert!(!encoded.windows(b"store.physical.wal.canonical-redo.v3".len()).any(|window| {
            window == b"store.physical.wal.canonical-redo.v3"
        }));
        assert_eq!(PhysicalRewriteRedo::decode(&encoded, 32).unwrap(), sample());
        assert_eq!(
            PhysicalRewriteRedo::decode(&encoded, 31),
            Err(PhysicalRewriteRedoDenial::CandidateLimit)
        );
        let mut unknown = encoded.clone();
        unknown[8] ^= 1;
        assert_eq!(
            PhysicalRewriteRedo::decode(&unknown, 32),
            Err(PhysicalRewriteRedoDenial::WrongDomain)
        );
        assert_eq!(
            PhysicalRewriteRedo::decode(&encoded[..encoded.len() - 1], 32),
            Err(PhysicalRewriteRedoDenial::Truncated)
        );
    }
}
