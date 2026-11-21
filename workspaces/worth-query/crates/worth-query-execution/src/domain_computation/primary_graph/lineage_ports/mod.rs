//! Lineage-layer port implementations for query/runtime-owned types.

pub(in crate::domain_computation::primary_graph) mod fact_conversion;
pub(in crate::domain_computation::primary_graph) mod lineage;
mod query_ports;
pub(in crate::domain_computation::primary_graph) mod read_observation;
pub(in crate::domain_computation::primary_graph) mod snapshot_lease;
pub(in crate::domain_computation::primary_graph) mod source_read_authorization;
mod source_query;

pub(in crate::domain_computation::primary_graph) use fact_conversion::SourceFactExpansionControlExt;
pub(in crate::domain_computation::primary_graph) use lineage::{
    downcast_managed_views, downcast_managed_views_arc,
};
pub(in crate::domain_computation::primary_graph) use read_observation::ReadObservationRuntimeExt;
pub(crate) use snapshot_lease::SnapshotLeaseRuntimeExt;
pub(in crate::domain_computation) use source_read_authorization::SourceReadAuthorizationRuntimeExt;
pub(in crate::domain_computation::primary_graph) use worth_query_execution_attempt::facade::{
    CommitAuthorityBindingExt, PrimaryGraphHandleWorkflowExt, WorkflowRetentionStore,
};
pub(crate) use worth_query_execution_attempt::facade::RetainedPreImageExt;
