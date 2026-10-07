use super::read_composition_current_execution::{
    execute_runtime_current_count_graph, execute_runtime_current_read_graph,
};
use super::read_composition_runtime::{
    execute_runtime_basis_context_read_graph, WorthQueryExecutedReadProduct,
};
use super::runtime_read_execution_receipts::{
    attach_graph_read_access_receipt, attach_read_intent_execution_evidence,
    provision_graph_indexes_for_read_binding,
};
use super::*;
use crate::intent_admission::dx::{
    non_admitted_runtime_violation, WorthQueryRuntimeIntentAdmissionReviewData,
};
use crate::intent_admission::{
    WorthQueryIntentAdmissionDecision, WorthQueryReadExecutionBinding,
    WorthQueryReadExecutionHandoff, WorthQueryReadExecutionIntentSeed,
};
use crate::query_context::ScopedQueryBasisContext;

impl WorthQueryWorkspace {
    pub fn read_family_intent(
        &mut self,
        family: &WorthQueryReadFamily,
    ) -> crate::intent_admission::WorthQueryWorkspaceReadIntentAuthoring<'_> {
        crate::intent_admission::WorthQueryWorkspaceReadIntentAuthoring::new(
            self,
            family.clone(),
            None,
            None,
        )
    }

    pub fn read_family_in_basis_context_intent(
        &mut self,
        family: &WorthQueryReadFamily,
        context: &ScopedQueryBasisContext,
    ) -> crate::intent_admission::WorthQueryWorkspaceReadIntentAuthoring<'_> {
        crate::intent_admission::WorthQueryWorkspaceReadIntentAuthoring::new(
            self,
            family.clone(),
            Some(context.clone()),
            None,
        )
    }

    pub fn read_family_intent_in_graph_read_authority(
        &mut self,
        family: &WorthQueryReadFamily,
        authority: &WorthQueryGraphReadAccessAuthorityContext,
    ) -> crate::intent_admission::WorthQueryWorkspaceReadIntentAuthoring<'_> {
        crate::intent_admission::WorthQueryWorkspaceReadIntentAuthoring::new(
            self,
            family.clone(),
            None,
            Some(authority.clone()),
        )
    }

    pub(crate) fn review_read_execution(
        &self,
        family: WorthQueryReadFamily,
        basis_context: Option<ScopedQueryBasisContext>,
    ) -> Result<WorthQueryRuntimeIntentAdmissionReviewData, WorthQueryRuntimeError> {
        self.runtime
            .review_runtime_read_execution(family, basis_context)
    }

    pub(crate) fn resolve_reviewed_admitted_read_execution_handoff(
        &self,
        review: WorthQueryRuntimeIntentAdmissionReviewData,
    ) -> Result<WorthQueryReadExecutionHandoff, WorthQueryRuntimeError> {
        self.runtime
            .resolve_reviewed_admitted_read_execution_handoff(review)
    }

    pub(crate) fn read_execution_non_admitted_error(
        &self,
        review: &WorthQueryRuntimeIntentAdmissionReviewData,
    ) -> WorthQueryRuntimeError {
        self.runtime.read_execution_non_admitted_error(review)
    }

    pub(crate) fn into_runtime_read_execution_binding(
        &self,
        handoff: WorthQueryReadExecutionHandoff,
    ) -> Result<WorthQueryReadExecutionBinding, WorthQueryRuntimeError> {
        self.runtime.prepare_read_execution_binding(handoff, None)
    }

    pub(crate) fn into_runtime_read_execution_binding_in_authority(
        &self,
        handoff: WorthQueryReadExecutionHandoff,
        authority: Option<&WorthQueryGraphReadAccessAuthorityContext>,
    ) -> Result<WorthQueryReadExecutionBinding, WorthQueryRuntimeError> {
        self.runtime
            .prepare_read_execution_binding(handoff, authority)
    }

    #[cfg(test)]
    pub(crate) fn into_runtime_read_execution_binding_with_access_plan(
        &self,
        handoff: WorthQueryReadExecutionHandoff,
        graph_read_access_plan: WorthQueryAdmittedGraphReadAccessPlan,
    ) -> Result<WorthQueryReadExecutionBinding, WorthQueryRuntimeError> {
        self.runtime
            .prepare_read_execution_binding_with_access_plan(handoff, graph_read_access_plan)
    }

    pub(crate) fn execute_bound_read_execution(
        &mut self,
        binding: WorthQueryReadExecutionBinding,
    ) -> Result<WorthQueryReadResult, WorthQueryRuntimeError> {
        self.runtime.execute_read_execution_binding(binding)
    }

    pub(crate) fn execute_bound_count_read_execution(
        &mut self,
        binding: WorthQueryReadExecutionBinding,
    ) -> Result<WorthQueryCountResult, WorthQueryRuntimeError> {
        self.runtime.execute_count_read_execution_binding(binding)
    }
}

