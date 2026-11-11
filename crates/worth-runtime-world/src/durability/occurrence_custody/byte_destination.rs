//! The actual pre-effect byte destination cannot be silently replaced after seal.
use super::*;
use crate::durability::artifact::WorldArtifactFrame;
use worth_execution::{
    ExecutionByteBuffer, ExecutionRequest, ExecutionRequestArrayDenial, MapKernelStop,
    WorkCeilingDenial,
};

#[derive(Debug)]
pub(super) enum DestinationPosture {
    Available,
    Consumed,
}
#[derive(Debug)]
pub(super) struct OccurrenceByteDestination {
    backing: Option<ExecutionByteBuffer>,
    posture: DestinationPosture,
}
impl OccurrenceByteDestination {
    pub(super) fn prepare(
        bytes: usize,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, WorldDurableMediumDenial> {
        request
            .consult()
            .map_err(WorldDurableMediumDenial::Request)?;
        let mut prepared = None;
        let outcome = request.run_native_with_checkpoint(|context| {
            prepared = Some((|| {
                context.poll().map_err(|cause| {
                    ExecutionRequestArrayDenial::Request(WorkCeilingDenial::Stopped(cause))
                })?;
                ExecutionByteBuffer::allocate_in_stage(bytes, *request, context)
            })());
        });
        let backing = match (outcome, prepared) {
            (Ok(_), Some(backing)) => backing,
            (Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped)), Some(Err(cause))) => {
                Err(cause)
            }
            (Err(cause), _) => Err(ExecutionRequestArrayDenial::Request(cause)),
            (Ok(_), None) => unreachable!("completed original destination allocation"),
        }
        .map_err(|cause| match cause {
            ExecutionRequestArrayDenial::Request(cause) => WorldDurableMediumDenial::Request(cause),
            ExecutionRequestArrayDenial::Allocation(cause) => {
                WorldDurableMediumDenial::Allocation(cause)
            }
        })?;
        Ok(Self {
            backing: Some(backing),
            posture: DestinationPosture::Available,
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
        // Consume before take/seal, including their refusal and unwind paths.
        self.posture.consume()?;
        let bytes = self
            .backing
            .take()
            .expect("one original destination is consumed")
            .seal()
            .map_err(WorldDurableMediumDenial::Allocation)?;
        Ok(WorldArtifactFrame::from_encoded(bytes))
    }
}

impl DestinationPosture {
    pub(super) fn consume(&mut self) -> Result<(), WorldDurableMediumDenial> {
        match self {
            Self::Available => {
                *self = Self::Consumed;
                Ok(())
            }
            Self::Consumed => Err(WorldDurableMediumDenial::OccurrenceReopenRequired),
        }
    }
}
