//! The Native touch vocabulary that can change one fact's support (PR C §1.4).
//!
//! Native certifies each candidate append's touches as kind, aspect field
//! locator or relation kind, never a record. A fact's keys name every touch
//! that could change its comparison; a fact Native cannot key that way is
//! conservative and every write marks it. Keys only select which derivations
//! are compared exactly: a key never accepts a change.
use super::WorthQueryApplicationObservedFact as Fact;
use worth_foundational::facade::AspectKey;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::mvcc::ValidatedMutationTouch as Touch;

/// One key a fact's support can be changed through.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::domain_computation::primary_graph) enum FactTouchKey {
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
pub(in crate::domain_computation::primary_graph) fn fact_keys(
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
pub(in crate::domain_computation::primary_graph) enum TouchProbe {
    /// Not a write: the record is exactly what it was.
    Nothing,
    /// These keys, plus every conservative fact.
    Keys(Vec<FactTouchKey>),
    /// Every fact (deletion, unrepresentable mutation).
    Everything,
}

pub(in crate::domain_computation::primary_graph) fn touch_probe(touch: &Touch) -> TouchProbe {
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
