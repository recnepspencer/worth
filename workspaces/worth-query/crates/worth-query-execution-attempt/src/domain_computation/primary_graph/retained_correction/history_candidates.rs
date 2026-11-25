//! Ready retained writers, enumerated as DATA for a bounded correction history.
use super::catalog::MaterialBody;
use super::*;
use worth_foundational::facade::DurableCompositePublicationAttemptIdentity;
use worth_relational::facade::history::RelationalCommitReceipt;

/// One retained writer whose correction material is ready.
///
/// Descriptive only: a candidate grants no disclosure or correction authority.
/// Listing resolves each candidate through ordinary source selection.
#[derive(Clone, Debug)]
pub struct RetainedCorrectionHistoryCandidate {
    pub original: DurableCompositePublicationAttemptIdentity,
    pub member: crate::domain_computation::application_aftermath::WorthQueryCorrectionSourceMember,
    pub commit: RelationalCommitReceipt,
}

impl CorrectionMaterialOwner {
    /// Every writer whose material is ready, in catalog order.
    ///
    /// O(retained material); expired, abandoned and captured-only writers are
    /// omitted because they can no longer be selected as correction sources.
    pub fn ready_history_candidates(
        &self,
    ) -> Result<Vec<RetainedCorrectionHistoryCandidate>, Denial> {
        let state = self.state();
        if state.pending.is_some() {
            return Err(Denial::PublicationPending);
        }
        Ok(state
            .catalog
            .material
            .iter()
            .filter_map(|((original, member), entry)| match &entry.body {
                MaterialBody::Ready { ready, .. } => Some(RetainedCorrectionHistoryCandidate {
                    original: *original,
                    member: *member,
                    commit: ready.commit.clone(),
                }),
                MaterialBody::Captured(_)
                | MaterialBody::Expired { .. }
                | MaterialBody::Abandoned => None,
            })
            .collect())
    }
}
