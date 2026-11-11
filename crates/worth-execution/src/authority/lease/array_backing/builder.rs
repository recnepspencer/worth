use super::super::fixed_backing::{
    BuildingFixedBacking, ExecutionAllocationDenial, ExecutionAllocationPolicy,
};
use super::ExecutionArray;

/// Move-only author for an exact logical number of independently moved values.
/// Charges `Layout::array::<T>(element_count)`; nested heaps remain their owners'.
/// No growth, mutable slice, arbitrary Vec adoption, or copied Clone payload.
/// Cancellation is observed per write. Deadline consultation uses bounded
/// append chunks: at most 32 scalar writes, with a 64 KiB granule or one
/// indivisible value. Allocation, explicit check_live, bounds errors and seal
/// poll immediately. A stopped prefix cannot seal.
/// Push consumes its input on refusal as well as success.
///
/// ```compile_fail
/// use worth_execution::ExecutionArrayBuilder;
/// fn duplicate(builder: &ExecutionArrayBuilder<u8>) -> ExecutionArrayBuilder<u8> {
///     builder.clone()
/// }
/// ```
pub struct ExecutionArrayBuilder<T> {
    building: BuildingFixedBacking<T>,
}

impl<T> ExecutionArrayBuilder<T> {
    pub fn allocate(
        element_count: usize,
        policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Self, ExecutionAllocationDenial> {
        Ok(Self {
            building: BuildingFixedBacking::allocate(element_count, policy)?,
        })
    }
    pub fn check_live(&self) -> Result<(), ExecutionAllocationDenial> {
        self.building.check_live()
    }
    pub fn push(&mut self, value: T) -> Result<(), ExecutionAllocationDenial> {
        self.building.push(value)
    }
    pub fn elements(&self) -> &[T] {
        self.building.elements()
    }
    pub fn len(&self) -> usize {
        self.building.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Exact logical admission; independent of the allocator's ZST capacity.
    pub fn element_count(&self) -> usize {
        self.building.element_count()
    }
    pub fn seal(self) -> Result<ExecutionArray<T>, ExecutionAllocationDenial> {
        Ok(ExecutionArray::from_owned(self.building.seal()?))
    }
}
impl<T> std::fmt::Debug for ExecutionArrayBuilder<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExecutionArrayBuilder")
            .field("written_elements", &self.len())
            .field("element_count", &self.element_count())
            .finish_non_exhaustive()
    }
}
