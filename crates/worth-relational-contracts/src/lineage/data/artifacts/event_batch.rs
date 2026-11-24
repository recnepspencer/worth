use serde::{Deserialize, Serialize};

use crate::lineage::data::{LineageEventKind, LineageEventRecord, LineageFinalizationCounters};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FinalizedLineageEventBatch {
    event_ids: Vec<u64>,
    events: Vec<LineageEventRecord>,
    counters: LineageFinalizationCounters,
}

impl FinalizedLineageEventBatch {
    pub fn new(events: Vec<LineageEventRecord>) -> Self {
        let counters = summarize_event_counters(&events);
        let event_ids = events.iter().map(|event| event.event_id).collect();
        Self {
            event_ids,
            events,
            counters,
        }
    }

    pub fn event_ids(&self) -> &[u64] {
        &self.event_ids
    }

    pub fn events(&self) -> &[LineageEventRecord] {
        &self.events
    }

    pub fn counters(&self) -> &LineageFinalizationCounters {
        &self.counters
    }
}

fn summarize_event_counters(events: &[LineageEventRecord]) -> LineageFinalizationCounters {
    let mut counters = LineageFinalizationCounters {
        event_batch_width: events.len(),
        ..LineageFinalizationCounters::default()
    };
    for event in events {
        match event.kind {
            LineageEventKind::Create => counters.created_event_count += 1,
            LineageEventKind::Replace => counters.replaced_event_count += 1,
            LineageEventKind::Retire => counters.retired_event_count += 1,
            LineageEventKind::Split | LineageEventKind::Merge | LineageEventKind::Restore => {}
        }
    }
    counters
}
