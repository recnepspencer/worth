//! The home's selected checkpoint: Relational's native checkpoint bytes,
//! filed under the canonical position they cover, as the anchored record
//! `relational.checkpoint`. Replacing it is one durable root decision, so a
//! crash leaves either the previous checkpoint or the new one, never a torn
//! mixture.
//!
//! Payload (little-endian): magic `WSRCKPT1`, format version `u16`, covered
//! position `u64`, then the native checkpoint bytes. Store's anchored-record
//! frame already binds the payload's digest and generation.

use worth_relational::facade::durability::{
    RelationalDurableCheckpointSource, RelationalNativeCheckpoint,
};
use worth_relational::facade::publication::PatchStreamPosition;
use worth_store::physical_runtime::{
    AnchoredRecordName, AnchoredRecordPublication, AnchoredRecordPublicationFailure,
    AnchoredRecordPublished, AnchoredRecordReadFailure, AnchoredRecordsDenial,
    PhysicalMutationDeadline, ServingPhysicalRuntime,
};

const ANCHOR: &str = "relational.checkpoint";
const MAGIC: &[u8; 8] = b"WSRCKPT1";
const FORMAT_VERSION: u16 = 1;
const HEADER_BYTES: usize = 18;
/// The largest checkpoint a home reads back. A larger record is refused, not
/// allocated.
const MAXIMUM_CHECKPOINT_BYTES: u64 = 1 << 30;
const PUBLICATION_DEADLINE_MILLIS: u64 = 120_000;

/// Why the selected checkpoint record could not be read or replaced.
#[derive(Debug)]
#[non_exhaustive]
pub enum RelationalStoreCheckpointRecordRefusal {
    Unavailable(AnchoredRecordsDenial),
    Read(AnchoredRecordReadFailure),
    /// The record is intact for Store but is not a checkpoint record this
    /// build can read.
    Malformed,
    Publication(AnchoredRecordPublicationFailure),
}

type Refusal = RelationalStoreCheckpointRecordRefusal;

fn anchor() -> AnchoredRecordName {
    AnchoredRecordName::new(ANCHOR).expect("the checkpoint anchor name is admissible")
}

/// The selected checkpoint, or `None` when the home has never checkpointed.
pub(crate) fn read(
    serving: &ServingPhysicalRuntime,
) -> Result<Option<RelationalDurableCheckpointSource>, Refusal> {
    let records = serving.anchored_records().map_err(Refusal::Unavailable)?;
    let Some(record) = records
        .read(anchor(), MAXIMUM_CHECKPOINT_BYTES)
        .map_err(Refusal::Read)?
    else {
        return Ok(None);
    };
    let bytes = record.into_bytes();
    let covered = decode_header(&bytes).ok_or(Refusal::Malformed)?;
    let checkpoint = RelationalNativeCheckpoint::from_untrusted_bytes(&bytes[HEADER_BYTES..]);
    Ok(Some(RelationalDurableCheckpointSource::new(
        PatchStreamPosition(covered),
        checkpoint,
    )))
}

/// Replace the selected checkpoint with `checkpoint`, filed under `covered`.
/// Returns the anchored generation the new checkpoint holds.
pub(crate) fn publish(
    serving: &ServingPhysicalRuntime,
    covered: PatchStreamPosition,
    checkpoint: &RelationalNativeCheckpoint,
) -> Result<u64, Refusal> {
    let records = serving.anchored_records().map_err(Refusal::Unavailable)?;
    let current = records
        .read(anchor(), MAXIMUM_CHECKPOINT_BYTES)
        .map_err(Refusal::Read)?
        .map_or(0, |record| record.version().generation());
    let mut payload = Vec::with_capacity(HEADER_BYTES + checkpoint.bytes().len());
    payload.extend_from_slice(MAGIC);
    payload.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    payload.extend_from_slice(&covered.0.to_le_bytes());
    payload.extend_from_slice(checkpoint.bytes());
    let deadline = PhysicalMutationDeadline::after_milliseconds(PUBLICATION_DEADLINE_MILLIS)
        .expect("the checkpoint deadline is representable");
    match records
        .publish(AnchoredRecordPublication::new(
            anchor(),
            current,
            payload,
            crate::physical_store::placement(),
            deadline,
        ))
        .map_err(Refusal::Publication)?
    {
        AnchoredRecordPublished::Published(version)
        | AnchoredRecordPublished::AlreadyPublished(version) => Ok(version.generation()),
    }
}

fn decode_header(bytes: &[u8]) -> Option<u64> {
    let rest = bytes.strip_prefix(MAGIC)?;
    let (version, rest) = rest.split_first_chunk::<2>()?;
    if u16::from_le_bytes(*version) != FORMAT_VERSION {
        return None;
    }
    let (covered, _) = rest.split_first_chunk::<8>()?;
    Some(u64::from_le_bytes(*covered))
}
