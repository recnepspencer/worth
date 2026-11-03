use crate::durability::data::{
    DurabilityError, RecoveryFailureClass, RelationalNativeCheckpointCaptureDenial as Denial,
};
use std::io::{self, Write};
mod control;
use control::CheckpointEncodingBuffer;
pub(crate) use control::CheckpointEncodingControl;
use worth_execution::ExecutionImmutableBytes;

struct SizingWriter<'scope, 'authority> {
    control: CheckpointEncodingControl<'scope, 'authority>,
    total: usize,
    failure: Option<Denial>,
}

impl<'scope, 'authority> SizingWriter<'scope, 'authority> {
    pub fn new(control: CheckpointEncodingControl<'scope, 'authority>) -> Self {
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

impl Write for SizingWriter<'_, '_> {
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

struct EmissionWriter<'a, 'request, 'authority> {
    control: CheckpointEncodingControl<'request, 'authority>,
    bytes: &'a mut CheckpointEncodingBuffer<'request, 'authority>,
    failure: Option<Denial>,
}

impl<'a, 'request, 'authority> EmissionWriter<'a, 'request, 'authority> {
    pub fn new(
        control: CheckpointEncodingControl<'request, 'authority>,
        bytes: &'a mut CheckpointEncodingBuffer<'request, 'authority>,
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

impl Write for EmissionWriter<'_, '_, '_> {
    fn write(&mut self, input: &[u8]) -> io::Result<usize> {
        if self.failure.is_some() {
            return Err(stopped());
        }
        for chunk in input.chunks(4096) {
            let result = self
                .control
                .checkpoint(chunk.len() as u64)
                .and_then(|()| self.bytes.append(chunk));
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
    control: CheckpointEncodingControl<'_, '_>,
    value: &impl serde::Serialize,
) -> Result<usize, Denial> {
    let mut writer = SizingWriter::new(control);
    let result = rmp_serde::encode::write_named(&mut writer, value);
    writer.finish(result)
}

pub(crate) fn encode_value(
    control: CheckpointEncodingControl<'_, '_>,
    prefix: &[u8],
    value: &impl serde::Serialize,
) -> Result<ExecutionImmutableBytes, Denial> {
    let total = count_value(control, value)?
        .checked_add(prefix.len())
        .ok_or_else(|| frame_mismatch("native checkpoint frame size overflow"))?;
    let mut bytes = CheckpointEncodingBuffer::allocate(control, total)?;
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
    control.checkpoint(0)?;
    bytes.seal()
}

pub(crate) fn copy_bytes(
    control: CheckpointEncodingControl<'_, '_>,
    input: &[u8],
) -> Result<ExecutionImmutableBytes, Denial> {
    let mut bytes = CheckpointEncodingBuffer::allocate(control, input.len())?;
    let mut writer = EmissionWriter::new(control, &mut bytes);
    let copied = writer.write_all(input);
    writer.finish(copied)?;
    control.checkpoint(0)?;
    bytes.seal()
}
