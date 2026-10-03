//! Observer-owned tag-7 wire interpretation. No producer codec is an oracle here.

mod no_release;

use crate::integrity_observation::sha256::Sha256;
pub(super) use no_release::NoRelease;

const DOMAIN: &[u8] = b"store.physical.checkpoint.released-drop-custody.v1";
const BATCH_DOMAIN: &[u8] = b"store.physical.checkpoint.released-drop-batches.v1";
const TIP_BYTES: usize = 24 + 32 + 24 + 32 + 80 + 80 + 8 + 32;
const PREFIX: usize = 8 + DOMAIN.len() + 2;
const BATCH_BYTES: usize =
    PREFIX + 24 + 8 + 32 + 2 + 24 + 32 + 32 + 24 + 32 + 80 + 80 + 8 + 32 + 56 + 8 + 32 + 1;
const ACCUMULATOR_BYTES: usize =
    PREFIX + 24 + 8 + 32 + 8 + 32 + 32 + 8 + 32 + 2 + 32 + TIP_BYTES + 8 + 32 + 1;

#[derive(Clone)]
pub(super) struct Batch {
    pub(super) bytes: Vec<u8>,
    pub(super) checkpoint: [u8; 24],
    pub(super) root_generation: u64,
    pub(super) root_sha: [u8; 32],
    pub(super) ordinal: u16,
    pub(super) descriptor: [u8; 24],
    pub(super) reservation: [u8; 24],
    pub(super) tip: [u8; TIP_BYTES],
    pub(super) cumulative: u64,
    pub(super) cumulative_digest: [u8; 32],
    pub(super) terminal: bool,
}

#[derive(Clone)]
pub(super) struct Accumulator {
    pub(super) checkpoint: [u8; 24],
    pub(super) root_generation: u64,
    pub(super) root_sha: [u8; 32],
    pub(super) batch_count: u16,
    pub(super) batch_digest: [u8; 32],
    pub(super) tip: [u8; TIP_BYTES],
    pub(super) cumulative: u64,
    pub(super) cumulative_digest: [u8; 32],
    pub(super) terminal: bool,
}

impl Accumulator {
    #[cfg(test)]
    pub(super) fn tip_descriptor(&self) -> [u8; 24] {
        self.tip[..24].try_into().expect("fixed tip")
    }
}

pub(super) enum Certificate {
    Batch(Batch),
    Accumulator(Accumulator),
    NoRelease(NoRelease),
}

pub(super) fn decode(bytes: &[u8]) -> Option<Certificate> {
    if bytes.len() < PREFIX
        || bytes[..8] != (DOMAIN.len() as u64).to_le_bytes()
        || &bytes[8..8 + DOMAIN.len()] != DOMAIN
        || bytes[PREFIX - 2] != 1
    {
        return None;
    }
    let mut cursor = Cursor::new(bytes, PREFIX);
    let result = match bytes[PREFIX - 1] {
        1 if bytes.len() == BATCH_BYTES => Certificate::Batch(read_batch(&mut cursor)?),
        2 if bytes.len() == ACCUMULATOR_BYTES => {
            Certificate::Accumulator(read_accumulator(&mut cursor)?)
        }
        3 if bytes.len() == no_release::WIRE_BYTES => {
            Certificate::NoRelease(no_release::read(&mut cursor)?)
        }
        _ => return None,
    };
    (cursor.offset == bytes.len()).then_some(result)
}

fn read_batch(cursor: &mut Cursor<'_>) -> Option<Batch> {
    let start = cursor.bytes;
    let checkpoint = cursor.take::<24>()?;
    let root_generation = cursor.u64()?;
    let root_sha = cursor.take::<32>()?;
    let ordinal = cursor.u16()?;
    let descriptor = cursor.take::<24>()?;
    let descriptor_sha = cursor.take::<32>()?;
    let custody = cursor.take::<32>()?;
    let reservation = cursor.take::<24>()?;
    let reservation_sha = cursor.take::<32>()?;
    let request = cursor.take::<80>()?;
    let fate = cursor.take::<80>()?;
    let candidate_generation = cursor.u64()?;
    let candidate_sha = cursor.take::<32>()?;
    let predecessor = cursor.take::<56>()?;
    let cumulative = cursor.u64()?;
    let cumulative_digest = cursor.take::<32>()?;
    let terminal = cursor.boolean()?;
    let tip = tip(
        descriptor,
        descriptor_sha,
        reservation,
        reservation_sha,
        request,
        fate,
        candidate_generation,
        candidate_sha,
    )?;
    if !valid_checkpoint(checkpoint)
        || root_generation == 0
        || root_generation < candidate_generation
        || root_sha == [0; 32]
        || ordinal >= 64
        || custody == [0; 32]
        || cumulative == 0
        || cumulative_digest == [0; 32]
        || (predecessor != [0; 56]
            && (!valid_record(&predecessor[..24])
                || predecessor[24..] == [0; 32]
                || predecessor[..24] == descriptor))
    {
        return None;
    }
    Some(Batch {
        bytes: start.to_vec(),
        checkpoint,
        root_generation,
        root_sha,
        ordinal,
        descriptor,
        reservation,
        tip,
        cumulative,
        cumulative_digest,
        terminal,
    })
}

