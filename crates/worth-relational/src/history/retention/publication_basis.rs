//! Live publication holds are owned by heads/candidates, never canonical origins.
use crate::branch::basis::RelationalBranchBasisDescriptor;
use crate::branch::root::RelationalBranchRoot;
use crate::history::data::CommitId;
use crate::identity::data::EntityId;
use crate::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaning;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use std::sync::{Arc, Weak};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct PublicationBasisRoute {
    family: Arc<str>,
    source: EntityId,
    partition: worth_foundational::CanonicalDigestId,
}
impl PublicationBasisRoute {
    pub(super) fn compare_meaning(
        &self,
        meaning: &RequiredOutputPublicationMeaning,
    ) -> std::cmp::Ordering {
        self.family
            .as_ref()
            .cmp(meaning.family.as_str())
            .then_with(|| self.source.cmp(&meaning.source_scope))
            .then_with(|| self.partition.cmp(&meaning.source_partition))
    }
    pub(crate) fn of(meaning: &RequiredOutputPublicationMeaning) -> Self {
        Self {
            family: Arc::from(meaning.family.as_str()),
            source: meaning.source_scope,
            partition: meaning.source_partition,
        }
    }
}

#[derive(Debug)]
pub(crate) struct RetainedPublicationBasis {
    pub(crate) publication: CommitId,
    pub(crate) descriptor: RelationalBranchBasisDescriptor,
    pub(crate) root: Arc<RelationalBranchRoot>,
    pub(crate) binding: super::RelationalBranchRetentionBinding,
    pub(crate) applied:
        Option<Arc<dyn crate::publication::live_publication_inputs::AppliedPublicationInputAccess>>,
    /// The publishing branch's own cell, so issuing this input does not
    /// depend on that branch still being registered after it is deleted.
    pub(crate) publication_cell: crate::branch::RelationalBranchPublicationCell,
    pub(crate) _hold: super::RelationalObservationRetentionObligation,
    pub(crate) _funding: Option<worth_execution::ExecutionMemoryReservation>,
}

pub(crate) trait PublicationBasisAccess: std::fmt::Debug + Send + Sync {
    fn routes(&self) -> &super::publication_basis_tree::PublicationBasisRoutes;
    fn belongs_to(&self, binding: &super::RelationalBranchRetentionBinding) -> bool;
}
impl PublicationBasisAccess for PublicationBasisImage {
    fn belongs_to(&self, binding: &super::RelationalBranchRetentionBinding) -> bool {
        self.owner
            .as_ref()
            .is_none_or(|owner| owner.same_owner(binding))
    }
    fn routes(&self) -> &super::publication_basis_tree::PublicationBasisRoutes {
        &self.routes
    }
}
#[derive(Debug, Default)]
pub(crate) struct PublicationBasisImage {
    owner: Option<super::RelationalBranchRetentionBinding>,
    pub(crate) routes: super::publication_basis_tree::PublicationBasisRoutes,
    // Every persistent node owns its own funding; this reservation covers only
    // the image header. Sharing nodes never loses their reservations.
    _funding: Option<worth_execution::ExecutionMemoryReservation>,
}
impl PublicationBasisImage {
    pub(crate) fn from_funded_routes(
        owner: super::RelationalBranchRetentionBinding,
        routes: super::publication_basis_tree::PublicationBasisRoutes,
        funding: Option<worth_execution::ExecutionMemoryReservation>,
    ) -> Arc<dyn PublicationBasisAccess> {
        Arc::new(Self {
            owner: Some(owner),
            routes,
            _funding: funding,
        })
    }

    /// A cold restored root has the canonical coordinate, but no operational hold.
    pub(crate) fn at_for_owner(
        root: &RelationalBranchRoot,
        binding: &super::RelationalBranchRetentionBinding,
    ) -> Arc<dyn PublicationBasisAccess> {
        root.publication_bases
            .get()
            .and_then(Weak::upgrade)
            .filter(|image| image.belongs_to(binding))
            .unwrap_or_else(|| Arc::new(Self::default()))
    }
}
pub(crate) fn request(
    cause: impl Into<crate::execution::denial_cause::RelationalExecutionDenialCause>,
) -> TransactionCommitError {
    TransactionCommitError::execution(crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenial {
        kind: crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenialKind::Cause(cause.into()),
        partition_identity: None,
    })
}
