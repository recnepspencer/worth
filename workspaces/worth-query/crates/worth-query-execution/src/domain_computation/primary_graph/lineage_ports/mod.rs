//! Primary-graph-scoped port implementations and extension traits bridging
//! `worth-query-execution` internals to `worth-query-execution-lineage`.

mod aftermath;
mod attempt;
mod commit_authority;
mod fact_conversion;
mod lineage;
mod read_observation;
mod snapshot_lease;
mod source_query;
mod source_read_authorization;
mod workflow_retention;

pub(crate) use aftermath::RetainedPreImageExt;
pub(in crate::domain_computation::primary_graph) use commit_authority::CommitAuthorityBindingExt;
pub(in crate::domain_computation::primary_graph) use fact_conversion::SourceFactExpansionControlExt;
pub(in crate::domain_computation::primary_graph) use lineage::{
    downcast_managed_views, downcast_managed_views_arc,
};
pub(in crate::domain_computation::primary_graph) use read_observation::ReadObservationRuntimeExt;
pub(in crate::domain_computation) use snapshot_lease::SnapshotLeaseRuntimeExt;
pub(in crate::domain_computation) use source_read_authorization::SourceReadAuthorizationRuntimeExt;
pub(in crate::domain_computation::primary_graph) use workflow_retention::{
    PrimaryGraphHandleWorkflowExt, WorkflowRetentionStore,
};
