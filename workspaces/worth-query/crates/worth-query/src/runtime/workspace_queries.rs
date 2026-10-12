use super::{
    WorthQueryCountResult, WorthQueryInspection, WorthQueryInspectionTarget, WorthQueryReadBuilder,
    WorthQueryReadFamily, WorthQueryReadGraph, WorthQueryReadResult, WorthQueryRuntimeError,
    WorthQueryWorkspace,
};

impl WorthQueryWorkspace {
    pub(crate) fn open_declared_live_graph<T>(
        &mut self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        name: impl Into<String>,
        read_graph: WorthQueryReadGraph,
        authority: &super::WorthQueryGraphReadAccessAuthorityContext,
    ) -> Result<super::WorthQueryLiveView<T>, WorthQueryRuntimeError> {
        self.runtime
            .admit_facade_family(super::WorthQueryRuntimeFacadeFamily::Read)?;
        let family = WorthQueryReadFamily::new_kernel_only("declared_live_resource", read_graph);
        self.read_family_intent_in_graph_read_authority(&family, authority)
            .admit()?
            .open_managed_live(request_execution, name)
    }

    pub(crate) fn open_admitted_live_binding<T>(
        &mut self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        name: impl Into<String>,
        binding: crate::intent_admission::WorthQueryReadExecutionBinding,
    ) -> Result<super::WorthQueryLiveView<T>, WorthQueryRuntimeError> {
        self.runtime
            .declare_live_view_from_read_binding(request_execution, name, binding)
    }

    pub(crate) fn execute_declared_read_graph_in_authority(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        read_graph: WorthQueryReadGraph,
        authority: &super::WorthQueryGraphReadAccessAuthorityContext,
    ) -> Result<WorthQueryReadResult, WorthQueryRuntimeError> {
        self.runtime
            .admit_facade_family(super::WorthQueryRuntimeFacadeFamily::Read)?;
        let family = WorthQueryReadFamily::new_kernel_only("declared_read", read_graph);
        self.read_family_intent_in_graph_read_authority(&family, authority)
            .execute(execution)
    }

    pub(crate) fn execute_declared_read_graph_in_basis_context(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        read_graph: WorthQueryReadGraph,
        context: &crate::query_context::ScopedQueryBasisContext,
    ) -> Result<WorthQueryReadResult, WorthQueryRuntimeError> {
        self.runtime
            .admit_facade_family(super::WorthQueryRuntimeFacadeFamily::Read)?;
        let family = WorthQueryReadFamily::new_kernel_only("declared_historical_read", read_graph);
        self.read_family_in_basis_context_intent(&family, context)
            .execute(execution)
    }

    pub(crate) fn execute_declared_count_graph_in_authority(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        read_graph: WorthQueryReadGraph,
        authority: &super::WorthQueryGraphReadAccessAuthorityContext,
    ) -> Result<WorthQueryCountResult, WorthQueryRuntimeError> {
        self.runtime
            .admit_facade_family(super::WorthQueryRuntimeFacadeFamily::Read)?;
        let family = WorthQueryReadFamily::new_kernel_only("declared_count", read_graph);
        self.read_family_intent_in_graph_read_authority(&family, authority)
            .admit()?
            .execute_count(execution)
    }

    pub(crate) fn compose_read(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        declaration: impl FnOnce(
            WorthQueryReadBuilder,
        ) -> Result<WorthQueryReadGraph, super::WorthQueryReadDenial>,
    ) -> Result<WorthQueryReadResult, WorthQueryRuntimeError> {
        self.runtime
            .admit_facade_family(super::WorthQueryRuntimeFacadeFamily::Read)?;
        let read_graph = declaration(WorthQueryReadBuilder::new())
            .map_err(WorthQueryRuntimeError::ReadCompositionDenied)?;
        let family = WorthQueryReadFamily::new_kernel_only("composed_read", read_graph);
        self.read_family_intent(&family).execute(execution)
    }

    pub(crate) fn define_read_family(
        &mut self,
        family_name: impl Into<String>,
        declaration: impl FnOnce(
            WorthQueryReadBuilder,
        ) -> Result<WorthQueryReadGraph, super::WorthQueryReadDenial>,
    ) -> Result<WorthQueryReadFamily, WorthQueryRuntimeError> {
        self.runtime
            .admit_facade_family(super::WorthQueryRuntimeFacadeFamily::Read)?;
        let read_graph = declaration(WorthQueryReadBuilder::new())
            .map_err(WorthQueryRuntimeError::ReadCompositionDenied)?;
        Ok(WorthQueryReadFamily::new_kernel_only(
            family_name,
            read_graph,
        ))
    }

    pub(crate) fn execute_read_family(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        family: &WorthQueryReadFamily,
    ) -> Result<WorthQueryReadResult, WorthQueryRuntimeError> {
        self.runtime
            .admit_facade_family(super::WorthQueryRuntimeFacadeFamily::Read)?;
        self.read_family_intent(family).execute(execution)
    }

    #[cfg(test)]
    pub(crate) fn execute_read_family_with_access_plan(
        &mut self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        family: &WorthQueryReadFamily,
        graph_read_access_plan: super::WorthQueryAdmittedGraphReadAccessPlan,
    ) -> Result<WorthQueryReadResult, WorthQueryRuntimeError> {
        self.runtime
            .admit_facade_family(super::WorthQueryRuntimeFacadeFamily::Read)?;
        let handoff = self
            .read_family_intent(family)
            .review()?
            .admitted_handoff()
            .ok_or_else(|| {
                WorthQueryRuntimeError::ReadCompositionDenied(super::WorthQueryReadDenial::new(
                    super::WorthQueryReadDenialKind::BasisPreflightDenied,
                    "read family was not admitted for access-plan execution",
                ))
            })?;
        let binding = self.into_runtime_read_execution_binding_with_access_plan(
            handoff,
            graph_read_access_plan,
        )?;
        self.execute_bound_read_execution(request_execution, binding)
    }

