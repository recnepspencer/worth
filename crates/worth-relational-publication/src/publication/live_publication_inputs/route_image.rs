//! Install the exact applied route pins into the selected branch's retained route image.
use worth_relational_contracts::internal_port as contracts;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::group_selection::CanonicalCommitEnvelopeBodyOwnerPort as _;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyFieldsPort as _;
use std::sync::Arc;
use worth_relational_branch::internal_port::branch::basis::{AdmittedRelationalBranchBasis, AdmittedRelationalBranchBasisFieldsPort as _, AdmittedRelationalBranchBasisOwnerPort as _};
use worth_relational_branch::internal_port::branch::canonical_runtime_frame::RelationalBranchBasisDescriptorOwnerPort as _;

use super::BoundLivePublicationInputs;
use worth_relational_branch::internal_port::branch::root::RelationalBranchRoot;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::CanonicalOutputPublication;
use worth_relational_branch::internal_port::history::retention::publication_basis::{PublicationBasisAccess, PublicationBasisImage, PublicationBasisRoute, RetainedPublicationBasis, request};
use worth_relational_branch::internal_port::history::retention::obligation::RelationalObservationRetentionObligation;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;

impl BoundLivePublicationInputs {
    /// O((changed routes) log(live routes)); unchanged images share one Arc.
    pub(crate) fn prepare(
        mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        next: &Arc<RelationalBranchRoot>,
    ) -> Result<Arc<dyn PublicationBasisAccess>, TransactionCommitError> {
        let previous = PublicationBasisImage::at_for_owner(
            &basis.root_read(),
            &basis.owner_inner().retention_binding,
        );
        let envelope = next
            .canonical_envelope()
            .expect("candidate has its canonical commit");
        let changes = envelope.output_origins().count()
            + (*envelope.owner_required_output_codec_retirements()).len();
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
                (std::mem::size_of::<PublicationBasisImage>() + 2 * std::mem::size_of::<usize>())
                    as u64,
            )
            .map_err(request)?;
        let mut routes = previous.routes().clone();
        let mut answer = None;
        execution
            .run_native_with_checkpoint(|context| {
                answer = Some((|| {
                    for origin in envelope.output_origins() {
                        contracts::execution::request_stage::checkpoint(
                            context,
                            (height + 1)
                                .saturating_mul(origin.meaning.family.len() + 1)
                                .saturating_mul(8) as u64,
                        )
                        .map_err(request)?;
                        let route = PublicationBasisRoute::of(&origin.meaning);
                        routes.remove(&route, execution, context)?;
                        let applied = self.take_verified(envelope, origin);
                        // Cold/deep-cloned coupled envelopes have no issuance
                        // for D. Never approximate that input with committed C.
                        if ((*envelope.owner_coupled_authored_publication()).is_some() && applied.is_none())
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
                                contracts::transactions::data::outcomes::transaction_commit_error::origin_denial("invalid publication basis frame")
                            })?
                        {
                            return Err(contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
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
                        }).ok_or_else(|| request(contracts::execution::denial_cause::RelationalExecutionDenialCause::ChargedBytesOverflow))?;
                        let funding = execution
                            .reserve_retained_memory(bytes as u64)
                            .map_err(request)?;
                        let hold = RelationalObservationRetentionObligation::acquire(
                            &basis.owner_inner().retention_binding,
                            Arc::clone(&basis.owner_inner().root),
                        )
                        .map_err(|_| {
                            contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                                "publication predecessor retention unavailable",
                            )
                        })?;
                        routes.insert(
                            route,
                            Arc::new(RetainedPublicationBasis {
                                publication: origin.commit.commit_id,
                                descriptor: basis.descriptor().clone(),
                                root: Arc::clone(&basis.owner_inner().root),
                                binding: basis.owner_inner().retention_binding.clone(),
                                applied,
                                publication_cell: basis.publication_cell().clone(),
                                _count_guard: worth_relational_branch::internal_port::history::retention::publication_basis::PublicationBasisCountGuard::acquire(
                                    &basis.owner_inner().retention_binding,
                                ),
                                _hold: hold,
                                _funding: funding,
                            }),
                            execution,
                            context,
                        )?;
                    }
                    for retired in envelope.owner_required_output_codec_retirements() {
                        contracts::execution::request_stage::checkpoint(
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
        Ok(PublicationBasisImage::from_funded_routes(
            basis.owner_inner().retention_binding.clone(),
            routes,
            funding,
        ))
    }
}
