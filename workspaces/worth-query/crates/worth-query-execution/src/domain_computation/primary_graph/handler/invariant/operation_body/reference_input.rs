//! The compiled reference uses the same native indexed selection and adjacency owners.
use super::*;
use crate::domain_computation::primary_graph::{
    operation_body::reference_input::{PreparedBodyReference, PreparedBodyReferences},
    operation_body::{PreparedBodyInputs, PreparedBodySelection},
    WorthQueryEntityResolutionDenial, WorthQueryEntityResolutionDenialKind as ResolutionKind,
    WorthQueryEntityResolutionRefusalKind as Refusal,
};
use worth_query_declaration::facade::application_schema::ApplicationOperationBodyProgram;
use worth_query_declaration::facade::application_schema::ApplicationOperationDecisionReadTarget as Read;
impl<S: ApplicationSchema, B: ApplicationMutationBinding<S>> DecisionReader<'_, '_, '_, S, B> {
    pub(in crate::domain_computation::primary_graph) fn body_references(
        &mut self,
        body: &ApplicationOperationBodyProgram,
        inputs: Option<&PreparedBodyInputs>,
    ) -> Result<PreparedBodyReferences<S>, HandlerExecutionDenial> {
        if body.reference_inputs().is_empty() {
            return Ok(PreparedBodyReferences {
                targets: Vec::new(),
                _memory: None,
            });
        }
        // Reference inputs are the final slots, after every historical input profile.
        let inputs = &inputs.expect("installed reference inputs").values;
        let values = &inputs[inputs.len() - body.reference_inputs().len()..];
        let bytes = body
            .reference_inputs()
            .iter()
            .zip(values)
            .try_fold(0usize, |sum, (reference, value)| {
                sum.checked_add(size_of::<PreparedBodyReference<S>>())?
                    .checked_add(size_of::<[PreparedBodySelection<S>; 2]>())?
                    .checked_add(reference.to.len().checked_mul(6)?)?
                    .checked_add(reference.relation.len())?
                    .checked_add(reference.from.len())?
                    .checked_add(6 * size_of::<usize>())?
                    .checked_add(
                        value
                            .as_ref()
                            .map_or(0, AspectValue::owned_allocation_capacity_bytes)
                            .checked_mul(2)?,
                    )
            })
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(|| {
                resource(WorthQueryManagedComputationResourceDenial::CapacityOverflow)
            })?;
        let memory = self.execution.reserve(bytes).map_err(resource)?;
        self.context_use.set(self.context_use.get().scope());
        let targets = self
            .execution
            .decision_stage(|context| {
                let mut targets = Vec::with_capacity(body.reference_inputs().len());
                for (reference, value) in body.reference_inputs().iter().zip(values) {
                    let relation = Read::Relation {
                        relation: reference.relation.clone(),
                        from: reference.from.clone(),
                        to: reference.to.clone(),
                    };
                    let current = self.reader.body_related_selection(
                        self.scope,
                        &relation,
                        self.execution,
                        context,
                    )?;
                    let selected = if let Some(value) = value {
                        let mut matches = self
                            .reader
                            .body_select_identity(&reference.field, value.clone())
                            .map_err(|denial| {
                                super::super::super::decision_reads::resolution_denial(
                                    denial,
                                    reference.field.field(),
                                )
                            })?;
                        if matches.len() > 1 {
                            return Err(resolution(
                                Refusal::AmbiguousEntity,
                                reference.field.field(),
                            ));
                        }
                        let identity = matches.pop().ok_or_else(|| {
                            resolution(Refusal::UnknownEntity, reference.field.field())
                        })?;
                        if !self.reader.body_field(
                            &identity,
                            &reference.field,
                            BodyFieldExpectation::Present,
                        )? {
                            return Err(resolution(
                                Refusal::UnknownEntity,
                                reference.field.field(),
                            ));
                        }
                        Some(
                            self.reader
                                .mutation_target(&identity)
                                .map_err(HandlerExecutionDenial::new)?,
                        )
                    } else {
                        None
                    };
                    targets.push(PreparedBodyReference {
                        current: current.map(|selected| selected.target),
                        selected,
                    });
                }
                Ok(targets)
            })
            .map_err(HandlerExecutionDenial::new)??;
        Ok(PreparedBodyReferences {
            targets,
            _memory: Some(memory),
        })
    }
}
fn resource(cause: WorthQueryManagedComputationResourceDenial) -> HandlerExecutionDenial {
    HandlerExecutionDenial::new(WorthQueryAdvancementDenial::Resource(cause))
}
fn resolution(kind: Refusal, subject: &str) -> HandlerExecutionDenial {
    HandlerExecutionDenial::new(WorthQueryEntityResolutionDenial::new(
        ResolutionKind::Refused(kind),
        subject,
    ))
}
