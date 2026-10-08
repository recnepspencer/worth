use super::*;

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
        for _ in 0..self
            .controls
            .resolve(self.application.output_demand_resource_profile())
            .settlement_attempts()
        {
            let progress = self.advance(fresh_request)?;
            if matches!(
                progress,
                WorthQueryApplicationOutputDemandProgress::Settled(_)
            ) {
                return Ok(progress);
            }
        }
        Ok(WorthQueryApplicationOutputDemandProgress::Pending)
    }

    pub fn advance(
        &mut self,
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
            _ => WorthQueryApplicationOutputDemandDenial::Demand(denial),
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
