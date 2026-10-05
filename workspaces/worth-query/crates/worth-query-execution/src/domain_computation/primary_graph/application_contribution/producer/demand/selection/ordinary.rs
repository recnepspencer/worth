//! Ordinary producer selection entry points.

use super::*;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    pub fn select_output_producer<Family>(
        &self,
        source: &WorthQueryObservedSource<
            <<Family as WorthQueryProducerOutputFamily<Schema>>::Source as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::Query,
        >,
        profile_kind: &'static str,
        maximum_work: usize,
    ) -> Result<WorthQuerySelectedApplicationProducer, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        self.select_output_producer_with_retained_basis::<Family>(
            source,
            profile_kind,
            maximum_work.min(
                self.output_demand_resource_profile()
                    .limits()
                    .source_currentness_work(),
            ),
            None,
        )
    }

    pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand) fn select_output_producer_with_retained_basis<
        Family,
    >(
        &self,
        source: &WorthQueryObservedSource<
            <<Family as WorthQueryProducerOutputFamily<Schema>>::Source as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::Query,
        >,
        profile_kind: &'static str,
        maximum_work: usize,
        retained_program_basis: Option<
            &crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation,
        >,
    ) -> Result<WorthQuerySelectedApplicationProducer, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        let mut remaining_work = maximum_work;
        self.select_output_producer_with_remaining::<Family>(
            source,
            profile_kind,
            &mut remaining_work,
            retained_program_basis,
        )
        .map(|(selected, _)| selected)
    }

    /// The admitted demand continues with this exact remaining allowance.
    pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand) fn select_output_producer_with_remaining<
        Family,
    >(
        &self,
        source: &WorthQueryObservedSource<
            <<Family as WorthQueryProducerOutputFamily<Schema>>::Source as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::Query,
        >,
        profile_kind: &'static str,
        remaining_work: &mut usize,
        retained_program_basis: Option<
            &crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation,
        >,
    ) -> Result<SelectedWithEntry<'_, Schema>, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        self.select_output_producer_with_remaining_core::<Family>(
            source,
            profile_kind,
            remaining_work,
            retained_program_basis,
            None,
        )
    }
}
