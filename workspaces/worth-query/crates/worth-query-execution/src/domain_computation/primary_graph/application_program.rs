use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_declaration::facade::application_query::{
    ApplicationQueryBinding, ApplicationQueryIntent,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, ApplicationStructuredValueBinding,
};

use super::application_contribution::WorthQueryApplicationOutputDemand;
use super::WorthQueryOutputDemandDenial;

/// Move-only owner evidence for one performed source publication.
///
/// ```compile_fail
/// use worth_query_execution::facade::primary_graph::WorthQueryPreparedRequiredOutputSource;
/// fn cannot_duplicate(source: WorthQueryPreparedRequiredOutputSource) {
///     let first = source;
///     let second = source;
///     drop((first, second));
/// }
/// ```
///
/// A descriptive receipt copy cannot recover or mint this owner evidence.
///
/// ```compile_fail
/// use worth_query_execution::facade::primary_graph::{
///     WorthQueryApplicationCommitReceipt, WorthQueryPrimaryGraphApplicationRuntime,
/// };
/// use worth_query_installation::facade::ApplicationSchema;
/// fn receipt_cannot_mint<Schema: ApplicationSchema>(
///     runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
///     receipt: &WorthQueryApplicationCommitReceipt,
/// ) {
///     let receipt_copy = receipt.clone();
///     let _ = runtime.prepared_required_output_source(&receipt_copy);
/// }
/// ```
///
/// Required admission is not a public escape hatch around an installed
/// program's prepared or parent-settlement evidence.
///
/// ```compile_fail
/// use worth_query_declaration::facade::{application_query::ApplicationQueryBinding, application_schema::ApplicationStructuredValueBinding};
/// use worth_query_execution::facade::{application_contribution::WorthQueryProducerOutputFamily, primary_graph::{WorthQueryApplicationOutputDemandSource, WorthQueryPrimaryGraphApplicationRuntime}};
/// use worth_query_installation::facade::ApplicationSchema;
/// type Source<Schema, Family> = <Family as WorthQueryProducerOutputFamily<Schema>>::Source;
/// fn bypass<Schema, Family>(
///     runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
///     source: WorthQueryApplicationOutputDemandSource<
///         <Source<Schema, Family> as ApplicationQueryBinding<Schema>>::Query,
///         <<Source<Schema, Family> as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value,
///     >,
/// ) where Schema: ApplicationSchema + 'static, Family: WorthQueryProducerOutputFamily<Schema> {
///     let _ = runtime.admit_required_output_demand::<Family>(source, 1, 1);
/// }
/// ```
pub struct WorthQueryPreparedRequiredOutputSource {
    pub(in crate::domain_computation::primary_graph) runtime_authority: u64,
    pub(in crate::domain_computation::primary_graph) source_commit:
        worth_runtime_world::facade::CompositeCommitIdentity,
    pub(in crate::domain_computation::primary_graph) source_scope:
        crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    pub(in crate::domain_computation::primary_graph) owner:
        super::application_output_demand::WorthQueryOutputDemandRegistry,
    pub(in crate::domain_computation::primary_graph) preparation:
        super::application_output_demand::WorthQueryRequiredOutputSourcePreparation,
}

impl Drop for WorthQueryPreparedRequiredOutputSource {
    fn drop(&mut self) {
        self.owner.release_prepared_token(&self.source_commit);
    }
}

/// A source publication succeeded, but its required-output recovery custody
/// could not be established.
pub struct WorthQueryRequiredOutputSourcePreparationFailure {
    pub(in crate::domain_computation::primary_graph) receipt:
        crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    pub(in crate::domain_computation::primary_graph) denial: WorthQueryOutputDemandDenial,
}

impl WorthQueryRequiredOutputSourcePreparationFailure {
    pub fn receipt(
        &self,
    ) -> &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt {
        &self.receipt
    }

    pub const fn denial(&self) -> &WorthQueryOutputDemandDenial {
        &self.denial
    }
}

/// One installed program connection from a performed source operation to a required output.
pub trait WorthQueryApplicationRequiredOutputConnection<Schema>: Sized + 'static
where
    Schema: ApplicationSchema,
{
    type Source: ApplicationMutationBinding<Schema>;
    type Demand: WorthQueryApplicationOutputDemand<Schema>;

    /// Canonical connection identity that must be present in the installed
    /// validated program before the source operation may publish.
    const IDENTITY: &'static str;

    fn validate_source(
        source: &<Self::Source as ApplicationMutationBinding<Schema>>::Input,
    ) -> Result<(), WorthQueryRequiredOutputConnectionDenial>;

    fn demand_from_committed_source(
        source: &<Self::Source as ApplicationMutationBinding<Schema>>::Input,
        result: &<Self::Source as ApplicationMutationBinding<Schema>>::Result,
    ) -> Self::Demand;
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryRequiredOutputConnectionDenial {
    subject: String,
}

impl WorthQueryRequiredOutputConnectionDenial {
    pub fn new(subject: impl Into<String>) -> Self {
        Self {
            subject: subject.into(),
        }
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }
}

impl std::fmt::Display for WorthQueryRequiredOutputConnectionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "required output connection denied: {}",
            self.subject
        )
    }
}

impl std::error::Error for WorthQueryRequiredOutputConnectionDenial {}
