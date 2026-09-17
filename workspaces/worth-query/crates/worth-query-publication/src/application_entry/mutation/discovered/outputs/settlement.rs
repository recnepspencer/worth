use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_execution::facade::application_contribution::{
    WorthQueryApplicationOutputDemand, WorthQueryProducerOutputFamily,
};
use worth_query_execution::facade::primary_graph::WorthQueryApplicationDependentOutputConnection;

use crate::application_entry::mutation::program_output_settlement::ProgramOutputRecord;
use crate::application_entry::mutation::WorthQueryApplicationProgramWork;
use crate::application_entry::{
    WorthQueryApplicationOutputDemandSettlement, WorthQueryApplicationReadObservation,
};

type ChildDemand<Schema, Connection> =
    <Connection as WorthQueryApplicationDependentOutputConnection<Schema>>::Demand;
type Family<Schema, Connection> =
    <ChildDemand<Schema, Connection> as WorthQueryApplicationOutputDemand<Schema>>::OutputFamily;
type Source<Schema, Connection> =
    <Family<Schema, Connection> as WorthQueryProducerOutputFamily<Schema>>::Source;
type Query<Schema, Connection> =
    <Source<Schema, Connection> as ApplicationQueryBinding<Schema>>::Query;

pub enum WorthQueryDiscoveredProgramOutputProgress<RootQuery, RootDemand> {
    Pending,
    Settled(WorthQueryDiscoveredProgramOutputSettlement<RootQuery, RootDemand>),
}

pub struct WorthQueryDiscoveredProgramOutputSettlement<RootQuery, RootDemand> {
    source: WorthQueryApplicationReadObservation,
    roots: Vec<(
        RootDemand,
        WorthQueryApplicationOutputDemandSettlement<RootQuery>,
    )>,
    superseded: Vec<RootDemand>,
    outputs: Vec<ProgramOutputRecord>,
    work: WorthQueryApplicationProgramWork,
}

impl<RootQuery, RootDemand> WorthQueryDiscoveredProgramOutputSettlement<RootQuery, RootDemand> {
    pub(super) fn new(
        source: WorthQueryApplicationReadObservation,
        roots: Vec<(
            RootDemand,
            WorthQueryApplicationOutputDemandSettlement<RootQuery>,
        )>,
        superseded: Vec<RootDemand>,
        outputs: Vec<ProgramOutputRecord>,
        work: WorthQueryApplicationProgramWork,
    ) -> Self {
        Self {
            source,
            roots,
            superseded,
            outputs,
            work,
        }
    }

    pub fn source_observation(&self) -> &WorthQueryApplicationReadObservation {
        &self.source
    }

    pub fn observation(&self) -> &WorthQueryApplicationReadObservation {
        self.roots
            .iter()
            .map(|(_, settled)| settled.observation())
            .chain(self.outputs.iter().map(ProgramOutputRecord::observation))
            .chain(std::iter::once(&self.source))
            .max_by_key(|observation| observation.selected_commit().ordinal())
            .expect("the performed source is retained even when discovery has no roots")
    }

    pub fn root_outputs(
        &self,
    ) -> impl Iterator<
        Item = (
            &RootDemand,
            &WorthQueryApplicationOutputDemandSettlement<RootQuery>,
        ),
    > {
        self.roots.iter().map(|(demand, settled)| (demand, settled))
    }

    pub fn superseded_roots(&self) -> impl Iterator<Item = &RootDemand> {
        self.superseded.iter()
    }

    pub fn output_count(&self) -> usize {
        self.roots.len() + self.outputs.len()
    }

    pub const fn work(&self) -> WorthQueryApplicationProgramWork {
        self.work
    }

    pub fn outputs_for<Schema, Connection>(
        &self,
    ) -> impl Iterator<
        Item = (
            &ChildDemand<Schema, Connection>,
            &WorthQueryApplicationOutputDemandSettlement<Query<Schema, Connection>>,
        ),
    >
    where
        Schema: ApplicationSchema,
        Connection: WorthQueryApplicationDependentOutputConnection<Schema> + 'static,
        ChildDemand<Schema, Connection>: 'static,
        Query<Schema, Connection>: 'static,
    {
        self.outputs
            .iter()
            .filter_map(ProgramOutputRecord::typed_for::<Schema, Connection>)
    }
}
