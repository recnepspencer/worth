//! Structural decoding of the durable persistent-name log.
//!
//! Decoding checks format identity, completeness, tags and canonical order,
//! and yields raw persisted claims. It never produces a persistent-name
//! identity: births and references stay raw bytes until readmission.

use std::collections::BTreeMap;

use worth_foundational::facade::AggregateIdentity;

use crate::contracts::aggregate_identity::group_lineage::AggregateGroupLineageEvent;
use crate::contracts::history::data::commit_receipt::RelationalCommitReceipt;
use crate::contracts::history::data::coordinates::{BranchId, CommitId};
use crate::contracts::identity::data::primitives::{LineageId, PartitionId, VersionId};
use crate::contracts::identity::data::record_ids::{EntityId, RelationId};
use crate::contracts::lineage::data::events::{LineageEventKind, LineageSubject};
use crate::contracts::lineage::data::naming::PersistentNameSubjectKind;
use crate::lineage::authority::{PersistedBirthClaim, PersistedNameBytes};

use super::format::{
    group_event_arity, group_event_from_parts, kind_from_tag, name_subject_from_tag,
    subject_from_tag, FORMAT_VERSION, FRAME_MAGIC, LOG_MAGIC,
};
use super::refusal::{PersistentNameDurabilityError as Refusal, PersistentNameWireSection};
use super::wire::{read_strictly_ascending, WireReader};

/// One persisted lineage event, with its naming payload still raw.
pub(super) struct DecodedLineageEvent {
    pub(super) event_id: u64,
    pub(super) commit: RelationalCommitReceipt,
    pub(super) branch_id: BranchId,
    pub(super) kind: LineageEventKind,
    pub(super) subject: LineageSubject,
    pub(super) sources: Vec<LineageId>,
    pub(super) targets: Vec<LineageId>,
    pub(super) birth: Option<PersistedBirthClaim>,
    pub(super) kept_name: Option<PersistedNameBytes>,
}

/// One persisted carrier declaration, with its names still raw.
pub(super) struct DecodedCarriers {
    pub(super) subject: PersistentNameSubjectKind,
    pub(super) key: PersistedNameBytes,
    pub(super) carriers: Vec<PersistedNameBytes>,
}

/// One structurally valid persisted commit frame.
pub(super) struct DecodedCommitFrame {
    pub(super) commit: CommitId,
    pub(super) min_retained_commit: Option<CommitId>,
    pub(super) events: Vec<DecodedLineageEvent>,
    pub(super) entity_holders: BTreeMap<LineageId, EntityId>,
    pub(super) relation_holders: BTreeMap<LineageId, RelationId>,
    pub(super) group_events: Vec<AggregateGroupLineageEvent>,
    pub(super) group_schemes: BTreeMap<AggregateIdentity, String>,
    pub(super) carriers: Vec<DecodedCarriers>,
}

/// Reads the log header; returns its branch and the frame bytes after it.
pub(super) fn decode_log_header(bytes: &[u8]) -> Result<(BranchId, &[u8]), Refusal> {
    let mut reader = WireReader::new(bytes, PersistentNameWireSection::LogHeader);
    read_format_identity(&mut reader, LOG_MAGIC, PersistentNameWireSection::LogHeader)?;
    let branch_id = BranchId(reader.text("branch id")?);
    Ok((branch_id, reader.remaining()))
}

/// Splits the next frame from `bytes`; returns its payload and the rest.
pub(super) fn split_commit_frame(bytes: &[u8]) -> Result<(&[u8], &[u8]), Refusal> {
    let mut reader = WireReader::new(bytes, PersistentNameWireSection::FrameHeader);
    read_format_identity(
        &mut reader,
        FRAME_MAGIC,
        PersistentNameWireSection::FrameHeader,
    )?;
    let len = reader.count()?;
    let mut payload = WireReader::new(reader.remaining(), PersistentNameWireSection::FramePayload);
    let frame = payload.take(len)?;
    Ok((frame, payload.remaining()))
}

fn read_format_identity(
    reader: &mut WireReader<'_>,
    magic: &[u8; 4],
    section: PersistentNameWireSection,
) -> Result<(), Refusal> {
    if reader.take(magic.len())? != magic {
        return Err(Refusal::UnrecognizedMagic { section });
    }
    let found = reader.u8()?;
    if found != FORMAT_VERSION {
        return Err(Refusal::UnsupportedFormatVersion {
            found,
            supported: FORMAT_VERSION,
        });
    }
    Ok(())
}

/// Decodes one frame payload, requiring it to be complete and canonical.
///
/// Complexity: O(payload bytes).
pub(super) fn decode_commit_frame(payload: &[u8]) -> Result<DecodedCommitFrame, Refusal> {
    let mut reader = WireReader::new(payload, PersistentNameWireSection::FramePayload);
    let commit = CommitId(reader.u64()?);
    let min_retained_commit = if reader.option_tag("retention floor")? {
        Some(CommitId(reader.u64()?))
    } else {
        None
    };
    if min_retained_commit.is_some_and(|floor| floor > commit) {
        return Err(Refusal::NonCanonical {
            field: "retention floor",
        });
    }
    let events = decode_events(&mut reader)?;
    let entity_holders = read_strictly_ascending(
        &mut reader,
        "entity holders",
        |reader| Ok((LineageId(reader.u64()?), decode_entity(reader)?)),
        |(lineage, _)| *lineage,
    )?
    .into_iter()
    .collect();
    let relation_holders = read_strictly_ascending(
        &mut reader,
        "relation holders",
        |reader| Ok((LineageId(reader.u64()?), decode_relation(reader)?)),
        |(lineage, _)| *lineage,
    )?
    .into_iter()
    .collect();
    let group_events =
        read_strictly_ascending(&mut reader, "group events", decode_group_event, |event| {
            *event
        })?;
    let group_schemes = read_strictly_ascending(
        &mut reader,
        "group schemes",
        |reader| {
            let group = AggregateIdentity::from_bytes(reader.digest()?);
            Ok((group, reader.text("group scheme id")?))
        },
        |(group, _)| *group,
    )?
    .into_iter()
    .collect();
    let carriers = read_strictly_ascending(
        &mut reader,
        "carrier declarations",
        decode_carriers,
        |declaration| (declaration.subject, declaration.key),
    )?;
    reader.finish()?;
    Ok(DecodedCommitFrame {
        commit,
        min_retained_commit,
        events,
        entity_holders,
        relation_holders,
        group_events,
        group_schemes,
        carriers,
    })
}

