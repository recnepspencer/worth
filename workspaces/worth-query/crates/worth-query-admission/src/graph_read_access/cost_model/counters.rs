#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryGraphReadCostEstimateCounters {
    requirement_row_count: usize,
    estimated_relation_row_count: usize,
    estimated_workset_row_count: usize,
    estimated_buffer_row_count: usize,
    edge_scan_count: usize,
    access_buffer_allocation_count: usize,
}

impl WorthQueryGraphReadCostEstimateCounters {
    pub fn requirement_row_count(&self) -> usize {
        self.requirement_row_count
    }

    pub fn estimated_relation_row_count(&self) -> usize {
        self.estimated_relation_row_count
    }

    pub fn estimated_workset_row_count(&self) -> usize {
        self.estimated_workset_row_count
    }

    pub fn estimated_buffer_row_count(&self) -> usize {
        self.estimated_buffer_row_count
    }

    pub fn edge_scan_count(&self) -> usize {
        self.edge_scan_count
    }

    pub fn access_buffer_allocation_count(&self) -> usize {
        self.access_buffer_allocation_count
    }

    pub(crate) fn new(
        requirement_row_count: usize,
        estimated_relation_row_count: usize,
        estimated_workset_row_count: usize,
        estimated_buffer_row_count: usize,
    ) -> Self {
        Self {
            requirement_row_count,
            estimated_relation_row_count,
            estimated_workset_row_count,
            estimated_buffer_row_count,
            edge_scan_count: 0,
            access_buffer_allocation_count: 0,
        }
    }

    pub(crate) fn digest_part(&self) -> String {
        let mut text = String::new();
        self.write_digest_part(&mut text)
            .expect("String formatting cannot fail");
        text
    }

    pub(crate) fn write_digest_part(&self, output: &mut dyn Write) -> fmt::Result {
        write!(
            output,
            "counters:rows:{}:relations:{}:worksets:{}:buffers:{}:edge_scans:{}:allocations:{}",
            self.requirement_row_count,
            self.estimated_relation_row_count,
            self.estimated_workset_row_count,
            self.estimated_buffer_row_count,
            self.edge_scan_count,
            self.access_buffer_allocation_count
        )
    }
}
use std::fmt::{self, Write};
