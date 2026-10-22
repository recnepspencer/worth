//! Canonical actor-envelope JSON in its native, execution-owned byte backing.
use std::io::{self, Write};

use serde::{Serialize, Serializer};
use worth_execution::{ExecutionByteBuffer, ExecutionImmutableBytes, ExecutionRequest};

use crate::domain_computation::authorization::WorkflowApprovalRequestEncodingDenial as Denial;

/// Clones retain the same text allocation and its original payer. No owned
/// String can be extracted; an independent field copy requires fresh admission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) struct FundedActorText(ExecutionImmutableBytes);

impl FundedActorText {
    pub(in crate::domain_computation) fn bytes(&self) -> &ExecutionImmutableBytes {
        &self.0
    }
    pub(in crate::domain_computation) fn as_str(&self) -> &str {
        std::str::from_utf8(self.0.bytes()).expect("serde JSON emits UTF-8")
    }
}

impl Serialize for FundedActorText {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl std::ops::Deref for FundedActorText {
    type Target = str;
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for FundedActorText {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

pub(in crate::domain_computation) fn encode(
    execution: &ExecutionRequest<'_, '_>,
    value: &impl Serialize,
    maximum: usize,
) -> Result<FundedActorText, Denial> {
    let mut counter = EnvelopeWriter {
        execution,
        output: None,
        written: 0,
        maximum,
        denial: None,
    };
    let counted = serde_json::to_writer(&mut counter, value);
    counter.finish(counted)?;
    let output = ExecutionByteBuffer::allocate(counter.written, execution.allocation_policy())
        .map_err(native_denial)?;
    let mut writer = EnvelopeWriter {
        execution,
        output: Some(output),
        written: 0,
        maximum: counter.written,
        denial: None,
    };
    let serialized = serde_json::to_writer(&mut writer, value);
    writer.finish(serialized)?;
    let output = writer.output.take().expect("serialization owns its buffer");
    Ok(FundedActorText(output.seal().map_err(native_denial)?))
}

struct EnvelopeWriter<'a, 'scope, 'authority> {
    execution: &'a ExecutionRequest<'scope, 'authority>,
    output: Option<ExecutionByteBuffer>,
    written: usize,
    maximum: usize,
    denial: Option<Denial>,
}

impl EnvelopeWriter<'_, '_, '_> {
    fn finish(&mut self, result: Result<(), serde_json::Error>) -> Result<(), Denial> {
        if let Some(denial) = self.denial.take() {
            return Err(denial);
        }
        result.map_err(|_| Denial::InvalidRequest)
    }

    fn append(&mut self, bytes: &[u8]) -> Result<(), Denial> {
        check_live(self.execution).map_err(|cause| Denial::Native(Box::new(cause)))?;
        let next = self
            .written
            .checked_add(bytes.len())
            .ok_or(Denial::TooLarge)?;
        if next > self.maximum {
            return Err(Denial::TooLarge);
        }
        if let Some(output) = &mut self.output {
            output.extend_from_slice(bytes).map_err(native_denial)?;
        }
        self.written = next;
        Ok(())
    }
}

impl Write for EnvelopeWriter<'_, '_, '_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        match self.append(bytes) {
            Ok(()) => Ok(bytes.len()),
            Err(denial) => {
                self.denial = Some(denial);
                Err(io::Error::other("workflow actor envelope stopped"))
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn native_denial(cause: worth_execution::ExecutionAllocationDenial) -> Denial {
    Denial::Native(Box::new(super::storage_denial(
        cause,
        "workflow actor JSON encoding",
    )))
}

pub(in crate::domain_computation) fn check_live(
    execution: &ExecutionRequest<'_, '_>,
) -> Result<(), crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial> {
    execution.check_live().map_err(|stop| {
            use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
            let cause = match stop {
                worth_execution::MapKernelStop::Cancelled => Cause::Cancelled,
                worth_execution::MapKernelStop::DeadlineElapsed => Cause::DeadlineElapsed,
                worth_execution::MapKernelStop::WorkCeiling => Cause::WorkExhausted,
                worth_execution::MapKernelStop::WorkCounterOverflow => Cause::WorkCounterOverflow,
                worth_execution::MapKernelStop::AccountingUnavailable => Cause::AccountingUnavailable,
                worth_execution::MapKernelStop::NestedStopped => Cause::NestedStopped,
            };
        crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::relational_read_stopped(cause, "workflow actor JSON payload")
    })
}
