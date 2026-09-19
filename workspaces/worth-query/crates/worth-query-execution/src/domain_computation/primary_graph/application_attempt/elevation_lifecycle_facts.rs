use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_relational::facade::identity::{EntityId, KindId};

use super::{WorthQueryApplicationAdjacencyDirection, WorthQueryApplicationObservedFact};

#[derive(Clone, Copy)]
pub(super) enum WorthQueryExpectedLifecycleRelation {
    Absent,
    Present(EntityId),
}

pub(super) struct WorthQueryElevationLifecycleFactExpectation<'a> {
    pub(super) elevation: EntityId,
    pub(super) review: EntityId,
    pub(super) resource: EntityId,
    pub(super) requester: EntityId,
    pub(super) approver: WorthQueryExpectedLifecycleRelation,
    pub(super) grant: EntityId,
    pub(super) reviewer: WorthQueryExpectedLifecycleRelation,
    pub(super) elevation_identity: (&'a AspectFieldLocator, &'a AspectValue),
    pub(super) reason: (&'a AspectFieldLocator, &'a AspectValue),
    pub(super) status: (&'a AspectFieldLocator, &'a AspectValue),
    pub(super) not_before: (&'a AspectFieldLocator, &'a AspectValue),
    pub(super) not_after: (&'a AspectFieldLocator, &'a AspectValue),
    pub(super) review_identity: (&'a AspectFieldLocator, &'a AspectValue),
    pub(super) review_type: (&'a AspectFieldLocator, &'a AspectValue),
    pub(super) review_status: (&'a AspectFieldLocator, &'a AspectValue),
    pub(super) absent_fields: &'a [(EntityId, &'a AspectFieldLocator)],
    pub(super) requester_relation: KindId,
    pub(super) approver_relation: KindId,
    pub(super) grant_relation: KindId,
    pub(super) resource_relation: Option<KindId>,
    pub(super) review_relation: KindId,
    pub(super) review_scope_relation: KindId,
    pub(super) reviewer_relation: KindId,
}

pub(super) fn lifecycle_facts_are_exact(
    facts: &[WorthQueryApplicationObservedFact],
    expected: WorthQueryElevationLifecycleFactExpectation<'_>,
) -> bool {
    let fields = [
        (expected.elevation, expected.elevation_identity),
        (expected.elevation, expected.reason),
        (expected.elevation, expected.status),
        (expected.elevation, expected.not_before),
        (expected.elevation, expected.not_after),
        (expected.review, expected.review_identity),
        (expected.review, expected.review_type),
        (expected.review, expected.review_status),
    ];
    facts.len()
        == 14 + usize::from(expected.resource_relation.is_some()) + expected.absent_fields.len()
        && fields
            .into_iter()
            .all(|(entity, (locator, value))| exact_field(facts, entity, locator, value))
        && expected
            .absent_fields
            .iter()
            .all(|(entity, locator)| exact_absent_field(facts, *entity, locator))
        && exact_relation_set(
            facts,
            expected.requester_relation,
            expected.elevation,
            WorthQueryApplicationAdjacencyDirection::Incoming,
            WorthQueryExpectedLifecycleRelation::Present(expected.requester),
        )
        && exact_relation_set(
            facts,
            expected.approver_relation,
            expected.elevation,
            WorthQueryApplicationAdjacencyDirection::Incoming,
            expected.approver,
        )
        && exact_relation_set(
            facts,
            expected.grant_relation,
            expected.elevation,
            WorthQueryApplicationAdjacencyDirection::Outgoing,
            WorthQueryExpectedLifecycleRelation::Present(expected.grant),
        )
        && expected.resource_relation.is_none_or(|relation| {
            exact_relation_set(
                facts,
                relation,
                expected.elevation,
                WorthQueryApplicationAdjacencyDirection::Outgoing,
                WorthQueryExpectedLifecycleRelation::Present(expected.resource),
            )
        })
        && exact_relation_set(
            facts,
            expected.review_relation,
            expected.elevation,
            WorthQueryApplicationAdjacencyDirection::Outgoing,
            WorthQueryExpectedLifecycleRelation::Present(expected.review),
        )
        && exact_relation_set(
            facts,
            expected.review_scope_relation,
            expected.review,
            WorthQueryApplicationAdjacencyDirection::Outgoing,
            WorthQueryExpectedLifecycleRelation::Present(expected.resource),
        )
        && exact_relation_set(
            facts,
            expected.reviewer_relation,
            expected.review,
            WorthQueryApplicationAdjacencyDirection::Incoming,
            expected.reviewer,
        )
}

fn exact_absent_field(
    facts: &[WorthQueryApplicationObservedFact],
    entity: EntityId,
    locator: &AspectFieldLocator,
) -> bool {
    facts
        .iter()
        .filter(|fact| {
            matches!(fact, WorthQueryApplicationObservedFact::AbsentField {
                entity_id,
                locator: observed_locator,
                ..
            } if *entity_id == entity && observed_locator == locator)
        })
        .count()
        == 1
}

fn exact_relation_set(
    facts: &[WorthQueryApplicationObservedFact],
    kind: KindId,
    anchor: EntityId,
    direction: WorthQueryApplicationAdjacencyDirection,
    expected: WorthQueryExpectedLifecycleRelation,
) -> bool {
    facts
        .iter()
        .filter(|fact| {
            matches!(
                fact,
                WorthQueryApplicationObservedFact::Adjacency {
                    relation_kind,
                    anchor: observed_anchor,
                    direction: observed_direction,
                    relations,
                    ..
                } if *relation_kind == kind
                    && *observed_anchor == anchor
                    && *observed_direction == direction
                    && relation_set_matches(relations, anchor, direction, expected)
            )
        })
        .count()
        == 1
}

fn exact_field(
    facts: &[WorthQueryApplicationObservedFact],
    entity: EntityId,
    locator: &AspectFieldLocator,
    value: &AspectValue,
) -> bool {
    facts
        .iter()
        .filter(|fact| {
            matches!(
                fact,
                WorthQueryApplicationObservedFact::Field {
                    entity_id,
                    locator: observed_locator,
                    value: observed_value,
                    ..
                } if *entity_id == entity && observed_locator == locator && observed_value == value
            )
        })
        .count()
        == 1
}

fn relation_set_matches(
    relations: &[super::fact::WorthQueryApplicationObservedRelation],
    anchor: EntityId,
    direction: WorthQueryApplicationAdjacencyDirection,
    expected: WorthQueryExpectedLifecycleRelation,
) -> bool {
    match expected {
        WorthQueryExpectedLifecycleRelation::Absent => relations.is_empty(),
        WorthQueryExpectedLifecycleRelation::Present(endpoint) => {
            relations.len() == 1
                && relations.iter().all(|relation| match direction {
                    WorthQueryApplicationAdjacencyDirection::Incoming => {
                        relation.from == endpoint && relation.to == anchor
                    }
                    WorthQueryApplicationAdjacencyDirection::Outgoing => {
                        relation.from == anchor && relation.to == endpoint
                    }
                })
        }
    }
}
