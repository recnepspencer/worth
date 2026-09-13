use worth_query_declaration::facade::application_query::{
    ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, ApplicationStructuredValueBinding,
};
use worth_query_execution::facade::application_contribution::{
    WorthQueryApplicationOutputDemand, WorthQueryProducerOutputFamily,
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryAdmittedOutputDemand, WorthQueryApplicationProjection, WorthQueryOutputDemandAdvance,
    WorthQueryOutputDemandNotifications, WorthQueryPrimaryGraphApplicationRuntime,
};

use super::request::{
    WorthQueryApplicationOutputDemandDenial, WorthQueryApplicationOutputDemandRequest,
    WorthQueryOutputDemandControls,
};
use super::settlement::WorthQueryApplicationOutputDemandSettlement;

type Family<Schema, Demand> = <Demand as WorthQueryApplicationOutputDemand<Schema>>::OutputFamily;
type SourceBinding<Schema, Demand> =
    <Family<Schema, Demand> as WorthQueryProducerOutputFamily<Schema>>::Source;
type SourceQuery<Schema, Demand> =
    <SourceBinding<Schema, Demand> as ApplicationQueryBinding<Schema>>::Query;
type SourceValue<Schema, Demand> = <<SourceBinding<Schema, Demand> as ApplicationQueryBinding<
    Schema,
>>::ResultBinding as ApplicationStructuredValueBinding>::Value;

pub enum WorthQueryApplicationOutputDemandProgress<Query> {
    Pending,
    Settled(WorthQueryApplicationOutputDemandSettlement<Query>),
}

pub struct WorthQueryApplicationOutputDemandHandle<'application, Schema, Demand>
where
    Schema: ApplicationSchema,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
{
    application: &'application WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    admitted: WorthQueryAdmittedOutputDemand<Schema, Family<Schema, Demand>>,
    demand: Demand,
    controls: WorthQueryOutputDemandControls,
    closed: bool,
}

impl<'application, 'principal, 'scope, Schema, Demand>
    WorthQueryApplicationOutputDemandRequest<'application, 'principal, 'scope, Schema, Demand>
where
    Schema: ApplicationSchema + 'static,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
    SourceValue<Schema, Demand>:
        WorthQueryApplicationProjection<Schema, SourceQuery<Schema, Demand>> + Clone,
    <SourceBinding<Schema, Demand> as ApplicationQueryBinding<Schema>>::Input:
        ApplicationQueryIntent<Schema, Binding = SourceBinding<Schema, Demand>>,
    <SourceBinding<Schema, Demand> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <SourceBinding<Schema, Demand> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
{
    pub fn start(
        self,
    ) -> Result<
        WorthQueryApplicationOutputDemandHandle<'application, Schema, Demand>,
        WorthQueryApplicationOutputDemandDenial,
    > {
        let source_result = crate::application_entry::WorthQueryApplicationRequest {
            application: self.application,
            principal: self.principal,
            scope: self.scope,
            branch: self.branch,
        }
        .query(self.demand.source_intent())
        .execute()
        .map_err(WorthQueryApplicationOutputDemandDenial::Source)?;
        let source = source_result
            .rows()
            .first()
            .cloned()
            .ok_or(WorthQueryApplicationOutputDemandDenial::MissingSource)?;
        let observed_source = source_result
            .observed_sources()
            .first()
            .cloned()
            .ok_or(WorthQueryApplicationOutputDemandDenial::MissingSource)?;
        let profile_kind = Family::<Schema, Demand>::profile_kind(&source);
        let admitted = self
            .application
            .admit_output_demand::<Family<Schema, Demand>>(
                source,
                observed_source,
                profile_kind,
                self.controls
                    .map_or(1, |controls| controls.maximum_work().get()),
                self.controls
                    .map_or(1, |controls| controls.maximum_retained_bytes().get()),
            )
            .map_err(WorthQueryApplicationOutputDemandDenial::Demand)?;
        Ok(WorthQueryApplicationOutputDemandHandle {
            application: self.application,
            admitted,
            demand: self.demand,
            controls: self.controls.unwrap_or_else(|| {
                WorthQueryOutputDemandControls::new(
                    std::num::NonZeroUsize::new(1).unwrap(),
                    std::num::NonZeroUsize::new(1).unwrap(),
                )
            }),
            closed: false,
        })
    }
}

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
            .map_err(WorthQueryApplicationOutputDemandDenial::Source)?;
        let _controls = self.controls;
        let progress = self
            .application
            .advance_output_demand(
                &self.admitted,
                fresh_request.principal,
                fresh_request.scope,
                fresh_request.branch.clone(),
                disclosure.into_output_demand_disclosure(),
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

impl<Schema, Demand> Drop for WorthQueryApplicationOutputDemandHandle<'_, Schema, Demand>
where
    Schema: ApplicationSchema,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
{
    fn drop(&mut self) {
        if !self.closed {
            self.admitted.close();
            self.closed = true;
        }
    }
}
