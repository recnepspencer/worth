//! Historical marks share one immutable ordered map between image replacements.
use super::HistoricalMarkState;
use im::OrdMap;
use std::alloc::Layout;
use std::fmt;
use std::ops::Deref;
use std::sync::{Arc, OnceLock};
use worth_execution::ExecutionMemoryReservation;
use worth_relational::facade::mvcc::{CompanionPreflightStop, PublicationCompanionPreflight};
use worth_relational::facade::publication::PatchStreamPosition;

type Entries = OrdMap<Option<PatchStreamPosition>, HistoricalMarkState>;

struct HistoryImage {
    entries: Entries,
    _memory: Option<ExecutionMemoryReservation>,
}

trait HistoryBacking: Send + Sync {
    fn entries(&self) -> &Entries;
}

impl HistoryBacking for HistoryImage {
    fn entries(&self) -> &Entries {
        &self.entries
    }
}

#[derive(Clone, Default)]
pub(in super::super) struct RetainedMarkHistory(Option<Arc<dyn HistoryBacking>>);

impl RetainedMarkHistory {
    pub(in super::super) fn appending(
        &self,
        position: Option<PatchStreamPosition>,
        row: HistoricalMarkState,
        context: &PublicationCompanionPreflight<'_>,
    ) -> Result<Self, CompanionPreflightStop> {
        let layout = Layout::new::<[usize; 2]>()
            .extend(Layout::new::<HistoryImage>())
            .map_err(|_| CompanionPreflightStop::PreparationMemoryCounterOverflow)?
            .0
            .pad_to_align();
        let bytes = u64::try_from(layout.size())
            .map_err(|_| CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        let memory = context
            .execution_request()
            .reserve_retained_memory(bytes)
            .map_err(|cause| CompanionPreflightStop::Execution(cause.into()))?;
        let mut entries = self.deref().clone();
        entries.insert(position, row);
        Ok(Self(Some(Arc::new(HistoryImage {
            entries,
            _memory: memory,
        }))))
    }
}

impl Deref for RetainedMarkHistory {
    type Target = Entries;

    fn deref(&self) -> &Entries {
        match &self.0 {
            Some(image) => image.entries(),
            None => {
                static EMPTY: OnceLock<Entries> = OnceLock::new();
                EMPTY.get_or_init(Entries::new)
            }
        }
    }
}

impl fmt::Debug for RetainedMarkHistory {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.deref().fmt(formatter)
    }
}
