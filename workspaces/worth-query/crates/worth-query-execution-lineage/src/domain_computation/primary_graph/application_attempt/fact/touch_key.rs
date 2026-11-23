//! The Native touch vocabulary that can change one fact's support (PR C §1.4).
//!
//! Native certifies each candidate append's touches as kind, aspect field
//! locator or relation kind, never a record. A fact's keys name every touch
//! that could change its comparison; a fact Native cannot key that way is
//! conservative and every write marks it. Keys only select which derivations
//! are compared exactly: a key never accepts a change.
use super::WorthQueryApplicationFactKey as Key;
use super::WorthQueryApplicationObservedFact as Fact;
use worth_foundational::facade::AspectKey;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::mvcc::ValidatedMutationTouch as Touch;

/// One key a fact's support can be changed through.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum FactTouchKey {
    /// Presence of an entity of this kind (creation, selection membership).
    Kind(KindId),
    /// Any field of this aspect on an entity of this kind.
    Aspect(KindId, AspectKey),
    /// Any field of this aspect, on an entity whose kind the fact does not hold.
    AnyKindAspect(AspectKey),
    /// Presence of an entity whose kind the fact does not hold.
    AnyKindPresence,
    /// Edges of this relation kind.
    Relation(KindId),
}

/// The keys of one fact, or `None` when it is conservative.
pub fn fact_keys(
    fact: &Fact,
) -> Option<Vec<FactTouchKey>> {
    use FactTouchKey::*;
    Some(match fact {
        Fact::Field { kind, locator, .. }
        | Fact::AbsentField { kind, locator, .. }
        | Fact::OptionalAspectAbsent { kind, locator, .. } => {
            vec![Aspect(*kind, locator.aspect().aspect_key().clone())]
        }
        Fact::AspectPresence { kind, aspect, .. } => vec![Aspect(*kind, aspect.clone())],
        Fact::Entity { kind, .. } | Fact::RetiredOutputEntity { kind, .. } => vec![Kind(*kind)],
        Fact::IndexedEntitySelection {
            entity_kind,
            locator,
            ..
        } => vec![
            Kind(*entity_kind),
            Aspect(*entity_kind, locator.aspect().aspect_key().clone()),
        ],
        Fact::Relation { relation_kind, .. }
        | Fact::Adjacency { relation_kind, .. }
        | Fact::SourceAdjacencyRevision { relation_kind, .. } => vec![Relation(*relation_kind)],
        Fact::SourceFieldRevision { locator, .. } => {
            vec![AnyKindAspect(locator.aspect().aspect_key().clone())]
        }
        Fact::SourceAspectRevision { aspect, .. } => vec![AnyKindAspect(aspect.clone())],
        Fact::SourceEntity { .. } => vec![AnyKindPresence],
        // Observations over many records, managed sources, publications,
        // authorization and workflow state: Native certifies no narrower key.
        _ => return None,
    })
}

/// What one certified touch marks.
pub enum TouchProbe {
    /// Not a write: the record is exactly what it was.
    Nothing,
    /// These keys, plus every conservative fact.
    Keys(Vec<FactTouchKey>),
    /// Every fact (deletion, unrepresentable mutation).
    Everything,
}

pub fn touch_probe(touch: &Touch) -> TouchProbe {
    use FactTouchKey::*;
    match touch {
        Touch::RevalidateEntity { .. } | Touch::RevalidateRelation { .. } => TouchProbe::Nothing,
        Touch::WriteEntityField { kind, locator } => {
            let aspect = locator.aspect().aspect_key().clone();
            TouchProbe::Keys(vec![Aspect(*kind, aspect.clone()), AnyKindAspect(aspect)])
        }
        Touch::CreateEntity { kind } => TouchProbe::Keys(vec![Kind(*kind), AnyKindPresence]),
        Touch::LinkRelation { kind }
        | Touch::UnlinkRelation { kind }
        | Touch::InitializeRelationField { kind, .. } => TouchProbe::Keys(vec![Relation(*kind)]),
        Touch::DeleteEntity { .. }
        | Touch::UnrepresentableEntityMutation { .. }
        | Touch::UnrepresentableRelationMutation { .. } => TouchProbe::Everything,
    }
}

