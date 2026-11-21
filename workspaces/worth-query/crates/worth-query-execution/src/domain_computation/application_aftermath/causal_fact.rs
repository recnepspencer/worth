//! Pending aftermath causality paired with lineage-owned committed aftermath types.

use worth_relational::facade::history::RelationalCommitReceipt;

pub use worth_query_execution_lineage::facade::domain_computation::application_aftermath::causal_fact::{
    WorthQueryAftermathCausalRole, WorthQueryCommittedAftermathCausality,
    WorthQueryCorrectionLineageBinding, WorthQueryRecordedCorrectionSource,
};

/// Query-owned semantic fact awaiting an ordinary Relational commit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPendingAftermathCausality {
    role: WorthQueryAftermathCausalRole,
    parent: RelationalCommitReceipt,
    target_branch: worth_relational::facade::history::BranchId,
    source: WorthQueryRecordedCorrectionSource,
    lineage: WorthQueryCorrectionLineageBinding,
    correction: std::sync::Arc<
        crate::domain_computation::primary_graph::WorthQueryPreparedCorrectionSource,
    >,
}

impl WorthQueryPendingAftermathCausality {
    pub(crate) fn recorded_correction(
        correction: std::sync::Arc<
            crate::domain_computation::primary_graph::WorthQueryPreparedCorrectionSource,
        >,
    ) -> Self {
        Self {
            role: correction.lineage_binding().role(),
            lineage: correction.lineage_binding().clone(),
            parent: correction.target_parent().clone(),
            target_branch: correction.target_branch().clone(),
            source: WorthQueryRecordedCorrectionSource::from_untrusted_parts(
                correction.source_world().clone(),
                correction.source_commit().clone(),
            ),
            correction,
        }
    }

    pub(crate) const fn role(&self) -> WorthQueryAftermathCausalRole {
        self.role
    }

    pub(crate) const fn parent(&self) -> &RelationalCommitReceipt {
        &self.parent
    }

    pub(crate) fn target_branch(&self) -> &worth_relational::facade::history::BranchId {
        &self.target_branch
    }

    pub(crate) fn recorded_source(&self) -> &WorthQueryRecordedCorrectionSource {
        &self.source
    }

    pub(crate) fn correction_lineage(&self) -> &WorthQueryCorrectionLineageBinding {
        &self.lineage
    }

    pub(crate) fn prepared_correction(
        &self,
    ) -> &crate::domain_computation::primary_graph::WorthQueryPreparedCorrectionSource {
        &self.correction
    }

    pub(crate) fn key(&self) -> String {
        self.correction.intent().causal_key()
    }
}

pub(crate) trait CommittedAftermathCausalityExt: Sized {
    fn seal(
        pending: WorthQueryPendingAftermathCausality,
        child: RelationalCommitReceipt,
        fact_record: worth_relational::facade::transactions::RecordRef,
    ) -> Option<Self>;

    fn answers(&self, pending: &WorthQueryPendingAftermathCausality) -> bool;
}

impl CommittedAftermathCausalityExt for WorthQueryCommittedAftermathCausality {
    fn seal(
        pending: WorthQueryPendingAftermathCausality,
        child: RelationalCommitReceipt,
        fact_record: worth_relational::facade::transactions::RecordRef,
    ) -> Option<Self> {
        Self::seal_parts(
            pending.role,
            pending.parent,
            pending.target_branch,
            pending.source,
            pending.lineage,
            child,
            fact_record,
        )
    }

    fn answers(&self, pending: &WorthQueryPendingAftermathCausality) -> bool {
        self.matches_parts(
            pending.role,
            &pending.parent,
            &pending.target_branch,
            &pending.source,
            &pending.lineage,
        )
    }
}

mod court_doors_for_test;
