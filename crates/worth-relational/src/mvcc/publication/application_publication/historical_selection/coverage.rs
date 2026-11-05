//! A graph selection cannot silently keep displaced publication custody.
use super::*;
use crate::branch::RelationalOwnedModelGraphSelection;

pub(crate) fn require_graph_coverage(
    execution: &ExecutionRequest<'_, '_>,
    graph: &RelationalOwnedModelGraphSelection,
    publication: Option<&RequiredOutputPublication>,
) -> Result<(), TransactionCommitError> {
    let entities = graph.entities();
    let count = entities.entities().count();
    let _funding = execution
        .reserve_retained_memory((count as u64).saturating_mul(128))
        .map_err(custody::execution_stop)?;
    let records = entities
        .entities()
        .map(RecordRef::Entity)
        .collect::<Vec<_>>();
    let selections = match publication {
        Some(RequiredOutputPublication::HistoricalSelection { selections, .. }) => {
            selections.as_slice()
        }
        _ => &[],
    };
    execution
        .run_native_with_checkpoint(|context| {
            let current = crate::publication::ownership::read(
                execution,
                entities.current_basis(),
                &records,
                context,
            )?;
            let selected = crate::publication::ownership::read(
                execution,
                entities.selected_basis(),
                &records,
                context,
            )?;
            for original in selected.origins() {
                context
                    .checkpoint(
                        (selections.len() + original.members.len() + original.relations.len() + 1)
                            as u64,
                    )
                    .map_err(custody::execution_stop)?;
                let selection = selections
                    .iter()
                    .find(|selection| selection.publication.origin.same_publication(original))
                    .ok_or_else(|| missing(original))?;
                if selection.selected.descriptor() != entities.selected_basis().descriptor()
                    || selection.current.descriptor() != entities.current_basis().descriptor()
                    || !entities
                        .selected_entities()
                        .contains(&original.meaning.source_scope)
                    || original
                        .members
                        .iter()
                        .filter(|m| m.posture != RequiredOutputRolePosture::Retire)
                        .any(|m| !entities.selected_entities().contains(&m.id))
                    || original
                        .relations
                        .iter()
                        .any(|edge| !graph.selected_relations().contains_key(&edge.id))
                {
                    return Err(missing(original));
                }
            }
            for original in current.origins() {
                context
                    .checkpoint(
                        (selections.len() + original.members.len() + original.relations.len() + 1)
                            as u64,
                    )
                    .map_err(custody::execution_stop)?;
                if !selections.iter().any(|selection| {
                    selection
                        .predecessor
                        .as_ref()
                        .is_some_and(|p| p.origin.same_publication(original))
                }) {
                    return Err(missing(original));
                }
            }
            for selection in selections {
                context
                    .checkpoint(selected.origins().len() as u64 + 1)
                    .map_err(custody::execution_stop)?;
                if !selected
                    .origins()
                    .any(|o| o.same_publication(&selection.publication.origin))
                {
                    return Err(refuse(
                        selection.publication.origin.commit.commit_id,
                        RecordRef::Entity(selection.publication.origin.meaning.source_scope),
                    ));
                }
            }
            Ok(())
        })
        .map_err(custody::execution_stop)?
        .0
}
fn missing(origin: &crate::history::data::CanonicalRequiredOutputOrigin) -> TransactionCommitError {
    TransactionCommitError::publication(PublicationError::authored_retirement(
        Cause::OwnedPublicationRequiresCascade {
            publication: origin.commit.commit_id,
            owner: origin.meaning.source_scope,
        },
        RecordRef::Entity(origin.meaning.source_scope),
    ))
}
