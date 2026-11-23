//! Which committed lineage events allocate a fresh `LineageId`, and how the
//! entity-allocating ones bind to the entities their commit created.
//!
//! `Create` and `Restore` are the only allocating kinds: a `Create` starts a
//! lineage from nothing, and a `Restore` re-activates a retired persistent
//! name onto a fresh lineage. Entity and relation subjects draw from the one
//! lineage-id allocator, so recovery accounts for every allocation when it
//! checks or advances that allocator, while checkpoint lineage nodes bind
//! exactly the entity allocations.

use crate::history::data::{CanonicalCommitEnvelope, RelationalCommitReceipt};
use crate::identity::data::{EntityId, LineageId};
use crate::lineage::data::{LineageEventKind, LineageEventRecord, LineageSubject};
use crate::publication::patch::data::RecordStructuralChange;
use crate::transactions::data::RecordRef;

/// One fresh lineage allocated by a committed lineage event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct LineageAllocation {
    pub(super) subject: LineageSubject,
    pub(super) lineage: LineageId,
}

/// Why a commit's allocating lineage events do not fit their exact shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LineageAllocationRefusal {
    /// A `Create` must have no sources and exactly one target.
    MalformedCreate { event_id: u64 },
    /// A `Restore` must have one retired source and one different target.
    MalformedRestore { event_id: u64 },
    /// An allocating event names a commit other than its envelope's.
    ForeignCommit { event_id: u64 },
    /// An explicit entity binding on a relation allocation.
    MalformedEntityBinding { event_id: u64 },
    /// Entity allocations and created entities are not one-to-one.
    EntityAllocationCountMismatch {
        allocations: usize,
        created_entities: usize,
    },
}

impl LineageAllocationRefusal {
    pub(super) fn describe(&self, commit_id: u64) -> String {
        match self {
            Self::MalformedCreate { event_id } => format!(
                "commit {commit_id} create lineage event {event_id} must have no sources and exactly one target"
            ),
            Self::MalformedRestore { event_id } => format!(
                "commit {commit_id} restore lineage event {event_id} must have one source and one different target"
            ),
            Self::ForeignCommit { event_id } => format!(
                "commit {commit_id} allocating lineage event {event_id} names another commit"
            ),
            Self::MalformedEntityBinding { event_id } => format!(
                "commit {commit_id} relation lineage event {event_id} names an entity"
            ),
            Self::EntityAllocationCountMismatch {
                allocations,
                created_entities,
            } => format!(
                "commit {commit_id} binds {allocations} entity lineage allocations to {created_entities} created entities"
            ),
        }
    }
}

/// The fresh lineage `event` allocates, or `None` when its kind continues or
/// ends lineages without allocating one.
pub(super) fn allocated_lineage(
    event: &LineageEventRecord,
) -> Result<Option<LineageAllocation>, LineageAllocationRefusal> {
    let event_id = event.event_id();
    let lineage = match event.kind() {
        LineageEventKind::Create => match (event.sources(), event.targets()) {
            ([], &[lineage]) => lineage,
            _ => return Err(LineageAllocationRefusal::MalformedCreate { event_id }),
        },
        LineageEventKind::Restore => match (event.sources(), event.targets()) {
            (&[source], &[lineage]) if source != lineage => lineage,
            _ => return Err(LineageAllocationRefusal::MalformedRestore { event_id }),
        },
        LineageEventKind::Replace
        | LineageEventKind::Split
        | LineageEventKind::Merge
        | LineageEventKind::Retire => return Ok(None),
    };
    Ok(Some(LineageAllocation {
        subject: event.subject(),
        lineage,
    }))
}

/// Every lineage allocation of `events`, in event order.
///
/// Complexity: O(events).
pub(super) fn commit_allocations(
    events: &[LineageEventRecord],
) -> Result<Vec<LineageAllocation>, LineageAllocationRefusal> {
    let mut allocations = Vec::new();
    for event in events {
        if let Some(allocation) = allocated_lineage(event)? {
            allocations.push(allocation);
        }
    }
    Ok(allocations)
}

