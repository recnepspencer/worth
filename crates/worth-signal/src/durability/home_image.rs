//! Opaque original owner frames form the Signal section of a Home checkpoint.
use super::{
    frame, SignalDurableReadSource, SignalOwnerReadmissionDenial as D,
    SignalRecordedIdentityContribution,
};
use std::{ops::Range, sync::Arc};
use worth_execution::{
    ExecutionImmutableBytes, ExecutionMemoryReservation, ExecutionRequest,
    ExecutionRequestArrayBuilder,
};
use worth_foundational::facade::{DurableSignalOwnerIdentity, SignalDurableJournal};
mod readmission;
mod tail;

pub const SIGNAL_HOME_IMAGE_MAXIMUM_BYTES: usize = 1 << 30;
const KIND: u32 = 5;

struct ImageSource {
    bytes: ExecutionImmutableBytes,
    region: Range<usize>,
    bounds: SignalRecordedIdentityContribution,
    definitions: Vec<Range<usize>>,
    advances: Vec<Range<usize>>,
    _memory: Option<ExecutionMemoryReservation>,
}
/// Descriptive, checksummed original source. Admission still requires the actual
/// medium, installation bindings and owner claim. No derived state is encoded.
#[derive(Clone)]
pub struct SignalAuthoredHomeImage {
    source: Arc<ImageSource>,
}
impl std::fmt::Debug for SignalAuthoredHomeImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignalAuthoredHomeImage")
            .field("owner", &self.owner())
            .field("definitions", &self.source.definitions.len())
            .field("advances", &self.source.advances.len())
            .finish()
    }
}
impl SignalAuthoredHomeImage {
    pub fn owner(&self) -> DurableSignalOwnerIdentity {
        self.source.bounds.owner()
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.source.bytes.bytes()[self.source.region.clone()]
    }
    pub fn readmit_region(
        execution: &ExecutionRequest<'_, '_>,
        bytes: ExecutionImmutableBytes,
        region: Range<usize>,
    ) -> Result<Self, D> {
        readmission::readmit(execution, bytes, region)
    }
    pub(crate) fn capture(
        source: &dyn SignalDurableReadSource,
        bounds: SignalRecordedIdentityContribution,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, D> {
        if source.owner() != bounds.owner() {
            return Err(D::ForeignOwner);
        }
        let definitions = source
            .frontier(SignalDurableJournal::Definitions, execution)
            .map_err(D::Medium)?;
        let advances = source
            .frontier(SignalDurableJournal::Advances, execution)
            .map_err(D::Medium)?;
        let count: usize = definitions
            .checked_add(advances)
            .and_then(|count| usize::try_from(count).ok())
            .filter(|count| *count <= SIGNAL_HOME_IMAGE_MAXIMUM_BYTES / (frame::HEADER + 8))
            .ok_or(D::MetadataCapacity)?;
        let mut records = ExecutionRequestArrayBuilder::allocate(*execution, count)
            .map_err(|cause| D::Frame(super::SignalDurableFrameDenial::Backing(cause)))?;
        for (journal, frontier) in [
            (SignalDurableJournal::Definitions, definitions),
            (SignalDurableJournal::Advances, advances),
        ] {
            for position in 1..=frontier {
                let bytes = source
                    .read(journal, position, execution)
                    .map_err(D::Medium)?;
                records
                    .push(bytes)
                    .map_err(|cause| D::Frame(super::SignalDurableFrameDenial::Backing(cause)))?;
            }
        }
        let records = records
            .seal()
            .map_err(|cause| D::Frame(super::SignalDurableFrameDenial::Backing(cause)))?;
        let bytes = frame::encode_bounded(
            KIND,
            execution,
            SIGNAL_HOME_IMAGE_MAXIMUM_BYTES,
            &mut |out| {
                bounds.write_image_bounds(out)?;
                let mut offset = 0usize;
                for count in [definitions, advances] {
                    out.number(count)?;
                    for _ in 0..count {
                        out.blob(records.elements()[offset].bytes())?;
                        offset += 1;
                    }
                }
                Ok(())
            },
        )
        .map_err(D::Frame)?;
        let end = bytes.len();
        Self::readmit_region(execution, bytes, 0..end)
    }
    fn locations(&self, journal: SignalDurableJournal) -> &[Range<usize>] {
        match journal {
            SignalDurableJournal::Definitions => &self.source.definitions,
            SignalDurableJournal::Advances => &self.source.advances,
        }
    }
    pub fn covered_through(&self, journal: SignalDurableJournal) -> u64 {
        self.locations(journal).len() as u64
    }
    fn read(
        &self,
        journal: SignalDurableJournal,
        position: u64,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<ExecutionImmutableBytes, super::SignalDurableMediumDenial> {
        use super::SignalDurableMediumDenial as M;
        let location = position
            .checked_sub(1)
            .and_then(|p| usize::try_from(p).ok())
            .and_then(|p| self.locations(journal).get(p))
            .ok_or(M::Unavailable)?;
        let bytes = &self.source.bytes.bytes()[location.clone()];
        frame::admit_work(execution, bytes.len()).map_err(M::Frame)?;
        frame::copy_bytes(execution, bytes).map_err(M::Frame)
    }
}
