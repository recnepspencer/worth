use std::mem::size_of;

use worth_query_declaration::facade::application_query::ApplicationQueryCardinality;
use worth_query_installation::facade::{
    WorthQueryAdmittedReadGraphPlanningInventory, WorthQueryInstalledGraphReadContract,
    WorthQueryReadGraphPlanningContract, WorthQueryReadGraphRelationDirection,
};

use super::{copied_bytes, inventory_stop, reserve_vec};
use crate::application_query::requirements::{
    fanout_posture, result_pressure, OrderingPostureScan,
};
use crate::application_query::WorthQueryApplicationQueryLane;
use crate::canonical_identity_derivation::WorthQueryCanonicalIdentityStop;
use crate::graph_read_access::{
    WorthQueryAdmittedGraphReadRelationDirection, WorthQueryGraphReadPlanningOrderingField,
    WorthQueryGraphReadPlanningPredicateField, WorthQueryGraphReadPlanningRelation,
    WorthQueryGraphReadPlanningShape, WorthQueryGraphReadPredicateFamily,
    WorthQueryGraphReadTraversalOperator,
};

pub(super) fn relations<Stop>(
    inventory: &WorthQueryAdmittedReadGraphPlanningInventory<'_>,
    admit: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
) -> Result<(Vec<WorthQueryGraphReadPlanningRelation>, bool), WorthQueryCanonicalIdentityStop<Stop>>
{
    let mut rows = reserve_vec(inventory.relation_count(), admit)?;
    let mut has_many = false;
    for relation in inventory
        .relations_admitted(admit)
        .map_err(inventory_stop)?
    {
        admit(2, 0).map_err(WorthQueryCanonicalIdentityStop::Admission)?;
        let name_bytes = copied_bytes::<Stop>(&[relation.relation.len()])?;
        let operator_bytes = u64::try_from(size_of::<WorthQueryGraphReadTraversalOperator>())
            .map_err(|_| WorthQueryCanonicalIdentityStop::AccountingOverflow)?;
        let scratch = name_bytes
            .checked_add(operator_bytes)
            .ok_or(WorthQueryCanonicalIdentityStop::AccountingOverflow)?;
        let work = name_bytes
            .checked_add(4)
            .ok_or(WorthQueryCanonicalIdentityStop::AccountingOverflow)?;
        admit(work, scratch).map_err(WorthQueryCanonicalIdentityStop::Admission)?;
        has_many |= relation.cardinality == ApplicationQueryCardinality::Many;
        rows.push(
            WorthQueryGraphReadPlanningRelation::from_admitted_reference(
                relation.relation,
                match relation.direction {
                    WorthQueryReadGraphRelationDirection::Forward => {
                        WorthQueryAdmittedGraphReadRelationDirection::Forward
                    }
                    WorthQueryReadGraphRelationDirection::Reverse => {
                        WorthQueryAdmittedGraphReadRelationDirection::Ancestor
                    }
                },
                relation.depth,
                vec![WorthQueryGraphReadTraversalOperator::DirectEdge],
            ),
        );
    }
    Ok((rows, has_many))
}

pub(super) fn finish<Stop>(
    graph: &WorthQueryInstalledGraphReadContract,
    inventory: &WorthQueryAdmittedReadGraphPlanningInventory<'_>,
    relations: Vec<WorthQueryGraphReadPlanningRelation>,
    has_many_relation: bool,
    lane: WorthQueryApplicationQueryLane,
    admit: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
) -> Result<WorthQueryGraphReadPlanningShape, WorthQueryCanonicalIdentityStop<Stop>> {
    let predicate_fields = predicate_fields(inventory, admit)?;
    let (ordering_fields, ordering_posture) = ordering_fields(inventory, lane, admit)?;
    admit(8, 0).map_err(WorthQueryCanonicalIdentityStop::Admission)?;
    let predicate_family = if inventory.predicate_count() == 0 && inventory.guard_count() == 0 {
        WorthQueryGraphReadPredicateFamily::None
    } else {
        WorthQueryGraphReadPredicateFamily::Equality
    };
    Ok(WorthQueryGraphReadPlanningShape::from_admitted_shape(
        relations,
        fanout_posture(inventory.relation_count()),
        result_pressure(
            graph.cardinality(),
            graph.projection_count(),
            has_many_relation,
        ),
    )
    .with_predicates(predicate_family, predicate_fields)
    .with_ordering(ordering_posture, ordering_fields)
    .with_relationship_proof_required(inventory.relation_count() != 0)
    .with_root_union_dedup_required(graph.root_union_dedup_required()))
}

