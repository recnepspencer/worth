//! Unconsumed input planning selects real Native readiness before borrowing a handler.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::coupled_outputs::DeclaredCoupledOutputDemand;
use worth_relational::facade::runtime::RelationalPinnedRead;

pub enum PreparedCoupledInputs {
    Ready(SelectedCoupledSource),
    Captured(Box<dyn execution::CapturedProducerInputRead>),
}
pub enum PlannedOutput {
    Prepared(Option<synchronous_outputs::AppliedOutputParent>),
    Recapture,
}
impl PreparedCoupledInputs {
    pub fn support_equal(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        pin: &RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
    ) -> Result<bool, CoupledSourceReadStop> {
        match self {
            Self::Ready(_) => Ok(true),
            Self::Captured(inputs) => inputs
                .support_equal(execution, pin)
                .map_err(CoupledSourceReadStop::capture),
        }
    }
}
impl<S: ApplicationSchema> CoupledApplicationAssembly<'_, '_, '_, S> {
    pub fn capture_inputs_one(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        before: &RelationalPinnedRead,
        demand: &DeclaredCoupledOutputDemand,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
        component: std::any::TypeId,
    ) -> Result<Option<PreparedCoupledInputs>, CoupledSourceReadStop> {
        let registry = &self.application.installed_producers;
        if registry
            .declared_coupled_source(demand)?
            .declaration
            .input_capture
            .is_none()
        {
            return Ok(None);
        }
        let selected = self.select_one(execution, before, demand, candidate)?;
        let execution::CoupledSourceRead::Source(source) = selected.read else {
            return Ok(Some(PreparedCoupledInputs::Ready(selected)));
        };
        let producer =
            registry.declared_coupled_producer(demand, source.profile, source.lifecycle)?;
        // A reopened route's capture reads its own chain's records as before
        // the chain (§1.5), like the round it captures for.
        let chain = self.continued_chain(
            execution,
            candidate,
            demand.family,
            source.scope,
            &source.partition,
        )?;
        let principal = self.principal.ok_or_else(|| {
            WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                demand.family,
            )
        })?;
        let inputs = producer.coupled_executor().capture_coupled_inputs(
            self.phase,
            self.application,
            principal,
            self.request,
            self.before,
            before,
            &selected.pin,
            source,
            producer.edition,
            component,
            chain.as_ref().map(|chain| &chain.own),
        )?;
        Ok(Some(PreparedCoupledInputs::Captured(inputs)))
    }
    pub fn prepare_planned_one(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        before: &RelationalPinnedRead,
        demand: &DeclaredCoupledOutputDemand,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
        inputs: Option<PreparedCoupledInputs>,
        component: Option<std::any::TypeId>,
    ) -> Result<PlannedOutput, CoupledSourceReadStop> {
        let Some(inputs) = inputs else {
            return self
                .prepare_one(execution, before, demand, candidate)
                .map(PlannedOutput::Prepared);
        };
        let current = candidate.pin_source(execution)?;
        let (read, captured) = match inputs {
            PreparedCoupledInputs::Ready(selected) => {
                let read = if selected.pin.candidate_identity().qualification()
                    == current.candidate_identity().qualification()
                {
                    selected.read
                } else {
                    self.read_source_one(execution, before, demand, candidate)?
                };
                if matches!(read, execution::CoupledSourceRead::Source(_)) {
                    return Ok(PlannedOutput::Recapture);
                }
                (read, None)
            }
            PreparedCoupledInputs::Captured(inputs) => {
                if !inputs
                    .support_equal(execution, &current)
                    .map_err(CoupledSourceReadStop::capture)?
                {
                    return Ok(PlannedOutput::Recapture);
                }
                let read = self.read_source_one(execution, before, demand, candidate)?;
                let captured = if let execution::CoupledSourceRead::Source(source) = &read {
                    let producer = self
                        .application
                        .installed_producers
                        .declared_coupled_producer(demand, source.profile, source.lifecycle)?;
                    if producer.declaration.identity != inputs.producer() {
                        return Ok(PlannedOutput::Recapture);
                    }
                    let component = component.ok_or_else(|| {
                        WorthQueryOutputDemandDenial::new(
                            WorthQueryOutputDemandDenialKind::ForeignSource,
                            demand.family,
                        )
                    })?;
                    Some((inputs, component))
                } else {
                    None
                };
                (read, captured)
            }
        };
        self.prepare_read_one(execution, before, demand, candidate, read, captured)
            .map(PlannedOutput::Prepared)
    }
}