impl WorthQueryRuntime {
    pub(crate) fn review_runtime_read_execution(
        &self,
        family: WorthQueryReadFamily,
        basis_context: Option<ScopedQueryBasisContext>,
    ) -> Result<WorthQueryRuntimeIntentAdmissionReviewData, WorthQueryRuntimeError> {
        self.admit_facade_family(WorthQueryRuntimeFacadeFamily::Read)?;
        let request = match basis_context {
            Some(context) => {
                let seed = WorthQueryReadExecutionIntentSeed::in_basis_context(family, context);
                crate::intent_admission::WorthQueryRawIntentAdmissionRequest::read_family_in_basis_context_entrypoint(seed)
            }
            None => {
                let seed = WorthQueryReadExecutionIntentSeed::current_runtime(family);
                crate::intent_admission::WorthQueryRawIntentAdmissionRequest::read_family_entrypoint(seed)
            }
        }
        .map_err(|violation| WorthQueryRuntimeError::ReadCompositionDenied(
            WorthQueryReadDenial::new(WorthQueryReadDenialKind::AuthoringDenied, violation.message()),
        ))?;
        Ok(WorthQueryRuntimeIntentAdmissionReviewData::from_request(
            request,
        ))
    }

    pub(crate) fn resolve_reviewed_admitted_read_execution_handoff(
        &self,
        review: WorthQueryRuntimeIntentAdmissionReviewData,
    ) -> Result<WorthQueryReadExecutionHandoff, WorthQueryRuntimeError> {
        match review.decision().clone() {
            WorthQueryIntentAdmissionDecision::Admitted(
                crate::intent_admission::WorthQueryAdmittedIntentPlan::ReadExecution(plan),
            ) => Ok(WorthQueryReadExecutionHandoff::from_plan(plan)),
            WorthQueryIntentAdmissionDecision::Admitted(_)
            | WorthQueryIntentAdmissionDecision::Advisory(_)
            | WorthQueryIntentAdmissionDecision::Violation(_) => {
                Err(self.read_execution_non_admitted_error(&review))
            }
        }
    }

    pub(crate) fn read_execution_non_admitted_error(
        &self,
        review: &WorthQueryRuntimeIntentAdmissionReviewData,
    ) -> WorthQueryRuntimeError {
        let violation = non_admitted_runtime_violation(review);
        WorthQueryRuntimeError::ReadCompositionDenied(WorthQueryReadDenial::new(
            WorthQueryReadDenialKind::BasisPreflightDenied,
            violation.message(),
        ))
    }

    pub(crate) fn prepare_read_execution_binding(
        &self,
        handoff: WorthQueryReadExecutionHandoff,
        graph_read_authority: Option<&WorthQueryGraphReadAccessAuthorityContext>,
    ) -> Result<WorthQueryReadExecutionBinding, WorthQueryRuntimeError> {
        let graph_read_access_plan =
            self.admit_graph_read_access_plan_for_handoff(&handoff, graph_read_authority)?;
        Ok(WorthQueryReadExecutionBinding::from_handoff(
            handoff,
            graph_read_access_plan,
        ))
    }

    #[cfg(test)]
    pub(crate) fn prepare_read_execution_binding_with_access_plan(
        &self,
        handoff: WorthQueryReadExecutionHandoff,
        graph_read_access_plan: WorthQueryAdmittedGraphReadAccessPlan,
    ) -> Result<WorthQueryReadExecutionBinding, WorthQueryRuntimeError> {
        validate_graph_read_access_plan_matches_handoff(&handoff, &graph_read_access_plan)?;
        Ok(WorthQueryReadExecutionBinding::from_handoff(
            handoff,
            graph_read_access_plan,
        ))
    }

    pub(crate) fn execute_read_execution_binding(
        &mut self,
        binding: WorthQueryReadExecutionBinding,
    ) -> Result<WorthQueryReadResult, WorthQueryRuntimeError> {
        self.execute_read_execution_product(binding, Self::execute_graph_read_binding)
    }

