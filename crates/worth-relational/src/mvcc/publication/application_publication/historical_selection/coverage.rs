//! A graph selection cannot silently keep or discard publication custody.
use super::*;
use crate::branch::RelationalOwnedModelGraphSelection;

pub(crate) fn require_graph_coverage(
    execution: &ExecutionRequest<'_, '_>,
    graph: &RelationalOwnedModelGraphSelection,
    publication: Option<&RequiredOutputPublication>,
) -> Result<(), TransactionCommitError> {
    let entities = graph.entities();
    let count = entities
        .current_entities()
        .len()
        .saturating_add(entities.selected_entities().len());
    let _funding = execution
        .reserve_retained_memory((count as u64).saturating_mul(128))
        .map_err(custody::execution_stop)?;
    let selections = match publication {
        Some(RequiredOutputPublication::HistoricalSelection { selections, .. }) => {
            selections.as_slice()
        }
        _ => &[],
    };
    execution
        .run_native_with_checkpoint(|context| {
            context
                .checkpoint(count as u64)
                .map_err(custody::execution_stop)?;
            let current_records = super::existing_entities(
                execution,
                entities.current_basis(),
                entities.entities(),
                context,
            )?;
            let selected_records = super::existing_entities(
                execution,
                entities.selected_basis(),
                entities.entities(),
                context,
            )?;
            let current = crate::publication::ownership::read(
                execution,
                entities.current_basis(),
                &current_records,
                context,
            )?;
            let selected = crate::publication::ownership::read(
                execution,
                entities.selected_basis(),
                &selected_records,
                context,
            )?;
            if let Some((publication, owner)) = selected
                .codec_retirement_subject()
                .or_else(|| current.codec_retirement_subject())
            {
                return Err(TransactionCommitError::publication(
                    PublicationError::authored_retirement(
                        Cause::OwnedPublicationRequiresCascade { publication, owner },
                        RecordRef::Entity(owner),
                    ),
                ));
            }
            let entity_work = entities
                .current_entities()
                .len()
                .checked_ilog2()
                .unwrap_or(0) as usize
                + entities
                    .selected_entities()
                    .len()
                    .checked_ilog2()
                    .unwrap_or(0) as usize
                + 3;
            let relation_work = graph.current_relations().len().checked_ilog2().unwrap_or(0)
                as usize
                + graph
                    .selected_relations()
                    .len()
                    .checked_ilog2()
                    .unwrap_or(0) as usize
                + 3;
            let charge = |origin: &crate::history::data::CanonicalRequiredOutputOrigin| {
                selections
                    .len()
                    .saturating_add(origin.members.len().saturating_mul(entity_work))
                    .saturating_add(origin.relations.len().saturating_mul(relation_work))
                    .saturating_add(entity_work)
                    .saturating_add(1) as u64
            };
            for original in selected.origins() {
                context
                    .checkpoint(charge(original))
                    .map_err(custody::execution_stop)?;
                let selection = selections
                    .iter()
                    .find(|s| s.select_version && s.publication.origin.same_publication(original))
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
                    .checkpoint(charge(original))
                    .map_err(custody::execution_stop)?;
                let source_owned = entities
                    .current_entities()
                    .contains(&original.meaning.source_scope)
                    || entities
                        .selected_entities()
                        .contains(&original.meaning.source_scope);
                let outside_members = original
                    .members
                    .iter()
                    .filter(|m| m.posture != RequiredOutputRolePosture::Retire)
                    .any(|m| {
                        !entities.current_entities().contains(&m.id)
                            && !entities.selected_entities().contains(&m.id)
                    });
                let outside_relations = original
                    .relations
                    .iter()
                    .any(|edge| !graph.current_relations().contains_key(&edge.id));
                let covered = selections.iter().any(|s| {
                    if s.select_version {
                        s.predecessor
                            .as_ref()
                            .is_some_and(|p| p.origin.same_publication(original))
                    } else {
                        s.publication.origin.same_publication(original)
                    }
                });
                if !source_owned || outside_members || outside_relations || !covered {
                    return Err(missing(original));
                }
            }
            for selection in selections {
                context
                    .checkpoint((selected.origins().len() + current.origins().len() + 1) as u64)
                    .map_err(custody::execution_stop)?;
                let reachable = if selection.select_version {
                    selected
                        .origins()
                        .any(|o| o.same_publication(&selection.publication.origin))
                } else {
                    let mut at_snapshot = false;
                    for origin in selected.origins() {
                        if super::same_route(
                            &origin.meaning,
                            &selection.publication.origin.meaning,
                            context,
                        )? {
                            at_snapshot = true;
                        }
                    }
                    !at_snapshot
                        && current
                            .origins()
                            .any(|o| o.same_publication(&selection.publication.origin))
                };
                if !reachable
                    || selection.selected.descriptor() != entities.selected_basis().descriptor()
                    || selection.current.descriptor() != entities.current_basis().descriptor()
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
