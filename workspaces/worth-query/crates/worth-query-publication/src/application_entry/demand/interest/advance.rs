use super::*;
use worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase as AdvancementPhase;

impl<Schema, Demand> WorthQueryApplicationOutputDemandHandle<'_, Schema, Demand>
where
    Schema: ApplicationSchema + 'static,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
    SourceValue<Schema, Demand>: 'static,
    SourceQuery<Schema, Demand>: 'static,
    <SourceBinding<Schema, Demand> as ApplicationQueryBinding<Schema>>::Input:
        ApplicationQueryIntent<Schema, Binding = SourceBinding<Schema, Demand>>,
    <SourceBinding<Schema, Demand> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <SourceBinding<Schema, Demand> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
    SourceValue<Schema, Demand>:
        WorthQueryApplicationProjection<Schema, SourceQuery<Schema, Demand>> + Clone,
{
    pub fn settle(
        &mut self,
        fresh_request: &crate::application_entry::WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<
        WorthQueryApplicationOutputDemandProgress<SourceQuery<Schema, Demand>>,
        WorthQueryApplicationOutputDemandDenial,
    > {
        self.application
            .with_application_advancement(fresh_request.scope, |phase| {
                for _ in 0..self
                    .controls
                    .resolve(self.application.output_demand_resource_profile())
                    .settlement_attempts()
                {
                    let progress = self.advance_in_advancement(&phase, fresh_request)?;
                    if matches!(
                        progress,
                        WorthQueryApplicationOutputDemandProgress::Settled(_)
                    ) {
                        return Ok(progress);
                    }
                }
                Ok(WorthQueryApplicationOutputDemandProgress::Pending)
            })
            .map_err(WorthQueryApplicationOutputDemandDenial::advancement)?
    }

    pub fn advance(
        &mut self,
        fresh_request: &crate::application_entry::WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<
        WorthQueryApplicationOutputDemandProgress<SourceQuery<Schema, Demand>>,
        WorthQueryApplicationOutputDemandDenial,
    > {
        self.application
            .with_application_advancement(fresh_request.scope, |phase| {
                self.advance_in_advancement(&phase, fresh_request)
            })
            .map_err(WorthQueryApplicationOutputDemandDenial::advancement)?
    }

    pub(in crate::application_entry) fn advance_in_advancement(
        &mut self,
        phase: &AdvancementPhase<'_>,
        fresh_request: &crate::application_entry::WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<
        WorthQueryApplicationOutputDemandProgress<SourceQuery<Schema, Demand>>,
        WorthQueryApplicationOutputDemandDenial,
    > {
        if self.closed {
            return Err(WorthQueryApplicationOutputDemandDenial::Closed);
        }
        if !std::ptr::eq(self.application, fresh_request.application) {
            return Err(WorthQueryApplicationOutputDemandDenial::FreshRequestMismatch);
        }
        let _controls = self.controls;
        let progress = if let Some((identity, revision)) = &self.selected_program {
            self.application.advance_selected_program_output_demand_from_retained(
                phase,
                &worth_query_execution::publication_boundary::program_publication_access(),
                &mut self.admitted,
                fresh_request.principal,
                fresh_request.scope,
                fresh_request.branch,
                identity.clone(),
                *revision,
            )
        } else {
            self.application.advance_output_demand_from_retained(
                phase,
                &mut self.admitted,
                fresh_request.principal,
                fresh_request.scope,
                fresh_request.branch,
            )
        }
        .map_err(|denial| match denial.kind() {
            worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenialKind::Superseded
                =>
            {
                WorthQueryApplicationOutputDemandDenial::Superseded
            }
            worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenialKind::ExecutionRequest(_)
                | worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenialKind::SourceQueryInstallation(_)
                | worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenialKind::SourcePrincipal(_)
                | worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenialKind::SourceScope(_)
                | worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenialKind::SourceQueryAdmission(_)
                | worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenialKind::SourceQueryExecution(_)
                | worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenialKind::ForeignSource
                | worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenialKind::MissingApplicableProducer
                | worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenialKind::AmbiguousApplicableProducer
                | worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenialKind::ProducerUnavailable
                | worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenialKind::ProducerDomainDenied
                | worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenialKind::RequestAuthorization(_)
                | worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenialKind::ProductSelection(_)
                | worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenialKind::SchedulingRejected
                | worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenialKind::SchedulingDeferred
                | worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenialKind::PublicationStale
                | worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenialKind::NoEffect
                | worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenialKind::Cancelled
                | worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenialKind::TimedOut
                | worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
                | worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded
                | worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenialKind::PublicationCapacityExceeded
                | worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenialKind::ForeignDemand
                | worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenialKind::ForeignSettlement
                | worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenialKind::IncompleteDependencyCoverage
                | worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable
                | worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenialKind::Closed
                | worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenialKind::DuplicatePerformedSource => WorthQueryApplicationOutputDemandDenial::Demand(denial),
        })?;
        match progress {
            WorthQueryOutputDemandAdvance::Pending => {
                Ok(WorthQueryApplicationOutputDemandProgress::Pending)
            }
            WorthQueryOutputDemandAdvance::Settled(receipt) => {
                Ok(WorthQueryApplicationOutputDemandProgress::Settled(
                    WorthQueryApplicationOutputDemandSettlement::new(
                        receipt,
                        self.admitted.observed_source().clone(),
                        self.admitted.checkpoint_readmission_work_units(),
                        self.admitted.checkpoint_readmission_work_bound(),
                        self.admitted
                            .checkpoint_readmission_charged_preparation_bytes(),
                    ),
                ))
            }
        }
    }

    pub fn close(&mut self) {
        if !self.closed {
            self.admitted.close();
        }
        self.closed = true;
    }

    pub fn notifications(
        &self,
    ) -> Result<WorthQueryOutputDemandNotifications, WorthQueryApplicationOutputDemandDenial> {
        self.admitted
            .notifications()
            .map_err(WorthQueryApplicationOutputDemandDenial::Demand)
    }
}
