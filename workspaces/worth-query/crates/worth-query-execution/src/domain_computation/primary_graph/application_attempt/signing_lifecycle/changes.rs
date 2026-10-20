//! Intersect final candidate writes with native preimages using Relational's canonical value equivalence.
use super::{
    attempt, capacity, resource, unavailable, Effect, EntityId, Fact, Lease, Locus, Outcome,
    QueryRequestExecution, Rule,
};
use crate::domain_computation::primary_graph::application_checkpoint::facts::FactSequence;
use crate::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryGraphLayout as Layout;
use std::collections::BTreeMap;
use worth_foundational::facade::{AspectKey, AspectValue, CanonicalFieldPath};
use worth_relational::facade::storage::AuthoritativeFieldComparisonKey as Comparison;

/// Operation-local work only: O(facts log facts + writes * declared subject fields).
/// No source document, history, or retained derived output is scanned here.
pub(super) fn select(
    rule: &Rule,
    lease: &Lease,
    effects: &[Effect],
    facts: &(impl FactSequence + ?Sized),
    execution: &QueryRequestExecution<'_>,
) -> Result<BTreeMap<EntityId, Vec<Locus>>, Outcome> {
    let layout: &Layout = &lease.layout;
    let mut before: BTreeMap<(EntityId, &AspectKey, &CanonicalFieldPath), Option<&AspectValue>> =
        BTreeMap::new();
    for fact in facts.iter() {
        match fact {
            Fact::Field {
                entity_id,
                locator,
                value,
                ..
            } => {
                before.insert(
                    (
                        *entity_id,
                        locator.aspect().aspect_key(),
                        locator.field_path(),
                    ),
                    Some(value),
                );
            }
            Fact::AbsentField {
                entity_id, locator, ..
            } => {
                before.insert(
                    (
                        *entity_id,
                        locator.aspect().aspect_key(),
                        locator.field_path(),
                    ),
                    None,
                );
            }
            _ => {}
        }
    }
    let mut changed = BTreeMap::<EntityId, Vec<Locus>>::new();
    let mut final_writes = BTreeMap::<(EntityId, &Locus), Option<&AspectValue>>::new();
    for effect in effects {
        let entity = match effect {
            Effect::UpdateEntity {
                entity, entity_id, ..
            }
            | Effect::PatchOptionalEntityFields {
                entity, entity_id, ..
            } if entity == &rule.spec().source_entity => *entity_id,
            Effect::DeleteEntity { entity_id } => {
                let request = execution.execution_request().map_err(resource)?;
                let kind = lease.handle().with_runtime(&request, |runtime| {
                    runtime
                        .read_truth()
                        .project_snapshot(lease.snapshot())
                        .ok_or_else(unavailable)?
                        .entity_record_with_projection_scope(
                            &request,
                            *entity_id,
                            worth_relational::facade::runtime::ProjectionAspectScope::empty(),
                            |record| Some(record.kind_id()),
                        )
                        .map_err(|cause| {
                            attempt(
                                super::super::super::WorthQueryApplicationAttemptDenial::from(
                                    cause,
                                ),
                            )
                        })?
                        .ok_or_else(unavailable)
                })?;
                if Some(kind) == layout.entity_kind(&rule.spec().source_entity) {
                    changed.insert(*entity_id, rule.subjects().to_vec());
                }
                continue;
            }
            _ => continue,
        };
        for field in rule.subjects() {
            let locator = layout
                .field_locator(field.entity(), field.aspect(), field.field())
                .ok_or_else(unavailable)?;
            let next = match effect {
                Effect::UpdateEntity { fields, .. } => fields.get(locator).map(Some),
                Effect::PatchOptionalEntityFields { fields, .. } => {
                    fields.get(locator).map(|write| write.value.as_ref())
                }
                _ => unreachable!("source field effect selected"),
            };
            let Some(next) = next else {
                continue;
            };
            final_writes.insert((entity, field), next);
        }
    }
    for ((entity, field), next) in final_writes {
        let locator = layout
            .field_locator(field.entity(), field.aspect(), field.field())
            .ok_or_else(unavailable)?;
        let previous = *before
            .get(&(entity, locator.aspect().aspect_key(), locator.field_path()))
            .ok_or_else(unavailable)?;
        if !equivalent(previous, next, execution)? {
            let fields = changed.entry(entity).or_default();
            if !fields.contains(field) {
                fields.push(field.clone());
            }
        }
    }
    Ok(changed)
}
fn equivalent(
    previous: Option<&AspectValue>,
    next: Option<&AspectValue>,
    execution: &QueryRequestExecution<'_>,
) -> Result<bool, Outcome> {
    let (Some(previous), Some(next)) = (previous, next) else {
        return Ok(previous.is_none() && next.is_none());
    };
    let bytes = Comparison::required_encoded_capacity_bytes(previous)
        .and_then(|bytes| bytes.checked_add(Comparison::required_encoded_capacity_bytes(next)?))
        .ok_or_else(capacity)?;
    let _memory = execution.reserve(bytes).map_err(resource)?;
    Ok(Comparison::from_aspect_value(previous) == Comparison::from_aspect_value(next))
}
