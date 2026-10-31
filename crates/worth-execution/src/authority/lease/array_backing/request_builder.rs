use super::super::fixed_backing::{BuildingFixedBacking, ExecutionAllocationDenial};
use super::ExecutionArray;
use crate::{ExecutionRequest, MapKernelStop, WorkCeilingDenial};

/// Original request refusal stays distinct from physical fixed-backing refusal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExecutionRequestArrayDenial {
    Request(WorkCeilingDenial),
    Allocation(ExecutionAllocationDenial),
}

/// Exact fixed storage authored while borrowing its genuine original request.
/// The immutable array retains only the same allocation's single memory ticket.
/// This constructor accepts no caller-supplied ticket or replacement request.
pub struct ExecutionRequestArrayBuilder<'request, 'authority, T> {
    request: ExecutionRequest<'request, 'authority>,
    building: BuildingFixedBacking<T>,
}

impl<'request, 'authority, T> ExecutionRequestArrayBuilder<'request, 'authority, T> {
    pub fn allocate(
        request: ExecutionRequest<'request, 'authority>,
        element_count: usize,
    ) -> Result<Self, ExecutionRequestArrayDenial> {
        use ExecutionRequestArrayDenial as Denial;
        request.consult().map_err(Denial::Request)?;
        let quote =
            BuildingFixedBacking::<T>::payload_quote(element_count).map_err(Denial::Allocation)?;
        let reservation = request
            .reserve_retained_memory(quote)
            .map_err(Denial::Request)?;
        let mut building = None;
        let outcome = request.run_native_with_checkpoint(|context| {
            building = Some((|| {
                context
                    .poll()
                    .map_err(|cause| Denial::Request(WorkCeilingDenial::Stopped(cause)))?;
                BuildingFixedBacking::allocate_reserved(element_count, reservation)
                    .map_err(Denial::Allocation)
            })());
        });
        let building = match (outcome, building) {
            (Ok(_), Some(building)) => building,
            (Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped)), Some(Err(cause))) => {
                Err(cause)
            }
            (Err(cause), _) => Err(Denial::Request(cause)),
            (Ok(_), None) => unreachable!("completed allocation callback"),
        }?;
        Ok(Self { request, building })
    }

    pub fn len(&self) -> usize {
        self.building.len()
    }
    pub fn is_empty(&self) -> bool {
        self.building.len() == 0
    }
    pub fn element_count(&self) -> usize {
        self.building.element_count()
    }
    pub fn elements(&self) -> &[T] {
        self.building.elements()
    }

    pub fn push(&mut self, value: T) -> Result<(), ExecutionRequestArrayDenial> {
        self.request
            .consult()
            .map_err(ExecutionRequestArrayDenial::Request)?;
        self.building
            .push(value)
            .map_err(ExecutionRequestArrayDenial::Allocation)?;
        self.request
            .consult()
            .map_err(ExecutionRequestArrayDenial::Request)
    }

    pub fn seal(self) -> Result<ExecutionArray<T>, ExecutionRequestArrayDenial> {
        let Self { request, building } = self;
        let (array, _) = request
            .run_native(|_| building.seal().map(ExecutionArray::from_owned))
            .map_err(ExecutionRequestArrayDenial::Request)?;
        array.map_err(ExecutionRequestArrayDenial::Allocation)
    }
}

impl<T> std::fmt::Debug for ExecutionRequestArrayBuilder<'_, '_, T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ExecutionRequestArrayBuilder")
            .field("written_elements", &self.len())
            .field("element_count", &self.element_count())
            .finish_non_exhaustive()
    }
}

impl ExecutionRequestArrayBuilder<'_, '_, u8> {
    /// Copy bytes into the already admitted fixed layout, consulting the same
    /// original request between bounded copies. This never grows the layout.
    pub fn extend_from_slice(&mut self, bytes: &[u8]) -> Result<(), ExecutionRequestArrayDenial> {
        use ExecutionRequestArrayDenial as Denial;
        self.request.consult().map_err(Denial::Request)?;
        for chunk in bytes.chunks(64 * 1024) {
            self.request.consult().map_err(Denial::Request)?;
            self.building
                .append_bytes(chunk)
                .map_err(Denial::Allocation)?;
        }
        self.request.consult().map_err(Denial::Request)
    }
}
