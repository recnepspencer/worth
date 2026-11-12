//! Live publication holds are owned by heads/candidates, never canonical origins.
use crate::{
    branch::{
        AdmittedRelationalBranchBasis, RelationalBranchBasisDescriptor, RelationalBranchRoot,
    },
    history::data::{CanonicalOutputPublication, CommitId},
    identity::data::EntityId,
    mvcc::RequiredOutputPublicationMeaning,
    transactions::data::TransactionCommitError,
};
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
    pub(crate) applied: Option<Arc<dyn crate::history::data::AppliedPublicationInputAccess>>,
    _hold: super::RelationalObservationRetentionObligation,
    _funding: Option<worth_execution::ExecutionMemoryReservation>,
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
    /// O((changed routes) log(live routes)); unchanged images share one Arc.
    pub(crate) fn prepare(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        next: &Arc<RelationalBranchRoot>,
    ) -> Result<Arc<dyn PublicationBasisAccess>, TransactionCommitError> {
        let previous = Self::at_for_owner(&basis.inner.root, &basis.inner.retention_binding);
        let envelope = next
            .canonical_envelope()
            .expect("candidate has its canonical commit");
        let changes =
            envelope.output_origins().count() + envelope.required_output_codec_retirements.len();
        if changes == 0 {
            return Ok(previous);
        }
        let height = usize::BITS as usize
            - previous
                .routes()
                .len()
                .saturating_add(changes)
                .max(1)
                .leading_zeros() as usize;
        let funding = execution
            .reserve_retained_memory(
                (std::mem::size_of::<Self>() + 2 * std::mem::size_of::<usize>()) as u64,
            )
            .map_err(request)?;
        let mut routes = previous.routes().clone();
        let mut answer = None;
        execution
            .run_native_with_checkpoint(|context| {
                answer = Some((|| {
                    for origin in envelope.output_origins() {
                        crate::execution::request_stage::checkpoint(
                            context,
                            (height + 1)
                                .saturating_mul(origin.meaning.family.len() + 1)
                                .saturating_mul(8) as u64,
                        )
                        .map_err(request)?;
                        let route = PublicationBasisRoute::of(&origin.meaning);
                        routes.remove(&route, execution, context)?;
                        let applied = crate::history::data::LivePublicationInputs::take_verified(envelope, origin);
                        // Cold/deep-cloned coupled envelopes have no issuance
                        // for D. Never approximate that input with committed C.
                        if (envelope.coupled_authored_publication.is_some() && applied.is_none())
                            || !origin.source_scope_owned
                            || matches!(
                                origin.publication,
                                CanonicalOutputPublication::ProducerRetirement { .. }
                                    | CanonicalOutputPublication::AuthoredRetirement { .. }
                            )
                        {
                            continue;
                        }
                        if origin.basis
                            != basis.descriptor().in_canonical_frame().map_err(|_| {
                                crate::mvcc::origin_denial("invalid publication basis frame")
                            })?
                        {
                            return Err(crate::mvcc::origin_denial(
                                "publication decision basis differs from candidate basis",
                            ));
                        }
                        let parent_bytes = basis.reference().target().as_basis()
                            .map_or(0, |target| target.parent_commit_ids().len())
                            .checked_mul(std::mem::size_of::<u64>());
                        let bytes = parent_bytes.and_then(|parents| {
                            [std::mem::size_of::<RetainedPublicationBasis>(),
                                4 * std::mem::size_of::<usize>(), parents,
                                origin.meaning.family.len(), origin.basis.branch_id().0.len(),
                                origin.basis.reference().branch_id().as_str().len()]
                                .into_iter().try_fold(0usize, usize::checked_add)
                        }).ok_or_else(|| request(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow))?;
                        let funding = execution
                            .reserve_retained_memory(bytes as u64)
                            .map_err(request)?;
                        let hold = super::RelationalObservationRetentionObligation::acquire(
                            &basis.inner.retention_binding,
                            Arc::clone(&basis.inner.root),
                        )
                        .map_err(|_| {
                            crate::mvcc::origin_denial(
                                "publication predecessor retention unavailable",
                            )
                        })?;
                        routes.insert(
                            route,
                            Arc::new(RetainedPublicationBasis {
                                publication: origin.commit.commit_id,
                                descriptor: basis.descriptor().clone(),
                                root: Arc::clone(&basis.inner.root),
                                binding: basis.inner.retention_binding.clone(),
                                applied,
                                _hold: hold,
                                _funding: funding,
                            }),
                            execution,
                            context,
                        )?;
                    }
                    for retired in &envelope.required_output_codec_retirements {
                        crate::execution::request_stage::checkpoint(
                            context,
                            (height + 1)
                                .saturating_mul(retired.meaning.family.len() + 1)
                                .saturating_mul(8) as u64,
                        )
                        .map_err(request)?;
                        routes.remove(
                            &PublicationBasisRoute::of(&retired.meaning),
                            execution,
                            context,
                        )?;
                    }
                    Ok(())
                })());
            })
            .map_err(request)?;
        answer.expect("completed retention preparation recorded its result")?;
        Ok(Arc::new(Self {
            owner: Some(basis.inner.retention_binding.clone()),
            routes,
            _funding: funding,
        }))
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
pub(super) fn request(
    cause: impl Into<crate::execution::RelationalExecutionDenialCause>,
) -> TransactionCommitError {
    TransactionCommitError::execution(crate::transactions::data::CommitExecutionDenial {
        kind: crate::transactions::data::CommitExecutionDenialKind::Cause(cause.into()),
        partition_identity: None,
    })
}
