use std::sync::Arc;

use super::{
    WorthServerProductOperationDenial, WorthServerProductOperationDenialCode,
    WorthServerProductOperationOutcome, WorthServerProductOperationPayload,
    WorthServerProductOperationSuccess, WorthServerScheduledProductOperation,
};

pub trait WorthServerProductApplicationAdapter: Send + Sync + 'static {
    fn execute(
        &self,
        operation: &WorthServerScheduledProductOperation,
    ) -> Result<WorthServerProductOperationSuccess, WorthServerProductAdapterExecutionError>;

    /// Executes a shared-read packet under the caller's bounded authority.
    /// Implementations must explicitly carry `lease` into any nested work.
    fn execute_with_lease(
        &self,
        operation: &WorthServerScheduledProductOperation,
        lease: &worth_execution::ExecutionResourceLease<'_>,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<
        Result<WorthServerProductOperationSuccess, WorthServerProductAdapterExecutionError>,
        worth_execution::MapKernelStop,
    >;
}

pub trait WorthServerProductPayloadSchemaValidator: Send + Sync + 'static {
    fn validate(
        &self,
        payload: &WorthServerProductOperationPayload,
    ) -> Result<(), WorthServerProductOperationDenial>;
}

pub trait WorthServerProductOperationErrorMap: Send + Sync + 'static {
    fn map_error(
        &self,
        error: WorthServerProductAdapterExecutionError,
    ) -> WorthServerProductOperationOutcome;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthServerProductAdapterExecutionError {
    Denied(WorthServerProductOperationDenial),
    InvalidResultArtifact(crate::WorthServerProductResultArtifactError),
    Failed { reason_key: String, detail: String },
}

impl WorthServerProductAdapterExecutionError {
    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        use super::execution_pipeline::read_batch_accounting::string;
        match self {
            Self::Denied(denial) => denial.owned_allocation_capacity_bytes(),
            Self::InvalidResultArtifact(error) => error.owned_allocation_capacity_bytes(),
            Self::Failed { reason_key, detail } => {
                string(reason_key).saturating_add(string(detail))
            }
        }
    }

    pub fn denied(denial: WorthServerProductOperationDenial) -> Self {
        Self::Denied(denial)
    }

    pub fn failed(reason_key: impl Into<String>, detail: impl Into<String>) -> Self {
        Self::Failed {
            reason_key: reason_key.into(),
            detail: detail.into(),
        }
    }

    pub fn invalid_result_artifact(error: crate::WorthServerProductResultArtifactError) -> Self {
        Self::InvalidResultArtifact(error)
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct WorthServerDefaultProductOperationErrorMap;

impl WorthServerProductOperationErrorMap for WorthServerDefaultProductOperationErrorMap {
    fn map_error(
        &self,
        error: WorthServerProductAdapterExecutionError,
    ) -> WorthServerProductOperationOutcome {
        match error {
            WorthServerProductAdapterExecutionError::Denied(denial) => {
                WorthServerProductOperationOutcome::Denied(
                    denial.with_code(WorthServerProductOperationDenialCode::ProductSemantic),
                )
            }
            WorthServerProductAdapterExecutionError::InvalidResultArtifact(error) => {
                WorthServerProductOperationOutcome::failed(
                    "invalid_result_artifact",
                    error.detail(),
                )
            }
            WorthServerProductAdapterExecutionError::Failed { reason_key, detail } => {
                WorthServerProductOperationOutcome::failed(reason_key, detail)
            }
        }
    }
}

fn default_error_map() -> Arc<dyn WorthServerProductOperationErrorMap> {
    Arc::new(WorthServerDefaultProductOperationErrorMap)
}

#[derive(Clone, Copy, Debug, Default)]
pub struct WorthServerProductOperationErrorMaps;

impl WorthServerProductOperationErrorMaps {
    pub fn passthrough() -> Arc<dyn WorthServerProductOperationErrorMap> {
        default_error_map()
    }
}
