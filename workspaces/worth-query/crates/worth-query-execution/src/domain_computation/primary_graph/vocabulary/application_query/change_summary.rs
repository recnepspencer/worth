//! Moved whole from `domain_computation::primary_graph::application_query::derived_view::change_summary`: the items below and every impl they own.
use crate::domain_computation::primary_graph::vocabulary::application_query::publication::ViewChange;
use worth_relational::facade::mvcc::PreparedRelationalChangeSummary;
use worth_relational::facade::publication::RecordStructuralChange;
use worth_relational::facade::transactions::RecordRef;

pub(in crate::domain_computation::primary_graph) fn changes_from_summary(
    summary: &PreparedRelationalChangeSummary,
    maximum_changes: usize,
) -> Option<Vec<ViewChange>> {
    let mut required = 0usize;
    for record in &summary.records {
        // Relation adjacency has no safe bounded target without endpoints.
        if matches!(record.target, RecordRef::Relation(_))
            && record.before_endpoints.is_none()
            && record.after_endpoints.is_none()
        {
            return None;
        }
        let additional = match record.target {
            RecordRef::Entity(_) => record.aspect_scopes.len().checked_add(usize::from(
                record.structural_change != RecordStructuralChange::Updated,
            ))?,
            RecordRef::Relation(_) => 4,
        };
        required = required.checked_add(additional)?;
        if required > maximum_changes {
            return None;
        }
    }
    let mut changes = Vec::with_capacity(required);
    for record in &summary.records {
        match record.target {
            RecordRef::Entity(entity) => {
                // Relational's exact native field transition projection
                // publishes scopes for changed values. An updated record with
                // no scopes has no field revision changes or lifecycle change.
                if record.structural_change != RecordStructuralChange::Updated {
                    changes.push(ViewChange::Entity(entity));
                }
                for scope in &record.aspect_scopes {
                    changes.push(match &scope.field_path {
                        Some(path) => {
                            ViewChange::Field(entity, scope.aspect_key.clone(), path.clone())
                        }
                        None => ViewChange::Aspect(entity, scope.aspect_key.clone()),
                    });
                }
            }
            RecordRef::Relation(_) => {
                for endpoints in [record.before_endpoints, record.after_endpoints]
                    .into_iter()
                    .flatten()
                {
                    changes.push(ViewChange::Adjacency(
                        endpoints.source,
                        endpoints.kind_id,
                        0,
                    ));
                    changes.push(ViewChange::Adjacency(
                        endpoints.target,
                        endpoints.kind_id,
                        1,
                    ));
                }
            }
        }
    }
    Some(changes)
}