fn read_accumulator(cursor: &mut Cursor<'_>) -> Option<Accumulator> {
    let checkpoint = cursor.take::<24>()?;
    let root_generation = cursor.u64()?;
    let root_sha = cursor.take::<32>()?;
    let prior_sequence = cursor.u64()?;
    let prior_root = cursor.take::<32>()?;
    let prior_acc = cursor.take::<32>()?;
    let prior_cumulative = cursor.u64()?;
    let prior_digest = cursor.take::<32>()?;
    let batch_count = cursor.u16()?;
    let batch_digest = cursor.take::<32>()?;
    let descriptor = cursor.take::<24>()?;
    let descriptor_sha = cursor.take::<32>()?;
    let reservation = cursor.take::<24>()?;
    let reservation_sha = cursor.take::<32>()?;
    let request = cursor.take::<80>()?;
    let fate = cursor.take::<80>()?;
    let candidate_generation = cursor.u64()?;
    let candidate_sha = cursor.take::<32>()?;
    let cumulative = cursor.u64()?;
    let cumulative_digest = cursor.take::<32>()?;
    let terminal = cursor.boolean()?;
    let tip = tip(
        descriptor,
        descriptor_sha,
        reservation,
        reservation_sha,
        request,
        fate,
        candidate_generation,
        candidate_sha,
    )?;
    let has_prior = prior_sequence != 0;
    if !valid_checkpoint(checkpoint)
        || root_generation == 0
        || root_generation < candidate_generation
        || root_sha == [0; 32]
        || cumulative == 0
        || cumulative_digest == [0; 32]
        || batch_count > 63
        || (batch_count == 0) != (batch_digest == [0; 32])
        || has_prior != (prior_root != [0; 32])
        || has_prior != (prior_acc != [0; 32])
        || has_prior != (prior_cumulative != 0)
        || has_prior != (prior_digest != [0; 32])
        || (has_prior && prior_sequence >= u64::from_le_bytes(checkpoint[16..24].try_into().ok()?))
        || (!has_prior && batch_count == 0)
        || (batch_count == 0
            && (cumulative != prior_cumulative || cumulative_digest != prior_digest))
        || (batch_count != 0 && cumulative <= prior_cumulative)
    {
        return None;
    }
    Some(Accumulator {
        checkpoint,
        root_generation,
        root_sha,
        batch_count,
        batch_digest,
        tip,
        cumulative,
        cumulative_digest,
        terminal,
    })
}

fn tip(
    descriptor: [u8; 24],
    descriptor_sha: [u8; 32],
    reservation: [u8; 24],
    reservation_sha: [u8; 32],
    request: [u8; 80],
    fate: [u8; 80],
    generation: u64,
    candidate_sha: [u8; 32],
) -> Option<[u8; TIP_BYTES]> {
    let lease_start = u64::from_le_bytes(request[64..72].try_into().ok()?);
    let lease_end = u64::from_le_bytes(request[72..80].try_into().ok()?);
    let lsn_start = u64::from_le_bytes(fate[..8].try_into().ok()?);
    let lsn_end = u64::from_le_bytes(fate[8..16].try_into().ok()?);
    if !valid_record(&descriptor)
        || !valid_record(&reservation)
        || descriptor == reservation
        || descriptor_sha == [0; 32]
        || reservation_sha == [0; 32]
        || request[..32] == [0; 32]
        || request[32..64] == [0; 32]
        || lease_end <= lease_start
        || lsn_start == 0
        || lsn_end <= lsn_start
        || fate[16..48] == [0; 32]
        || fate[48..80] == [0; 32]
        || generation == 0
        || candidate_sha == [0; 32]
    {
        return None;
    }
    let mut out = [0; TIP_BYTES];
    let mut pos = 0;
    for part in [
        &descriptor[..],
        &descriptor_sha,
        &reservation,
        &reservation_sha,
        &request,
        &fate,
        &generation.to_le_bytes(),
        &candidate_sha,
    ] {
        out[pos..pos + part.len()].copy_from_slice(part);
        pos += part.len();
    }
    Some(out)
}

fn valid_checkpoint(bytes: [u8; 24]) -> bool {
    bytes[..16] != [0; 16] && bytes[16..] != [0; 8]
}
fn valid_record(bytes: &[u8]) -> bool {
    bytes.len() == 24 && bytes[..16] != [0; 16] && bytes[16..] != [0; 8]
}

pub(super) fn batch_digest(batches: &[Batch]) -> Option<[u8; 32]> {
    let first = batches.first()?;
    if batches.len() > 63 {
        return None;
    }
    let mut sha = Sha256::new();
    sha.update(&(BATCH_DOMAIN.len() as u64).to_le_bytes());
    sha.update(BATCH_DOMAIN);
    sha.update(&(batches.len() as u16).to_le_bytes());
    for (index, batch) in batches.iter().enumerate() {
        if usize::from(batch.ordinal) != index
            || batch.checkpoint != first.checkpoint
            || batch.root_generation != first.root_generation
            || batch.root_sha != first.root_sha
            || batches[..index].iter().any(|prior| {
                prior.descriptor == batch.descriptor || prior.reservation == batch.reservation
            })
            || (index > 0 && batch.cumulative <= batches[index - 1].cumulative)
        {
            return None;
        }
        sha.update(&(batch.bytes.len() as u32).to_le_bytes());
        sha.update(&batch.bytes);
    }
    Some(sha.finish())
}

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}
impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8], offset: usize) -> Self {
        Self { bytes, offset }
    }
    fn take<const N: usize>(&mut self) -> Option<[u8; N]> {
        let end = self.offset.checked_add(N)?;
        let result = self.bytes.get(self.offset..end)?.try_into().ok()?;
        self.offset = end;
        Some(result)
    }
    fn u64(&mut self) -> Option<u64> {
        Some(u64::from_le_bytes(self.take()?))
    }
    fn u16(&mut self) -> Option<u16> {
        Some(u16::from_le_bytes(self.take()?))
    }
    fn boolean(&mut self) -> Option<bool> {
        match self.take::<1>()?[0] {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        }
    }
}
