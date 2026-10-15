//! Prepare selection preimages inside the body's native decision observation.
use super::{
    ApplicationMutationBinding, ApplicationSchema, DecisionReader, HandlerExecutionDenial,
};
use crate::domain_computation::primary_graph::{
    operation_body::{PreparedBodySelection, PreparedBodySelections},
    WorthQueryAdvancementDenial, WorthQueryManagedComputationResourceDenial,
};
use worth_query_declaration::facade::application_schema::ApplicationOperationDecisionReadTarget;
impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>>
    DecisionReader<'_, '_, '_, Schema, Binding>
{
    pub(in crate::domain_computation::primary_graph) fn body_selections(
        &mut self,
        selections: &[ApplicationOperationDecisionReadTarget],
    ) -> Result<PreparedBodySelections<Schema>, HandlerExecutionDenial> {
        if selections.is_empty() {
            return Ok(PreparedBodySelections {
                targets: Vec::new(),
                _memory: None,
            });
        }
        let bytes = selections
            .iter()
            .try_fold(0usize, |total, selection| {
                let ApplicationOperationDecisionReadTarget::Relation { relation, from, to } =
                    selection
                else {
                    unreachable!("installed selection is a relation");
                };
                total
                    .checked_add(size_of::<(
                        ApplicationOperationDecisionReadTarget,
                        Option<PreparedBodySelection<Schema>>,
                    )>())
                    .and_then(|bytes| bytes.checked_add(relation.len()))
                    .and_then(|bytes| bytes.checked_add(from.len()))
                    .and_then(|bytes| bytes.checked_add(to.len()))
                    // The native identity and mutation target share this Arc name.
                    // Admit conversion's temporary String plus the Arc counters/name.
                    .and_then(|bytes| bytes.checked_add(to.len()))
                    .and_then(|bytes| bytes.checked_add(to.len()))
                    .and_then(|bytes| bytes.checked_add(2 * size_of::<usize>()))
            })
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(|| {
                HandlerExecutionDenial::new(WorthQueryAdvancementDenial::Resource(
                    WorthQueryManagedComputationResourceDenial::CapacityOverflow,
                ))
            })?;
        let execution = self.execution;
        let memory = execution.reserve(bytes).map_err(|cause| {
            HandlerExecutionDenial::new(WorthQueryAdvancementDenial::Resource(cause))
        })?;
        self.context_use.set(self.context_use.get().scope());
        let targets = execution
            .decision_stage(|context| {
                let mut targets = Vec::with_capacity(selections.len());
                for selection in selections {
                    let target = self
                        .reader
                        .body_related_selection(self.scope, selection, execution, context)?;
                    targets.push((selection.clone(), target));
                }
                Ok(targets)
            })
            .map_err(HandlerExecutionDenial::new)??;
        Ok(PreparedBodySelections {
            targets,
            _memory: Some(memory),
        })
    }
}
