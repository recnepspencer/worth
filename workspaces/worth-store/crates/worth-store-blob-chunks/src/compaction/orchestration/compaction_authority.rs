use crate::compaction::receipt_construction::published_observation::BlobCompactionPublishedObservation;
use crate::compaction::transitions::execute_rewrite::BlobCompactionRewriteExecution;
use crate::compaction::transitions::publish_rewrite;
use crate::compaction::types::{BlobCompactionIntent, BlobCompactionRewritePlan};
use crate::{BlobCompactionDenial, BlobCompactionEquivalence};
use worth_store_authority::StoreCurrentAuthorityWitness;
use worth_store_physical_isolation::CompactionReadPlanCompletion;

/// Store-owned blob compaction authority.
///
/// External callers cannot mint this authority marker directly:
///
/// ```compile_fail
/// let _authority = worth_store_blob_chunks::BlobCompactionAuthority::store_owned();
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobCompactionAuthority {
    current_authority: StoreCurrentAuthorityWitness,
}

impl BlobCompactionAuthority {
    pub const fn from_current_store_authority(
        current_authority: StoreCurrentAuthorityWitness,
    ) -> Self {
        Self { current_authority }
    }

    pub fn plan_compaction(
        &self,
        intent: BlobCompactionIntent,
    ) -> Result<BlobCompactionRewritePlan, BlobCompactionDenial> {
        BlobCompactionRewritePlan::admit(intent)
    }

    pub fn execute_rewrite(
        &self,
        plan: BlobCompactionRewritePlan,
        equivalence: BlobCompactionEquivalence,
        read_plan_completion: CompactionReadPlanCompletion,
    ) -> Result<BlobCompactionRewriteExecution, BlobCompactionDenial> {
        BlobCompactionRewriteExecution::from_plan(plan, equivalence, read_plan_completion)
    }

    pub fn publish_rewrite(
        &self,
        execution: BlobCompactionRewriteExecution,
    ) -> Result<BlobCompactionPublishedObservation, BlobCompactionDenial> {
        publish_rewrite::publish_rewrite(execution)
    }
}
