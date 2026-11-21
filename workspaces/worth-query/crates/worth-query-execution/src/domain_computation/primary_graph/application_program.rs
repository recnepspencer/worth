use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_declaration::facade::application_query::{
    ApplicationQueryBinding, ApplicationQueryIntent,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, ApplicationStructuredValueBinding,
};

use super::application_contribution::WorthQueryApplicationOutputDemand;
use super::WorthQueryOutputDemandDenial;

pub use worth_query_execution_lineage::facade::primary_graph::application_program::WorthQueryPreparedRequiredOutputSource;

/// A source publication succeeded, but its required-output recovery custody
/// could not be established.
pub struct WorthQueryRequiredOutputSourcePreparationFailure {
    pub(in crate::domain_computation::primary_graph) receipt:
        crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    pub(in crate::domain_computation::primary_graph) denial: WorthQueryOutputDemandDenial,
    pub(in crate::domain_computation::primary_graph) source:
        super::application_installation::WorthQueryUnpublishedProgramOutputSource,
    pub(in crate::domain_computation::primary_graph) carrier:
        super::application_installation::WorthQueryRecoveredProgramOutputSource,
}

impl WorthQueryRequiredOutputSourcePreparationFailure {
    /// Both actual one-shot native delivery parts remain available for promotion.
    pub fn has_original_delivery(&self) -> bool {
        self.carrier.has_original_delivery()
    }

    pub fn receipt(
        &self,
    ) -> &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt {
        &self.receipt
    }

    pub const fn denial(&self) -> &WorthQueryOutputDemandDenial {
        &self.denial
    }
    /// Original move-only preparation; descriptive access cannot promote it.
    pub fn source(
        &self,
    ) -> &super::application_installation::WorthQueryUnpublishedProgramOutputSource {
        &self.source
    }
    /// Reassembles a refused transfer from the exact owners returned by it.
    pub fn returned(
        receipt: crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
        denial: WorthQueryOutputDemandDenial,
        source: super::application_installation::WorthQueryUnpublishedProgramOutputSource,
        carrier: super::application_installation::WorthQueryRecoveredProgramOutputSource,
    ) -> Self {
        Self {
            receipt,
            denial,
            source,
            carrier,
        }
    }
    /// Returns every original native owner for a fresh checked retention retry.
    pub fn into_parts(
        self,
    ) -> (
        crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
        WorthQueryOutputDemandDenial,
        super::application_installation::WorthQueryUnpublishedProgramOutputSource,
        super::application_installation::WorthQueryRecoveredProgramOutputSource,
    ) {
        (self.receipt, self.denial, self.source, self.carrier)
    }
}

pub use worth_query_declaration::facade::application_program::{
    WorthQueryApplicationRequiredOutputConnection, WorthQueryApplicationRequiredOutputSource,
    WorthQueryRequiredOutputConnectionDenial,
};

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
#[cfg(test)]
#[path = "application_program_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;

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
