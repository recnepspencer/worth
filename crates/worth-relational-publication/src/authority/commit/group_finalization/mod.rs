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
use contracts::history::data::commit_receipt::RelationalCommitReceipt;
use contracts::lineage::data::artifacts::finalization_artifact::PreparedLineageFinalization;
use contracts::lineage::data::naming_denial::LineageNameDenial;
use contracts::lineage::data::postures::GroupPartitionClaim;
use contracts::publication::bundle::publication_status::PublicationStage;
use contracts::publication::data::publication_error::PublicationError;
use contracts::transactions::data::outcomes::commit_preparation::{
    CommitPreparationError, CommitPreparationErrorOwnerPort as _,
};
use contracts::transactions::data::outcomes::transaction_commit_error::{
    TransactionCommitError, TransactionCommitErrorOwnerPort as _,
};
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::owned_graph_selection::CanonicalOwnedGraphSelection;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_lineage::internal_port::durability::naming::encode_group_lineage_frame;
use worth_relational_lineage::internal_port::lineage::groups::{
    CommittedGroupTransition, GroupStateDenial,
};
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

    fn into_commit_error(self, commit: &RelationalCommitReceipt) -> TransactionCommitError {
        match self {
            Self::Naming { scheme_id, denial } => {
                TransactionCommitError::lineage_name_denial(LineageNameDenial::GroupNaming {
                    scheme_id,
                    denial,
                })
            }
            Self::Unavailable(GroupStateDenial::StorageUnavailable(cause)) => {
                TransactionCommitError::preparation(CommitPreparationError::storage_unavailable(
                    commit.branch_id.clone(),
                    Some(commit.commit_id.0),
                    commit.version_id,
                    contracts::storage::data::storage_unavailable::RelationalStorageUnavailable {
                        subject: contracts::storage::data::storage_unavailable::RelationalStorageSubject::Lineage {
                            first: contracts::identity::data::primitives::LineageId(0),
                            last: contracts::identity::data::primitives::LineageId(0),
                        },
                        cause,
                    },
                ))
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
    commit: &RelationalCommitReceipt,
    claims: &[GroupPartitionClaim],
    selection: Option<&CanonicalOwnedGraphSelection>,
    basis: &BranchNamingBasisGuard<'_>,
    prepared: &mut PreparedLineageFinalization,
) -> Result<Vec<CommittedGroupTransition>, TransactionCommitError> {
    let state = match basis.group_state() {
        Ok(state) => state,
        Err(denial @ GroupStateDenial::StorageUnavailable(_)) => {
            return Err(GroupRefusal::unavailable(denial.clone()).into_commit_error(commit));
        }
        Err(denial) if !claims.is_empty() || selection.is_some() => {
            return Err(GroupRefusal::unavailable(denial.clone()).into_commit_error(commit));
        }
        Err(_) => return Ok(Vec::new()),
    };
    if state.is_none() && claims.is_empty() {
        return Ok(Vec::new());
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
    };
    basis
        .settle()
        .map_err(|denial| super::lineage_preparation::naming_basis_commit_error(commit, denial))?;
    let transitions = transitions.map_err(|refusal| refusal.into_commit_error(commit))?;
    if transitions.is_empty() {
        return Ok(Vec::new());
    }
    if let Some(state) = state {
        state
            .verify(&transitions)
            .map_err(|denial| GroupRefusal::unavailable(denial).into_commit_error(commit))?;
    }
    let frame = encode_group_lineage_frame(&transitions).map_err(|refusal| {
        TransactionCommitError::publication(PublicationError::new(
            PublicationStage::BundleAssembly,
            format!("group lineage frame refused: {refusal:?}"),
        ))
    })?;
    prepared.attach_group_lineage(frame);
    Ok(transitions)
}
