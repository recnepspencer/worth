use worth_execution::{ExecutionAllocationPolicy, ExecutionByteBuffer, ExecutionImmutableBytes};

use super::super::persisted_checkpoint::{
    CaptureSectionRecorder, PersistedDurableCheckpointFileRef,
};
use crate::durability::data::{
    DurableCheckpoint, NativeCheckpointSectionBytes, RelationalNativeCheckpointCaptureDenial,
};

mod writer;
use writer::{frame_mismatch, EmissionWriter, SizingWriter};

/// Count and emit the same borrowed image. Image/alias/serializer metadata heaps
/// remain separate from the selected fixed payload backing admission.
pub(crate) fn encode_checkpoint(
    checkpoint: &DurableCheckpoint,
    policy: ExecutionAllocationPolicy<'_, '_>,
) -> Result<
    (ExecutionImmutableBytes, NativeCheckpointSectionBytes),
    RelationalNativeCheckpointCaptureDenial,
> {
    policy.check_live()?;
    let mut sizing = SizingWriter::new(policy);
    let counted = rmp_serde::encode::write_named(
        &mut sizing,
        &PersistedDurableCheckpointFileRef::new(checkpoint),
    );
    let total = sizing.finish(counted)?;
    let mut bytes = ExecutionByteBuffer::allocate(total, policy)?;
    let recorder = CaptureSectionRecorder::default();
    let mut emission = EmissionWriter::new(&mut bytes);
    let emitted = {
        let mut writer = recorder.writer(&mut emission);
        rmp_serde::encode::write_named(
            &mut writer,
            &PersistedDurableCheckpointFileRef::measured(checkpoint, &recorder),
        )
    };
    emission.finish(emitted)?;
    if bytes.len() != total {
        return Err(frame_mismatch(
            "native checkpoint count and emission lengths differ",
        ));
    }
    let sections = recorder.finish(total).ok_or_else(|| {
        frame_mismatch("native checkpoint section accounting did not cover emitted bytes")
    })?;
    Ok((bytes.seal()?, sections))
}
