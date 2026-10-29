//! Actual pre-World native candidate races preserve terminal source custody.
pub use super::{fixture::*, readback::*, schema::*};
pub use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};
pub use std::{num::NonZeroUsize, time::Duration};
pub use worth_query_host::facade::primary_graph::BridgeExecutionBasisFinalizationFailureKind;
pub use worth_query_host::facade::{
    application_entry::*, declaration::application_schema::*, primary_graph::*,
};

pub fn assert_native_failure(failure: &WorthQueryMutationCleanupFailure) {
    assert_eq!(
        failure.kind(),
        WorthQueryMutationCleanupFailureKind::Native(
            WorthQueryManagedRunCleanupFailureKind::BridgeFinalization(
                BridgeExecutionBasisFinalizationFailureKind::SignalRuntimeThreadAffinityViolation
            )
        )
    );
}
#[path = "managed_cleanup/retained_capacity.rs"]
pub mod retained_capacity;
