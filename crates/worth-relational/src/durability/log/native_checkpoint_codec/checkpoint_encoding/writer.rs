use crate::durability::data::{
    DurabilityError, RecoveryFailureClass, RelationalNativeCheckpointCaptureDenial as Denial,
};
use std::io::{self, Write};
use worth_execution::{ExecutionRequest, ExecutionRequestArrayBuilder};

pub(super) struct SizingWriter<'scope, 'authority> {
    request: ExecutionRequest<'scope, 'authority>,
    total: usize,
    failure: Option<Denial>,
}

impl<'scope, 'authority> SizingWriter<'scope, 'authority> {
    pub fn new(request: ExecutionRequest<'scope, 'authority>) -> Self {
        Self {
            request,
            total: 0,
            failure: None,
        }
    }
    pub fn finish(self, result: Result<(), impl std::fmt::Display>) -> Result<usize, Denial> {
        complete(result, self.failure)?;
        self.request
            .consult()
            .map_err(|cause| Denial::Request(cause.into()))?;
        Ok(self.total)
    }
}

impl Write for SizingWriter<'_, '_> {
    fn write(&mut self, input: &[u8]) -> io::Result<usize> {
        if self.failure.is_some() {
            return Err(stopped());
        }
        if let Err(error) = self.request.consult() {
            self.failure = Some(Denial::Request(error.into()));
            return Err(stopped());
        }
        match self
            .total
            .checked_add(input.len())
            .filter(|total| *total <= isize::MAX as usize)
        {
            Some(total) => {
                self.total = total;
                Ok(input.len())
            }
            None => {
                self.failure = Some(
                    DurabilityError::new(
                        RecoveryFailureClass::CheckpointSizeOverflow,
                        "native checkpoint encoded byte length is not representable",
                    )
                    .into(),
                );
                Err(stopped())
            }
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub(super) struct EmissionWriter<'a, 'scope, 'authority> {
    bytes: &'a mut ExecutionRequestArrayBuilder<'scope, 'authority, u8>,
    failure: Option<Denial>,
}

impl<'a, 'scope, 'authority> EmissionWriter<'a, 'scope, 'authority> {
    pub fn new(bytes: &'a mut ExecutionRequestArrayBuilder<'scope, 'authority, u8>) -> Self {
        Self {
            bytes,
            failure: None,
        }
    }
    pub fn finish(self, result: Result<(), impl std::fmt::Display>) -> Result<(), Denial> {
        complete(result, self.failure)?;
        Ok(())
    }
}

impl Write for EmissionWriter<'_, '_, '_> {
    fn write(&mut self, input: &[u8]) -> io::Result<usize> {
        if self.failure.is_some() {
            return Err(stopped());
        }
        match self.bytes.extend_from_slice(input) {
            Ok(()) => Ok(input.len()),
            Err(error) => {
                self.failure =
                    Some(crate::durability::checkpoint_history::capture::array_capture_stop(error));
                Err(stopped())
            }
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn complete(
    result: Result<(), impl std::fmt::Display>,
    failure: Option<Denial>,
) -> Result<(), Denial> {
    if let Some(error) = failure {
        return Err(error);
    }
    result.map_err(encoding_failure)
}

pub(super) fn encoding_failure(error: impl std::fmt::Display) -> Denial {
    DurabilityError::new(
        RecoveryFailureClass::DurableIoFailure,
        format!("failed to encode native checkpoint: {error}"),
    )
    .into()
}

pub(super) fn frame_mismatch(detail: &str) -> Denial {
    DurabilityError::new(RecoveryFailureClass::CheckpointFrameSizeMismatch, detail).into()
}

fn stopped() -> io::Error {
    io::Error::other("native checkpoint byte writer stopped")
}
