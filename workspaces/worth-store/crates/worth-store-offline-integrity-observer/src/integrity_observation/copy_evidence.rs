//! Independent interpretation of extent-copy WAL facts. This parser reads
//! complete fields at verified WAL boundaries and never scans user payloads.
use super::{
    families::durable_frame::{read_u32, read_u64},
    sha256::sha256,
};

const INTENT_DOMAIN: &[u8] = b"store.physical.extent-copy.v1";
const CLASSIFIED_INTENT_DOMAIN: &[u8] = b"store.physical.extent-copy.v2";
const FINAL_DOMAIN: &[u8] = b"store.physical.extent-copy-publication.v1";
const CURRENT_PROJECTION_DOMAIN: &[u8] = b"store.physical.recovery-projection.v16";
const BINDING_DOMAIN: &[u8] = b"store.physical.mutation-attempt-binding.v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CopyIntentFact {
    pub(crate) operation: [u8; 32],
    pub(crate) lsn: u64,
    pub(crate) digest: [u8; 32],
    pub(crate) source_root: u64,
    pub(crate) extent: u64,
    pub(crate) source_generation: u64,
    pub(crate) destination_generation: u64,
    pub(crate) source: (u64, u64, u64),
    pub(crate) destination: (u64, u64, u64),
    pub(crate) record: [u8; 24],
    pub(crate) payload_bytes: u64,
    pub(crate) alignment: u64,
    pub(crate) chunk_count: u32,
    pub(crate) source_route_metadata: [u8; 7],
    pub(crate) destination_route_metadata: [u8; 7],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CopyResolutionFact {
    Cancelled {
        operation: [u8; 32],
        intent_lsn: u64,
        intent_digest: [u8; 32],
    },
    Published {
        operation: [u8; 32],
        intent_lsn: u64,
        intent_digest: [u8; 32],
        root: u64,
        publication_lsn: u64,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CopyFinalFact {
    pub(crate) intent: CopyIntentFact,
    pub(crate) source_root: u64,
    pub(crate) result_root: u64,
    pub(crate) publication_lsn: u64,
}

impl CopyIntentFact {
    pub(crate) fn decode(payload: &[u8], lsn: (u64, u64)) -> Option<Self> {
        let (body, classified) = if let Some(body) = payload.strip_prefix(CLASSIFIED_INTENT_DOMAIN)
        {
            (body, true)
        } else {
            (payload.strip_prefix(INTENT_DOMAIN)?, false)
        };
        let body = body.strip_prefix(&[1])?;
        if body.len() != (if classified { 214 } else { 200 })
            || lsn.0 == 0
            || lsn.0.checked_add(1) != Some(lsn.1)
            || read_u64(body, 64) != 0
        {
            return None;
        }
        let operation: [u8; 32] = body[..32].try_into().ok()?;
        let record: [u8; 24] = body[40..64].try_into().ok()?;
        let source_root = read_u64(body, 32);
        let extent = read_u64(body, 72);
        let source_generation = read_u64(body, 80);
        let destination_generation = read_u64(body, 88);
        let source = (read_u64(body, 96), read_u64(body, 104), read_u64(body, 112));
        let destination = (
            read_u64(body, 120),
            read_u64(body, 128),
            read_u64(body, 136),
        );
        let payload_bytes = read_u64(body, 144);
        let alignment = read_u64(body, 152);
        let maximum_frame_bytes = read_u32(body, 160);
        let chunk_count = read_u32(body, 164);
        let (source_route_metadata, destination_route_metadata) = if classified {
            let source: [u8; 7] = body[200..207].try_into().ok()?;
            let destination: [u8; 7] = body[207..214].try_into().ok()?;
            if !valid_route_metadata(&source)
                || source[0] == 0
                || !valid_route_metadata(&destination)
                || source[..4] != destination[..4]
            {
                return None;
            }
            (source, destination)
        } else {
            ([0; 7], [0; 7])
        };
        if operation == [0; 32]
            || record[..16] == [0; 16]
            || read_u64(&record, 16) == 0
            || source_root == 0
            || extent == 0
            || source_generation == 0
            || destination_generation != source_generation.checked_add(1)?
            || source.0 == 0
            || destination.0 == 0
            || source.0 == destination.0
            || source.2 == 0
            || source.2 != destination.2
            || payload_bytes == 0
            || alignment == 0
            || !alignment.is_power_of_two()
            || source.1 % alignment != 0
            || destination.1 % alignment != 0
            || source.2 % alignment != 0
            || source.1.checked_add(source.2).is_none()
            || destination.1.checked_add(destination.2).is_none()
            || maximum_frame_bytes == 0
            || chunk_count == 0
        {
            return None;
        }
        Some(Self {
            operation,
            lsn: lsn.0,
            digest: sha256(payload),
            source_root,
            extent,
            source_generation,
            destination_generation,
            source,
            destination,
            record,
            payload_bytes,
            alignment,
            chunk_count,
            source_route_metadata,
            destination_route_metadata,
        })
    }
}

impl CopyResolutionFact {
    pub(crate) fn decode(payload: &[u8], lsn: (u64, u64)) -> Option<Self> {
        let body = payload.strip_prefix(INTENT_DOMAIN)?.strip_prefix(&[2])?;
        if body.len() != 89 || lsn.0 == 0 || lsn.0.checked_add(1) != Some(lsn.1) {
            return None;
        }
        let operation = body[..32].try_into().ok()?;
        let intent_digest = body[32..64].try_into().ok()?;
        let intent_lsn = read_u64(body, 64);
        if operation == [0; 32] || intent_lsn == 0 || intent_lsn >= lsn.0 {
            return None;
        }
        match body[72] {
            0 if read_u64(body, 73) == 0 && read_u64(body, 81) == 0 => Some(Self::Cancelled {
                operation,
                intent_lsn,
                intent_digest,
            }),
            1 if read_u64(body, 73) != 0 && read_u64(body, 81) > intent_lsn => {
                Some(Self::Published {
                    operation,
                    intent_lsn,
                    intent_digest,
                    root: read_u64(body, 73),
                    publication_lsn: read_u64(body, 81),
                })
            }
            _ => None,
        }
    }
    pub(crate) fn identity(self) -> ([u8; 32], u64, [u8; 32]) {
        match self {
            Self::Cancelled {
                operation,
                intent_lsn,
                intent_digest,
            }
            | Self::Published {
                operation,
                intent_lsn,
                intent_digest,
                ..
            } => (operation, intent_lsn, intent_digest),
        }
    }
}

impl CopyFinalFact {
    pub(crate) fn decode(payload: &[u8], store: [u8; 16], lsn: (u64, u64)) -> Option<Self> {
        let mut outer = Cursor(payload);
        let binding = outer.field()?;
        let redo = outer.field()?;
        outer.end()?;
        let mut binding = Cursor(binding);
        if binding.field()? != BINDING_DOMAIN {
            return None;
        }
        let operation: [u8; 32] = binding.fixed_field(32)?.try_into().ok()?;
        if binding.fixed_field(16)? != store {
            return None;
        }
        binding.fixed_field(32)?;
        binding.take(16)?;
        binding.fixed_field(32)?;
        binding.fixed_field(32)?;
        if binding.fixed_field(16)? != store {
            return None;
        }
        binding.take(24)?;
        binding.fixed_field(32)?;
        let member = binding.take(8)?;
        if read_u32(member, 0) == 0 || read_u32(member, 0) > read_u32(member, 4) {
            return None;
        }
        binding.fixed_field(32)?;
        binding.fixed_field(32)?;
        let interval = binding.take(16)?;
        if (read_u64(interval, 0), read_u64(interval, 8)) != lsn
            || binding.fixed_field(32)? != sha256(redo)
        {
            return None;
        }
        binding.end()?;
        let mut redo = Cursor(redo);
        if redo.field()? != FINAL_DOMAIN {
            return None;
        }
        let publication_lsn = redo.u64()?;
        if publication_lsn != lsn.0 || publication_lsn.checked_add(1) != Some(lsn.1) {
            return None;
        }
        let projection = redo.field()?;
        redo.end()?;
        let mut projection = Cursor(projection);
        if projection.field()? != CURRENT_PROJECTION_DOMAIN {
            return None;
        }
        let source_root = projection.u64()?;
        if projection.field()?.is_empty() || projection.u64()? != 1 {
            return None;
        }
        let record = projection.fixed_field(24)?;
        if projection.take(1)? != [1] {
            return None;
        }
        let intent_bytes = projection.field()?;
        let intent = parse_embedded_intent(intent_bytes)?;
        let intent_lsn = projection.u64()?;
        let digest = projection.take(32)?;
        if intent_lsn == 0
            || intent_lsn >= publication_lsn
            || intent.digest != digest
            || intent.operation != operation
            || record != intent.record
            || source_root < intent.source_root
            || projection.u64()? != 1
        {
            return None;
        }
        let destination = projection.fixed_field(80)?;
        if destination[0] != 2
            || destination[1..25] != intent.record
            || read_u64(destination, 25) != intent.extent
            || read_u64(destination, 33) != intent.destination_generation
            || read_u64(destination, 41) != intent.payload_bytes
            || (
                read_u64(destination, 49),
                read_u64(destination, 57),
                read_u64(destination, 65),
            ) != intent.destination
            || destination[73..] != intent.destination_route_metadata
            || projection.u64()? != 0
            || projection.u64()? != 0
            || projection.field()? != [0]
        {
            return None;
        }
        projection.end()?;
        Some(Self {
            intent: CopyIntentFact {
                lsn: intent_lsn,
                ..intent
            },
            source_root,
            result_root: source_root.checked_add(1)?,
            publication_lsn,
        })
    }
}

fn parse_embedded_intent(bytes: &[u8]) -> Option<CopyIntentFact> {
    // Embedded recipe has no WAL interval. Supply one only for structural decode;
    // the separately observed intent frame establishes its actual LSN.
    let mut fact = CopyIntentFact::decode(bytes, (1, 2))?;
    fact.lsn = 0;
    Some(fact)
}

fn valid_route_metadata(bytes: &[u8]) -> bool {
    if bytes.len() != 7 || bytes[5..] != [0; 2] || bytes[4] > 2 {
        return false;
    }
    let family = u16::from_le_bytes([bytes[2], bytes[3]]);
    match (bytes[0], bytes[1], family) {
        (0, 0, 0) => bytes[4] == 0,
        (1, 0, 0) | (4, 0, 0) => true,
        (2, 1..=16, 0) => true,
        (3, 0, 1..) => true,
        _ => false,
    }
}

struct Cursor<'a>(&'a [u8]);
impl<'a> Cursor<'a> {
    fn take(&mut self, length: usize) -> Option<&'a [u8]> {
        let result = self.0.get(..length)?;
        self.0 = &self.0[length..];
        Some(result)
    }
    fn u64(&mut self) -> Option<u64> {
        Some(read_u64(self.take(8)?, 0))
    }
    fn field(&mut self) -> Option<&'a [u8]> {
        let length = usize::try_from(self.u64()?).ok()?;
        self.take(length)
    }
    fn fixed_field(&mut self, length: usize) -> Option<&'a [u8]> {
        self.field().filter(|field| field.len() == length)
    }
    fn end(&self) -> Option<()> {
        self.0.is_empty().then_some(())
    }
}

#[cfg(test)]
#[path = "copy_evidence/tests.rs"]
mod tests;
