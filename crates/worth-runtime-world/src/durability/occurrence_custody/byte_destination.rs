//! The actual pre-effect byte destination cannot be silently replaced after seal.
use super::*;
use crate::durability::artifact::WorldArtifactFrame;
use worth_execution::{ExecutionByteBuffer, ExecutionMemoryReservation, ExecutionRequest};

#[derive(Debug)]
enum DestinationPosture {
    Available,
    Consumed,
}
#[derive(Debug)]
pub(super) struct OccurrenceByteDestination {
    backing: Option<ExecutionByteBuffer>,
    posture: DestinationPosture,
    funding: Option<ExecutionMemoryReservation>,
}
impl OccurrenceByteDestination {
    pub(super) fn prepare(
        bytes: usize,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, WorldDurableMediumDenial> {
        // A leased buffer owns its grant internally. System allocation used by
        // serial requests is explicitly uncharged, so retain its request grant
        // alongside the same buffer and later its sealed frame instead.
        let funding = if request.is_leased() {
            None
        } else {
            let quote = u64::try_from(bytes).map_err(|_| {
                WorldDurableMediumDenial::Frame(super::super::WorldDurableFrameDenial::SizeLimit)
            })?;
            request
                .reserve_retained_memory(quote)
                .map_err(WorldDurableMediumDenial::Request)?
        };
        let backing = ExecutionByteBuffer::allocate(bytes, request.allocation_policy())
            .map_err(WorldDurableMediumDenial::Allocation)?;
        Ok(Self {
            backing: Some(backing),
            posture: DestinationPosture::Available,
            funding,
        })
    }
    pub(super) fn backing(&mut self) -> Result<&mut ExecutionByteBuffer, WorldDurableMediumDenial> {
        match self.posture {
            DestinationPosture::Available => Ok(self
                .backing
                .as_mut()
                .expect("available destination keeps its original backing")),
            DestinationPosture::Consumed => Err(WorldDurableMediumDenial::OccurrenceReopenRequired),
        }
    }
    pub(super) fn seal(&mut self) -> Result<WorldArtifactFrame, WorldDurableMediumDenial> {
        match self.posture {
            DestinationPosture::Available => {}
            DestinationPosture::Consumed => {
                return Err(WorldDurableMediumDenial::OccurrenceReopenRequired)
            }
        }
        // This state changes before take/seal, including their unwind paths.
        self.posture = DestinationPosture::Consumed;
        let funding = self.funding.take();
        let bytes = self
            .backing
            .take()
            .expect("one original destination is consumed")
            .seal()
            .map_err(WorldDurableMediumDenial::Allocation)?;
        Ok(WorldArtifactFrame::from_streamed(bytes, funding))
    }
}
