//! Intact current-contract refusal before interpreting native checkpoint bytes.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryPrimaryGraphInstallationDenial as Denial,
    WorthQueryPrimaryGraphInstallationDenialKind as Kind,
};

#[derive(Debug)]
pub struct CheckpointDecodeDenial {
    kind: Kind,
    message: String,
}
impl CheckpointDecodeDenial {
    pub fn installation(cause: Denial) -> Self {
        Self {
            kind: cause.kind(),
            message: cause.subject().to_owned(),
        }
    }
    pub fn execution(cause: worth_execution::WorkCeilingDenial) -> Self {
        let native =
            worth_relational::facade::transactions::RelationalExecutionDenialCause::from(cause);
        Self::native_execution(
            native,
            format!("checkpoint reconstruction stopped: {cause:?}"),
        )
    }
    pub fn native_execution(
        native: worth_relational::facade::transactions::RelationalExecutionDenialCause,
        message: String,
    ) -> Self {
        let kind =
            match super::super::provider::relational_execution_denial::relational_execution_kind(
                native, None,
            ) {
                Ok(kind) => Kind::ExecutionDenied { kind },
                Err(kind) => Kind::ExecutionControlStopped { kind },
            };
        Self { kind, message }
    }
    pub fn allocator() -> Self {
        Self {
            kind: Kind::ExecutionDenied {
                kind:
                    crate::domain_computation::WorthQueryProviderSessionDenialKind::AllocationDenied,
            },
            message: "checkpoint metadata allocation refused".to_owned(),
        }
    }
    pub fn capacity() -> Self {
        Self::execution(worth_execution::WorkCeilingDenial::Admission(
            worth_execution::LeaseDenial::ChargedBytesOverflow,
        ))
    }
    pub fn from_fact(cause: FactDecodeDenial) -> Self {
        use super::super::application_attempt::retained_decision_facts::StoreDenial;
        use worth_execution::{ExecutionAllocationDenialClass, MapKernelStop, WorkCeilingDenial};
        match cause {
            FactDecodeDenial::Format(message) => message.into(),
            FactDecodeDenial::Execution(cause) => Self::native_execution(cause,
                "checkpoint fact reconstruction stopped".to_owned()),
            FactDecodeDenial::Retention(StoreDenial::ExecutionRequest(cause)) => Self::execution(cause),
            FactDecodeDenial::Retention(StoreDenial::Content(cause)) => Self::native_execution(
                cause.into_execution_cause(),
                format!("checkpoint content reconstruction refused: {cause}"),
            ),
            FactDecodeDenial::Retention(StoreDenial::Allocation(cause)) => match cause.classify() {
                ExecutionAllocationDenialClass::Request(stop) => Self::execution(stop),
                ExecutionAllocationDenialClass::Physical(_) => Self {
                    kind: Kind::ExecutionDenied { kind: crate::domain_computation::WorthQueryProviderSessionDenialKind::AllocationDenied },
                    message: cause.to_string(),
                },
            },
            FactDecodeDenial::Retention(StoreDenial::RequestInterruption(cause)) => {
                use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption as I;
                Self::execution(WorkCeilingDenial::Stopped(match cause {
                    I::Cancelled => MapKernelStop::Cancelled,
                    I::DeadlineExceeded => MapKernelStop::DeadlineElapsed,
                }))
            },
            FactDecodeDenial::Retention(StoreDenial::PhysicalAllocation) => Self::allocator(),
            FactDecodeDenial::Retention(StoreDenial::Representability) => Self::capacity(),
            FactDecodeDenial::Retention(StoreDenial::ContentInconsistency) => Self::native_execution(
                worth_relational::facade::transactions::RelationalExecutionDenialCause::ContentInconsistency,
                "checkpoint retained content disagrees with its identity".to_owned(),
            ),
            FactDecodeDenial::Retention(StoreDenial::ConflictingBody) =>
                "checkpoint facts contain conflicting bodies".to_owned().into(),
            FactDecodeDenial::Retention(StoreDenial::InvalidOrdinal) =>
                "checkpoint facts contain an invalid fixture ordinal".to_owned().into(),
        }
    }
    pub fn unsupported_producer_facts(observed: u16) -> Self {
        Self {
            kind: Kind::UnsupportedCheckpointProducerFactVersion {
                observed,
                supported: facts::WIRE_VERSION,
            },
            message: format!("checkpoint producer fact wire version {observed} is unsupported"),
        }
    }
    pub fn into_installation_denial(self) -> Denial {
        Denial::new(self.kind, self.message)
    }
}
impl From<String> for CheckpointDecodeDenial {
    fn from(message: String) -> Self {
        Self {
            kind: Kind::CheckpointRecoveryRejected,
            message,
        }
    }
}
impl std::ops::Deref for CheckpointDecodeDenial {
    type Target = str;
    fn deref(&self) -> &str {
        &self.message
    }
}
impl std::fmt::Display for CheckpointDecodeDenial {
    fn fmt(&self, writer: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writer.write_str(&self.message)
    }
}
impl std::error::Error for CheckpointDecodeDenial {}
