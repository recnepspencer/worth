//! Read the recorded decision basis of the exact publication selected for succession.
use super::RelationalRequiredOutputPublication;
use crate::{
    branch::{AdmittedRelationalBranchBasis, RelationalBranchBasisDenial},
    history::data::CommitId,
    history::retention::PublicationBasisImage,
    runtime::RelationalRuntime,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelationalPredecessorBasisOrigin {
    HistoricalSelection,
    AuthoredRestoration,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RelationalPredecessorBasisDenial {
    PredecessorBasisUnavailable,
    PredecessorBasisUnavailableForOrigin(RelationalPredecessorBasisOrigin),
    PredecessorMismatch,
    BasisAdmission(RelationalBranchBasisDenial),
    Selection(crate::transactions::data::TransactionCommitError),
    Execution(crate::execution::RelationalExecutionDenialCause),
}

impl RelationalRuntime {
    /// Requires genuine publication custody and the exact predecessor commit
    /// selected by the candidate's succession fence. This performs no current
    /// observation and adds no dependency or publication authority.
    ///
    /// A first publication or a cold restored root has no retained decision
    /// basis. Its canonical coordinate is reserved by origin.basis, but that
    /// descriptor never reconstructs versions or approximates with current state.
    /// Cost: logarithmic route-index lookup plus descriptor metadata, no truth copy.
    pub fn predecessor_publication_basis(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        selected: &AdmittedRelationalBranchBasis,
        predecessor: Option<&RelationalRequiredOutputPublication>,
        expected_predecessor: Option<CommitId>,
    ) -> Result<AdmittedRelationalBranchBasis, RelationalPredecessorBasisDenial> {
        use RelationalPredecessorBasisDenial as Denial;
        let Some(prior) = predecessor else {
            return Err(Denial::PredecessorBasisUnavailable);
        };
        if prior.runtime_instance_id != self.runtime_instance_id()
            || selected.identity().runtime_instance_id() != self.runtime_instance_id()
            || prior.authority.publication_coordinate().0 != prior.origin.commit.commit_id.0
            || expected_predecessor != Some(prior.origin.commit.commit_id)
        {
            return Err(Denial::PredecessorMismatch);
        }
        if !prior.origin.source_scope_owned {
            return Err(Denial::PredecessorBasisUnavailable);
        }
        // Restoration records the navigation candidate's precommit basis, not
        // the selected producer's consumed inputs. #94 reserves that contract.
        let restored = match prior.origin.publication {
            crate::history::data::CanonicalOutputPublication::HistoricalSelection { .. } => {
                Some(RelationalPredecessorBasisOrigin::HistoricalSelection)
            }
            crate::history::data::CanonicalOutputPublication::AuthoredRestoration { .. } => {
                Some(RelationalPredecessorBasisOrigin::AuthoredRestoration)
            }
            _ => None,
        };
        if let Some(origin) = restored {
            return Err(Denial::PredecessorBasisUnavailableForOrigin(origin));
        }
        // Unrelated publication changes may replace the image wrapper while
        // sharing this still-live route's hold. The live head is only a pin
        // locator: values always come from the matched hold's recorded root.
        let image = selected
            .publication_cell()
            .head_retention()
            .publication_bases()
            .unwrap_or_else(|| {
                PublicationBasisImage::at_for_owner(
                    &selected.inner.root,
                    &self.history.retention_binding(),
                )
            });
        let work = u64::from(usize::BITS - image.routes().len().max(1).leading_zeros())
            .saturating_add(1)
            .saturating_mul(prior.origin.meaning.family.len() as u64 + 1)
            .saturating_mul(2);
        let positions = &selected.inner.root.output_publication_positions;
        let position = execution
            .run_native_with_checkpoint(|context| {
                crate::execution::request_stage::checkpoint(context, work)?;
                Ok(positions.get(&prior.origin.meaning, context))
            })
            .map_err(|stop| Denial::Execution(stop.into()))?
            .0
            .map_err(|stop: worth_execution::MapKernelStop| Denial::Execution(stop.into()))?
            .map_err(Denial::Selection)?;
        let retained = image
            .routes()
            .get_by_meaning(&prior.origin.meaning)
            .ok_or(Denial::PredecessorBasisUnavailable)?;
        use crate::history::data::RequiredOutputRoutePosition;
        if position
            != Some(RequiredOutputRoutePosition::Published(
                prior.origin.commit.commit_id,
            ))
        {
            return Err(
                if position.is_none() && positions.unresolved_prefix().is_some() {
                    Denial::PredecessorBasisUnavailable
                } else {
                    Denial::PredecessorMismatch
                },
            );
        }
        if retained.publication != prior.origin.commit.commit_id
            || retained
                .descriptor
                .in_canonical_frame()
                .map_err(|_| Denial::PredecessorMismatch)?
                != prior.origin.basis
        {
            return Err(Denial::PredecessorMismatch);
        }
        let cell = self
            .history
            .branch_cell(retained.descriptor.branch_id())
            .ok_or(Denial::PredecessorBasisUnavailable)?;
        crate::branch::issue_admitted_relational_branch_basis_with_retention(
            retained.descriptor.clone(),
            cell.identity().clone(),
            std::sync::Arc::clone(&retained.root),
            cell.publication_cell(),
            &retained.binding,
        )
        .and_then(|basis| cell.register_basis(basis))
        .map_err(Denial::BasisAdmission)
    }
}
