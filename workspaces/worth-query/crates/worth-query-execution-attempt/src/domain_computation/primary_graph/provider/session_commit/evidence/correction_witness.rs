//! Minimal copied performed evidence; ordinary receipt observation is unchanged.
use super::WorthQueryCompletedCommitEvidenceStore;
use crate::domain_computation::primary_graph::provider::correction_witness::{
    admit, retain, run, CorrectionPerformedWitness, CorrectionWitnessDenial as Denial,
};
use std::sync::Arc;
use worth_execution::{ExecutionRequest, MapKernelStop};
use worth_relational::facade::history::RelationalCommitReceipt;

impl WorthQueryCompletedCommitEvidenceStore {
    pub fn capture_correction_witness(
        &self,
        execution: ExecutionRequest<'_, '_>,
        receipt: &RelationalCommitReceipt,
    ) -> Result<Option<Arc<CorrectionPerformedWitness>>, Denial> {
        run(execution, |context| {
            let comparisons = (self.by_commit.len() as u64)
                .checked_add(1)
                .ok_or(Denial::Interrupted(MapKernelStop::WorkCounterOverflow))?;
            let work = (receipt.parents.len() as u64)
                .checked_mul(std::mem::size_of_val(&receipt.commit_id) as u64)
                .and_then(|work| work.checked_add(receipt.branch_id.0.len() as u64))
                .and_then(|work| work.checked_add(std::mem::size_of_val(receipt) as u64))
                .and_then(|work| {
                    work.checked_add(
                        comparisons.checked_mul(std::mem::size_of_val(&receipt.commit_id) as u64)?,
                    )
                })
                .ok_or(Denial::Interrupted(MapKernelStop::WorkCounterOverflow))?;
            admit(context, work)?;
            let Some(completed) = self
                .by_commit
                .get(&receipt.commit_id)
                .filter(|completed| completed.commit_reference() == receipt)
            else {
                return Ok(None);
            };
            retain(
                execution,
                context,
                completed.committed_product_publication().composite_commit(),
                completed.aftermath_causality(),
            )
            .map(Some)
        })
    }
}

pub mod court_doors_for_test;