/// The one record a single-record fact reads, if it reads exactly one.
pub fn fact_record(
    fact: &Fact,
) -> Option<worth_relational::facade::identity::EntityId> {
    match fact {
        Fact::Entity { entity_id, .. }
        | Fact::Field { entity_id, .. }
        | Fact::AbsentField { entity_id, .. }
        | Fact::OptionalAspectAbsent { entity_id, .. }
        | Fact::AspectPresence { entity_id, .. }
        | Fact::RetiredOutputEntity { entity_id, .. }
        | Fact::SourceEntity { entity_id }
        | Fact::SourceAspectRevision { entity_id, .. }
        | Fact::SourceFieldRevision { entity_id, .. } => Some(*entity_id),
        _ => None,
    }
}

/// Whether a successor round's own-output view serves `fact` (§1.5): a
/// field, aspect-presence or presence read of one record, which it reads at
/// the chain head's pre-pin when the chain wrote that record (#277). Every
/// other single-record fact (source revisions, a retired output) is read at
/// the current image. `head_served_record` states the same rule for keys.
pub fn overlay_served(fact: &Fact) -> bool {
    matches!(
        fact,
        Fact::Field { .. }
            | Fact::AbsentField { .. }
            | Fact::OptionalAspectAbsent { .. }
            | Fact::AspectPresence { .. }
            | Fact::Entity { .. }
    )
}

/// The one record `key` reads, when a successor round's own-output view
/// serves that key at the chain head's pre-pin for a record its chain wrote:
/// a field, an aspect's presence, or the record's own presence (#277). The
/// read set's completion and a carried computation read both ask here, so a
/// covered record is never seen with its fields at the head and its presence
/// at the current image.
pub fn head_served_record(key: &Key) -> Option<worth_relational::facade::identity::EntityId> {
    match key {
        Key::Field { entity_id, .. }
        | Key::AspectPresence { entity_id, .. }
        | Key::Entity { entity_id, .. } => Some(*entity_id),
        _ => None,
    }
}

/// Whether the own-output view serves `fact` as a relation list (#277): the
/// current list minus the chain's own links plus the links it removed. Every
/// relation or adjacency fact a successor records is served or untouched by
/// the chain, so its links and unlinks cannot have changed what it read.
pub fn relation_served(fact: &Fact) -> bool {
    matches!(fact, Fact::Relation { .. } | Fact::Adjacency { .. })
}

/// Whether a certified touch could change what `fact` read. Field reads and
/// indexed selections compare the field path; every other fact compares
/// its keys, and a conservative fact always could.
pub fn touch_changes(
    touch: &Touch,
    fact: &Fact,
) -> bool {
    let overlaps = |left: &worth_foundational::facade::AspectFieldLocator,
                    right: &worth_foundational::facade::AspectFieldLocator| {
        left.aspect().aspect_key() == right.aspect().aspect_key() && {
            let (a, b) = (left.field_path().fields(), right.field_path().fields());
            a.starts_with(b) || b.starts_with(a)
        }
    };
    match (touch, fact) {
        (
            Touch::WriteEntityField { kind, locator },
            Fact::Field {
                kind: read,
                locator: field,
                ..
            }
            | Fact::AbsentField {
                kind: read,
                locator: field,
                ..
            }
            | Fact::IndexedEntitySelection {
                entity_kind: read,
                locator: field,
                ..
            },
        ) => kind == read && overlaps(locator, field),
        _ => match (touch_probe(touch), fact_keys(fact)) {
            (TouchProbe::Nothing, _) => false,
            (TouchProbe::Everything, _) | (TouchProbe::Keys(_), None) => true,
            (TouchProbe::Keys(touched), Some(read)) => touched.iter().any(|key| read.contains(key)),
        },
    }
}
