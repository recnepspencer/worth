//! The canonical shape of one persisted lineage event, checked before any of
//! its naming payload is readmitted.
//!
//! Each kind has exactly one source/target arity; the receipt names the
//! event's own branch; births belong only to `Create`; kept names only to
//! the kinds that carry one name across a record change. A tampered frame
//! that bends any of these refuses typed instead of reaching the index,
//! where a `Retire` carrying targets would otherwise record forward edges.

use std::collections::BTreeSet;

use worth_relational_contracts::internal_port::identity::data::primitives::LineageId;
use worth_relational_contracts::internal_port::lineage::data::events::LineageEventKind;

use super::frame_decoding::DecodedLineageEvent;
use super::refusal::PersistentNameDurabilityError as Refusal;

/// Refuses `event` unless its arity, branch, and naming payload are canonical.
///
/// Complexity: O(sources + targets).
pub(super) fn require_canonical_event_shape(event: &DecodedLineageEvent) -> Result<(), Refusal> {
    let refusal = |detail| Refusal::InvalidEventShape {
        event_id: event.event_id,
        detail,
    };
    if let Some(detail) = arity_violation(event.kind, &event.sources, &event.targets) {
        return Err(refusal(detail));
    }
    if event.commit.branch_id != event.branch_id {
        return Err(refusal(
            "the receipt names a different branch than the event",
        ));
    }
    if event.birth.is_some() && event.kind != LineageEventKind::Create {
        return Err(refusal("only a Create can carry a name birth"));
    }
    let keeps_name = matches!(
        event.kind,
        LineageEventKind::Create
            | LineageEventKind::Replace
            | LineageEventKind::Merge
            | LineageEventKind::Restore
    );
    if event.kept_name.is_some() && !keeps_name {
        return Err(refusal(
            "only Create, Replace, Merge, or Restore can keep a name",
        ));
    }
    Ok(())
}

/// The one arity each kind may have, or why `sources`/`targets` break it.
///
/// A `Merge` keeping its keeper in place lists the keeper's lineage among
/// both its sources and its target, so merges never require disjointness.
fn arity_violation(
    kind: LineageEventKind,
    sources: &[LineageId],
    targets: &[LineageId],
) -> Option<&'static str> {
    match kind {
        LineageEventKind::Create => (!(sources.is_empty() && targets.len() == 1))
            .then_some("a Create has no sources and exactly one target"),
        LineageEventKind::Retire => (!(sources.len() == 1 && targets.is_empty()))
            .then_some("a Retire has exactly one source and no targets"),
        LineageEventKind::Restore | LineageEventKind::Replace => match (sources, targets) {
            (&[source], &[target]) if source != target => None,
            _ => Some("a Restore or Replace has one source and one different target"),
        },
        LineageEventKind::Split => match sources {
            &[source] if targets.len() >= 2 && distinct(targets) && !targets.contains(&source) => {
                None
            }
            _ => Some("a Split has one source and at least two distinct other targets"),
        },
        LineageEventKind::Merge => {
            (!(sources.len() >= 2 && distinct(sources) && targets.len() == 1))
                .then_some("a Merge has at least two distinct sources and exactly one target")
        }
    }
}

fn distinct(lineages: &[LineageId]) -> bool {
    lineages.iter().collect::<BTreeSet<_>>().len() == lineages.len()
}