/// The highest lineage any `Create` or `Restore` of either subject in
/// `events` allocates; the lineage-id allocator must resume past it.
///
/// Complexity: O(events).
pub(super) fn maximum_allocated_lineage<'event>(
    events: impl IntoIterator<Item = &'event LineageEventRecord>,
) -> Option<u64> {
    events
        .into_iter()
        .filter_map(|event| allocated_lineage(event).ok().flatten())
        .map(|allocation| allocation.lineage.0)
        .max()
}

/// One commit's lineage events and the entities its patch created.
pub(crate) struct CommitAllocationView<'commit> {
    pub(crate) commit: &'commit RelationalCommitReceipt,
    pub(crate) events: &'commit [LineageEventRecord],
    pub(crate) created_entities: Vec<EntityId>,
}

impl<'commit> CommitAllocationView<'commit> {
    pub(crate) fn of_envelope(envelope: &'commit CanonicalCommitEnvelope) -> Self {
        let created_entities = envelope
            .patch
            .authoritative_record_patches
            .iter()
            .filter(|patch| patch.structural_change == RecordStructuralChange::Created)
            .filter_map(|patch| match patch.target {
                RecordRef::Entity(entity_id) => Some(entity_id),
                RecordRef::Relation(_) => None,
            })
            .collect();
        Self {
            commit: &envelope.commit,
            events: envelope.lineage_events(),
            created_entities,
        }
    }
}

/// Binds each entity allocation of one commit to the entity it created.
///
/// Commit finalization emits allocating events in ascending record-target
/// order, so the k-th entity allocation in event order binds the k-th
/// created entity in ascending `EntityId` order. This holds for `Create`
/// and `Restore` alike and never depends on patch order. An allocation onto
/// an entity the commit did not create (a selection restore at the original
/// identity) names its entity on the event and binds directly; it can never
/// also be one of the created entities.
///
/// Complexity: O(events + created entities log created entities).
pub(crate) fn entity_allocation_bindings(
    view: &CommitAllocationView<'_>,
) -> Result<Vec<(LineageId, EntityId)>, LineageAllocationRefusal> {
    let mut entity_lineages = Vec::new();
    let mut bound = Vec::new();
    for event in view.events {
        let Some(allocation) = allocated_lineage(event)? else {
            continue;
        };
        if event.commit() != view.commit {
            return Err(LineageAllocationRefusal::ForeignCommit {
                event_id: event.event_id(),
            });
        }
        match (allocation.subject, event.entity_id()) {
            (LineageSubject::Entity, Some(entity_id)) => {
                bound.push((allocation.lineage, entity_id))
            }
            (LineageSubject::Entity, None) => entity_lineages.push(allocation.lineage),
            (LineageSubject::Relation, Some(_)) => {
                return Err(LineageAllocationRefusal::MalformedEntityBinding {
                    event_id: event.event_id(),
                })
            }
            (LineageSubject::Relation, None) => {}
        }
    }
    let mut created_entities = view.created_entities.clone();
    created_entities.sort_unstable();
    if entity_lineages.len() != created_entities.len() {
        return Err(LineageAllocationRefusal::EntityAllocationCountMismatch {
            allocations: entity_lineages.len(),
            created_entities: created_entities.len(),
        });
    }
    if bound
        .iter()
        .any(|(_, entity)| created_entities.binary_search(entity).is_ok())
    {
        return Err(LineageAllocationRefusal::EntityAllocationCountMismatch {
            allocations: entity_lineages.len() + bound.len(),
            created_entities: created_entities.len(),
        });
    }
    let mut bindings: Vec<_> = entity_lineages.into_iter().zip(created_entities).collect();
    bindings.extend(bound);
    Ok(bindings)
}
