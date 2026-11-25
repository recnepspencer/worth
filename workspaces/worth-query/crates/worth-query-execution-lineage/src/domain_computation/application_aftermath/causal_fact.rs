//! Query aftermath meaning carried into Relational-owned commit history.

use worth_relational::facade::history::{CommitId, RelationalCommitReceipt};
use worth_relational::facade::transactions::RecordRef;
pub mod correction_lineage;
pub use correction_lineage::WorthQueryCorrectionLineageBinding;

/// Query meaning of one co-committed aftermath relation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryAftermathCausalRole {
    RecordedReverse,
    RecordedReapply,
}

impl WorthQueryAftermathCausalRole {
    pub const fn code(self) -> u64 {
        match self {
            Self::RecordedReverse => 3,
            Self::RecordedReapply => 4,
        }
    }
}

/// Exact historical source, distinct from the new commit's actual ancestry parent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryRecordedCorrectionSource {
    world: worth_runtime_world::facade::CompositeCommitIdentity,
    relational: RelationalCommitReceipt,
    member: super::correction_coordinates::WorthQueryCorrectionSourceMember,
}
impl WorthQueryRecordedCorrectionSource {
    /// Descriptive coordinates to be resolved against retained native evidence.
    pub fn from_untrusted_parts(
        world: worth_runtime_world::facade::CompositeCommitIdentity,
        relational: RelationalCommitReceipt,
        member: super::correction_coordinates::WorthQueryCorrectionSourceMember,
    ) -> Self {
        Self {
            world,
            relational,
            member,
        }
    }
    /// The writer of the commit's coupled group this source names (#368).
    pub const fn member(&self) -> super::correction_coordinates::WorthQueryCorrectionSourceMember {
        self.member
    }

    pub fn world(&self) -> &worth_runtime_world::facade::CompositeCommitIdentity {
        &self.world
    }
    pub fn relational(&self) -> &RelationalCommitReceipt {
        &self.relational
    }
}

/// Query aftermath meaning sealed from one Relational commit and its exact
/// co-committed fact record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryCommittedAftermathCausality {
    role: WorthQueryAftermathCausalRole,
    parent: RelationalCommitReceipt,
    child: RelationalCommitReceipt,
    fact_record: RecordRef,
    source: Box<WorthQueryRecordedCorrectionSource>,
    lineage: Box<WorthQueryCorrectionLineageBinding>,
}

impl WorthQueryCommittedAftermathCausality {
    pub fn owned_receipt_backing_bytes(&self) -> Option<u64> {
        receipt_backing_bytes(&self.parent)?
            .checked_add(receipt_backing_bytes(&self.child)?)?
            .checked_add(receipt_backing_bytes(self.source.relational())?)?
            .checked_add(self.lineage.owned_receipt_backing_bytes()?)
    }

    pub fn seal_parts(
        role: WorthQueryAftermathCausalRole,
        parent: RelationalCommitReceipt,
        target_branch: worth_relational::facade::history::BranchId,
        source: WorthQueryRecordedCorrectionSource,
        lineage: WorthQueryCorrectionLineageBinding,
        child: RelationalCommitReceipt,
        fact_record: RecordRef,
    ) -> Option<Self> {
        (child.branch_id == target_branch && child.parents.as_slice() == [parent.commit_id])
            .then_some(Self {
                role,
                parent,
                child,
                fact_record,
                source: Box::new(source),
                lineage: Box::new(lineage),
            })
    }

    pub const fn role(&self) -> WorthQueryAftermathCausalRole {
        self.role
    }

    pub const fn parent(&self) -> &RelationalCommitReceipt {
        &self.parent
    }

    pub const fn child(&self) -> &RelationalCommitReceipt {
        &self.child
    }
    pub fn recorded_source(&self) -> &WorthQueryRecordedCorrectionSource {
        &self.source
    }
    pub fn correction_lineage(&self) -> &WorthQueryCorrectionLineageBinding {
        &self.lineage
    }

    pub const fn fact_record(&self) -> &RecordRef {
        &self.fact_record
    }

    pub const fn parent_commit_id(&self) -> CommitId {
        self.parent.commit_id
    }

    /// Whether this sealed fact matches the expected co-committed parameters.
    pub fn matches_parts(
        &self,
        role: WorthQueryAftermathCausalRole,
        parent: &RelationalCommitReceipt,
        target_branch: &worth_relational::facade::history::BranchId,
        source: &WorthQueryRecordedCorrectionSource,
        lineage: &WorthQueryCorrectionLineageBinding,
    ) -> bool {
        self.role == role
            && &self.parent == parent
            && self.source.as_ref() == source
            && self.lineage.as_ref() == lineage
            && &self.child.branch_id == target_branch
    }
}

fn receipt_backing_bytes(receipt: &RelationalCommitReceipt) -> Option<u64> {
    (receipt.parents.capacity() as u64)
        .checked_mul(std::mem::size_of::<CommitId>() as u64)?
        .checked_add(receipt.branch_id.0.capacity() as u64)
}

#[cfg(test)]
#[path = "causal_fact_inline_type_budget.rs"]
pub mod inline_type_budget;

mod court_doors_for_test;
