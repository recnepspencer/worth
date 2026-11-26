//! Read the recorded decision basis of the exact publication selected for succession.
use super::RelationalRequiredOutputPublication;
use contracts::branch::basis_denial::RelationalBranchBasisDenial;
use contracts::history::data::coordinates::CommitId;
use worth_relational_branch::internal_port::branch::basis::{
    AdmittedRelationalBranchBasis, AdmittedRelationalBranchBasisFieldsPort as _,
    AdmittedRelationalBranchBasisOwnerPort as _,
};
use worth_relational_branch::internal_port::branch::canonical_runtime_frame::RelationalBranchBasisDescriptorOwnerPort as _;
use worth_relational_branch::internal_port::history::retention::publication_basis::PublicationBasisImage;
use worth_relational_branch::internal_port::runtime::state::subsystems::history::HistorySubsystem;
use worth_relational_contracts::internal_port as contracts;

/// Exact consumed inputs. A coupled image is an applied proposal, never a
/// committed branch basis or a snapshot. Only the publication owner issues it.
#[derive(Clone, Debug)]
pub struct RelationalPredecessorInput {
    before: AdmittedRelationalBranchBasis,
    applied: Option<
        std::sync::Arc<
            dyn worth_relational_branch::internal_port::history::retention::publication_basis::RetainedAppliedPublicationPin,
        >,
    >,
}
impl RelationalPredecessorInput {
    /// The genuine committed basis before the transaction. For an applied
    /// input this supplies index ancestry, not the writer's input values.
    pub fn before_basis(&self) -> &AdmittedRelationalBranchBasis {
        &self.before
    }
    pub fn applied_read(
        &self,
    ) -> Option<
        &crate::visibility::materialization::read_records::reader::pinned_read::RelationalPinnedRead<
            crate::visibility::materialization::read_records::reader::pinned_read::RelationalAppliedCandidateReadIdentity,
        >,
    >{
        self.applied
            .as_deref()
            .map(crate::publication::live_publication_inputs::retained_applied_read)
    }
}

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
    Selection(
        contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
    ),
    Execution(contracts::execution::denial_cause::RelationalExecutionDenialCause),
}

/// Requires genuine publication custody and the exact predecessor commit
/// selected by the candidate's succession fence. This performs no current
/// observation and adds no dependency or publication authority.
///
/// A first publication or a cold restored root has no retained decision
/// basis. Its canonical coordinate is reserved by origin.basis, but that
/// descriptor never reconstructs versions or approximates with current state.
/// Cost: logarithmic route-index lookup plus descriptor metadata, no truth copy.
pub fn predecessor_publication_basis(
    runtime_instance_id: u64,
    history: &HistorySubsystem,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    selected: &AdmittedRelationalBranchBasis,
    predecessor: Option<&RelationalRequiredOutputPublication>,
    expected_predecessor: Option<CommitId>,
) -> Result<AdmittedRelationalBranchBasis, RelationalPredecessorBasisDenial> {
    let input = predecessor_publication_input(
        runtime_instance_id,
        history,
        execution,
        selected,
        predecessor,
        expected_predecessor,
    )?;
    if input.applied.is_some() {
        return Err(RelationalPredecessorBasisDenial::PredecessorBasisUnavailable);
    }
    Ok(input.before)
}

/// Select genuine committed or pre-writer applied inputs from the same
/// authenticated route retention. No input is reconstructed from C.
pub fn predecessor_publication_input(
    runtime_instance_id: u64,
    history: &HistorySubsystem,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    selected: &AdmittedRelationalBranchBasis,
    predecessor: Option<&RelationalRequiredOutputPublication>,
    expected_predecessor: Option<CommitId>,
) -> Result<RelationalPredecessorInput, RelationalPredecessorBasisDenial> {
    use RelationalPredecessorBasisDenial as Denial;
    let Some(prior) = predecessor else {
        return Err(Denial::PredecessorBasisUnavailable);
    };
    if prior.runtime_instance_id != runtime_instance_id
        || selected.identity().runtime_instance_id() != runtime_instance_id
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
        worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::CanonicalOutputPublication::HistoricalSelection { .. } => {
            Some(RelationalPredecessorBasisOrigin::HistoricalSelection)
        }
        worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::CanonicalOutputPublication::AuthoredRestoration { .. } => {
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
            PublicationBasisImage::at_for_owner(&selected.root_read(), &history.retention_binding())
        });
    let work = u64::from(usize::BITS - image.routes().len().max(1).leading_zeros())
        .saturating_add(1)
        .saturating_mul(prior.origin.meaning.family.len() as u64 + 1)
        .saturating_mul(2);
    let positions = &selected.owner_inner().root.output_publication_positions;
    let position = execution
        .run_native_with_checkpoint(|context| {
            contracts::execution::request_stage::checkpoint(context, work)?;
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
    use worth_relational_branch::internal_port::history::data::required_output_positions::RequiredOutputRoutePosition;
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
    // The hold carries the publishing branch's cell: a fork keeps this
    // input after that branch is deleted and leaves the registry.
    let cell = &retained.publication_cell;
    worth_relational_branch::internal_port::branch::basis_admission::issue_admitted_relational_branch_basis_with_retention(
        retained.descriptor.clone(),
        cell.identity().clone(),
        std::sync::Arc::clone(&retained.root),
        cell.clone(),
        &retained.binding,
    )
    .and_then(|basis| cell.register_basis(basis))
    .map_err(Denial::BasisAdmission)
    .map(|before| RelationalPredecessorInput {
        before,
        applied: retained.applied.clone(),
    })
}