/// Carrier keys order as `PersistentNameKey` does: by subject, then digest.
fn decode_carriers(reader: &mut WireReader<'_>) -> Result<DecodedCarriers, Refusal> {
    let subject = name_subject_from_tag("carrier key subject", reader.u8()?)?;
    let key = reader.digest()?;
    let carriers =
        read_strictly_ascending(reader, "carriers", |reader| reader.digest(), |raw| *raw)?;
    Ok(DecodedCarriers {
        subject,
        key,
        carriers,
    })
}

fn decode_events(reader: &mut WireReader<'_>) -> Result<Vec<DecodedLineageEvent>, Refusal> {
    let count = reader.count()?;
    let mut events: Vec<DecodedLineageEvent> = Vec::new();
    for _ in 0..count {
        let event = decode_event(reader)?;
        if events
            .last()
            .is_some_and(|previous| previous.event_id >= event.event_id)
        {
            return Err(Refusal::NonCanonical {
                field: "lineage event order",
            });
        }
        events.push(event);
    }
    Ok(events)
}

fn decode_event(reader: &mut WireReader<'_>) -> Result<DecodedLineageEvent, Refusal> {
    let event_id = reader.u64()?;
    let commit = decode_receipt(reader)?;
    let branch_id = BranchId(reader.text("event branch id")?);
    let kind = kind_from_tag(reader.u8()?)?;
    let subject = subject_from_tag(reader.u8()?)?;
    let sources = decode_lineages(reader)?;
    let targets = decode_lineages(reader)?;
    let birth = if reader.option_tag("name birth")? {
        Some(decode_birth(reader)?)
    } else {
        None
    };
    let kept_name = if reader.option_tag("kept name")? {
        Some(reader.digest()?)
    } else {
        None
    };
    Ok(DecodedLineageEvent {
        event_id,
        commit,
        branch_id,
        kind,
        subject,
        sources,
        targets,
        birth,
        kept_name,
    })
}

fn decode_receipt(reader: &mut WireReader<'_>) -> Result<RelationalCommitReceipt, Refusal> {
    let commit_id = CommitId(reader.u64()?);
    let version_id = VersionId(reader.u64()?);
    let branch_id = BranchId(reader.text("receipt branch id")?);
    let count = reader.count()?;
    let mut parents = Vec::new();
    for _ in 0..count {
        parents.push(CommitId(reader.u64()?));
    }
    Ok(RelationalCommitReceipt {
        commit_id,
        version_id,
        branch_id,
        parents,
    })
}

fn decode_lineages(reader: &mut WireReader<'_>) -> Result<Vec<LineageId>, Refusal> {
    let count = reader.count()?;
    let mut lineages = Vec::new();
    for _ in 0..count {
        lineages.push(LineageId(reader.u64()?));
    }
    Ok(lineages)
}

/// Reads a birth's typed parts exactly as stored; parents keep their stored
/// order so readmission can refuse an unsorted birth.
fn decode_birth(reader: &mut WireReader<'_>) -> Result<PersistedBirthClaim, Refusal> {
    let stored_identity = reader.digest()?;
    let subject_kind = name_subject_from_tag("birth subject", reader.u8()?)?;
    let scheme_id = reader.text("birth scheme id")?;
    let occurrence_key = reader.text("birth occurrence key")?;
    let role_path = reader.text("birth role path")?;
    let count = reader.count()?;
    let mut parent_identities = Vec::new();
    for _ in 0..count {
        parent_identities.push(reader.digest()?);
    }
    let canonical_discriminator = reader.bytes()?;
    Ok(PersistedBirthClaim {
        stored_identity,
        subject_kind,
        scheme_id,
        occurrence_key,
        role_path,
        parent_identities,
        canonical_discriminator,
    })
}

pub(super) fn decode_entity(reader: &mut WireReader<'_>) -> Result<EntityId, Refusal> {
    let (partition, slot, generation) = (reader.u32()?, reader.u64()?, reader.u32()?);
    Ok(EntityId::new(PartitionId(partition), slot, generation))
}

fn decode_relation(reader: &mut WireReader<'_>) -> Result<RelationId, Refusal> {
    let (partition, slot, generation) = (reader.u32()?, reader.u64()?, reader.u32()?);
    Ok(RelationId::new(PartitionId(partition), slot, generation))
}

pub(super) fn decode_group_event(
    reader: &mut WireReader<'_>,
) -> Result<AggregateGroupLineageEvent, Refusal> {
    let tag = reader.u8()?;
    let arity = group_event_arity(tag)?;
    let first = AggregateIdentity::from_bytes(reader.digest()?);
    let second = if arity == 2 {
        Some(AggregateIdentity::from_bytes(reader.digest()?))
    } else {
        None
    };
    group_event_from_parts(tag, first, second)
}