    pub(crate) fn execute_count_read_execution_binding(
        &mut self,
        binding: WorthQueryReadExecutionBinding,
    ) -> Result<WorthQueryCountResult, WorthQueryRuntimeError> {
        self.execute_read_execution_product(binding, Self::execute_count_graph_read_binding)
    }

    fn execute_read_execution_product<Product>(
        &mut self,
        binding: WorthQueryReadExecutionBinding,
        execute: impl FnOnce(
            &mut Self,
            &WorthQueryReadExecutionBinding,
        )
            -> Result<WorthQueryExecutedReadProduct<Product>, WorthQueryRuntimeError>,
    ) -> Result<Product, WorthQueryRuntimeError>
    where
        Product: WorthQueryReadExecutionProduct,
    {
        self.admit_facade_family(WorthQueryRuntimeFacadeFamily::Read)?;
        let snapshot_identity = self.current_snapshot_identity()?.evidence_identity();
        let ephemeral_graph_index_receipt =
            provision_graph_indexes_for_read_binding(&binding, snapshot_identity.as_str())?;
        let mut executed_read = execute(self, &binding)?;
        executed_read.record_ephemeral_index_receipt(ephemeral_graph_index_receipt.as_ref());

        attach_graph_read_access_receipt(
            &mut executed_read,
            &binding,
            snapshot_identity.as_str(),
            ephemeral_graph_index_receipt,
        );
        attach_read_intent_execution_evidence(&mut executed_read, &binding, &snapshot_identity);

        Ok(executed_read.into_product())
    }

    fn execute_graph_read_binding(
        &mut self,
        binding: &WorthQueryReadExecutionBinding,
    ) -> Result<super::read_composition_runtime::WorthQueryExecutedReadGraph, WorthQueryRuntimeError>
    {
        match binding.basis_context() {
            Some(context) => execute_runtime_basis_context_read_graph(
                self,
                binding.read_family().read_graph(),
                context,
            ),
            None => execute_runtime_current_read_graph(self, binding.read_family().read_graph()),
        }
        .map_err(WorthQueryRuntimeError::ReadCompositionDenied)
    }

    fn execute_count_graph_read_binding(
        &mut self,
        binding: &WorthQueryReadExecutionBinding,
    ) -> Result<super::read_composition_runtime::WorthQueryExecutedCountGraph, WorthQueryRuntimeError>
    {
        if binding.basis_context().is_some() {
            return Err(WorthQueryRuntimeError::ReadCompositionDenied(
                WorthQueryReadDenial::new(
                    WorthQueryReadDenialKind::BasisPreflightDenied,
                    "count aggregate execution does not yet admit reusable basis contexts",
                ),
            ));
        }
        execute_runtime_current_count_graph(self, binding.read_family().read_graph())
            .map_err(WorthQueryRuntimeError::ReadCompositionDenied)
    }
}

#[cfg(test)]
fn validate_graph_read_access_plan_matches_handoff(
    handoff: &WorthQueryReadExecutionHandoff,
    graph_read_access_plan: &WorthQueryAdmittedGraphReadAccessPlan,
) -> Result<(), WorthQueryRuntimeError> {
    let planned_read_graph_digest = graph_read_access_plan
        .admission()
        .requirement_set()
        .read_graph_digest();
    let handoff_read_graph = handoff.read_family().read_graph();
    if &handoff_read_graph.evidence_identity().canonical_digest_id() == planned_read_graph_digest {
        return Ok(());
    }
    let planned_read_graph_digest = planned_read_graph_digest.render_hex();
    let handoff_read_graph_digest = handoff_read_graph.digest();
    Err(WorthQueryRuntimeError::ReadCompositionDenied(
        WorthQueryReadDenial::new(
            WorthQueryReadDenialKind::BasisPreflightDenied,
            format!(
                "graph read access plan was admitted for read graph `{planned_read_graph_digest}` but execution handoff carries `{handoff_read_graph_digest}`"
            ),
        )
        .with_access_plan_binding_mismatch(WorthQueryReadAccessPlanBindingMismatch::new(
            &planned_read_graph_digest,
            handoff_read_graph_digest,
            graph_read_access_plan.digest(),
            graph_read_access_plan.admission().digest(),
        )),
    ))
}
