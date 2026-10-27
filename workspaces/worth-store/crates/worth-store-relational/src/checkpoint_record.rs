//! The home's selected checkpoint: Relational's native checkpoint bytes,
//! filed under the canonical position and the branch transition sequence
//! they cover, as the anchored record `relational.checkpoint`. Replacing it
//! is one durable root decision, so a crash leaves either the previous
//! checkpoint or the new one, never a torn mixture.
//!
//! Payload (little-endian): magic `WSRCKPT1`, format version `u16` (3),
//! covered position `u64`, covered transition sequence `u64`, then the native
//! checkpoint bytes. Store's anchored-record frame already binds the
//! payload's digest and generation. A record of another version is refused,
//! never reinterpreted.

use worth_relational::facade::branch::RelationalBranchTransitionSequence;
use worth_relational::facade::durability::{
    RelationalCheckpointMember, RelationalDurableCheckpointSource, RelationalNativeCheckpoint,
};
use worth_relational::facade::publication::PatchStreamPosition;
use worth_store::physical_runtime::{
    AnchoredRecordMember, AnchoredRecordName, AnchoredRecordPublication,
    AnchoredRecordPublicationFailure, AnchoredRecordPublished, AnchoredRecordReadFailure,
    AnchoredRecordsDenial, PhysicalMutationDeadline, ServingPhysicalRuntime,
};

const ANCHOR: &str = "relational.checkpoint";
const MAGIC: &[u8; 8] = b"WSRCKPT1";
const FORMAT_VERSION: u16 = 3;
const HEADER_BYTES: usize = 26;
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
    /// A capture completed after a later recovery basis was selected. It may
    /// not remove that basis's member membership or regress either frontier.
    StaleCapture,
    Publication(AnchoredRecordPublicationFailure),
}

type Refusal = RelationalStoreCheckpointRecordRefusal;

fn anchor() -> AnchoredRecordName {
    AnchoredRecordName::new(ANCHOR).expect("the checkpoint anchor name is admissible")
}

/// The selected checkpoint, or `None` when the home has never checkpointed.
pub(crate) fn read(
    serving: &ServingPhysicalRuntime,
    cold_source: &crate::cold_source::StoreColdSource,
) -> Result<Option<RelationalDurableCheckpointSource>, Refusal> {
    let records = serving.anchored_records().map_err(Refusal::Unavailable)?;
    let Some(record) = records
        .read(anchor(), MAXIMUM_CHECKPOINT_BYTES)
        .map_err(Refusal::Read)?
    else {
        return Ok(None);
    };
    cold_source.select_members(record.version().generation(), record.members());
    let bytes = record.into_bytes();
    let (covered, covered_transitions) = decode_header(&bytes).ok_or(Refusal::Malformed)?;
    let checkpoint = RelationalNativeCheckpoint::from_untrusted_bytes(&bytes[HEADER_BYTES..]);
    Ok(Some(RelationalDurableCheckpointSource::new(
        PatchStreamPosition(covered),
        RelationalBranchTransitionSequence(covered_transitions),
        checkpoint,
    )))
}

/// Replace the selected checkpoint with `checkpoint`, filed under `covered`
/// and `covered_transitions`. Returns the anchored generation the new
/// checkpoint holds.
pub(crate) fn publish(
    serving: &ServingPhysicalRuntime,
    cold_source: &crate::cold_source::StoreColdSource,
    covered: PatchStreamPosition,
    covered_transitions: RelationalBranchTransitionSequence,
    checkpoint: &RelationalNativeCheckpoint,
) -> Result<u64, Refusal> {
    // The fence precedes reading the anchor and lasts through selection. New
    // read custody cannot arrive in the gap between member union and cutover.
    let retained = cold_source.protect_member_selection();
    let records = serving.anchored_records().map_err(Refusal::Unavailable)?;
    let previous = records
        .read(anchor(), MAXIMUM_CHECKPOINT_BYTES)
        .map_err(Refusal::Read)?;
    let current = previous
        .as_ref()
        .map_or(0, |record| record.version().generation());
    if let Some(previous) = &previous {
        let (position, transitions) = decode_header(previous.bytes()).ok_or(Refusal::Malformed)?;
        if covered.0 < position || covered_transitions.0 < transitions {
            return Err(Refusal::StaleCapture);
        }
    }
    let old_members = previous
        .as_ref()
        .map(|record| record.members())
        .unwrap_or(&[])
        .iter()
        .map(|member| ((member.sha256(), member.length()), *member))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut members = checkpoint
        .members()
        .iter()
        .map(|member| {
            let key = member.locator();
            if let Some(locator) = old_members.get(&(key.sha256, key.length)) {
                return Ok((key, AnchoredRecordMember::Carried { locator: *locator }));
            }
            match member {
                RelationalCheckpointMember::New { locator, bytes } => Ok((
                    key,
                    AnchoredRecordMember::New {
                        key: locator.sha256,
                        bytes: bytes.to_vec(),
                    },
                )),
                RelationalCheckpointMember::Carried { .. } => Err(Refusal::Malformed),
            }
        })
        .collect::<Result<std::collections::BTreeMap<_, _>, _>>()?;
    for key in retained.member_locators() {
        if let std::collections::btree_map::Entry::Vacant(entry) = members.entry(key) {
            let locator = old_members
                .get(&(key.sha256, key.length))
                .ok_or(Refusal::Malformed)?;
            entry.insert(AnchoredRecordMember::Carried { locator: *locator });
        }
    }
    let mut payload = Vec::with_capacity(HEADER_BYTES + checkpoint.bytes().len());
    payload.extend_from_slice(MAGIC);
    payload.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    payload.extend_from_slice(&covered.0.to_le_bytes());
    payload.extend_from_slice(&covered_transitions.0.to_le_bytes());
    payload.extend_from_slice(checkpoint.bytes());
    let deadline = PhysicalMutationDeadline::after_milliseconds(PUBLICATION_DEADLINE_MILLIS)
        .expect("the checkpoint deadline is representable");
    match records
        .publish(
            AnchoredRecordPublication::new(
                anchor(),
                current,
                payload,
                crate::physical_store::placement(),
                deadline,
            )
            .with_members(members.into_values().collect()),
        )
        .map_err(Refusal::Publication)?
    {
        AnchoredRecordPublished::Published(membership)
        | AnchoredRecordPublished::AlreadyPublished(membership) => {
            cold_source.select_members(membership.version().generation(), membership.members());
            Ok(membership.version().generation())
        }
    }
}

/// The covered position and transition sequence, or `None` for a record
/// that is not a current checkpoint record.
fn decode_header(bytes: &[u8]) -> Option<(u64, u64)> {
    let rest = bytes.strip_prefix(MAGIC)?;
    let (version, rest) = rest.split_first_chunk::<2>()?;
    if u16::from_le_bytes(*version) != FORMAT_VERSION {
        return None;
    }
    let (covered, rest) = rest.split_first_chunk::<8>()?;
    let (covered_transitions, _) = rest.split_first_chunk::<8>()?;
    Some((
        u64::from_le_bytes(*covered),
        u64::from_le_bytes(*covered_transitions),
    ))
}
