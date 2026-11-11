//! A declaration supplies producers; canonical Native facts decide the trigger.
use super::*;
use worth_relational::facade::mvcc::{
    RelationalAuthoredCoupledOwners, RelationalAuthoredCoupledProducerAssembly,
};

pub(in crate::domain_computation::primary_graph) trait CoupledProducerAssembly:
    RelationalAuthoredCoupledProducerAssembly
{
    fn prepared_members(&self) -> Vec<super::super::CoupledApplicationMember>;
    fn admit_validated(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        provider: &WorthQueryPrimaryGraphProvider,
        candidate: &worth_relational::facade::mvcc::ValidatedRelationalProposal,
    ) -> Result<(), WorthQueryInvariantExecutionFailure>;
}

pub(super) struct NativeAuthoredCoupling<'a> {
    pub(super) owners: RelationalAuthoredCoupledOwners,
    pub(super) assembly: &'a dyn RelationalAuthoredCoupledProducerAssembly,
}
impl WorthQueryPrimaryGraphProvider {
    pub(super) fn native_authored_coupling<'a>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        session: WorthQueryProviderSessionView<'_>,
        material: &mut ApplicationInvariantCandidateMaterial,
        assembly: Option<&'a dyn RelationalAuthoredCoupledProducerAssembly>,
    ) -> Result<Option<NativeAuthoredCoupling<'a>>, WorthQueryInvariantExecutionFailure> {
        let Some(publisher) = &self.graph.application_publication else {
            return Ok(None);
        };
        let owners = match &material.restoration_plan {
            Some(restoration) => self
                .graph
                .with_runtime(execution, |runtime| {
                    runtime.readmit_authored_restoration_coupled_owners(
                        execution,
                        &publisher.native,
                        restoration.clone(),
                    )
                })
                .map_err(map_validation_failure)?,
            None => {
                if assembly.is_none() {
                    return Ok(None);
                }
                let selected = {
                    let attempts = self.attempts.lock().unwrap_or_else(|p| p.into_inner());
                    let staged = attempts
                        .staged_attempt(session)
                        .ok_or_else(missing_candidate_failure)?;
                    super::super::application_attempt_state::coupled_publication::authored_retirement_input(&self.graph.layout, &staged)?
                };
                let Some((anchor, co_anchors, roles)) = selected else {
                    return Ok(None);
                };
                Some(
                    self.graph
                        .with_runtime(execution, |runtime| {
                            runtime.readmit_authored_coupled_owners(
                                execution,
                                &publisher.native,
                                material.product.observation().basis().relational_basis(),
                                anchor,
                                &co_anchors,
                                &roles,
                            )
                        })
                        .map_err(map_validation_failure)?,
                )
            }
        };
        let Some(owners) = owners.filter(|owners| owners.required_owner_count() != 0) else {
            // A declaration does not run a producer when Native found no live
            // surviving writer. The established single/cascade path owns this edit.
            return Ok(None);
        };
        let Some(assembly) = assembly else {
            owners
                .require_independent()
                .map_err(map_validation_failure)?;
            unreachable!("nonempty Native owner set requires declared coupling");
        };
        material.restoration_plan = None;
        material.restoration_publication = None;
        Ok(Some(NativeAuthoredCoupling { owners, assembly }))
    }
}
