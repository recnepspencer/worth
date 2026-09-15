use std::any::Any;

use worth_query_declaration::facade::application_program::{
    ApplicationProgramDefinition, ApplicationProgramInventoryIdentity,
};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, ApplicationStructuredValueBinding,
};
use worth_query_execution::facade::application_contribution::{
    WorthQueryApplicationOutputDemand, WorthQueryProducerOutputFamily,
};
use worth_query_execution::facade::application_installation::{
    WorthQueryProgramApplicationRuntime, WorthQuerySettledProgramOutput,
};
use worth_query_execution::facade::primary_graph::WorthQueryApplicationProjection;

use crate::application_entry::demand::{
    WorthQueryApplicationOutputDemandDenial, WorthQueryProgramNodeHandle,
    WorthQueryProgramNodeProgress,
};
use crate::application_entry::WorthQueryApplicationReadObservation;

type Family<Schema, Demand> = <Demand as WorthQueryApplicationOutputDemand<Schema>>::OutputFamily;
type Source<Schema, Demand> =
    <Family<Schema, Demand> as WorthQueryProducerOutputFamily<Schema>>::Source;
type SourceQuery<Schema, Demand> =
    <Source<Schema, Demand> as ApplicationQueryBinding<Schema>>::Query;
type SourceValue<Schema, Demand> =
    <<Source<Schema, Demand> as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value;

pub(in crate::application_entry) struct ErasedProgramSettlement {
    pub(super) feature: &'static str,
    pub(super) feature_type: std::any::TypeId,
    pub(super) demand: std::sync::Arc<dyn Any>,
    pub(super) authority: Box<dyn Any>,
    pub(super) observation: WorthQueryApplicationReadObservation,
}

pub(in crate::application_entry) trait ErasedProgramNode<'application, Schema>
where
    Schema: ApplicationSchema,
{
    fn feature(&self) -> &'static str;
    fn notifications(
        &self,
    ) -> Result<
        worth_query_execution::facade::primary_graph::WorthQueryOutputDemandNotifications,
        WorthQueryApplicationOutputDemandDenial,
    >;
    fn advance(
        &mut self,
        request: &crate::application_entry::WorthQueryApplicationRequest<
            'application,
            '_,
            '_,
            Schema,
        >,
    ) -> Result<Option<ErasedProgramSettlement>, WorthQueryApplicationOutputDemandDenial>;
    fn close(&mut self);
}

pub(in crate::application_entry) struct TypedProgramNode<
    'application,
    Schema,
    Program,
    Inventory,
    Demand,
> where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Inventory: ApplicationProgramInventoryIdentity,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
{
    feature: &'static str,
    feature_type: std::any::TypeId,
    handle: WorthQueryProgramNodeHandle<'application, Schema, Program, Inventory, Demand>,
    settled: bool,
}

impl<'application, Schema, Program, Inventory, Demand>
    TypedProgramNode<'application, Schema, Program, Inventory, Demand>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Inventory: ApplicationProgramInventoryIdentity,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
{
    pub(super) fn new(
        feature: &'static str,
        feature_type: std::any::TypeId,
        handle: WorthQueryProgramNodeHandle<'application, Schema, Program, Inventory, Demand>,
    ) -> Self {
        Self {
            feature,
            feature_type,
            handle,
            settled: false,
        }
    }
}

impl<'application, Schema, Program, Inventory, Demand> ErasedProgramNode<'application, Schema>
    for TypedProgramNode<'application, Schema, Program, Inventory, Demand>
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema>,
    Inventory: ApplicationProgramInventoryIdentity,
    Demand: WorthQueryApplicationOutputDemand<Schema> + Clone + 'static,
    SourceQuery<Schema, Demand>: 'static,
    SourceValue<Schema, Demand>:
        WorthQueryApplicationProjection<Schema, SourceQuery<Schema, Demand>> + Clone + 'static,
    <Source<Schema, Demand> as ApplicationQueryBinding<Schema>>::Input:
        ApplicationQueryIntent<Schema, Binding = Source<Schema, Demand>>,
    <Source<Schema, Demand> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <Source<Schema, Demand> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
{
    fn feature(&self) -> &'static str {
        self.feature
    }

    fn notifications(
        &self,
    ) -> Result<
        worth_query_execution::facade::primary_graph::WorthQueryOutputDemandNotifications,
        WorthQueryApplicationOutputDemandDenial,
    > {
        self.handle.notifications()
    }

    fn advance(
        &mut self,
        request: &crate::application_entry::WorthQueryApplicationRequest<
            'application,
            '_,
            '_,
            Schema,
        >,
    ) -> Result<Option<ErasedProgramSettlement>, WorthQueryApplicationOutputDemandDenial> {
        if self.settled {
            return Ok(None);
        }
        match self.handle.advance(request)? {
            WorthQueryProgramNodeProgress::Pending => Ok(None),
            WorthQueryProgramNodeProgress::Settled {
                settlement,
                authority,
            } => {
                self.settled = true;
                self.handle.close();
                Ok(Some(ErasedProgramSettlement {
                    feature: self.feature,
                    feature_type: self.feature_type,
                    demand: std::sync::Arc::new(self.handle.demand().clone()),
                    authority: Box::new(authority) as Box<dyn Any>,
                    observation: settlement.observation().clone(),
                }))
            }
        }
    }

    fn close(&mut self) {
        if !self.settled {
            self.handle.close();
        }
        self.settled = true;
    }
}

pub(super) fn downcast_authority<Schema, Program, Inventory, Demand>(
    settlement: &ErasedProgramSettlement,
) -> Option<&WorthQuerySettledProgramOutput<Schema, Program, Inventory, Demand>>
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema>,
    Inventory: ApplicationProgramInventoryIdentity,
    Demand: WorthQueryApplicationOutputDemand<Schema> + 'static,
{
    settlement.authority.downcast_ref()
}

pub(super) fn runtime_matches<Schema, Program>(
    application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
    request: &crate::application_entry::WorthQueryApplicationRequest<'_, '_, '_, Schema>,
) -> bool
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    std::ptr::eq(application.runtime(), request.application)
}
