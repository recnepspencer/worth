use crate::durability::data::{
    DurabilityError, RecoveryFailureClass, RelationalNativeCheckpointCaptureDenial as Denial,
};
use std::io::{self, Write};
use worth_execution::{ExecutionAllocationPolicy, ExecutionByteBuffer};

pub(super) struct SizingWriter<'scope, 'authority> {
    policy: ExecutionAllocationPolicy<'scope, 'authority>,
    total: usize,
    failure: Option<Denial>,
}

impl<'scope, 'authority> SizingWriter<'scope, 'authority> {
    pub fn new(policy: ExecutionAllocationPolicy<'scope, 'authority>) -> Self {
        Self {
            policy,
            total: 0,
            failure: None,
        }
    }
    pub fn finish(self, result: Result<(), impl std::fmt::Display>) -> Result<usize, Denial> {
        complete(result, self.failure)?;
        self.policy.check_live()?;
        Ok(self.total)
    }
}

impl Write for SizingWriter<'_, '_> {
    fn write(&mut self, input: &[u8]) -> io::Result<usize> {
        if self.failure.is_some() {
            return Err(stopped());
        }
        if let Err(error) = self.policy.check_live() {
            self.failure = Some(error.into());
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

pub(super) struct EmissionWriter<'a> {
    bytes: &'a mut ExecutionByteBuffer,
    failure: Option<Denial>,
}

impl<'a> EmissionWriter<'a> {
    pub fn new(bytes: &'a mut ExecutionByteBuffer) -> Self {
        Self {
            bytes,
            failure: None,
        }
    }
    pub fn finish(self, result: Result<(), impl std::fmt::Display>) -> Result<(), Denial> {
        complete(result, self.failure)?;
        self.bytes.check_live()?;
        Ok(())
    }
}

impl Write for EmissionWriter<'_> {
    fn write(&mut self, input: &[u8]) -> io::Result<usize> {
        if self.failure.is_some() {
            return Err(stopped());
        }
        match self.bytes.extend_from_slice(input) {
            Ok(()) => Ok(input.len()),
            Err(error) => {
                self.failure = Some(error.into());
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
