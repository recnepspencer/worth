//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryApplicationReadObservation>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_query/retained_read.rs", "WorthQueryApplicationReadObservation", 128 * 1024);
        owner.field::<u64>("runtime_authority");
        owner.field::<ApplicationSchemaBindingIdentity>("schema_binding");
        owner.field::<WorthQueryProductObservationLease>("product");
        owner.field::<Option<
            Arc<super::super::super::application_output_demand::OriginalSettledProducerObservation>,
        >>("producer");
        owner.field::<Option<worth_execution::ExecutionMemoryReservation>>("_memory");
    }
}
