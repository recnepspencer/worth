use std::any::Any;

use worth_query_declaration::facade::application_program::ApplicationConnectionShape;
use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_execution::facade::application_contribution::{
    WorthQueryApplicationOutputDemand, WorthQueryProducerOutputFamily,
};
use worth_query_execution::facade::primary_graph::WorthQueryApplicationDependentOutputConnection;

type Demand<Schema, Connection> =
    <Connection as WorthQueryApplicationDependentOutputConnection<Schema>>::Demand;
type Family<Schema, Connection> =
    <Demand<Schema, Connection> as WorthQueryApplicationOutputDemand<Schema>>::OutputFamily;
type Source<Schema, Connection> =
    <Family<Schema, Connection> as WorthQueryProducerOutputFamily<Schema>>::Source;
type Query<Schema, Connection> =
    <Source<Schema, Connection> as ApplicationQueryBinding<Schema>>::Query;

pub enum WorthQueryApplicationProgramOutputProgress<RootQuery> {
    Pending,
    Settled(WorthQueryApplicationProgramOutputSettlement<RootQuery>),
}

pub struct WorthQueryApplicationProgramOutputSettlement<RootQuery> {
    pub(super) root:
        crate::application_entry::WorthQueryApplicationOutputDemandSettlement<RootQuery>,
    pub(super) outputs: Vec<ProgramOutputRecord>,
    pub(super) work: super::program_output_work::WorthQueryApplicationProgramWork,
}

impl<RootQuery> WorthQueryApplicationProgramOutputSettlement<RootQuery> {
    pub fn root_receipt(
        &self,
    ) -> &worth_query_execution::facade::primary_graph::WorthQueryApplicationCommitReceipt {
        self.root.receipt()
    }

    pub fn root_observation(
        &self,
    ) -> &crate::application_entry::WorthQueryApplicationReadObservation {
        self.root.observation()
    }

    pub fn observation(&self) -> &crate::application_entry::WorthQueryApplicationReadObservation {
        self.outputs
            .iter()
            .map(|output| &output.observation)
            .chain(std::iter::once(self.root.observation()))
            .max_by_key(|observation| observation.selected_commit().ordinal())
            .expect("one program settlement always retains its root output")
    }

    pub fn output_count(&self) -> usize {
        self.outputs.len()
    }

    pub const fn work(&self) -> super::program_output_work::WorthQueryApplicationProgramWork {
        self.work
    }

    pub fn outputs_for<Schema, Connection>(
        &self,
    ) -> impl Iterator<
        Item = (
            &Demand<Schema, Connection>,
            &crate::application_entry::WorthQueryApplicationOutputDemandSettlement<
                Query<Schema, Connection>,
            >,
        ),
    >
    where
        Schema: ApplicationSchema,
        Connection: WorthQueryApplicationDependentOutputConnection<Schema> + 'static,
        Demand<Schema, Connection>: 'static,
        Query<Schema, Connection>: 'static,
    {
        self.outputs.iter().filter_map(|output| {
            (output.connection_identity == Connection::IDENTITY).then(|| {
                (
                    output
                        .demand
                        .downcast_ref::<Demand<Schema, Connection>>()
                        .expect("a typed program output retains its declared demand"),
                    output
                        .settlement
                        .downcast_ref::<
                            crate::application_entry::WorthQueryApplicationOutputDemandSettlement<
                                Query<Schema, Connection>,
                            >,
                        >()
                        .expect("a typed program output retains its declared settlement"),
                )
            })
        })
    }

    pub fn outputs_for_instance<'output, Schema, Connection>(
        &'output self,
        composition_instance: &'output str,
    ) -> impl Iterator<
        Item = (
            &'output Demand<Schema, Connection>,
            &'output crate::application_entry::WorthQueryApplicationOutputDemandSettlement<
                Query<Schema, Connection>,
            >,
        ),
    > + 'output
    where
        Schema: ApplicationSchema,
        Connection: WorthQueryApplicationDependentOutputConnection<Schema> + 'static,
        Demand<Schema, Connection>: 'static,
        Query<Schema, Connection>: 'static,
    {
        self.outputs_for::<Schema, Connection>()
            .zip(
                self.outputs
                    .iter()
                    .filter(|output| output.connection_identity == Connection::IDENTITY),
            )
            .filter_map(move |(typed, output)| {
                (output.composition_instance == composition_instance).then_some(typed)
            })
    }
}

#[doc(hidden)]
pub struct ProgramOutputRecord {
    connection_identity: &'static str,
    composition_instance: &'static str,
    demand: Box<dyn Any>,
    settlement: Box<dyn Any>,
    receipt: worth_query_execution::facade::primary_graph::WorthQueryApplicationCommitReceipt,
    readiness_delivery: Option<
        worth_query_execution::facade::primary_graph::WorthQueryOutputReadinessDeliveryEvidence,
    >,
    observation: crate::application_entry::WorthQueryApplicationReadObservation,
}

impl ProgramOutputRecord {
    pub(super) fn typed_for<Schema, Connection>(
        &self,
    ) -> Option<(
        &Demand<Schema, Connection>,
        &crate::application_entry::WorthQueryApplicationOutputDemandSettlement<
            Query<Schema, Connection>,
        >,
    )>
    where
        Schema: ApplicationSchema,
        Connection: WorthQueryApplicationDependentOutputConnection<Schema> + 'static,
        Demand<Schema, Connection>: 'static,
        Query<Schema, Connection>: 'static,
    {
        (self.connection_identity == Connection::IDENTITY).then(|| {
            (
                self.demand
                    .downcast_ref::<Demand<Schema, Connection>>()
                    .expect("a typed program output retains its declared demand"),
                self.settlement
                    .downcast_ref::<
                        crate::application_entry::WorthQueryApplicationOutputDemandSettlement<
                            Query<Schema, Connection>,
                        >,
                    >()
                    .expect("a typed program output retains its declared settlement"),
            )
        })
    }

    pub(super) fn observation(
        &self,
    ) -> &crate::application_entry::WorthQueryApplicationReadObservation {
        &self.observation
    }

    pub(super) fn receipt(
        &self,
    ) -> &worth_query_execution::facade::primary_graph::WorthQueryApplicationCommitReceipt {
        &self.receipt
    }

    pub(super) fn readiness_delivery(
        &self,
    ) -> Option<
        &worth_query_execution::facade::primary_graph::WorthQueryOutputReadinessDeliveryEvidence,
    > {
        self.readiness_delivery.as_ref()
    }

    pub(super) fn new<Schema, Connection>(
        demand: Demand<Schema, <Connection as ApplicationConnectionShape<Schema>>::Binding>,
        settlement: crate::application_entry::WorthQueryApplicationOutputDemandSettlement<
            Query<Schema, <Connection as ApplicationConnectionShape<Schema>>::Binding>,
        >,
    ) -> Self
    where
        Schema: ApplicationSchema,
        Connection: ApplicationConnectionShape<Schema>,
        Connection::Binding: WorthQueryApplicationDependentOutputConnection<Schema> + 'static,
        Demand<Schema, Connection::Binding>: 'static,
        Query<Schema, Connection::Binding>: 'static,
    {
        let observation = settlement.observation().retained_clone();
        let receipt = settlement.receipt().clone();
        let readiness_delivery = settlement.readiness_delivery().cloned();
        let declaration = Connection::declaration();
        Self {
            connection_identity: declaration.identity(),
            composition_instance: declaration.target_instance(),
            demand: Box::new(demand),
            settlement: Box::new(settlement),
            receipt,
            readiness_delivery,
            observation,
        }
    }
}
