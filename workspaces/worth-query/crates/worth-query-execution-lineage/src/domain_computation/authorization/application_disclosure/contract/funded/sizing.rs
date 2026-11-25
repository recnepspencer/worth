//! Exact pre-allocation byte sizing for funded one-shot disclosure compilation:
//! covers every result-slot key variant, internal rule/group table, temporary
//! validation lookup key (predicates, filters, related predicates, managed
//! projections, live scope/target fields, graph-side Orbit/Components keys),
//! and closed-vocabulary lane cones.
use super::super::*;
use super::{Failure, StageError};
use crate::domain_computation::primary_graph::admitted_capture::{
    checkpoint, storage::tree_clone_bytes,
};
use std::{alloc::Layout, sync::atomic::AtomicUsize};
use worth_execution::{MapKernelContext, MapKernelFailure};
use worth_query_declaration::facade::application_query::ApplicationQueryObservableInfluence;

pub(super) fn backing_bytes<Schema, Query, Parameters, QueryResult, Scope>(
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<u64, Failure> {
    let ovf = || MapKernelFailure::Domain(StageError::Memory);
    let contract = query.disclosure();
    let count = contract.rules().len();
    let mut bytes =
        tree_clone_bytes::<(ApplicationQueryResultSlotKey, AdmittedDisclosureRule)>(count)
            .and_then(|b| b.checked_mul(2))
            .ok_or_else(ovf)?;
    bytes = bytes
        .checked_add(
            tree_clone_bytes::<((String, String, String), Vec<AdmittedDisclosureRule>)>(count)
                .ok_or_else(ovf)?,
        )
        .and_then(|b| {
            b.checked_add(tree_clone_bytes::<(
                (String, String, String, RelationDirection),
                Vec<AdmittedDisclosureRule>,
            )>(count)?)
        })
        .ok_or_else(ovf)?;
    let mut internal_fields = 0usize;
    let mut internal_rels = 0usize;
    for rule in contract.rules() {
        checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
        match rule.selector() {
            ApplicationQueryDisclosureSelector::InternalField { .. } => internal_fields += 1,
            ApplicationQueryDisclosureSelector::InternalRelation { .. } => internal_rels += 1,
            _ => {}
        }
    }
    let total_internal = internal_fields.checked_add(internal_rels).ok_or_else(ovf)?;
    let vec_tables =
        Layout::array::<AdmittedDisclosureRule>(total_internal.checked_mul(2).ok_or_else(ovf)?)
            .map_err(|_| ovf())?
            .size()
            .checked_add(
                Layout::array::<((String, String, String), GovernedInternalRuleGroup)>(
                    internal_fields,
                )
                .map_err(|_| ovf())?
                .size(),
            )
            .and_then(|n| {
                n.checked_add(
                    Layout::array::<(
                        (String, String, String, RelationDirection),
                        GovernedInternalRuleGroup,
                    )>(internal_rels)
                    .ok()?
                    .size(),
                )
            })
            .ok_or_else(ovf)?;
    bytes = add(bytes, vec_tables)?;
    bytes = bytes
        .checked_add(
            tree_clone_bytes::<((&str, &str, &str), usize)>(internal_fields).ok_or_else(ovf)?,
        )
        .and_then(|b| {
            b.checked_add(tree_clone_bytes::<(
                (&str, &str, &str, RelationDirection),
                usize,
            )>(internal_rels)?)
        })
        .ok_or_else(ovf)?;
    let text = contract
        .classification()
        .len()
        .checked_add(contract.capability_name().map_or(0, str::len))
        .and_then(|n| n.checked_add(contract.capability_type().map_or(0, str::len)))
        .and_then(|n| n.checked_add(query.name().len()))
        .ok_or_else(ovf)?;
    bytes = add(bytes, text)?;
    let header = Layout::new::<[AtomicUsize; 2]>()
        .extend(Layout::new::<WorthQueryAdmittedApplicationDisclosureRule>())
        .map_err(|_| ovf())?
        .0
        .pad_to_align()
        .size();
    for rule in contract.rules() {
        checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
        bytes = add(bytes, result_slot_key_bytes(rule.selector())?)?;
        bytes = add(bytes, header)?;
        bytes = add(
            bytes,
            rule.disclosure_value().owned_allocation_capacity_bytes(),
        )?;
        bytes = bytes
            .checked_add(
                tree_clone_bytes::<ApplicationQueryObservableInfluence>(
                    rule.influence().permitted().len(),
                )
                .ok_or_else(ovf)?,
            )
            .ok_or_else(ovf)?;
        if let Some((entity, aspect, field)) = rule.selector().field_contract() {
            let names = entity
                .len()
                .checked_add(aspect.len())
                .and_then(|n| n.checked_add(field.len()))
                .and_then(|n| n.checked_mul(4))
                .ok_or_else(ovf)?;
            bytes = add(bytes, names)?;
            if let Some(mask) = rule.selector().projection_mask() {
                bytes = add(bytes, mask.owned_allocation_capacity_bytes())?;
            }
            if let Some(mask) = rule.selector().diagnostic_mask() {
                bytes = add(bytes, mask.owned_allocation_capacity_bytes())?;
            }
        }
        if let ApplicationQueryDisclosureSelector::InternalRelation {
            relation, from, to, ..
        } = rule.selector()
        {
            let names = relation
                .len()
                .checked_add(from.len())
                .and_then(|n| n.checked_add(to.len()))
                .and_then(|n| n.checked_mul(2))
                .ok_or_else(ovf)?;
            bytes = add(bytes, names)?;
        }
    }
    bytes = add(bytes, validation_lookup_key_bytes(query, context)?)?;
    let vocabulary = ApplicationQueryObservableInfluence::ALL.len();
    let cone_bytes = tree_clone_bytes::<ApplicationQueryObservableInfluence>(vocabulary)
        .and_then(|b| b.checked_mul(2))
        .ok_or_else(ovf)?;
    bytes = bytes.checked_add(cone_bytes).ok_or_else(ovf)?;
    add(
        bytes,
        Layout::array::<ApplicationQueryObservableInfluence>(vocabulary)
            .map_err(|_| ovf())?
            .size(),
    )
}

fn sum_usize(lens: &[usize]) -> Result<usize, Failure> {
    lens.iter()
        .try_fold(0usize, |acc, &n| acc.checked_add(n))
        .ok_or(MapKernelFailure::Domain(StageError::Memory))
}

fn orbit_key_bytes(
    orbit: &worth_query_declaration::facade::application_query::ApplicationQueryOrbitMeaning,
) -> Result<usize, Failure> {
    sum_usize(&[
        orbit.query_type().len(),
        orbit.slot_type().len(),
        orbit.relation().len(),
        orbit.from().len(),
        orbit.to().len(),
        orbit.output_name().len(),
    ])
}

fn component_key_bytes(
    comp: &worth_query_declaration::facade::application_query::ApplicationQueryComponentMeaning,
) -> Result<usize, Failure> {
    sum_usize(&[
        comp.query_type().len(),
        comp.slot_type().len(),
        comp.set().len(),
        comp.member().len(),
        comp.output_name().len(),
    ])
}

/// Accounts heap backing for every `ApplicationQueryResultSlotKey` variant
/// that the one-shot disclosure compiler allocates into `result_rules`.
fn result_slot_key_bytes(selector: &ApplicationQueryDisclosureSelector) -> Result<usize, Failure> {
    match selector {
        ApplicationQueryDisclosureSelector::Field {
            query_type,
            slot_type,
            entity,
            aspect,
            field,
            output_name,
            value_type,
            ..
        } => sum_usize(&[
            entity.capacity(),
            aspect.capacity(),
            field.capacity(),
            output_name.capacity(),
            query_type.retained_heap_bytes(),
            slot_type.retained_heap_bytes(),
            value_type.retained_heap_bytes(),
        ]),
        ApplicationQueryDisclosureSelector::Relation {
            query_type,
            slot_type,
            relation,
            from,
            to,
            output_name,
            ..
        } => sum_usize(&[
            relation.capacity(),
            from.capacity(),
            to.capacity(),
            output_name.capacity(),
            query_type.retained_heap_bytes(),
            slot_type.retained_heap_bytes(),
        ]),
        ApplicationQueryDisclosureSelector::Orbit(orbit) => orbit_key_bytes(orbit),
        ApplicationQueryDisclosureSelector::Components(comp) => component_key_bytes(comp),
        _ => Ok(0),
    }
}

/// Accounts every temporary lookup/comparison key built during compilation and
/// `validate_influence` (predicates, filters, related predicates, managed
/// projections, live scope/target fields, root-path guards, ordering, and
/// graph-side Orbit/Components keys coexisting with selector keys).
fn validation_lookup_key_bytes<Schema, Query, Parameters, QueryResult, Scope>(
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<usize, Failure> {
    let ovf = || MapKernelFailure::Domain(StageError::Memory);
    let mut total = 0usize;
    let mut add3 = |a: &str, b: &str, c: &str| -> Result<(), Failure> {
        total = total
            .checked_add(a.len())
            .and_then(|n| n.checked_add(b.len()))
            .and_then(|n| n.checked_add(c.len()))
            .ok_or_else(ovf)?;
        Ok(())
    };
    let graph = query.read_graph();
    if graph.managed_index_selection().is_some() {
        for projection in graph.projections() {
            checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
            add3(projection.entity(), projection.aspect(), projection.field())?;
        }
    }
    for f in graph
        .predicates()
        .iter()
        .map(|p| p.field())
        .chain(
            graph
                .where_filter()
                .map_or(&[][..], |w| w.fields())
                .iter()
                .map(|f| f.field()),
        )
        .chain(graph.ordering().iter().map(|o| o.field()))
    {
        checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
        add3(f.0, f.1, f.2)?;
    }
    for related in graph.related_predicates() {
        checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
        add3(related.relation(), related.from(), related.to())?;
        for t in related.target_predicates() {
            let f = t.field();
            add3(f.0, f.1, f.2)?;
        }
        for f in related
            .target_where_filter()
            .map_or(&[][..], |w| w.fields())
        {
            let f = f.field();
            add3(f.0, f.1, f.2)?;
        }
    }
    for guard in graph.root_paths().iter().flat_map(|p| p.guards()) {
        checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
        add3(
            guard.entity(),
            guard.aspect().as_str(),
            guard.field().as_str(),
        )?;
    }
    if let Some(live) = query.live() {
        for id in [live.scope_identity(), live.target_identity()] {
            checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
            add3(id.entity(), id.aspect(), id.field())?;
        }
    }
    // Graph-side Orbit/Components keys constructed in `admit_rule` (selector
    // comparison) and `validate_influence` coexist with inserted result keys.
    if let Some(orbit) = graph.orbit() {
        checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
        let key = orbit_key_bytes(orbit)?.checked_mul(2).ok_or_else(ovf)?;
        total = total.checked_add(key).ok_or_else(ovf)?;
    }
    if let Some(comp) = graph.component_selection() {
        checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
        let key = component_key_bytes(comp)?.checked_mul(2).ok_or_else(ovf)?;
        total = total.checked_add(key).ok_or_else(ovf)?;
    }
    Ok(total)
}

fn add(bytes: u64, extra: usize) -> Result<u64, Failure> {
    u64::try_from(extra)
        .ok()
        .and_then(|extra| bytes.checked_add(extra))
        .ok_or(MapKernelFailure::Domain(StageError::Memory))
}
