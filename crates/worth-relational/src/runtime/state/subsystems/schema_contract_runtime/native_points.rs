//! Request-funded, immutable execution-point selection shared with its configuration.
use crate::execution::RelationalExecutionDenialCause as Denial;
use crate::schema::SchemaCompilationControl;
use crate::validation::data::{InvariantExecutionPoint as Point, InvariantRegistration};
use std::{alloc::Layout, ops::Deref, sync::Arc};
#[derive(Debug)]
pub(crate) struct NativeInvariantPoints {
    points: [Vec<InvariantRegistration>; Point::COUNT],
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl Deref for NativeInvariantPoints {
    type Target = [Vec<InvariantRegistration>; Point::COUNT];
    fn deref(&self) -> &Self::Target {
        &self.points
    }
}
impl NativeInvariantPoints {
    pub(super) fn compile(
        declared: &[InvariantRegistration],
        native: &[InvariantRegistration],
        control: SchemaCompilationControl<'_, '_>,
    ) -> Result<Arc<Self>, Denial> {
        let mut counts = [0usize; Point::COUNT];
        let mut bytes = Layout::new::<[usize; 2]>()
            .extend(Layout::new::<Self>())
            .map_err(|_| Denial::MemoryOverflow)?
            .0
            .pad_to_align()
            .size();
        for (point, count) in counts.iter_mut().enumerate() {
            for registration in declared.iter().chain(native) {
                control.checkpoint()?;
                if includes(point, registration) {
                    *count = count.checked_add(1).ok_or(Denial::MemoryOverflow)?;
                    let nested =
                        usize::try_from(registration.rule.owned_allocation_capacity_bytes())
                            .map_err(|_| Denial::MemoryOverflow)?;
                    bytes = bytes
                        .checked_add(size_of::<InvariantRegistration>())
                        .and_then(|n| n.checked_add(nested))
                        .ok_or(Denial::MemoryOverflow)?;
                }
            }
        }
        let memory = control.reserve(bytes)?;
        let mut points = std::array::from_fn(|point| Vec::with_capacity(counts[point]));
        for (point, entries) in points.iter_mut().enumerate() {
            for rejects_deletion in [true, false] {
                for registration in declared.iter().chain(native) {
                    control.checkpoint()?;
                    if includes(point, registration)
                        && rejects_live_endpoint_deletion(registration) == rejects_deletion
                    {
                        entries.push(registration.clone());
                    }
                }
            }
        }
        Ok(Arc::new(Self {
            points,
            _memory: memory,
        }))
    }
}
fn includes(point: usize, registration: &InvariantRegistration) -> bool {
    point == registration.execution_point as usize
        || (point == Point::ProposedState as usize
            && Point::ProposedState.includes(registration.execution_point))
}

fn rejects_live_endpoint_deletion(registration: &InvariantRegistration) -> bool {
    matches!(
        &registration.rule,
        crate::validation::data::InvariantRule::EndpointDeletionIntegrityContract(contract)
            if contract.mode
                == crate::schema::data::EndpointDeletionIntegrityMode::RejectDeleteWithLiveRelations
    )
}
