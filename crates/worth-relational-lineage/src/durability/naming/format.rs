//! Closed tag tables of the durable group lineage frame.
//!
//! Tag zero is never assigned, so zeroed bytes never decode as a variant.

use worth_foundational::facade::AggregateIdentity;

use crate::contracts::aggregate_identity::group_lineage::AggregateGroupLineageEvent;

use super::refusal::PersistentNameDurabilityError as Refusal;

/// A group lineage event as its tag and the one or two groups it names.
pub(crate) const fn group_event_parts(
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
pub(crate) fn group_event_arity(tag: u8) -> Result<usize, Refusal> {
    match tag {
        1 | 2 | 5 => Ok(1),
        3 | 4 => Ok(2),
        tag => Err(Refusal::InvalidTag {
            field: "group event",
            tag,
        }),
    }
}

pub(crate) fn group_event_from_parts(
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
