//! One original allocation keeps immutable meaning, frame and their funding together.
use super::*;
use crate::durability::{DecodedWorldRecord, WorldDurableMediumDenial as Denial};
use std::sync::OnceLock;
use worth_execution::{ExecutionImmutableBytes, ExecutionMemoryReservation, ExecutionRequest};

#[derive(Debug)]
enum WorldArtifactRecordOrigin {
    Owned(WorldRecord),
    Readmitted(Arc<DecodedWorldRecord>),
}
#[derive(Debug)]
pub(crate) struct WorldArtifactFrame {
    bytes: ExecutionImmutableBytes,
    _extra_funding: Option<ExecutionMemoryReservation>,
}
impl WorldArtifactFrame {
    pub(super) fn from_encoded(bytes: ExecutionImmutableBytes) -> Self {
        Self {
            bytes,
            _extra_funding: None,
        }
    }
    pub(in crate::durability) fn from_streamed(
        bytes: ExecutionImmutableBytes,
        funding: Option<ExecutionMemoryReservation>,
    ) -> Self {
        Self {
            bytes,
            _extra_funding: funding,
        }
    }
}
#[derive(Debug)]
pub(super) struct WorldArtifactMeaning {
    record: OnceLock<WorldArtifactRecordOrigin>,
    bytes: OnceLock<WorldArtifactFrame>,
    _funding: Option<ExecutionMemoryReservation>,
}
impl WorldArtifactMeaning {
    pub(super) fn record(&self) -> &WorldRecord {
        match self
            .record
            .get()
            .expect("only completed artifacts expose meaning")
        {
            WorldArtifactRecordOrigin::Owned(record) => record,
            WorldArtifactRecordOrigin::Readmitted(record) => &record.record,
        }
    }
    pub(super) fn bytes(&self) -> &[u8] {
        self.bytes
            .get()
            .expect("only completed artifacts expose their frame")
            .bytes
            .bytes()
    }
}

pub(crate) struct PreparedWorldArtifact {
    original: Arc<WorldArtifactMeaning>,
}
impl std::fmt::Debug for PreparedWorldArtifact {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreparedWorldArtifact")
            .finish_non_exhaustive()
    }
}
impl PreparedWorldArtifact {
    pub(super) fn prepare(
        names: usize,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Denial> {
        let layout = std::alloc::Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
            .extend(std::alloc::Layout::new::<WorldArtifactMeaning>())
            .map_err(|_| WorldDurableFrameDenial::SizeLimit)?
            .0
            .pad_to_align();
        let quote = layout
            .size()
            .checked_add(names)
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(WorldDurableFrameDenial::SizeLimit)?;
        let funding = request
            .reserve_retained_memory(quote)
            .map_err(Denial::Request)?;
        Ok(Self {
            original: Arc::new(WorldArtifactMeaning {
                record: OnceLock::new(),
                bytes: OnceLock::new(),
                _funding: funding,
            }),
        })
    }
    pub(crate) fn prepare_occurrence(
        open: &WorldOpenAttemptRecord,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(Self, WorldOpenAttemptRecord), Denial> {
        let input = super::record_input::WorldOpenAttemptInput::from_record(open);
        let prepared = Self::prepare(input.name_bytes()?, request)?;
        let copied = input.materialize()?;
        Ok((prepared, copied))
    }
    pub(in crate::durability) fn complete_owned(
        self,
        record: WorldRecord,
        bytes: WorldArtifactFrame,
    ) -> WorldDurableArtifact {
        self.complete(WorldArtifactRecordOrigin::Owned(record), bytes)
    }
    pub(super) fn complete_readmitted(
        self,
        record: Arc<DecodedWorldRecord>,
        bytes: WorldArtifactFrame,
    ) -> WorldDurableArtifact {
        self.complete(WorldArtifactRecordOrigin::Readmitted(record), bytes)
    }
    fn complete(
        self,
        record: WorldArtifactRecordOrigin,
        bytes: WorldArtifactFrame,
    ) -> WorldDurableArtifact {
        self.original
            .record
            .set(record)
            .expect("one consumed preparation installs its meaning once");
        self.original
            .bytes
            .set(bytes)
            .expect("one consumed preparation installs its frame once");
        WorldDurableArtifact {
            original: self.original,
        }
    }
}
