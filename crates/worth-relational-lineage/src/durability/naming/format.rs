//! Format identity and closed tag tables of the durable persistent-name log.
//!
//! Tag zero is never assigned, so zeroed bytes never decode as a variant.

use worth_foundational::facade::AggregateIdentity;

use crate::contracts::aggregate_identity::group_lineage::AggregateGroupLineageEvent;
use crate::contracts::lineage::data::events::{LineageEventKind, LineageSubject};
use crate::contracts::lineage::data::naming::PersistentNameSubjectKind;

use super::refusal::PersistentNameDurabilityError as Refusal;

/// The only format version this build writes and reads.
pub const FORMAT_VERSION: u8 = 1;

pub(super) const LOG_MAGIC: &[u8; 4] = b"WPNL";
pub(super) const FRAME_MAGIC: &[u8; 4] = b"WPNF";

pub(super) const fn kind_tag(kind: LineageEventKind) -> u8 {
    match kind {
        LineageEventKind::Create => 1,
        LineageEventKind::Replace => 2,
        LineageEventKind::Split => 3,
        LineageEventKind::Merge => 4,
        LineageEventKind::Retire => 5,
        LineageEventKind::Restore => 6,
    }
}

pub(super) fn kind_from_tag(tag: u8) -> Result<LineageEventKind, Refusal> {
    Ok(match tag {
        1 => LineageEventKind::Create,
        2 => LineageEventKind::Replace,
        3 => LineageEventKind::Split,
        4 => LineageEventKind::Merge,
        5 => LineageEventKind::Retire,
        6 => LineageEventKind::Restore,
        tag => {
            return Err(Refusal::InvalidTag {
                field: "event kind",
                tag,
            })
        }
    })
}

pub(super) const fn subject_tag(subject: LineageSubject) -> u8 {
    match subject {
        LineageSubject::Entity => 1,
        LineageSubject::Relation => 2,
    }
}

pub(super) fn subject_from_tag(tag: u8) -> Result<LineageSubject, Refusal> {
    match tag {
        1 => Ok(LineageSubject::Entity),
        2 => Ok(LineageSubject::Relation),
        tag => Err(Refusal::InvalidTag {
            field: "event subject",
            tag,
        }),
    }
}

pub(super) const fn name_subject_tag(subject: PersistentNameSubjectKind) -> u8 {
    match subject {
        PersistentNameSubjectKind::Entity => 1,
        PersistentNameSubjectKind::Relation => 2,
        PersistentNameSubjectKind::Group => 3,
    }
}

pub(super) fn name_subject_from_tag(
    field: &'static str,
    tag: u8,
) -> Result<PersistentNameSubjectKind, Refusal> {
    match tag {
        1 => Ok(PersistentNameSubjectKind::Entity),
        2 => Ok(PersistentNameSubjectKind::Relation),
        3 => Ok(PersistentNameSubjectKind::Group),
        tag => Err(Refusal::InvalidTag { field, tag }),
    }
}

/// A group lineage event as its tag and the one or two groups it names.
pub(super) const fn group_event_parts(
    event: AggregateGroupLineageEvent,
) -> (u8, AggregateIdentity, Option<AggregateIdentity>) {
    match event {
        AggregateGroupLineageEvent::Born { group } => (1, group, None),
        AggregateGroupLineageEvent::Revived { group } => (2, group, None),
        AggregateGroupLineageEvent::SplitFrom { fragment, source } => (3, fragment, Some(source)),
        AggregateGroupLineageEvent::MergedInto { absorbed, keeper } => (4, absorbed, Some(keeper)),
        AggregateGroupLineageEvent::Retired { group } => (5, group, None),
    }
}

/// How many groups a group-event tag names, or a refusal for unknown tags.
pub(super) fn group_event_arity(tag: u8) -> Result<usize, Refusal> {
    match tag {
        1 | 2 | 5 => Ok(1),
        3 | 4 => Ok(2),
        tag => Err(Refusal::InvalidTag {
            field: "group event",
            tag,
        }),
    }
}

pub(super) fn group_event_from_parts(
    tag: u8,
    first: AggregateIdentity,
    second: Option<AggregateIdentity>,
) -> Result<AggregateGroupLineageEvent, Refusal> {
    Ok(match (tag, second) {
        (1, None) => AggregateGroupLineageEvent::Born { group: first },
        (2, None) => AggregateGroupLineageEvent::Revived { group: first },
        (3, Some(source)) => AggregateGroupLineageEvent::SplitFrom {
            fragment: first,
            source,
        },
        (4, Some(keeper)) => AggregateGroupLineageEvent::MergedInto {
            absorbed: first,
            keeper,
        },
        (5, None) => AggregateGroupLineageEvent::Retired { group: first },
        (tag, _) => {
            return Err(Refusal::InvalidTag {
                field: "group event",
                tag,
            })
        }
    })
}
