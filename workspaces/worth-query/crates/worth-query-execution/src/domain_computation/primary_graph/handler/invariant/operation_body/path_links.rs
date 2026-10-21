//! Resolve exact live anchors before any creation; no candidate-time graph lookup.
use super::*;
use crate::domain_computation::primary_graph::{
    operation_body::PreparedBodyPathLinks, WorthQueryApplicationAttemptDenial as Denial,
    WorthQueryApplicationAttemptDenialKind as Kind, WorthQueryInvariantMutationTarget,
};
use worth_query_declaration::facade::application_schema::ApplicationOperationBodyProgram;
impl<S: ApplicationSchema, B: ApplicationMutationBinding<S>> DecisionReader<'_, '_, '_, S, B> {
    pub(in crate::domain_computation::primary_graph) fn body_path_links(
        &mut self,
        body: &ApplicationOperationBodyProgram,
    ) -> Result<PreparedBodyPathLinks<S>, HandlerExecutionDenial> {
        let paths = || {
            body.creation()
                .into_iter()
                .flat_map(|c| c.initialization_sites())
                .flat_map(|c| c.path_links())
        };
        let count = paths().count();
        if count == 0 {
            return Ok(PreparedBodyPathLinks {
                targets: Vec::new(),
                _memory: None,
            });
        }
        let bytes = paths()
            .try_fold(0usize, |sum, step| {
                step.path.steps().iter().try_fold(
                    sum.checked_add(size_of::<WorthQueryInvariantMutationTarget<S, ()>>())?,
                    |sum, hop| {
                        sum.checked_add(hop.child_entity().len().checked_mul(2)?)?
                            .checked_add(2 * size_of::<usize>())
                    },
                )
            })
            .and_then(|n| u64::try_from(n).ok())
            .ok_or_else(|| {
                resource(WorthQueryManagedComputationResourceDenial::CapacityOverflow)
            })?;
        let memory = self.execution.reserve(bytes).map_err(resource)?;
        self.context_use.set(self.context_use.get().scope());
        let targets = self
            .execution
            .decision_stage(|context| {
                let mut targets = Vec::with_capacity(count);
                for path in paths() {
                    let selected = self
                        .reader
                        .body_path_selection(self.scope, &path.path, self.execution, context)?
                        .ok_or_else(|| {
                            HandlerExecutionDenial::new(Denial::new(
                                Kind::MissingAuthoritativeFact,
                                path.path.terminal_entity(),
                            ))
                        })?;
                    targets.push(selected.target);
                }
                Ok(targets)
            })
            .map_err(HandlerExecutionDenial::new)??;
        Ok(PreparedBodyPathLinks {
            targets,
            _memory: Some(memory),
        })
    }
}
fn resource(cause: WorthQueryManagedComputationResourceDenial) -> HandlerExecutionDenial {
    HandlerExecutionDenial::new(WorthQueryAdvancementDenial::Resource(cause))
}
