//! Implementations of `worth-query-execution-attempt` runtime method ports.

use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_execution_attempt::facade::ports::{
    EffectProgramOriginalInputRuntimePort, ProducerIdentityRuntimePort,
    SelectedProductOperationPort, WorkflowOperationAuthorityRuntimePort,
};
use worth_query_execution_runtime::facade::ports::AdvancementPhaseInternalPort as _;
use worth_query_installation::facade::{ApplicationSchema, WorthQueryInstalledApplicationSchema};

use crate::basis::WorthQueryProductBranchReadIdentity;
use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;
use crate::domain_computation::primary_graph::application_attempt::check_request_live;
use crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider;
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementPhase, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationAttemptDenialKind, WorthQueryPrimaryGraphApplicationRuntime,
    WorthQuerySelectedProductOperation,
};

impl<Schema> WorkflowOperationAuthorityRuntimePort<Schema>
    for WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    fn validate_workflow_operation_authority(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryRequestScope,
        runtime_authority: u64,
        observation: &WorthQueryProductBranchReadIdentity,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let phase_request = match phase.execution_request_for(&self.runtime.product_runtime) {
            Ok(request) => request,
            Err(_) => {
                return Err(WorthQueryApplicationAttemptDenial::new(
                    WorthQueryApplicationAttemptDenialKind::ForeignPhase,
                    "application-read-phase",
                ))
            }
        };
        let execution = &phase_request;
        let mismatch = || {
            WorthQueryApplicationAttemptDenial::new(
                WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
                "workflow operation authority is no longer current",
            )
        };
        if self.runtime.authority_identity().as_u64() != runtime_authority {
            return Err(mismatch());
        }
        check_request_live(request, "workflow operation authority")?;
        let selected = self
            .on_branch(observation.product_branch())
            .select_with_request(execution)
            .map_err(|denial| match denial {
                crate::basis::WorthQueryProductBranchAdmissionDenial::Stopped(cause) => {
                    WorthQueryApplicationAttemptDenial::new(
                        match cause {
                            Ok(kind) => WorthQueryApplicationAttemptDenialKind::ExecutionRequest(kind),
                            Err(crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled) => {
                                WorthQueryApplicationAttemptDenialKind::Cancelled
                            }
                            Err(crate::domain_computation::WorthQueryProviderSessionControlStopKind::TimedOut) => {
                                WorthQueryApplicationAttemptDenialKind::DeadlineExceeded
                            }
                        },
                        "workflow operation authority",
                    )
                }
                crate::basis::WorthQueryProductBranchAdmissionDenial::Refused(_) => mismatch(),
            })?;
        let current =
            WorthQueryProductBranchReadIdentity::from_observation(selected.product().observation());
        if !observation.same_branch_occurrence(&current) {
            return Err(mismatch());
        }
        Ok(())
    }
}

impl<Schema> SelectedProductOperationPort<Schema> for WorthQuerySelectedProductOperation<'_, Schema>
where
    Schema: ApplicationSchema,
{
    fn authority_identity_u64(&self) -> u64 {
        self.application().runtime.authority_identity().as_u64()
    }

    fn branch_read_identity(&self) -> WorthQueryProductBranchReadIdentity {
        WorthQueryProductBranchReadIdentity::from_observation(self.product().observation())
    }
}

impl<Schema> EffectProgramOriginalInputRuntimePort<Schema>
    for WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    fn authority_identity(&self) -> WorthQueryRuntimeAuthorityIdentity {
        self.runtime.authority_identity()
    }

    fn installed_schema(&self) -> &WorthQueryInstalledApplicationSchema<Schema> {
        self.installed_schema()
    }

    fn execution_request_for_phase<'phase>(
        &self,
        phase: &'phase WorthQueryAdvancementPhase<'_>,
    ) -> Result<worth_execution::ExecutionRequest<'phase, 'phase>, WorthQueryApplicationAttemptDenial>
    {
        phase
            .execution_request_for(&self.runtime.product_runtime)
            .map_err(|_| {
                WorthQueryApplicationAttemptDenial::new(
                    WorthQueryApplicationAttemptDenialKind::ForeignPhase,
                    "original source capture",
                )
            })
    }
}

impl<Schema> ProducerIdentityRuntimePort for WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    fn primary_provider(&self) -> &WorthQueryPrimaryGraphProvider {
        &self.runtime.primary_provider
    }

    fn posts_settlement(
        &self,
        settlement: &crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity,
    ) -> Result<bool, ()> {
        self.runtime
            .output_demands
            .posts_settlement(settlement)
            .map_err(|_| ())
    }
}
