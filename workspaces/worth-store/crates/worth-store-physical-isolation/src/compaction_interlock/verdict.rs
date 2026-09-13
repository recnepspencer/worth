use super::{
    CompactionCutoverStabilityProof, CompactionReadInterlockDenial, CompactionRecoveryEvidence,
    CompactionRewritePublication,
};
use crate::StablePhysicalReadReceipt;

#[derive(Debug, Clone)]
pub struct ReadDuringCompactionVerdict {
    proof: CompactionCutoverStabilityProof,
    pre_cutover_read: StablePhysicalReadReceipt,
    post_cutover_read: StablePhysicalReadReceipt,
}

impl ReadDuringCompactionVerdict {
    pub fn from_stability_proof(
        proof: CompactionCutoverStabilityProof,
        pre_cutover_read: StablePhysicalReadReceipt,
        post_cutover_read: StablePhysicalReadReceipt,
    ) -> Result<Self, CompactionReadInterlockDenial> {
        if proof.pre_cutover_root().epoch() == proof.post_cutover_root().epoch() {
            return Err(CompactionReadInterlockDenial::MixedRootDuringCompaction);
        }
        let protected = proof.publication().delta().plan().protected();
        let pre_release = pre_cutover_read.read_plan_release();
        if pre_release.root() != proof.pre_cutover_root()
            || pre_release.footprint_basis() != protected.footprint_basis()
        {
            return Err(CompactionReadInterlockDenial::PreCutoverReadReceiptMismatch);
        }
        let post_release = post_cutover_read.read_plan_release();
        if post_release.root() != proof.post_cutover_root()
            || post_release.root_epoch() != proof.publication().delta().plan().target_epoch()
        {
            return Err(CompactionReadInterlockDenial::PostCutoverReadReceiptMismatch);
        }
        Ok(Self {
            proof,
            pre_cutover_read,
            post_cutover_read,
        })
    }

    pub const fn pre_cutover_reader_retained_old_structure(&self) -> bool {
        self.proof.publication().delta().plan().reclaim_deferred()
    }

    pub const fn post_cutover_reader_observed_new_epoch(&self) -> bool {
        self.proof.post_cutover_root().epoch().get()
            == self.proof.publication().delta().plan().target_epoch().get()
    }

    pub const fn proof(&self) -> &CompactionCutoverStabilityProof {
        &self.proof
    }

    pub const fn pre_cutover_read(&self) -> StablePhysicalReadReceipt {
        self.pre_cutover_read
    }

    pub const fn post_cutover_read(&self) -> StablePhysicalReadReceipt {
        self.post_cutover_read
    }
}

pub fn execute_read_during_compaction_cutover(
    publication: CompactionRewritePublication,
    recovery: CompactionRecoveryEvidence,
    pre_cutover_read: StablePhysicalReadReceipt,
    post_cutover_read: StablePhysicalReadReceipt,
) -> Result<ReadDuringCompactionVerdict, CompactionReadInterlockDenial> {
    let proof = CompactionCutoverStabilityProof::admit(publication, recovery)?;
    ReadDuringCompactionVerdict::from_stability_proof(proof, pre_cutover_read, post_cutover_read)
}
