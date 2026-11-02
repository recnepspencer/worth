use serde_json::Value;
use worth_proof::TransitionOutcome;

use crate::{
    WorthServerCompatibilityPreparedRequest, WorthServerQueryHandoffDeferred,
    WorthServerQueryHandoffDenial, WorthServerQueryHandoffFailure,
    WorthServerQueryHandoffRebindRequired, WorthServerQueryHandoffStale,
};

pub type WorthServerCompatibilityMutationOutcome<T> = TransitionOutcome<
    T,
    WorthServerQueryHandoffDenial,
    WorthServerQueryHandoffDeferred,
    WorthServerQueryHandoffStale,
    WorthServerQueryHandoffRebindRequired,
    WorthServerQueryHandoffFailure,
>;

#[derive(Clone, Debug)]
pub struct WorthServerCompatibilityMutationExecutionInput {
    prepared_request: WorthServerCompatibilityPreparedRequest,
    operation_name: String,
    body: Value,
}

impl WorthServerCompatibilityMutationExecutionInput {
    pub fn new(
        prepared_request: WorthServerCompatibilityPreparedRequest,
        operation_name: impl Into<String>,
        body: Value,
    ) -> Self {
        Self {
            prepared_request,
            operation_name: operation_name.into().trim().to_string(),
            body,
        }
    }

    pub(crate) fn into_parts(self) -> (WorthServerCompatibilityPreparedRequest, String, Value) {
        (self.prepared_request, self.operation_name, self.body)
    }
}

/// Keyed application results preserve every actual Query terminal, including
/// unpublished and unresolved custody. The direct lane is explicitly unkeyed.
#[derive(Debug)]
pub enum WorthServerCompatibilityMutationExecution {
    Direct(super::WorthServerCompatibilityMutation),
    Application(super::WorthServerQueryMutationAnswer),
}
