//! The original reader loan carries a non-driving output dependency.
use super::*;
use crate::domain_computation::primary_graph::{
    application_contribution::{WorthQueryApplicationOutputDemand, WorthQueryProducerOutputFamily},
    invariant_projection::WorthQueryObservedOutputAvailability,
};
use worth_query_declaration::facade::{
    application_query::{
        ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeBinding,
        ApplicationQueryScopeResolution,
    },
    application_schema::{ApplicationEntityMarkerIdentity, ApplicationStructuredValueBinding},
};
impl<Schema, Operation> WorthQueryComputationReader<'_, '_, '_, Schema, Operation>
where
    Schema: ApplicationSchema + 'static,
{
    /// Observe current output or source-aligned Pending without driving upstream.
    pub fn output_availability<Demand, Producer>(
        &mut self, producer: &WorthQueryInvariantEntityIdentity<Schema, Producer>, demand: Demand,
    ) -> Result<WorthQueryObservedOutputAvailability<Schema,
        <Demand::OutputFamily as WorthQueryProducerOutputFamily<Schema>>::Entity>, WorthQueryComputationReadDenial>
    where
        Demand: WorthQueryApplicationOutputDemand<Schema>,
        <Demand::OutputFamily as WorthQueryProducerOutputFamily<Schema>>::Source: ApplicationQueryBinding<Schema>,
        <<Demand::OutputFamily as WorthQueryProducerOutputFamily<Schema>>::Source as ApplicationQueryBinding<Schema>>::Input:
            ApplicationQueryIntent<Schema, Binding = <Demand::OutputFamily as WorthQueryProducerOutputFamily<Schema>>::Source>,
        <<Demand::OutputFamily as WorthQueryProducerOutputFamily<Schema>>::Source as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeBinding<Schema, Scope = Producer> + ApplicationQueryScopeResolution<Schema,
                <<Demand::OutputFamily as WorthQueryProducerOutputFamily<Schema>>::Source as ApplicationQueryBinding<Schema>>::PrincipalIdentity>,
        <<<Demand::OutputFamily as WorthQueryProducerOutputFamily<Schema>>::Source as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value:
            crate::domain_computation::primary_graph::WorthQueryApplicationProjection<Schema,
                <<Demand::OutputFamily as WorthQueryProducerOutputFamily<Schema>>::Source as ApplicationQueryBinding<Schema>>::Query>,
        Producer: ApplicationEntityMarkerIdentity<Schema> + OperationReads<Operation> + 'static,
        <Demand::OutputFamily as WorthQueryProducerOutputFamily<Schema>>::Entity: OperationReads<Operation>,
    {
        self.reader
            .output_availability(producer, demand)
            .map_err(|stop| WorthQueryComputationReadDenial::OutputAvailability(Box::new(stop)))
    }
}
