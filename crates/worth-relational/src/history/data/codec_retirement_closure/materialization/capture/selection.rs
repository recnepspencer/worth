//! Actual relation birth identifies the group; its native route locates current custody.
use super::*;
use crate::history::data::RequiredOutputRoutePosition as Position;
use crate::identity::data::RelationId;
use worth_execution::{ExecutionResourceLease, MapKernelContext};

pub(super) fn at_source(
    runtime: &RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
    record: &RelationalMaterializationRecord,
    context: &mut MapKernelContext<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<Option<Arc<CanonicalCommitEnvelope>>, TransactionCommitError> {
    let relation_id = match record {
        RelationalMaterializationRecord::Entity { entity_id, .. } => {
            return optional(super::super::super::membership::latest(
                runtime, basis, *entity_id, context, lease,
            ))
        }
        RelationalMaterializationRecord::Relation { relation_id, .. } => *relation_id,
    };
    let Some(birth) = optional(relation_birth(runtime, basis, relation_id, context))? else {
        return Ok(None);
    };
    let origin = birth
        .required_output_origin
        .as_deref()
        .expect("validated actual producer birth");
    let positions = &basis.inner.root.output_publication_positions;
    let position = match positions.get(&origin.meaning, context)? {
        Some(position) => Some(position),
        None if positions.unresolved_prefix().is_some() => {
            optional(super::super::super::route_selection::select_at(
                runtime,
                basis.inner.root.commit_id(),
                &origin.meaning,
                context,
                lease,
            ))?
            .flatten()
        }
        None => None,
    };
    let Some(Position::Published(commit)) = position else {
        return Ok(None);
    };
    // Indexed publication is a locator, never ownership authority. A retired
    // route cannot fall back to an older endpoint or an older publication.
    optional((|| {
        let current = runtime
            .history
            .canonical_envelope(commit)
            .ok_or("current incident producer publication absent")?;
        let current_origin = current.validate_output_membership_header()?;
        charge(
            context,
            current_origin.relations.len()
                + origin.meaning.family.len()
                + current_origin.meaning.family.len()
                + 1,
        )?;
        if current_origin.birth != origin.birth
            || current_origin.birth_producer != origin.birth_producer
            || current_origin.birth_dependency != origin.birth_dependency
            || !current_origin.meaning.same_route(&origin.meaning)
            || !current_origin
                .relations
                .iter()
                .any(|edge| edge.id == relation_id)
        {
            return Err(
                "current incident producer differs from actual birth group or relation custody"
                    .into(),
            );
        }
        Ok(current)
    })())
}

fn relation_birth(
    runtime: &RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
    relation_id: RelationId,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<Arc<CanonicalCommitEnvelope>, Stop> {
    charge(context, 3)?;
    let partition = basis
        .inner
        .root
        .partition_state(relation_id.partition_id)
        .ok_or("incident producer relation partition absent")?;
    let created = partition
        .relation_arena
        .created_at_for_slot(relation_id.slot_index())
        .ok_or("incident producer relation birth position absent")?;
    let birth = runtime
        .history
        .canonical_envelope_for_version(created)
        .ok_or("incident producer exact canonical birth absent")?;
    let origin = birth.validate_output_membership_header()?;
    // Actual Created allocation/endpoints establish producer ownership. Current
    // incidence and caller-supplied restoration fields provide no such proof.
    crate::history::data::publication_relations::validate_owned(
        &*basis.inner.root,
        relation_id,
        origin,
        &mut |version| runtime.history.canonical_envelope_for_version(version),
        context,
    )?;
    Ok(birth)
}

fn optional<T>(result: Result<T, Stop>) -> Result<Option<T>, TransactionCommitError> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(Stop::Payload(_)) => Ok(None),
        Err(cause) => Err(stop(cause)),
    }
}
