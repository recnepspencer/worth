//! Installed field selection, Native scope and admitted managed-source copies.
use super::*;
use std::collections::{BTreeMap, BTreeSet};
use worth_relational::facade::runtime::{ProjectionAspectRequirement, ProjectionAspectScope};

pub(super) fn project_fields(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    native: &worth_relational::facade::runtime::VisibilityProjectionView,
    record: worth_relational::facade::runtime::EntityProjectionRecord<'_>,
    fields: &[&WorthQueryInstalledGraphProjection],
    mut projected: Vec<WorthQueryApplicationProjectedField>,
    work: &mut ResultTreeWork,
) -> Result<Vec<WorthQueryApplicationProjectedField>, WorthQueryApplicationReadExecutionDenial> {
    for projection in fields {
        work.reads.record_fields(1);
        let Some(value) = field_presence::value(
            record.observe_aspect_field(projection.aspect_key(), projection.field_key()),
            projection.presence(),
            projection.result_path(),
        )?
        else {
            continue;
        };
        if value.value_family() != projection.scalar_family() {
            return Err(projection_denial(projection.result_path()));
        }

        work.material_field(projection.slot_type(), value)?;
        let field = if matches!(
            value,
            worth_foundational::facade::AspectValue::ContentRef(_)
        ) {
            let locator = worth_foundational::facade::AspectFieldLocator::new(
                worth_foundational::facade::LocatorAuthority::Authoritative,
                projection.aspect_key().clone(),
                worth_foundational::facade::CanonicalFieldPath::single(
                    projection.field_key().clone(),
                ),
            );
            let content = native
                .retain_entity_field_content(
                    execution,
                    record.entity_id(),
                    record.kind_id(),
                    &locator,
                )
                .map_err(|cause| {
                    super::super::denial::content_read_denial(cause, projection.result_path())
                })?;
            WorthQueryApplicationProjectedField::source(projection, value.clone(), content)
                .map_err(|cause| {
                    super::super::denial::content_read_denial(cause, projection.result_path())
                })?
        } else {
            WorthQueryApplicationProjectedField::new(projection, value.clone())
        };
        projected.push(field);
    }
    Ok(projected)
}

pub(super) fn direct_projections<'a>(
    contract: &'a WorthQueryInstalledGraphReadContract,
    governance: &crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationQueryGovernance,
    parent: &str,
    work: &ResultTreeWork,
) -> Result<Vec<&'a WorthQueryInstalledGraphProjection>, WorthQueryApplicationReadExecutionDenial> {
    let selected = contract.projections().iter().filter(|projection| {
            let field = (projection.entity(), projection.aspect(), projection.field());
            let disclosed = governance
                .admit_disclosed_projection(
                    projection.slot_key_identity().as_ref(),
                    field,
                    projection.field_key(),
                )
                .is_some();
            let internal_ordering = contract.ordering().iter().any(|ordering| {
                ordering.collection_path() == parent
                    && ordering.field() == field
                    && governance
                        .admit_internal_projection(
                            ordering.field(),
                            ordering.field_key(),
                            worth_query_declaration::facade::application_query::ApplicationQueryObservableInfluence::Ordering,
                        )
                        .is_some()
            });
            projection.parent_path() == parent
                && (disclosed || internal_ordering)
        });
    work.collect_material(contract.projections().len(), selected, parent)
}

pub(super) fn projection_scope(
    fields: &[&WorthQueryInstalledGraphProjection],
) -> ProjectionAspectScope {
    let mut keys = BTreeMap::new();
    for field in fields {
        keys.entry(field.aspect_key().clone())
            .or_insert_with(BTreeSet::new)
            .insert(field.field_key().clone());
    }
    ProjectionAspectScope::from_requirements(
        keys.into_iter()
            .map(|(aspect, fields)| ProjectionAspectRequirement::fields(aspect, fields)),
    )
}
