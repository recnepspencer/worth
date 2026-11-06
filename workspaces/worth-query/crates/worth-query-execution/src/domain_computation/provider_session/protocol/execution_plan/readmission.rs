//! Physical readmission consumes the receiver only after grant admission.
use super::*;

impl<'run> WorthQueryAdmittedProviderExecutionPlan<'run> {
    pub(in crate::domain_computation::provider_session::protocol) fn readmit_physical(
        mut self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'run>,
        admission: WorthQueryProviderSessionTokenAdmission,
    ) -> Result<
        super::super::WorthQueryPreparedProviderSession<'run>,
        WorthQueryProviderSessionFailure,
    > {
        self.require_current_home(phase.world_owner())?;
        let request = self.request_for_session(phase)?;
        let expected_generation = admission.reserved_generation();
        self.counters.called_provider();
        let invocation = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.provider.readmit_session(
                &WorthQueryProviderExecutionPlanView::new(request, &self.contract),
                admission,
            )
        }));
        let token = match invocation {
            Ok(Ok(token)) => token,
            Ok(Err(failure)) => {
                return Err(failure.at_stage(
                    WorthQueryProviderSessionProtocolStage::PlanReadmission,
                    self.counters,
                ));
            }
            Err(_) => {
                return Err(WorthQueryProviderSessionFailure::new(
                    WorthQueryProviderSessionDenialKind::ProviderPanicked,
                    WorthQueryProviderSessionProtocolStage::PlanReadmission,
                    "provider panicked while readmitting the sealed execution plan",
                    self.counters,
                )
                .with_recovery_posture(
                    WorthQueryProviderSessionRecoveryPosture::RecoveryRequired,
                ));
            }
        };
        if !token.belongs_to(&self.contract) || token.generation() != expected_generation {
            return Err(WorthQueryProviderSessionFailure::new(
                WorthQueryProviderSessionDenialKind::TokenNotMintedForPlan,
                WorthQueryProviderSessionProtocolStage::PlanReadmission,
                "provider returned a token minted for another plan or generation",
                self.counters,
            )
            .with_recovery_posture(WorthQueryProviderSessionRecoveryPosture::RecoveryRequired));
        }
        self.counters.minted_token();
        super::super::WorthQueryPreparedProviderSession::prepare_readmitted(
            WorthQueryProviderSessionAffinity::mint(
                request,
                self.run,
                self.contract,
                self.product,
                self.provider,
                token,
            ),
            self.counters,
        )
    }
    pub(in crate::domain_computation::provider_session::protocol) fn request_for_session(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'run>,
    ) -> Result<worth_execution::ExecutionRequest<'run, 'run>, WorthQueryProviderSessionFailure>
    {
        let (source_instance, owner_identity) = match &self.run {
            WorthQueryProviderRunBorrow::Direct(run) => (
                run.execution_source_instance_id(),
                run.execution_owner_identity(),
            ),
            WorthQueryProviderRunBorrow::Workflow(run) => (
                run.execution_source_instance_id(),
                run.execution_owner_identity(),
            ),
        };
        match &self.product {
            WorthQueryProviderProductAffinity::Standalone => match owner_identity {
                Some(owner) => phase.request_for_owner(owner),
                None => phase.request_for_source(source_instance),
            },
            WorthQueryProviderProductAffinity::Application(product) => {
                phase.request_for_owner(product.observation().branch_identity().owner_identity())
            }
        }
        .map_err(|cause| {
            failure(
                crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::from(cause)
                    .provider_denial_cause()
                    .expect("foreign custody is a resource refusal"),
                "provider plan belongs to another advancement runtime",
                &self.counters,
            )
        })
    }
}
