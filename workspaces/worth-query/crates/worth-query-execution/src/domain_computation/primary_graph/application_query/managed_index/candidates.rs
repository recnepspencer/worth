//! Output-sized source-slot membership, without raw entity or mutation authority.
use crate::domain_computation::primary_graph::access_strategy::ordered_storage;
use worth_execution::{
    ExecutionArray, ExecutionOrderedMap, ExecutionRequest, ExecutionRequestArrayBuilder,
};
use {
    super::source_rows::SourceImageView, super::WorthQueryManagedIndexSourceRows,
    crate::domain_computation::primary_graph::vocabulary::application_query::source_rows::WorthQueryManagedIndexSourceSlot as Slot,
    crate::domain_computation::vocabulary::primary_graph::control::WorthQueryManagedIndexControl,
    crate::domain_computation::vocabulary::primary_graph::stop::WorthQueryManagedIndexStop as Stop,
};

/// Candidate source slots. Final ordinary projection and disclosure remain authoritative.
pub struct WorthQueryManagedIndexCandidateWriter<'a> {
    image: SourceImageView<'a>,
    selected: ExecutionOrderedMap<Slot, ()>,
    execution: ExecutionRequest<'a, 'a>,
    control: &'a WorthQueryManagedIndexControl<'a>,
}
impl<'a> WorthQueryManagedIndexCandidateWriter<'a> {
    pub(super) fn allocate<S, I>(
        execution: &ExecutionRequest<'a, 'a>,
        source: &WorthQueryManagedIndexSourceRows<'a, S, I>,
        control: &'a WorthQueryManagedIndexControl<'a>,
    ) -> Result<Self, Stop>
    where
        S: worth_query_declaration::facade::application_schema::ApplicationSchema,
        I: worth_query_declaration::facade::application_query::ApplicationManagedIndexBinding<S>,
    {
        control.poll()?;
        Ok(Self {
            image: source.image(),
            selected: ExecutionOrderedMap::default(),
            execution: *execution,
            control,
        })
    }
    pub fn emit(&mut self, slot: Slot) -> Result<(), Stop> {
        self.control.checkpoint(1)?;
        self.image.validate_slot(slot, self.control)?;
        ordered_storage::lookup(self.control, self.selected.len(), size_of::<Slot>())?;
        if !self.selected.contains_key(&slot) {
            ordered_storage::insert(
                &mut self.selected,
                &slot,
                &(),
                &self.execution,
                self.control,
            )?;
        }
        self.control.poll()
    }
    pub(super) fn finish(
        self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<ExecutionArray<Slot>, Stop> {
        let mut selected = ExecutionRequestArrayBuilder::allocate(*execution, self.selected.len())
            .map_err(|cause| self.control.refuse(Stop::Storage(cause)))?;
        for slot in self.selected.keys().copied() {
            self.control.checkpoint(1)?;
            selected
                .push(slot)
                .map_err(|cause| self.control.refuse(Stop::Storage(cause)))?;
        }
        let selected = selected
            .seal()
            .map_err(|cause| self.control.refuse(Stop::Storage(cause)))?;
        self.control.poll()?;
        Ok(selected)
    }
    pub fn len(&self) -> usize {
        self.selected.len()
    }
    pub fn is_empty(&self) -> bool {
        self.selected.is_empty()
    }
}
