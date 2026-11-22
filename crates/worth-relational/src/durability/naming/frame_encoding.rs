//! Canonical encoding of the durable persistent-name log.
//!
//! A log is one header naming its format version and branch, followed by one
//! self-delimiting frame per committed naming batch. Frames append at commit
//! boundaries, so a log torn mid-append ends in a truncated frame that the
//! decoder refuses typed.

use crate::history::data::{BranchId, CommitId, RelationalCommitReceipt};
use crate::identity::data::{EntityId, LineageId, RelationId};
use crate::lineage::data::{LineageEventRecord, PersistentNameBirthRecord, PersistentNameKey};
use crate::lineage::index::PersistentNameCommittedBatch;

use super::format::{
    group_event_parts, kind_tag, name_subject_tag, subject_tag, FORMAT_VERSION, FRAME_MAGIC,
    LOG_MAGIC,
};
use super::refusal::PersistentNameDurabilityError as Refusal;
use super::wire::WireWriter;

/// Encodes a complete log: its header and one frame per committed batch,
/// each paired with the retention floor in force after it.
///
/// Complexity: O(total encoded bytes).
pub fn encode_durable_persistent_name_log<'batch>(
    branch_id: &BranchId,
    commits: impl IntoIterator<Item = (&'batch PersistentNameCommittedBatch, Option<CommitId>)>,
) -> Result<Vec<u8>, Refusal> {
    let mut log = encode_log_header(branch_id)?;
    for (batch, min_retained_commit) in commits {
        log.extend(encode_commit_frame(batch, min_retained_commit)?);
    }
    Ok(log)
}

/// Encodes the log header written once before the first frame.
pub fn encode_log_header(branch_id: &BranchId) -> Result<Vec<u8>, Refusal> {
    let mut writer = WireWriter::default();
    writer.raw(LOG_MAGIC);
    writer.u8(FORMAT_VERSION);
    writer.text("branch id", &branch_id.0)?;
    Ok(writer.into_bytes())
}

/// Encodes one committed batch as an appendable, self-delimiting frame.
///
/// Complexity: O(batch events + holders + group events + carriers).
pub fn encode_commit_frame(
    batch: &PersistentNameCommittedBatch,
    min_retained_commit: Option<CommitId>,
) -> Result<Vec<u8>, Refusal> {
    let payload = encode_frame_payload(batch, min_retained_commit)?;
    let mut writer = WireWriter::default();
    writer.raw(FRAME_MAGIC);
    writer.u8(FORMAT_VERSION);
    writer.bytes("frame payload", &payload)?;
    Ok(writer.into_bytes())
}

fn encode_frame_payload(
    batch: &PersistentNameCommittedBatch,
    min_retained_commit: Option<CommitId>,
) -> Result<Vec<u8>, Refusal> {
    let mut writer = WireWriter::default();
    writer.u64(batch.commit.0);
    writer.option_tag(min_retained_commit.is_some());
    if let Some(floor) = min_retained_commit {
        writer.u64(floor.0);
    }
    let ascending_events = batch
        .lineage_events
        .windows(2)
        .all(|pair| pair[0].event_id() < pair[1].event_id());
    if !ascending_events {
        return Err(Refusal::NonCanonical {
            field: "lineage event order",
        });
    }
    writer.count("lineage events", batch.lineage_events.len())?;
    for event in &batch.lineage_events {
        encode_event(&mut writer, event)?;
    }
    writer.count("entity holders", batch.entity_holders.len())?;
    for (lineage, entity) in &batch.entity_holders {
        writer.u64(lineage.0);
        encode_entity(&mut writer, *entity);
    }
    writer.count("relation holders", batch.relation_holders.len())?;
    for (lineage, relation) in &batch.relation_holders {
        writer.u64(lineage.0);
        encode_relation(&mut writer, *relation);
    }
    writer.count("group events", batch.group_events.len())?;
    for event in &batch.group_events {
        let (tag, first, second) = group_event_parts(*event);
        writer.u8(tag);
        writer.digest(first.bytes());
        if let Some(second) = second {
            writer.digest(second.bytes());
        }
    }
    writer.count("carrier declarations", batch.carriers.len())?;
    for (key, carriers) in &batch.carriers {
        encode_name_key(&mut writer, key);
        writer.count("carriers", carriers.len())?;
        for carrier in carriers {
            writer.digest(carrier.as_bytes());
        }
    }
    Ok(writer.into_bytes())
}

fn encode_event(writer: &mut WireWriter, event: &LineageEventRecord) -> Result<(), Refusal> {
    writer.u64(event.event_id());
    encode_receipt(writer, event.commit())?;
    writer.text("event branch id", &event.branch_id().0)?;
    writer.u8(kind_tag(event.kind()));
    writer.u8(subject_tag(event.subject()));
    encode_lineages(writer, "event sources", event.sources())?;
    encode_lineages(writer, "event targets", event.targets())?;
    writer.option_tag(event.name_birth().is_some());
    if let Some(birth) = event.name_birth() {
        encode_birth(writer, birth)?;
    }
    writer.option_tag(event.kept_name().is_some());
    if let Some(kept) = event.kept_name() {
        writer.digest(kept.as_bytes());
    }
    Ok(())
}

fn encode_receipt(
    writer: &mut WireWriter,
    receipt: &RelationalCommitReceipt,
) -> Result<(), Refusal> {
    writer.u64(receipt.commit_id.0);
    writer.u64(receipt.version_id.0);
    writer.text("receipt branch id", &receipt.branch_id.0)?;
    writer.count("receipt parents", receipt.parents.len())?;
    for parent in &receipt.parents {
        writer.u64(parent.0);
    }
    Ok(())
}

fn encode_lineages(
    writer: &mut WireWriter,
    field: &'static str,
    lineages: &[LineageId],
) -> Result<(), Refusal> {
    writer.count(field, lineages.len())?;
    for lineage in lineages {
        writer.u64(lineage.0);
    }
    Ok(())
}

fn encode_birth(writer: &mut WireWriter, birth: &PersistentNameBirthRecord) -> Result<(), Refusal> {
    let parts = birth.parts();
    writer.digest(birth.identity().as_bytes());
    writer.u8(name_subject_tag(birth.subject_kind()));
    writer.text("birth scheme id", parts.scheme_id())?;
    writer.text("birth occurrence key", parts.occurrence_key())?;
    writer.text("birth role path", parts.role_path())?;
    writer.count("birth parents", parts.parent_identities().len())?;
    for parent in parts.parent_identities() {
        writer.digest(parent.as_bytes());
    }
    writer.bytes("birth discriminator", parts.canonical_discriminator())
}

fn encode_entity(writer: &mut WireWriter, entity: EntityId) {
    writer.u32(entity.partition_value());
    writer.u64(entity.local_slot_value());
    writer.u32(entity.generation_value());
}

fn encode_relation(writer: &mut WireWriter, relation: RelationId) {
    writer.u32(relation.partition_value());
    writer.u64(relation.local_slot_value());
    writer.u32(relation.generation_value());
}

fn encode_name_key(writer: &mut WireWriter, key: &PersistentNameKey) {
    writer.u8(name_subject_tag(key.subject_kind()));
    writer.digest(key.digest());
}
