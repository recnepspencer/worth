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
        for _ in 0..self.controls.maximum_work().get() {
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
        let disclosure = fresh_request
            .query(self.demand.source_intent())
            .execute()
            .map_err(WorthQueryApplicationOutputDemandDenial::Source)?
            .into_output_demand_source();
        let _controls = self.controls;
        let progress = self
            .application
            .advance_output_demand(
                &mut self.admitted,
                fresh_request.principal,
                fresh_request.scope,
                fresh_request.branch.clone(),
                disclosure,
            )
            .map_err(|denial| {
                if denial.kind()
                    == worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenialKind::Superseded
                {
                    WorthQueryApplicationOutputDemandDenial::Superseded
                } else {
                    WorthQueryApplicationOutputDemandDenial::Demand(denial)
                }
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
