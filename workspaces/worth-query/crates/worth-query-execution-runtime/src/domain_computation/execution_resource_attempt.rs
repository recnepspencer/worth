use worth_query_admission::facade::resource_admission::{
    WorthQueryAdmittedExecutionResourcePlan, WorthQueryAdmittedWorkflowResourcePlan,
    WorthQueryExecutionResourceAdmissionDenial, WorthQueryExecutionResourceAdmissionDenialKind,
};
use worth_query_admission::integration::{
    reserve_execution_resource_plan, reserve_workflow_resource_plan,
};
use worth_query_installation::facade::WorthQueryDomainHandleDenialKind;

use super::{
    WorthQueryDirectExecutionResourceAttempt, WorthQueryExecutionBoundOperationAuthority,
    WorthQueryExecutionRuntime, WorthQueryWorkflowExecutionResourceAttempt,
};

impl WorthQueryExecutionRuntime {
    pub(crate) fn start_reserved_direct_resource_attempt(
        &self,
        authority: &WorthQueryExecutionBoundOperationAuthority,
        resources: worth_query_admission::integration::WorthQueryCapacityReservedExecutionResourcePlan,
    ) -> Result<WorthQueryDirectExecutionResourceAttempt, WorthQueryExecutionResourceAdmissionDenial>
    {
        self.validate_bound_authority(authority, resources.resources().counters())?;
        if !authority.admits_direct_plan(resources.resources()) {
            return Err(resource_plan_authority_mismatch(
                resources.resources().counters(),
            ));
        }
        Ok(WorthQueryDirectExecutionResourceAttempt::start(
            resources, authority,
        ))
    }

    pub fn start_direct_resource_attempt(
        &self,
        authority: &WorthQueryExecutionBoundOperationAuthority,
        resources: WorthQueryAdmittedExecutionResourcePlan,
    ) -> Result<WorthQueryDirectExecutionResourceAttempt, WorthQueryExecutionResourceAdmissionDenial>
    {
        self.validate_bound_authority(authority, resources.counters())?;
        if !authority.admits_direct_plan(&resources) {
            return Err(resource_plan_authority_mismatch(resources.counters()));
        }
        let mut counters = resources.counters();
        counters.capacity_reservation_checks += 1;
        let resources =
            reserve_execution_resource_plan(resources).ok_or_else(|| backpressured(counters))?;
        Ok(WorthQueryDirectExecutionResourceAttempt::start(
            resources, authority,
        ))
    }

    pub fn start_workflow_resource_attempt(
        &self,
        authority: &WorthQueryExecutionBoundOperationAuthority,
        resources: WorthQueryAdmittedWorkflowResourcePlan,
    ) -> Result<
        WorthQueryWorkflowExecutionResourceAttempt,
        WorthQueryExecutionResourceAdmissionDenial,
    > {
        self.validate_bound_authority(authority, resources.counters())?;
        if !authority.admits_workflow_plan(&resources) {
            return Err(resource_plan_authority_mismatch(resources.counters()));
        }
        let mut counters = resources.counters();
        counters.capacity_reservation_checks += 1;
        let resources =
            reserve_workflow_resource_plan(resources).ok_or_else(|| backpressured(counters))?;
        Ok(WorthQueryWorkflowExecutionResourceAttempt::start(
            resources, authority,
        ))
    }

    fn validate_bound_authority(
        &self,
        authority: &WorthQueryExecutionBoundOperationAuthority,
        counters: worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceAdmissionCounters,
    ) -> Result<(), WorthQueryExecutionResourceAdmissionDenial> {
        if authority.belongs_to(self) {
            if authority.belongs_to_current_installation(self) {
                Ok(())
            } else {
                Err(WorthQueryExecutionResourceAdmissionDenial::new(
                    WorthQueryExecutionResourceAdmissionDenialKind::RuntimeAuthority(
                        WorthQueryDomainHandleDenialKind::StaleInstallationGeneration,
                    ),
                    "bound operation belongs to a stale installation generation",
                    counters,
                ))
            }
        } else {
            Err(WorthQueryExecutionResourceAdmissionDenial::new(
                WorthQueryExecutionResourceAdmissionDenialKind::ForeignExecutionRuntime,
                "bound operation belongs to a different execution runtime",
                counters,
            ))
        }
    }
}

fn resource_plan_authority_mismatch(
    counters: worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceAdmissionCounters,
) -> WorthQueryExecutionResourceAdmissionDenial {
    WorthQueryExecutionResourceAdmissionDenial::new(
        WorthQueryExecutionResourceAdmissionDenialKind::ResourcePlanAuthorityMismatch,
        "admitted resource plan does not belong to the exact bound operation and resource contract",
        counters,
    )
}

fn backpressured(
    counters: worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceAdmissionCounters,
) -> WorthQueryExecutionResourceAdmissionDenial {
    WorthQueryExecutionResourceAdmissionDenial::new(
        WorthQueryExecutionResourceAdmissionDenialKind::Backpressured,
        "execution resource capacity is saturated",
        counters,
    )
}
