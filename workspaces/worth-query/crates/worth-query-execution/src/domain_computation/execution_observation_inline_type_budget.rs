//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryExecutionObservation>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/execution_observation.rs", "WorthQueryExecutionObservation", 128 * 1024);
        owner.field::<owned_control_scope::OwnedControlScope>("_controls");
        owner.field::<worth_signal::facade::SignalRequestWorkObservation>("_signal");
        owner.field::<worth_runtime_world::facade::RuntimeWorldReadObservation>("_world");
    }
}
