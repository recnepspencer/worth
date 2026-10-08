//! Ordinary batch custody shares the actual issued Query snapshot. It never
//! registers a replacement snapshot or reinspects the selected program.
use super::*;
use crate::domain_computation::primary_graph::application_query::batch::{
    WorthQueryApplicationQueryBatchAdmission, WorthQueryApplicationQueryBatchResourceDenial,
};

impl WorthQueryApplicationBasisLease {
    pub(in crate::domain_computation::primary_graph) fn retain_in_batch(
        &mut self,
        batch: &WorthQueryApplicationQueryBatchAdmission,
    ) -> Result<Self, WorthQueryApplicationQueryBatchResourceDenial> {
        if !matches!(self.custody, BasisCustody::Shared(_)) {
            let bytes = crate::domain_computation::primary_graph::output_lineage::invalidation::arc_bytes::<BasisLeaseCore>()
                .and_then(|n| usize::try_from(n).ok())
                .ok_or(WorthQueryApplicationQueryBatchResourceDenial::CounterOverflow)?;
            let claim = batch.claim_source_memory(bytes)?;
            let BasisCustody::Exclusive(mut core) =
                std::mem::replace(&mut self.custody, BasisCustody::Transitioning)
            else {
                unreachable!("only exclusive custody enters the sharing phase")
            };
            core._batch_shared = Some(claim);
            self.custody = BasisCustody::Shared(Arc::new(core));
        }
        let descriptor = &self.identity.descriptor;
        let parents = descriptor
            .reference()
            .target()
            .as_basis()
            .map_or(0, |target| target.parent_commit_ids().len());
        // The descriptor owns these initialized String/Vec values. Product
        // selection identity consists of fixed fields and Arc-backed names;
        // cloning it makes no independent name payload allocation.
        let bytes = parents
            .checked_mul(std::mem::size_of::<u64>())
            .and_then(|n| n.checked_add(self.identity.branch_id.0.len()))
            .and_then(|n| n.checked_add(descriptor.branch_id().0.len()))
            .and_then(|n| n.checked_add(descriptor.reference().branch_id().as_str().len()))
            .ok_or(WorthQueryApplicationQueryBatchResourceDenial::CounterOverflow)?;
        let claim = batch.claim_source_memory(bytes)?;
        let BasisCustody::Shared(core) = &self.custody else {
            unreachable!("a retained batch share follows promotion")
        };
        Ok(Self {
            identity: self.identity.clone(),
            custody: BasisCustody::Shared(Arc::clone(core)),
            _batch_identity: Some(claim),
        })
    }
}
