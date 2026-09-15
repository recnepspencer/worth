use super::*;

pub enum WorthQueryApplicationProgramOutputProgress<Schema, Program, Inventory>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Inventory: ApplicationProgramInventoryIdentity,
{
    Pending,
    Settled(WorthQueryApplicationProgramOutputSettlement<Schema, Program, Inventory>),
}

pub struct WorthQueryApplicationProgramOutputSettlement<Schema, Program, Inventory>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Inventory: ApplicationProgramInventoryIdentity,
{
    pub(super) outputs: Vec<SettledProgramOutput>,
    pub(super) basis: crate::application_entry::WorthQueryApplicationReadObservation,
    pub(super) root_observation: crate::application_entry::WorthQueryApplicationReadObservation,
    pub(super) marker: std::marker::PhantomData<fn() -> (Schema, Program, Inventory)>,
}

pub(super) struct SettledProgramOutput {
    pub(super) feature_type: std::any::TypeId,
    pub(super) port_type: std::any::TypeId,
    pub(super) demand: std::sync::Arc<dyn std::any::Any>,
    pub(super) observation: crate::application_entry::WorthQueryApplicationReadObservation,
    pub(super) receipt:
        worth_query_execution::facade::primary_graph::WorthQueryApplicationCommitReceipt,
}

/// One exact typed output occurrence retained by a completed program inventory.
pub struct WorthQueryApplicationProgramOutputOccurrence<'a, Demand> {
    demand: &'a Demand,
    observation: &'a crate::application_entry::WorthQueryApplicationReadObservation,
    receipt: &'a worth_query_execution::facade::primary_graph::WorthQueryApplicationCommitReceipt,
}

impl<Demand> Copy for WorthQueryApplicationProgramOutputOccurrence<'_, Demand> {}

impl<Demand> Clone for WorthQueryApplicationProgramOutputOccurrence<'_, Demand> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, Demand> WorthQueryApplicationProgramOutputOccurrence<'a, Demand> {
    /// The typed output demand that selected this occurrence.
    pub const fn demand(&self) -> &'a Demand {
        self.demand
    }

    /// The exact retained observation at which the output settled.
    pub const fn observation(
        &self,
    ) -> &'a crate::application_entry::WorthQueryApplicationReadObservation {
        self.observation
    }

    /// The immutable World publication receipt for this output occurrence.
    pub const fn receipt(
        &self,
    ) -> &'a worth_query_execution::facade::primary_graph::WorthQueryApplicationCommitReceipt {
        self.receipt
    }
}

impl<Schema, Program, Inventory>
    WorthQueryApplicationProgramOutputSettlement<Schema, Program, Inventory>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Inventory: ApplicationProgramInventoryIdentity,
{
    /// The latest commit among independently settled inventory outputs.
    pub fn latest_observation(
        &self,
    ) -> &crate::application_entry::WorthQueryApplicationReadObservation {
        &self.basis
    }

    pub fn root_observation(
        &self,
    ) -> &crate::application_entry::WorthQueryApplicationReadObservation {
        &self.root_observation
    }

    pub fn output_observations<Feature, Port>(
        &self,
    ) -> impl Iterator<Item = &crate::application_entry::WorthQueryApplicationReadObservation>
    where
        Feature: ApplicationFeature<Schema>,
        Port: ApplicationOutputPort<Schema, Feature>,
    {
        self.outputs
            .iter()
            .filter(|output| {
                output.feature_type == std::any::TypeId::of::<Feature>()
                    && output.port_type == std::any::TypeId::of::<Port>()
            })
            .map(|output| &output.observation)
    }

    pub fn output_occurrences<Feature, Port, Demand>(
        &self,
    ) -> impl Iterator<Item = WorthQueryApplicationProgramOutputOccurrence<'_, Demand>>
    where
        Feature: ApplicationFeature<Schema>,
        Port: ApplicationOutputPort<Schema, Feature>,
        Demand: 'static,
    {
        self.outputs
            .iter()
            .filter(|output| {
                output.feature_type == std::any::TypeId::of::<Feature>()
                    && output.port_type == std::any::TypeId::of::<Port>()
            })
            .filter_map(|output| {
                output.demand.downcast_ref::<Demand>().map(|demand| {
                    WorthQueryApplicationProgramOutputOccurrence {
                        demand,
                        observation: &output.observation,
                        receipt: &output.receipt,
                    }
                })
            })
    }
}
