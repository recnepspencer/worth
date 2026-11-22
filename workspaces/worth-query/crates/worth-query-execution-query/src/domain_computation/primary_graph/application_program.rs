use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_declaration::facade::application_query::{
    ApplicationQueryBinding, ApplicationQueryIntent,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, ApplicationStructuredValueBinding,
};

use super::application_contribution::WorthQueryApplicationOutputDemand;

pub use worth_query_declaration::facade::application_program::{
    WorthQueryApplicationRequiredOutputConnection, WorthQueryApplicationRequiredOutputSource,
    WorthQueryRequiredOutputConnectionDenial,
};
pub use worth_query_execution_lineage::facade::primary_graph::application_program::WorthQueryPreparedRequiredOutputSource;

/// A performed source whose required root demands are resolved by one typed
/// query at that source publication's retained observation.
pub trait WorthQueryApplicationDiscoveredOutputConnection<Schema>: Sized + 'static
where
    Schema: ApplicationSchema,
{
    type Source: ApplicationMutationBinding<Schema>;
    type Discovery: ApplicationQueryIntent<Schema> + Clone + Send + Sync;
    type Demand: WorthQueryApplicationOutputDemand<Schema>;

    const IDENTITY: &'static str;

    fn discovery_from_source(
        source: &<Self::Source as ApplicationMutationBinding<Schema>>::Input,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial>;

    fn demands_from_discovery(
        discovery: &<<<Self::Discovery as ApplicationQueryIntent<Schema>>::Binding as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial>;
}

/// One typed transitive connection from a settled root output to every
/// dependent output occurrence discovered at that exact retained result.
pub trait WorthQueryApplicationDependentOutputConnection<Schema>: Sized + 'static
where
    Schema: ApplicationSchema,
{
    type RootDemand: WorthQueryApplicationOutputDemand<Schema>;
    type Discovery: ApplicationQueryIntent<Schema>;
    type Demand: WorthQueryApplicationOutputDemand<Schema>;

    const IDENTITY: &'static str;

    fn discovery_from_root(
        root: &Self::RootDemand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial>;

    fn demands_from_discovery(
        discovery: &<<<Self::Discovery as ApplicationQueryIntent<Schema>>::Binding as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial>;
}

/// Complete predecessor query for one dependent connection. Query owns draining
/// the selected relation; each admitted row maps to one real parent demand.
pub trait WorthQueryApplicationPrerequisiteOutputConnection<S: ApplicationSchema>:
    WorthQueryApplicationDependentOutputConnection<S>
{
    type Prerequisites: worth_query_declaration::facade::application_query::ApplicationQueryIntent<
        S,
    >;
    type PrerequisiteRow: worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
    type Selection: 'static;
    fn prerequisite_rows() -> Self::Selection;
    fn prerequisites_from_target(
        target: &Self::Demand,
    ) -> Result<Self::Prerequisites, WorthQueryRequiredOutputConnectionDenial>;
    fn parent_from_prerequisite(
        row: &<Self::PrerequisiteRow as worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding>::Value,
    ) -> Result<Self::RootDemand, WorthQueryRequiredOutputConnectionDenial>;
}
