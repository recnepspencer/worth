//! Stream one owner's recorded component into exactly admitted payload backing.
use super::{copying, RecordedRelationalComponentReference, RelationalRecordedComponentRefusal};
use std::io::{self, Write};
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest, ExecutionRequestArrayBuilder};

impl RecordedRelationalComponentReference {
    /// Encode descriptive Native source coordinates. This grants no readmission
    /// or publication authority; the receiver must readmit the original source.
    pub fn to_json_in(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<ExecutionImmutableBytes, RelationalRecordedComponentRefusal> {
        let size = emit(self, request, Destination::Count(0))?.count();
        let backing = ExecutionRequestArrayBuilder::allocate(*request, size)
            .map_err(RelationalRecordedComponentRefusal::PayloadBacking)?;
        match emit(self, request, Destination::Backing(backing))? {
            Destination::Backing(backing) => backing
                .seal_immutable_bytes()
                .map_err(RelationalRecordedComponentRefusal::PayloadBacking),
            Destination::Count(_) => unreachable!("emission preserves its admitted destination"),
        }
    }
}

enum Destination<'request, 'authority> {
    Count(usize),
    Backing(ExecutionRequestArrayBuilder<'request, 'authority, u8>),
}
impl Destination<'_, '_> {
    fn count(self) -> usize {
        match self {
            Self::Count(size) => size,
            Self::Backing(_) => unreachable!("only the allocation-free sizing pass is counted"),
        }
    }
}
struct ComponentWriter<'request, 'authority> {
    request: ExecutionRequest<'request, 'authority>,
    destination: Destination<'request, 'authority>,
    refusal: Option<RelationalRecordedComponentRefusal>,
}
impl Write for ComponentWriter<'_, '_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let result = (|| {
            self.request
                .run_native_with_checkpoint(|context| {
                    if context.has_accounting() {
                        context.checkpoint(bytes.len() as u64)
                    } else {
                        context.poll()
                    }
                })
                .map_err(|cause| RelationalRecordedComponentRefusal::Request(cause.into()))?
                .0
                .map_err(|cause| {
                    RelationalRecordedComponentRefusal::Request(
                        worth_execution::WorkCeilingDenial::Stopped(cause).into(),
                    )
                })?;
            match &mut self.destination {
                Destination::Count(count) => {
                    *count = count
                        .checked_add(bytes.len())
                        .ok_or_else(copying::malformed)?;
                }
                Destination::Backing(backing) => backing
                    .extend_from_slice(bytes)
                    .map_err(RelationalRecordedComponentRefusal::PayloadBacking)?,
            }
            Ok(bytes.len())
        })();
        result.map_err(|cause| {
            self.refusal = Some(cause);
            io::Error::other("Native recorded component emission refused")
        })
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
fn emit<'request, 'authority>(
    reference: &RecordedRelationalComponentReference,
    request: &ExecutionRequest<'request, 'authority>,
    destination: Destination<'request, 'authority>,
) -> Result<Destination<'request, 'authority>, RelationalRecordedComponentRefusal> {
    let mut writer = ComponentWriter {
        request: *request,
        destination,
        refusal: None,
    };
    serde_json::to_writer(&mut writer, reference)
        .map_err(|_| writer.refusal.take().unwrap_or_else(copying::malformed))?;
    Ok(writer.destination)
}
