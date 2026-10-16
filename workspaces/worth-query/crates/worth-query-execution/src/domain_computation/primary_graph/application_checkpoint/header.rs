//! Intact current-contract refusal before interpreting native checkpoint bytes.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryPrimaryGraphInstallationDenial as Denial,
    WorthQueryPrimaryGraphInstallationDenialKind as Kind,
};

#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) struct CheckpointDecodeDenial {
    kind: Kind,
    message: String,
}
impl CheckpointDecodeDenial {
    pub(in crate::domain_computation::primary_graph) fn execution(
        cause: worth_execution::WorkCeilingDenial,
    ) -> Self {
        let native =
            worth_relational::facade::transactions::RelationalExecutionDenialCause::from(cause);
        Self::native_execution(
            native,
            format!("checkpoint reconstruction stopped: {cause:?}"),
        )
    }
    fn native_execution(
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
    pub(in crate::domain_computation::primary_graph) fn allocator() -> Self {
        Self {
            kind: Kind::ExecutionDenied {
                kind:
                    crate::domain_computation::WorthQueryProviderSessionDenialKind::AllocationDenied,
            },
            message: "checkpoint metadata allocation refused".to_owned(),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn capacity() -> Self {
        Self::execution(worth_execution::WorkCeilingDenial::Admission(
            worth_execution::LeaseDenial::ChargedBytesOverflow,
        ))
    }
    pub(in crate::domain_computation::primary_graph) fn from_fact(cause: FactDecodeDenial) -> Self {
        use super::super::application_attempt::retained_decision_facts::StoreDenial;
        use worth_execution::{ExecutionAllocationDenialClass, MapKernelStop, WorkCeilingDenial};
        match cause {
            FactDecodeDenial::Format(message) => message.into(),
            FactDecodeDenial::Execution(cause) => Self::native_execution(cause,
                "checkpoint fact reconstruction stopped".to_owned()),
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
            FactDecodeDenial::Retention(StoreDenial::Representability) => Self::capacity(),
            FactDecodeDenial::Retention(StoreDenial::ConflictingBody) =>
                "checkpoint facts contain conflicting bodies".to_owned().into(),
            #[cfg(test)]
            FactDecodeDenial::Retention(StoreDenial::InvalidOrdinal) =>
                "checkpoint facts contain an invalid fixture ordinal".to_owned().into(),
        }
    }
    pub(super) fn unsupported_producer_facts(observed: u16) -> Self {
        Self {
            kind: Kind::UnsupportedCheckpointProducerFactVersion {
                observed,
                supported: facts::WIRE_VERSION,
            },
            message: format!("checkpoint producer fact wire version {observed} is unsupported"),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn into_installation_denial(self) -> Denial {
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
pub(super) fn admit(bytes: &[u8]) -> Result<(), CheckpointDecodeDenial> {
    let body_start = MAGIC.len() + CHECKSUM_BYTES;
    // Check the old minimum framing before reading the declared version. A
    // well-formed displaced frame is unsupported, not a short current frame.
    if bytes.len() < body_start + 2 || &bytes[..MAGIC.len()] != MAGIC {
        return Err("Query application checkpoint header is invalid"
            .to_owned()
            .into());
    }
    let body = &bytes[body_start..];
    if Sha256::digest(body).as_slice() != &bytes[MAGIC.len()..body_start] {
        return Err("Query application checkpoint checksum differs"
            .to_owned()
            .into());
    }
    let version = u16::from_be_bytes([body[0], body[1]]);
    if version != FORMAT_VERSION {
        return Err(CheckpointDecodeDenial {
            kind: Kind::UnsupportedCheckpointFormat {
                observed: version,
                supported: FORMAT_VERSION,
            },
            message: format!("Query application checkpoint format {version} is unsupported"),
        });
    }
    if bytes.len() < HEADER_BYTES {
        return Err("Query application checkpoint header is invalid"
            .to_owned()
            .into());
    }
    let facts = u16::from_be_bytes([body[2], body[3]]);
    let instances = u16::from_be_bytes([body[4], body[5]]);
    if facts != WORKFLOW_FACTS || instances != WORKFLOW_INSTANCES {
        return Err(CheckpointDecodeDenial {
            kind: Kind::UnsupportedCheckpointWorkflowContract { facts, instances },
            message: "Query application checkpoint workflow contract is unsupported".to_owned(),
        });
    }
    Ok(())
}