    pub(crate) fn execute_read_family_in_basis_context(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        family: &WorthQueryReadFamily,
        context: &crate::query_context::ScopedQueryBasisContext,
    ) -> Result<WorthQueryReadResult, WorthQueryRuntimeError> {
        self.runtime
            .admit_facade_family(super::WorthQueryRuntimeFacadeFamily::Read)?;
        self.read_family_in_basis_context_intent(family, context)
            .execute(execution)
    }

    pub fn explain_graph_read_access_shape(
        &self,
        family: &WorthQueryReadFamily,
    ) -> Result<
        super::WorthQueryGraphReadAccessShapeExplanation,
        super::WorthQueryGraphReadAccessShapeExplanationError,
    > {
        let authority =
            super::WorthQueryGraphReadAccessAuthorityContext::runtime_current_compatibility();
        super::explain_graph_read_access_shape_for_family_in_authority_with_lookup(
            family,
            &authority,
            self.runtime.installed_domain_execution_index(),
        )
    }

    #[cfg(test)]
    pub(crate) fn admit_graph_read_access_authority(
        &self,
        request: super::WorthQueryGraphReadAccessAuthorityRequest,
    ) -> Result<
        super::WorthQueryGraphReadAccessAuthorityContext,
        super::WorthQueryGraphReadAccessAuthorityDenial,
    > {
        super::admit_graph_read_access_authority(request)
    }

    #[cfg(test)]
    pub(crate) fn admit_graph_read_access_authority_from_policy_tenant_request(
        &self,
        request: super::WorthQueryGraphReadPolicyTenantAuthorityRequest,
    ) -> Result<
        super::WorthQueryGraphReadAccessAuthorityContext,
        super::WorthQueryGraphReadAccessAuthorityDenial,
    > {
        super::admit_graph_read_access_authority_from_policy_tenant_request(request)
    }

    pub fn explain_graph_read_access_shape_in_authority(
        &self,
        family: &WorthQueryReadFamily,
        authority: &super::WorthQueryGraphReadAccessAuthorityContext,
    ) -> Result<
        super::WorthQueryGraphReadAccessShapeExplanation,
        super::WorthQueryGraphReadAccessShapeExplanationError,
    > {
        super::explain_graph_read_access_shape_for_family_in_authority_with_lookup(
            family,
            authority,
            self.runtime.installed_domain_execution_index(),
        )
    }

    #[cfg(test)]
    pub(crate) fn admit_graph_read_access_in_authority(
        &self,
        family: &WorthQueryReadFamily,
        authority: &super::WorthQueryGraphReadAccessAuthorityContext,
    ) -> Result<
        super::WorthQueryGraphReadAccessAdmission,
        super::WorthQueryGraphReadAccessShapeExplanationError,
    > {
        self.admit_graph_read_access_for_family_in_authority(family, authority)
    }

    pub fn explain_boolean_selectivity_shape(
        &self,
        family: &WorthQueryReadFamily,
    ) -> Result<
        super::WorthQueryBooleanSelectivityShape,
        super::WorthQueryGraphReadAccessShapeExplanationError,
    > {
        let authority =
            super::WorthQueryGraphReadAccessAuthorityContext::runtime_current_compatibility();
        super::explain_boolean_selectivity_shape_for_family_in_authority_with_lookup(
            family,
            &authority,
            self.runtime.installed_domain_execution_index(),
        )
    }

    #[cfg(test)]
    pub(crate) fn plan_live_graph_read_access(
        &self,
        family: &WorthQueryReadFamily,
        budget: super::WorthQueryLiveGraphReadMaintenanceBudget,
    ) -> Result<super::WorthQueryLiveGraphReadAccessPlan, super::WorthQueryLiveGraphReadAccessDenial>
    {
        let one_shot = match self.runtime.admit_graph_read_access_for_family(family) {
            Ok(admission) => {
                super::WorthQueryAdmittedGraphReadAccessPlan::from_admission(admission)
            }
            Err(error) => {
                return Err(super::WorthQueryLiveGraphReadAccessDenial::new(
                    super::WorthQueryLiveGraphReadAccessPosture::DeniedLiveMaintenanceSupport,
                    family.family_digest(),
                    &budget,
                    format!("live graph read access planning could not derive one-shot access: {error:?}"),
                ));
            }
        };
        let one_shot = match one_shot {
            None => {
                return Err(super::WorthQueryLiveGraphReadAccessDenial::new(
                    super::WorthQueryLiveGraphReadAccessPosture::DeniedLiveMaintenanceSupport,
                    family.family_digest(),
                    &budget,
                    "live graph read access planning requires an admitted one-shot access plan",
                ));
            }
            Some(one_shot) => one_shot,
        };
        super::WorthQueryLiveGraphReadAccessPlan::from_one_shot_access_plan(&one_shot, budget)
    }

    pub(crate) fn inspect<'a, T>(
        &'a self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        target: T,
    ) -> Result<WorthQueryInspection, WorthQueryRuntimeError>
    where
        T: Into<WorthQueryInspectionTarget<'a>>,
    {
        self.runtime.inspect(execution, target)
    }
}
