//! Replay claims for a commit's durable group lineage.
//!
//! Tail replay re-executes each commit through the production commit path,
//! so its group transitions are re-named by the group naming authority. The
//! claim for one recorded partition declares every next-state group the
//! commit recorded, and dissolves every prior group it touched (each kept or
//! retired one): with the whole touched scope declared, the authority's
//! naming is a function of the branch's replayed prior groups alone, and
//! reconciliation then requires the replayed frame to equal the durable one.
//! A history selection derives its groups again from its selection, so it
//! contributes no claim.

use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::{
    CanonicalCommitEnvelope, CanonicalCommitEnvelopeBodyOwnerPort as _,
};
use worth_relational_contracts::internal_port as contracts;
use worth_relational_lineage::internal_port::durability::naming::decode_group_lineage_frame;
use worth_relational_lineage::internal_port::lineage::groups::GroupTransitionKind;

use contracts::aggregate_identity::transition::group::model::AggregateGroupOrigin;
use contracts::durability::data::recovery_errors::{DurabilityError, RecoveryFailureClass};
use contracts::lineage::data::postures::GroupPartitionClaim;

/// Complexity: O(frame bytes).
pub fn replay_group_claims(
    envelope: &CanonicalCommitEnvelope,
) -> Result<Vec<GroupPartitionClaim>, DurabilityError> {
    let transitions = decode_group_lineage_frame(envelope.published_lineage().group_lineage())
        .map_err(|refusal| {
            DurabilityError::new(
                RecoveryFailureClass::ReplayFailure,
                format!(
                    "commit {} group lineage: {refusal:?}",
                    envelope.commit.commit_id.0
                ),
            )
        })?;
    Ok(transitions
        .into_iter()
        .filter(|committed| committed.kind == GroupTransitionKind::Partition)
        .map(|committed| {
            let transition = committed.transition;
            let kept = transition
                .assignments
                .iter()
                .filter(|assignment| assignment.origin == AggregateGroupOrigin::Kept)
                .map(|assignment| assignment.identity);
            GroupPartitionClaim {
                scheme: committed.scheme,
                dissolved: transition.retired.iter().copied().chain(kept).collect(),
                groups: transition
                    .assignments
                    .iter()
                    .map(|assignment| {
                        let members = assignment.members.iter();
                        members.map(|(name, member)| (*name, member.role)).collect()
                    })
                    .collect(),
            }
        })
        .collect())
}
