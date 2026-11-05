//! Moved whole from `domain_computation::primary_graph::application_query::managed_index::provider`: the items below and every impl they own.
use worth_query_declaration::facade::application_query::ApplicationManagedIndexBinding;
use worth_query_declaration::facade::application_schema::ApplicationScalarValueBinding;

/// The actual declared probe codec's domain value.
pub type WorthQueryManagedIndexProbeValue<S, I> = <<I as ApplicationManagedIndexBinding<S>>::ProbeBinding as ApplicationScalarValueBinding>::Value;
