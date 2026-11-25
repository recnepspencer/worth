//! Live publication holds are owned by heads/candidates, never canonical origins.
use worth_relational_contracts::internal_port::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;
use crate::branch::basis::RelationalBranchBasisDescriptor;
use crate::branch::root::RelationalBranchRoot;
use crate::history::data::CommitId;
use crate::identity::data::EntityId;
use crate::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaning;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use std::sync::{Arc, Weak};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PublicationBasisRoute {
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
    pub fn of(meaning: &RequiredOutputPublicationMeaning) -> Self {
        Self {
            family: Arc::from(meaning.family.as_str()),
            source: meaning.source_scope,
            partition: meaning.source_partition,
        }
    }
}

/// The original applied read is retained here for the publication that issued it.
pub trait RetainedAppliedPublicationPin: std::fmt::Debug + Send + Sync + std::any::Any {}

/// Counts a head-owned publication predecessor basis while it is retained,
/// so such holds are never mistaken for detached retention obligations.
#[derive(Debug)]
pub struct PublicationBasisCountGuard(Weak<super::owner::RelationalBranchRetentionOwnerInner>);

impl PublicationBasisCountGuard {
    pub fn acquire(binding: &super::RelationalBranchRetentionBinding) -> Self {
        if let Some(owner) = binding.owner.upgrade() {
            owner
                .live_publication_basis_count
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        Self(binding.owner.clone())
    }
}

impl Drop for PublicationBasisCountGuard {
    fn drop(&mut self) {
        if let Some(owner) = self.0.upgrade() {
            owner
                .live_publication_basis_count
                .fetch_sub(1, std::sync::atomic::Ordering::Release);
        }
    }
}

#[derive(Debug)]
pub struct RetainedPublicationBasis {
    pub publication: CommitId,
    pub descriptor: RelationalBranchBasisDescriptor,
    pub root: Arc<RelationalBranchRoot>,
    pub binding: super::RelationalBranchRetentionBinding,
    pub applied: Option<Arc<dyn RetainedAppliedPublicationPin>>,
    /// The publishing branch's own cell, so issuing this input does not
    /// depend on that branch still being registered after it is deleted.
    pub publication_cell:
        crate::branch::reference_publication_cell::RelationalBranchPublicationCell,
    pub _count_guard: PublicationBasisCountGuard,
    pub _hold: super::RelationalObservationRetentionObligation,
    pub _funding: Option<worth_execution::ExecutionMemoryReservation>,
}

pub trait PublicationBasisAccess: std::fmt::Debug + Send + Sync {
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
pub struct PublicationBasisImage {
    owner: Option<super::RelationalBranchRetentionBinding>,
    pub routes: super::publication_basis_tree::PublicationBasisRoutes,
    // Every persistent node owns its own funding; this reservation covers only
    // the image header. Sharing nodes never loses their reservations.
    _funding: Option<worth_execution::ExecutionMemoryReservation>,
}
impl PublicationBasisImage {
    pub fn from_funded_routes(
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
    pub fn at_for_owner(
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
pub fn request(
    cause: impl Into<crate::execution::denial_cause::RelationalExecutionDenialCause>,
) -> TransactionCommitError {
    TransactionCommitError::execution(crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenial {
        kind: crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenialKind::Cause(cause.into()),
        partition_identity: None,
    })
}