fn predicate_fields<Stop>(
    inventory: &WorthQueryAdmittedReadGraphPlanningInventory<'_>,
    admit: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
) -> Result<Vec<WorthQueryGraphReadPlanningPredicateField>, WorthQueryCanonicalIdentityStop<Stop>> {
    let count = inventory
        .predicate_count()
        .checked_add(inventory.guard_count())
        .ok_or(WorthQueryCanonicalIdentityStop::AccountingOverflow)?;
    let mut rows = reserve_vec(count, admit)?;
    for predicate in inventory
        .predicates_admitted(admit)
        .map_err(inventory_stop)?
    {
        admit(1, 0).map_err(WorthQueryCanonicalIdentityStop::Admission)?;
        rows.push(owned_predicate_field(
            predicate.aspect,
            predicate.field,
            predicate.scalar_family.canonical_name(),
            admit,
        )?);
    }
    for guard in inventory.guards_admitted(admit).map_err(inventory_stop)? {
        admit(1, 0).map_err(WorthQueryCanonicalIdentityStop::Admission)?;
        rows.push(owned_predicate_field(
            guard.aspect,
            guard.field,
            guard.scalar_family.canonical_name(),
            admit,
        )?);
    }
    Ok(rows)
}

fn owned_predicate_field<Stop>(
    aspect: &worth_foundational::facade::AspectKey,
    field: &worth_foundational::facade::FieldKey,
    family: &str,
    admit: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
) -> Result<WorthQueryGraphReadPlanningPredicateField, WorthQueryCanonicalIdentityStop<Stop>> {
    admit(3, 0).map_err(WorthQueryCanonicalIdentityStop::Admission)?;
    let bytes = copied_bytes::<Stop>(&[aspect.as_str().len(), field.as_str().len(), family.len()])?;
    admit(
        bytes
            .checked_add(1)
            .ok_or(WorthQueryCanonicalIdentityStop::AccountingOverflow)?,
        bytes,
    )
    .map_err(WorthQueryCanonicalIdentityStop::Admission)?;
    Ok(
        WorthQueryGraphReadPlanningPredicateField::from_admitted_field(
            aspect.clone(),
            field.clone(),
            family,
        ),
    )
}

fn ordering_fields<Stop>(
    inventory: &WorthQueryAdmittedReadGraphPlanningInventory<'_>,
    lane: WorthQueryApplicationQueryLane,
    admit: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
) -> Result<
    (
        Vec<WorthQueryGraphReadPlanningOrderingField>,
        crate::graph_read_access::WorthQueryGraphReadOrderingPosture,
    ),
    WorthQueryCanonicalIdentityStop<Stop>,
> {
    let mut rows = reserve_vec(inventory.ordering_count(), admit)?;
    let mut posture = OrderingPostureScan::new();
    for ordering in inventory
        .orderings_admitted(admit)
        .map_err(inventory_stop)?
    {
        admit(4, 0).map_err(WorthQueryCanonicalIdentityStop::Admission)?;
        posture.observe(ordering.mechanism);
        let direction = match ordering.direction {
            worth_query_declaration::facade::application_query::ApplicationQueryOrderingDirection::Ascending => "ascending",
            worth_query_declaration::facade::application_query::ApplicationQueryOrderingDirection::Descending => "descending",
        };
        let family = ordering.scalar_family.canonical_name();
        admit(5, 0).map_err(WorthQueryCanonicalIdentityStop::Admission)?;
        let bytes = copied_bytes::<Stop>(&[
            ordering.collection_path.len(),
            ordering.aspect.as_str().len(),
            ordering.field.as_str().len(),
            direction.len(),
            family.len(),
        ])?;
        admit(
            bytes
                .checked_add(1)
                .ok_or(WorthQueryCanonicalIdentityStop::AccountingOverflow)?,
            bytes,
        )
        .map_err(WorthQueryCanonicalIdentityStop::Admission)?;
        rows.push(
            WorthQueryGraphReadPlanningOrderingField::from_admitted_field(
                ordering.collection_path,
                ordering.aspect.clone(),
                ordering.field.clone(),
                direction,
                family,
            ),
        );
    }
    Ok((rows, posture.finish(lane, inventory.ordering_count())))
}
