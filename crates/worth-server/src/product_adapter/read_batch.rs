use crate::WorthServerOperationSchedulerCounters;
use worth_execution::{MapDenial, MapKernelStop, MapStop};
use worth_foundational::ExecutionReport;
use worth_foundational::PartitionIdentity;

use super::WorthServerCompletedProductOperation;

#[derive(Debug)]
pub enum WorthServerProductReadBatchStop {
    Preflight(MapKernelStop),
    Preparation(super::WorthServerProductOperationSurfaceDenial),
    PacketAdmission(MapDenial),
    PacketStopped {
        boundary: Option<PartitionIdentity>,
        reason: MapStop<()>,
        report: ExecutionReport,
    },
    Finalization {
        ordinal: usize,
        denial: super::WorthServerProductOperationSurfaceDenial,
    },
}

#[derive(Clone, Debug)]
pub struct WorthServerExecutedProductReadBatch {
    operations: Vec<WorthServerCompletedProductOperation>,
    counters: WorthServerOperationSchedulerCounters,
    canonical_digest: String,
    execution_report: Option<ExecutionReport>,
}

impl WorthServerExecutedProductReadBatch {
    pub(crate) fn from_checked_parts(
        operations: Vec<WorthServerCompletedProductOperation>,
        counters: WorthServerOperationSchedulerCounters,
        canonical_digest: String,
    ) -> Self {
        Self {
            operations,
            counters,
            canonical_digest,
            execution_report: None,
        }
    }

    pub(crate) fn new(
        operations: Vec<WorthServerCompletedProductOperation>,
        counters: WorthServerOperationSchedulerCounters,
    ) -> Self {
        let canonical_digest = format!(
            "worth-server-product-read-batch-v1|counters={:?}|operations={}",
            counters,
            operations
                .iter()
                .map(|operation| operation.envelope().canonical_digest())
                .collect::<Vec<_>>()
                .join("|")
        );
        Self {
            operations,
            counters,
            canonical_digest,
            execution_report: None,
        }
    }

    pub(crate) fn with_execution_report(mut self, report: ExecutionReport) -> Self {
        self.execution_report = Some(report);
        self
    }

    pub fn execution_report(&self) -> Option<ExecutionReport> {
        self.execution_report
    }

    pub fn operations(&self) -> &[WorthServerCompletedProductOperation] {
        &self.operations
    }

    pub fn counters(&self) -> &WorthServerOperationSchedulerCounters {
        &self.counters
    }

    pub fn canonical_digest(&self) -> &str {
        &self.canonical_digest
    }
}
