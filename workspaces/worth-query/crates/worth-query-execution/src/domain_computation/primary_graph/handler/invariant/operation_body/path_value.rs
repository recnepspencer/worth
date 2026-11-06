//! Select and observe all existing destinations before any candidate write.
mod aliases;
use super::*;
use crate::domain_computation::primary_graph::vocabulary::operation_body::field_expectation::BodyFieldExpectation;
use crate::domain_computation::vocabulary::request_execution::execution_refusal::WorthQueryManagedComputationResourceDenial;
use worth_query_declaration::facade::application_schema::ApplicationOperationBodyProgram;
use {crate::domain_computation::primary_graph::vocabulary::invariant_projection::WorthQueryInvariantMutationTarget, crate::domain_computation::primary_graph::vocabulary::operation_body::path_value::PreparedBodyPathWrites, crate::domain_computation::primary_graph::vocabulary::application_attempt::attempt_denial::WorthQueryApplicationAttemptDenial as Denial, crate::domain_computation::vocabulary::primary_graph::kind::WorthQueryApplicationAttemptDenialKind as Kind};
impl<S: ApplicationSchema, B: ApplicationMutationBinding<S>> DecisionReader<'_, '_, '_, S, B> {
    pub(in crate::domain_computation::primary_graph) fn body_path_writes(
        &mut self,
        body: &ApplicationOperationBodyProgram,
    ) -> Result<PreparedBodyPathWrites<S>, HandlerExecutionDenial> {
        let steps = body.path_value_assignments();
        if steps.is_empty() {
            return Ok(PreparedBodyPathWrites {
                targets: Vec::new(),
                _memory: None,
            });
        }
        let bytes = steps
            .iter()
            .try_fold(0usize, |sum, step| {
                // Each native hop creates a shared Arc identity/target name via a
                // temporary String. Fund those backings as well as target slots;
                // the claim covers the final name through candidate assembly.
                step.target.path.steps().iter().try_fold(
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
                let mut targets: Vec<WorthQueryInvariantMutationTarget<S, ()>> =
                    Vec::with_capacity(steps.len());
                for step in steps {
                    let selected = self
                        .reader
                        .body_path_selection(
                            self.scope,
                            &step.target.path,
                            self.execution,
                            context,
                        )?
                        .ok_or_else(|| {
                            denied(
                                Kind::MissingAuthoritativeFact,
                                step.target.path.terminal_entity(),
                            )
                        })?;
                    if !self.reader.body_field(
                        &selected.identity,
                        &step.target.field,
                        BodyFieldExpectation::Present,
                    )? {
                        return Err(denied(
                            Kind::MissingAuthoritativeFact,
                            step.target.field.field(),
                        ));
                    }
                    targets.push(selected.target);
                }
                Ok(targets)
            })
            .map_err(HandlerExecutionDenial::new)??;
        aliases::reject(self.scope.entity_id(), body, &targets, self.execution)?;
        Ok(PreparedBodyPathWrites {
            targets,
            _memory: Some(memory),
        })
    }
}
fn resource(cause: WorthQueryManagedComputationResourceDenial) -> HandlerExecutionDenial {
    HandlerExecutionDenial::new(WorthQueryAdvancementDenial::Resource(cause))
}
fn denied(kind: Kind, target: &str) -> HandlerExecutionDenial {
    HandlerExecutionDenial::new(Denial::new(kind, target))
}
