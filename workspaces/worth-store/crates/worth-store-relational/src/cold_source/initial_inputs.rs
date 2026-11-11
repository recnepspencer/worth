//! Native initialization bytes from the same protected physical cut.
use super::{Cause, StoreColdSource};
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest, ExecutionRequestArrayBuilder};
use worth_relational::facade::durability::RelationalDurableOwnedReadRefusal as Refusal;
use worth_store::physical_runtime::AnchoredRecordName;

impl StoreColdSource {
    pub(super) fn owned_initial_inputs(
        &self,
        request: &ExecutionRequest<'_, '_>,
        original: &(dyn std::any::Any + Send + Sync),
        limit: u64,
    ) -> Result<Option<ExecutionImmutableBytes>, Refusal> {
        let cut = self.validate_native_read(original)?;
        let records = self
            .serving()
            .anchored_records()
            .map_err(|_| Refusal::Storage(Cause::SourceUnavailable))?;
        let Some(mut stream) = records
            .open_payload_from(
                &cut.reader,
                request,
                AnchoredRecordName::new("relational.initial-inputs")
                    .expect("static Native initialization anchor"),
                limit,
            )
            .map_err(read_failure)?
        else {
            return Ok(None);
        };
        let length = usize::try_from(stream.payload_bytes())
            .map_err(|_| Refusal::Storage(Cause::MemoryExhausted))?;
        let mut bytes =
            ExecutionRequestArrayBuilder::allocate(*request, length).map_err(Refusal::Buffer)?;
        let mut scratch = [0; 16 * 1024];
        loop {
            request.consult().map_err(Refusal::from)?;
            let count = stream.read_next(&mut scratch).map_err(read_failure)?;
            if count == 0 {
                break;
            }
            if count > length - bytes.len() {
                return Err(Refusal::Storage(Cause::Damaged));
            }
            bytes
                .extend_from_slice(&scratch[..count])
                .map_err(Refusal::Buffer)?;
        }
        if bytes.len() != length || stream.verified_version().is_none() {
            return Err(Refusal::Storage(Cause::Damaged));
        }
        Ok(Some(bytes.seal_immutable_bytes().map_err(Refusal::Buffer)?))
    }
}
fn read_failure(cause: worth_store::physical_runtime::AnchoredRecordReadFailure) -> Refusal {
    use worth_store::physical_runtime::AnchoredRecordReadFailure as Failure;
    match cause {
        Failure::Execution(cause) => Refusal::from(cause),
        Failure::Allocation(cause) => Refusal::Buffer(
            worth_execution::ExecutionRequestArrayDenial::Allocation(cause),
        ),
        Failure::ForeignReader => Refusal::ForeignProtection,
        Failure::ProtectionBusy => Refusal::Busy,
        _ => Refusal::Storage(Cause::Damaged),
    }
}
