//! The actual protected W reader and original Home selection stay together.
mod inspection;
mod operation;
mod read;
mod selection;
use std::sync::{Arc, Weak};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_runtime_world::facade::{WorldCheckpointReadSource, WorldDurableMediumDenial as Denial};
use worth_store::physical_runtime::ServingPhysicalRuntime;

pub(super) struct ApplicationWorldCheckpointSource {
    selected: crate::checkpoint::SelectedHomeSource,
    serving: Arc<ServingPhysicalRuntime>,
    owner: Weak<crate::checkpoint::owner::HomeCheckpointOwner>,
    _funding: Option<ExecutionMemoryReservation>,
}
impl std::fmt::Debug for ApplicationWorldCheckpointSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ApplicationWorldCheckpointSource")
    }
}
pub(super) fn capture(
    medium: &super::ApplicationWorldMedium,
    request: &ExecutionRequest<'_, '_>,
) -> Result<Box<dyn WorldCheckpointReadSource>, Denial> {
    // No payload I/O occurs while World's capture reservation is held. Admit
    // the exact source carrier before acquiring its one actual registration.
    let funding = request
        .reserve_retained_memory(std::mem::size_of::<ApplicationWorldCheckpointSource>() as u64)
        .map_err(Denial::Request)?;
    let selected = crate::checkpoint::SelectedHomeSource::capture(&medium.serving, request)
        .map_err(read::denial)?;
    // From here the actual reader is retained before another fallible action.
    Ok(Box::new(ApplicationWorldCheckpointSource {
        selected,
        serving: medium.serving.clone(),
        owner: medium.checkpoint.clone(),
        _funding: funding,
    }))
}
pub(in crate::world_section) fn readmission_source(
    home: &crate::ReadmittedApplicationHome,
    request: &ExecutionRequest<'_, '_>,
) -> Result<Box<dyn WorldCheckpointReadSource>, Denial> {
    let funding = request
        .reserve_retained_memory(std::mem::size_of::<ApplicationWorldCheckpointSource>() as u64)
        .map_err(Denial::Request)?;
    let selected = home
        .context
        .selected_home_source
        .as_ref()
        .ok_or(Denial::Unavailable)?
        .clone();
    Ok(Box::new(ApplicationWorldCheckpointSource {
        selected,
        serving: home.context.serving.clone(),
        owner: Arc::downgrade(&home.context.checkpoint),
        _funding: funding,
    }))
}
impl WorldCheckpointReadSource for ApplicationWorldCheckpointSource {
    fn read(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<worth_runtime_world::facade::WorldCheckpointReadView, Denial> {
        read::view(self, request)
    }
    fn read_journal_record(
        &self,
        request: &ExecutionRequest<'_, '_>,
        journal: worth_foundational::facade::WorldDurableJournal,
        position: u64,
    ) -> Result<worth_runtime_world::facade::WorldCheckpointJournalRecord, Denial> {
        read::journal(self, request, journal, position)
    }
    fn observe_selection(
        &self,
        subject: &worth_runtime_world::facade::WorldCheckpointSubject,
    ) -> Result<Option<worth_runtime_world::facade::WorldCheckpointSelection>, Denial> {
        selection::observe(self, subject)
    }
}
