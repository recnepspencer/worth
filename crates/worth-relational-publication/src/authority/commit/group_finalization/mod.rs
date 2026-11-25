//! Commit-time group naming: every installed group scheme a commit touches
//! gets one committed transition, encoded into the commit's canonical group
//! lineage frame.
//!
//! A history selection derives its groups alone, restoring the selected
//! commit's state; every other commit names its declared partitions, and the
//! groups its retirements shrink, through the group naming authority. The
//! transitions are verified against the branch's group state before the frame
//! exists, so a refused transition leaves the commit with zero effects.

mod commit_names;
mod partition;
mod selection;

use contracts::aggregate_identity::transition::group::denial::AggregateGroupNamingDenial;
use contracts::lineage::data::artifacts::finalization_artifact::PreparedLineageFinalization;
use contracts::lineage::data::naming_denial::LineageNameDenial;
use contracts::lineage::data::postures::GroupPartitionClaim;
use contracts::publication::bundle::publication_status::PublicationStage;
use contracts::publication::data::publication_error::PublicationError;
use contracts::transactions::data::outcomes::transaction_commit_error::{
    TransactionCommitError, TransactionCommitErrorOwnerPort as _,
};
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::owned_graph_selection::CanonicalOwnedGraphSelection;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_lineage::internal_port::durability::naming::encode_group_lineage_frame;
use worth_relational_lineage::internal_port::lineage::groups::GroupStateDenial;
use worth_relational_lineage::internal_port::runtime::state::subsystems::lineage::naming_basis_binding::BranchNamingBasisGuard;

/// A refused group transition, before it becomes a commit refusal.
pub(super) enum GroupRefusal {
    Naming {
        scheme_id: String,
        denial: AggregateGroupNamingDenial,
    },
    Unavailable(GroupStateDenial),
}

impl GroupRefusal {
    pub(super) fn new(scheme_id: &str, denial: AggregateGroupNamingDenial) -> Self {
        Self::Naming {
            scheme_id: scheme_id.to_owned(),
            denial,
        }
    }

    pub(super) fn unavailable(denial: GroupStateDenial) -> Self {
        Self::Unavailable(denial)
    }

    fn into_commit_error(self) -> TransactionCommitError {
        match self {
            Self::Naming { scheme_id, denial } => {
                TransactionCommitError::lineage_name_denial(LineageNameDenial::GroupNaming {
                    scheme_id,
                    denial,
                })
            }
            Self::Unavailable(denial) => {
                TransactionCommitError::publication(PublicationError::new(
                    PublicationStage::BundleAssembly,
                    format!("branch group state is unavailable: {denial:?}"),
                ))
            }
        }
    }
}

/// Names the commit's group transitions and attaches their canonical frame
/// to `prepared`.
///
/// Complexity: O(touched schemes * scoped members * log live groups).
pub(crate) fn finalize_commit_groups(
    claims: &[GroupPartitionClaim],
    selection: Option<&CanonicalOwnedGraphSelection>,
    basis: &BranchNamingBasisGuard<'_>,
    prepared: &mut PreparedLineageFinalization,
) -> Result<(), TransactionCommitError> {
    let state = match basis.group_state() {
        Ok(state) => state,
        Err(denial) if !claims.is_empty() || selection.is_some() => {
            return Err(GroupRefusal::unavailable(denial.clone()).into_commit_error());
        }
        Err(_) => return Ok(()),
    };
    if state.is_none() && claims.is_empty() {
        return Ok(());
    }
    let outcome = commit_names::CommitNameOutcome::of(prepared, basis);
    let transitions = match selection {
        Some(_) if !claims.is_empty() => {
            let scheme_id = claims[0].scheme.id().as_str();
            Err(GroupRefusal::new(
                scheme_id,
                AggregateGroupNamingDenial::SelectionClaim,
            ))
        }
        Some(selection) => selection::selection_transitions(selection, basis, state, &outcome),
        None => partition::partition_transitions(claims, state, &outcome),
    }
    .map_err(GroupRefusal::into_commit_error)?;
    if transitions.is_empty() {
        return Ok(());
    }
    if let Some(state) = state {
        state
            .verify(&transitions)
            .map_err(|denial| GroupRefusal::unavailable(denial).into_commit_error())?;
    }
    let frame = encode_group_lineage_frame(&transitions).map_err(|refusal| {
        TransactionCommitError::publication(PublicationError::new(
            PublicationStage::BundleAssembly,
            format!("group lineage frame refused: {refusal:?}"),
        ))
    })?;
    prepared.attach_group_lineage(frame);
    Ok(())
}
