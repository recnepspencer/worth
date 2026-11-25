use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct LineageFinalizationCounters {
    pub event_batch_width: usize,
    pub created_event_count: usize,
    pub replaced_event_count: usize,
    pub retired_event_count: usize,
}
