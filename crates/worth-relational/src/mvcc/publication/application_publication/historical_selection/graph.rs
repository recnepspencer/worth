//! Read each complete Native roster once for the graph's two actual bases.
use super::*;
impl RelationalRuntime {
    pub fn prepare_historical_output_selections(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        graph: &crate::branch::RelationalOwnedModelGraphSelection,
    ) -> Result<Vec<RelationalHistoricalOutputSelection>, TransactionCommitError> {
        let entities = graph.entities();
        let current = entities.current_basis();
        let selected = entities.selected_basis();
        let subject = RecordRef::Entity(entities.model());
        require_bases(self, installed, current, selected, &subject)?;
        installed.require_current_ownership_storage(current, entities.model())?;
        installed.require_current_ownership_storage(selected, entities.model())?;
        let count = graph
            .current_entities()
            .len()
            .saturating_add(graph.restored_entities().len());
        let _records_funding = execution
            .reserve_retained_memory(
                (count as u64).saturating_mul(4 * std::mem::size_of::<RecordRef>() as u64),
            )
            .map_err(custody::execution_stop)?;
        execution
            .run_native_with_checkpoint(|context| {
                crate::execution::request_stage::checkpoint(context, count as u64)
                    .map_err(custody::execution_stop)?;
                let selected_records =
                    existing_entities(execution, selected, graph.boundary_entities(), context)?;
                let current_records =
                    existing_entities(execution, current, graph.boundary_entities(), context)?;
                let selected_roster = Arc::new(crate::publication::ownership::read(
                    execution,
                    selected,
                    &selected_records,
                    context,
                )?);
                let current_roster = Arc::new(crate::publication::ownership::read(
                    execution,
                    current,
                    &current_records,
                    context,
                )?);
                if let Some((publication, owner)) = selected_roster
                    .codec_retirement_subject()
                    .or_else(|| current_roster.codec_retirement_subject())
                {
                    return Err(TransactionCommitError::publication(
                        PublicationError::authored_retirement(
                            Cause::OwnedPublicationRequiresCascade { publication, owner },
                            RecordRef::Entity(owner),
                        ),
                    ));
                }
                let memory = funding(
                    execution,
                    selected_roster
                        .coordinates()
                        .len()
                        .saturating_add(current_roster.coordinates().len()),
                    context,
                )?;
                let mut selections = Vec::new();
                for (origin, ordinal) in selected_roster.coordinates() {
                    crate::execution::request_stage::checkpoint(context, 1)
                        .map_err(custody::execution_stop)?;
                    let publication = admit_publication(
                        installed,
                        selected,
                        Arc::clone(&selected_roster),
                        Arc::clone(origin),
                        ordinal,
                    )?;
                    let predecessor = on_route(
                        installed,
                        current,
                        Arc::clone(&current_roster),
                        &origin.meaning,
                        context,
                    )?;
                    selections.push(RelationalHistoricalOutputSelection {
                        current: current.clone(),
                        selected: selected.clone(),
                        publication,
                        predecessor,
                        select_version: true,
                        _funding: Arc::clone(&memory),
                    });
                }
                for (origin, ordinal) in current_roster.coordinates() {
                    let mut exists = false;
                    for selected_origin in selected_roster.origins() {
                        if same_route(&origin.meaning, &selected_origin.meaning, context)? {
                            exists = true;
                            break;
                        }
                    }
                    if !exists {
                        let publication = admit_publication(
                            installed,
                            current,
                            Arc::clone(&current_roster),
                            Arc::clone(origin),
                            ordinal,
                        )?;
                        selections.push(RelationalHistoricalOutputSelection {
                            current: current.clone(),
                            selected: selected.clone(),
                            publication,
                            predecessor: None,
                            select_version: false,
                            _funding: Arc::clone(&memory),
                        });
                    }
                }
                Ok(selections)
            })
            .map_err(custody::execution_stop)?
            .0
    }
}
