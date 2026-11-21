use super::resource_lifecycle::WorthQueryApplicationBasisSelectionIdentity;

pub use worth_query_execution_lineage::facade::primary_graph::application_query::observed_source::{
    comparison, denial, expression_context, fact_conversion, fact_sources, footprint_accounting,
    realized_footprint, retained_parameters, root_selection, scope_selector, source_identity,
    AppliedSourcePublication, DeclaredComponentSelection, DeclaredManagedIndexSelection,
    DeclaredOrbitSelection, ManagedSourceDependencies, ObservedQueryExpressionContext,
    RetainedExpressionContext, WorthQueryBoundSourceExpectation,
    WorthQueryCheckpointSourceIdentity, WorthQueryObservedAdjacencyRevision,
    WorthQueryObservedFieldRevision, WorthQueryObservedRootSelection,
    WorthQueryObservedScopeSelector, WorthQueryObservedSource, WorthQueryObservedSourceEpoch,
    WorthQueryObservedSourceFootprint, WorthQueryObservedSourceSelection,
    WorthQueryRetainedObservedParameters, WorthQueryRuntimeSourceIdentity,
    WorthQuerySourceExpectationDenial, WorthQuerySourceExpectationDenialKind,
    WorthQuerySourceExpectationKind,
};
pub(in crate::domain_computation::primary_graph) use fact_conversion::SourceFactExpansionControl;

pub(crate) mod prepared_expectation;
mod result_set;
pub(crate) mod retained_dependencies;
mod workflow_capture;
pub use prepared_expectation::WorthQueryPendingSourceExpectation;
pub(in crate::domain_computation::primary_graph) use prepared_expectation::{
    BoundStableObservedSourceFacts, PreparedObservedSourceExpectation,
};
pub use result_set::WorthQueryObservedResultSet;

mod applied;
pub(in crate::domain_computation::primary_graph) use applied::AppliedQuerySource;

pub(crate) mod applied_support;
pub(in crate::domain_computation::primary_graph) use applied_support::AppliedQuerySupport;

mod original_operation;
pub use original_operation::{
    capture_original_operation_source, WorthQueryOriginalSourceData,
    WorthQueryReadmittedOriginalSource,
};
