use crate::durability::data::{
    DurabilityError, RecoveryFailureClass, RelationalNativeCheckpointCaptureDenial as Denial,
};
use std::io::{self, Write};
mod control;
pub(crate) use control::CheckpointEncodingControl;
use worth_execution::{ExecutionByteBuffer, ExecutionImmutableBytes};

/// The writers charge through the capture's one control; see its work rate.
type Control<'c, 'stage, 'scope, 'lease, 'request, 'authority> =
    &'c mut CheckpointEncodingControl<'stage, 'scope, 'lease, 'request, 'authority>;

struct SizingWriter<'c, 'stage, 'scope, 'lease, 'request, 'authority> {
    control: Control<'c, 'stage, 'scope, 'lease, 'request, 'authority>,
    total: usize,
    failure: Option<Denial>,
}

impl<'c, 'stage, 'scope, 'lease, 'request, 'authority>
    SizingWriter<'c, 'stage, 'scope, 'lease, 'request, 'authority>
{
    pub fn new(control: Control<'c, 'stage, 'scope, 'lease, 'request, 'authority>) -> Self {
        Self {
            control,
            total: 0,
            failure: None,
        }
    }
    pub fn finish(self, result: Result<(), impl std::fmt::Display>) -> Result<usize, Denial> {
        complete(result, self.failure)?;
        self.control.checkpoint(0)?;
        Ok(self.total)
    }
}

impl Write for SizingWriter<'_, '_, '_, '_, '_, '_> {
    fn write(&mut self, input: &[u8]) -> io::Result<usize> {
        if self.failure.is_some() {
            return Err(stopped());
        }
        for chunk in input.chunks(4096) {
            if let Err(error) = self.control.checkpoint(chunk.len() as u64) {
                self.failure = Some(error);
                return Err(stopped());
            }
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

struct EmissionWriter<'a, 'c, 'stage, 'scope, 'lease, 'request, 'authority> {
    control: Control<'c, 'stage, 'scope, 'lease, 'request, 'authority>,
    bytes: &'a mut ExecutionByteBuffer,
    failure: Option<Denial>,
}

impl<'a, 'c, 'stage, 'scope, 'lease, 'request, 'authority>
    EmissionWriter<'a, 'c, 'stage, 'scope, 'lease, 'request, 'authority>
{
    pub fn new(
        control: Control<'c, 'stage, 'scope, 'lease, 'request, 'authority>,
        bytes: &'a mut ExecutionByteBuffer,
    ) -> Self {
        Self {
            control,
            bytes,
            failure: None,
        }
    }
    pub fn finish(self, result: Result<(), impl std::fmt::Display>) -> Result<(), Denial> {
        complete(result, self.failure)?;
        self.control.checkpoint(0)?;
        Ok(())
    }
}

impl Write for EmissionWriter<'_, '_, '_, '_, '_, '_, '_> {
    fn write(&mut self, input: &[u8]) -> io::Result<usize> {
        if self.failure.is_some() {
            return Err(stopped());
        }
        for chunk in input.chunks(4096) {
            let result = self.control.checkpoint(chunk.len() as u64).and_then(|()| {
                self.bytes
                    .extend_from_slice(chunk)
                    .map_err(Denial::Allocation)
            });
            if let Err(error) = result {
                self.failure = Some(error);
                return Err(stopped());
            }
        }
        Ok(input.len())
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

pub(crate) fn count_value(
    control: &mut CheckpointEncodingControl<'_, '_, '_, '_, '_>,
    value: &impl serde::Serialize,
) -> Result<usize, Denial> {
    let mut writer = SizingWriter::new(control);
    let result = rmp_serde::encode::write_named(&mut writer, value);
    writer.finish(result)
}

pub(crate) fn encode_value(
    control: &mut CheckpointEncodingControl<'_, '_, '_, '_, '_>,
    prefix: &[u8],
    value: &impl serde::Serialize,
) -> Result<ExecutionImmutableBytes, Denial> {
    let total = count_value(control, value)?
        .checked_add(prefix.len())
        .ok_or_else(|| frame_mismatch("native checkpoint frame size overflow"))?;
    let mut bytes = control.allocate(total)?;
    let mut writer = EmissionWriter::new(control, &mut bytes);
    let emitted = writer.write_all(prefix).and_then(|()| {
        rmp_serde::encode::write_named(&mut writer, value).map_err(io::Error::other)
    });
    writer.finish(emitted)?;
    if bytes.len() != total {
        return Err(frame_mismatch(
            "native checkpoint count and emission lengths differ",
        ));
    }
    bytes.seal().map_err(Denial::Allocation)
}

pub(crate) fn copy_bytes(
    control: &mut CheckpointEncodingControl<'_, '_, '_, '_, '_>,
    input: &[u8],
) -> Result<ExecutionImmutableBytes, Denial> {
    let mut bytes = control.allocate(input.len())?;
    let mut writer = EmissionWriter::new(control, &mut bytes);
    let copied = writer.write_all(input);
    writer.finish(copied)?;
    bytes.seal().map_err(Denial::Allocation)
}
