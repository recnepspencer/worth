//! Incremental canonical commitment serialization under the invoking request.
use crate::durability::data::{
    DurabilityError, RecoveryFailureClass, RelationalRecoveryStop as Stop,
};
use sha2::{Digest, Sha256};
use std::io::{self, Write};
use worth_execution::{ExecutionRequest, MapKernelContext, MapKernelStop, WorkCeilingDenial};

pub(super) fn in_context<T>(
    request: &ExecutionRequest<'_, '_>,
    run: impl FnOnce(&mut MapKernelContext<'_, '_>) -> Result<T, Stop>,
) -> Result<T, Stop> {
    request
        .consult()
        .map_err(|cause| Stop::Request(cause.into()))?;
    let mut result = None;
    let scope = request.run_native_with_checkpoint(|context| {
        result = Some(run(context));
    });
    match (scope, result) {
        (Ok(_), Some(result)) => result,
        (Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped)), Some(Err(cause))) => {
            Err(cause)
        }
        (Err(cause), _) => Err(Stop::Request(cause.into())),
        (Ok(_), None) => unreachable!("completed original-request commitment callback"),
    }
}

pub(super) struct CommitmentWriter<'context, 'scope, 'authority> {
    pub(super) hash: Sha256,
    context: &'context mut MapKernelContext<'scope, 'authority>,
    stop: Option<MapKernelStop>,
}
impl<'c, 's, 'a> CommitmentWriter<'c, 's, 'a> {
    pub(super) fn new(hash: Sha256, context: &'c mut MapKernelContext<'s, 'a>) -> Self {
        Self {
            hash,
            context,
            stop: None,
        }
    }
    pub(super) fn value(&mut self, value: &impl serde::Serialize) -> Result<(), Stop> {
        let result = rmp_serde::encode::write_named(&mut *self, value);
        if let Some(cause) = self.stop.take() {
            return Err(Stop::Request(cause.into()));
        }
        result.map_err(|_| corrupt())
    }
    pub(super) fn bytes(&mut self, bytes: &[u8]) -> Result<(), Stop> {
        let result = self.write_all(bytes);
        if let Some(cause) = self.stop.take() {
            return Err(Stop::Request(cause.into()));
        }
        result.map_err(|_| corrupt())
    }
}
impl Write for CommitmentWriter<'_, '_, '_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        for chunk in bytes.chunks(64 * 1024) {
            let result = if self.context.has_accounting() {
                self.context.checkpoint(chunk.len() as u64)
            } else {
                self.context.poll()
            };
            if let Err(cause) = result {
                self.stop = Some(cause);
                return Err(io::ErrorKind::Other.into());
            }
            self.hash.update(chunk);
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
fn corrupt() -> Stop {
    DurabilityError::new(
        RecoveryFailureClass::CorruptCanonicalArtifact,
        "canonical commitment serializer refused admitted DATA",
    )
    .into()
}
