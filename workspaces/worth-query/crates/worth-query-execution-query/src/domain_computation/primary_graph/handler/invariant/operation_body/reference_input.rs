//! The compiled reference uses the same native indexed selection and adjacency owners.
use super::*;
use crate::domain_computation::primary_graph::{
    operation_body::reference_input::{PreparedBodyReference, PreparedBodyReferences},
    operation_body::{PreparedBodyInputs, PreparedBodySelection},
    WorthQueryApplicationAttemptDenial as Denial, WorthQueryApplicationAttemptDenialKind as Kind,
    WorthQueryEntityResolutionDenial, WorthQueryEntityResolutionDenialKind as ResolutionKind,
    WorthQueryEntityResolutionRefusalKind as Refusal, WorthQueryInvariantEntityIdentity,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationBodyProgram, ApplicationOperationBodyReferenceInput as Reference,
    ApplicationOperationDecisionReadTarget as Read,
};
impl<S: ApplicationSchema, B: ApplicationMutationBinding<S>> DecisionReader<'_, '_, '_, S, B> {
    pub(crate) fn body_references(
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
        let missing = || {
            HandlerExecutionDenial::new(Denial::new(
                Kind::MissingAuthoritativeFact,
                "reference input",
            ))
        };
        let inputs = inputs.ok_or_else(missing)?;
        let first = (inputs.values.len())
            .checked_sub(body.reference_inputs().len())
            .ok_or_else(missing)?;
        let values = &inputs.values[first..];
        if inputs.reference_linked.len() != values.len() {
            return Err(missing());
        }
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
        let execution = self.execution;
        let targets = execution
            .decision_stage(|context| {
                let mut targets = Vec::with_capacity(body.reference_inputs().len());
                let selections = body
                    .reference_inputs()
                    .iter()
                    .zip(values)
                    .zip(&inputs.reference_linked);
                for ((reference, value), linked) in selections {
                    let relation = Read::Relation {
                        relation: reference.relation.clone(),
                        from: reference.from.clone(),
                        to: reference.to.clone(),
                    };
                    let selected = match value {
                        Some(value) => Some(self.body_reference_target(reference, value)?),
                        None => None,
                    };
                    targets.push(match (linked, selected) {
                        (Some(linked), Some(selected)) => {
                            self.body_toggle_endpoints(&relation, &selected, *linked, context)?
                        }
                        (None, selected) => {
                            self.body_replacement_endpoints(&relation, selected, context)?
                        }
                        // A toggle captures a required key; prepared inputs never omit it.
                        (Some(_), None) => return Err(missing()),
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

    /// Resolves the one present entity the captured identity key names.
    fn body_reference_target(
        &mut self,
        reference: &Reference,
        value: &AspectValue,
    ) -> Result<WorthQueryInvariantEntityIdentity<S, ()>, HandlerExecutionDenial> {
        let subject = reference.field.field();
        let mut matches = self
            .reader
            .body_select_identity(&reference.field, value.clone())
            .map_err(|denial| {
                super::super::super::decision_reads::resolution_denial(denial, subject)
            })?;
        if matches.len() > 1 {
            return Err(resolution(Refusal::AmbiguousEntity, subject));
        }
        let identity = matches
            .pop()
            .ok_or_else(|| resolution(Refusal::UnknownEntity, subject))?;
        if !self
            .reader
            .body_field(&identity, &reference.field, BodyFieldExpectation::Present)?
        {
            return Err(resolution(Refusal::UnknownEntity, subject));
        }
        Ok(identity)
    }

    /// Nullable replacement: the current at-most-one selection versus the input's target.
    fn body_replacement_endpoints(
        &mut self,
        relation: &Read,
        selected: Option<WorthQueryInvariantEntityIdentity<S, ()>>,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<PreparedBodyReference<S>, HandlerExecutionDenial> {
        let current =
            self.reader
                .body_related_selection(self.scope, relation, self.execution, context)?;
        Ok(PreparedBodyReference {
            current: current.map(|current| current.target),
            selected: selected
                .map(|identity| self.reader.mutation_target(&identity))
                .transpose()
                .map_err(HandlerExecutionDenial::new)?,
        })
    }

    /// Exact toggle: the edge to `target` is current when present and selected when
    /// `linked`, so equal endpoints leave the graph unchanged.
    fn body_toggle_endpoints(
        &mut self,
        relation: &Read,
        target: &WorthQueryInvariantEntityIdentity<S, ()>,
        linked: bool,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<PreparedBodyReference<S>, HandlerExecutionDenial> {
        let present = self.reader.body_related_contains(
            self.scope,
            relation,
            target,
            self.execution,
            context,
        )?;
        let endpoint = |reader: &mut Self, wanted: bool| {
            wanted
                .then(|| reader.reader.mutation_target(target))
                .transpose()
                .map_err(HandlerExecutionDenial::new)
        };
        Ok(PreparedBodyReference {
            current: endpoint(self, present)?,
            selected: endpoint(self, linked)?,
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
