//! Minimal copied performed evidence; ordinary receipt observation is unchanged.
use super::WorthQueryCompletedCommitEvidenceStore;
use crate::domain_computation::primary_graph::provider::correction_witness::{
    admit, run, CorrectionPerformedWitness, CorrectionWitnessDenial as Denial,
};
use std::{alloc::Layout, sync::Arc};
use worth_execution::{ExecutionRequest, MapKernelStop, WorkCeilingDenial};
use worth_relational::facade::history::RelationalCommitReceipt;

impl WorthQueryCompletedCommitEvidenceStore {
    pub(in crate::domain_computation::primary_graph) fn capture_correction_witness(
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
            let causality = completed.aftermath_causality();
            let backing = match causality {
                Some(causality) => causality
                    .owned_receipt_backing_bytes()
                    .ok_or(Denial::SizeOverflow)?,
                None => 0,
            };
            let root = Layout::new::<(usize, usize)>()
                .extend(Layout::new::<CorrectionPerformedWitness>())
                .map_err(|_| Denial::SizeOverflow)?
                .0
                .pad_to_align()
                .size() as u64;
            let bytes = root.checked_add(backing).ok_or(Denial::SizeOverflow)?;
            admit(context, bytes)?;
            let mut memory = execution
                .reserve_retained_memory(bytes)
                .map_err(Denial::CopyAdmission)?;
            let aftermath = causality.cloned();
            let actual = root
                .checked_add(match &aftermath {
                    Some(aftermath) => aftermath
                        .owned_receipt_backing_bytes()
                        .ok_or(Denial::SizeOverflow)?,
                    None => 0,
                })
                .ok_or(Denial::SizeOverflow)?;
            // These fixed String/Vec<CommitId> clones copy their initialized
            // payload, covered by the source's admitted capacities above.
            if actual > bytes {
                return Err(Denial::SizeOverflow);
            }
            if let Some(memory) = &mut memory {
                memory.resize(actual).map_err(|cause| {
                    Denial::Request(WorkCeilingDenial::Admission(
                        worth_execution::LeaseDenial::MemoryExhausted(cause),
                    ))
                })?;
            }
            Ok(Some(Arc::new(CorrectionPerformedWitness {
                world: completed
                    .committed_product_publication()
                    .composite_commit()
                    .clone(),
                aftermath,
                _memory: memory,
            })))
        })
    }
}
