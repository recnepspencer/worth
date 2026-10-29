//! Public ordinary batch courts using the existing installed temporal world.
#[path = "retained_query_batch/installed_profile.rs"]
pub mod installed_profile;
pub use super::{
    adapters::block_on,
    schema::current_read::TemporalIntentReadRequest,
    world::{self, CourtroomWorld},
};
pub use worth_query_host::facade::application_entry::{
    WorthQueryApplicationQueryBatchDenial, WorthQueryApplicationRequestExt,
};

pub fn read(identity: &str) -> TemporalIntentReadRequest {
    TemporalIntentReadRequest {
        identity: identity.to_owned(),
    }
}

#[path = "retained_query_batch/pre_kernel_cancellation_keeps_its_real_execution_cause.rs"]
pub mod pre_kernel_cancellation_keeps_its_real_execution_cause;

#[path = "retained_query_batch/direct_limits.rs"]
pub mod direct_limits;
