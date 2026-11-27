//! Replay claims for a commit's durable group lineage.
//!
//! Tail replay re-executes each commit through the production commit path,
//! so its group transitions are re-named by the group naming authority. The
//! claim for one recorded partition declares every next-state group the
//! commit recorded, and dissolves every prior group it touched (each kept or
//! retired one): with the whole touched scope declared, the authority's
//! naming is a function of the branch's replayed prior groups alone, and
//! reconciliation then requires the replayed frame to equal the durable one.
//! A kept group's recorded member delta is completed against the replayed
//! prior group, so recovery (never an ordinary commit) walks that group's
//! members. A history selection derives its groups again from its
//! selection, so it contributes no claim.

use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::{
    CanonicalCommitEnvelope, CanonicalCommitEnvelopeBodyOwnerPort as _,
};
use worth_relational_contracts::internal_port as contracts;
use worth_relational_lineage::internal_port::durability::naming::decode_group_lineage_frame;
use worth_relational_lineage::internal_port::lineage::groups::{
    CommittedGroupTransition, GroupTransitionKind,
};
use worth_relational_lineage::internal_port::runtime::state::subsystems::lineage::naming_basis_binding::BranchNamingBasisGuard;

use contracts::aggregate_identity::transition::group::delta::AggregateGroupChange;
use contracts::durability::data::recovery_errors::{DurabilityError, RecoveryFailureClass};
use contracts::lineage::data::postures::GroupPartitionClaim;

/// The commit's recorded partition transitions.
///
/// Complexity: O(frame bytes).
pub fn replay_group_transitions(
    envelope: &CanonicalCommitEnvelope,
) -> Result<Vec<CommittedGroupTransition>, DurabilityError> {
    let transitions = decode_group_lineage_frame(envelope.published_lineage().group_lineage())
        .map_err(|refusal| replay_failure(envelope, format!("{refusal:?}")))?;
    Ok(transitions
        .into_iter()
        .filter(|committed| committed.kind == GroupTransitionKind::Partition)
        .collect())
}

/// The claims re-declaring `transitions` against the branch's replayed
/// group state, read through `basis`.
///
/// Complexity: O(recorded members + each kept group's members).
pub fn replay_group_claims(
    envelope: &CanonicalCommitEnvelope,
    transitions: Vec<CommittedGroupTransition>,
    basis: &BranchNamingBasisGuard<'_>,
) -> Result<Vec<GroupPartitionClaim>, DurabilityError> {
    if transitions.is_empty() {
        return Ok(Vec::new());
    }
    let state = basis.group_state().map_err(|denial| {
        let detail = format!("branch group state is unavailable: {denial:?}");
        replay_failure(envelope, detail)
    })?;
    let mut claims = Vec::with_capacity(transitions.len());
    for committed in transitions {
        let transition = committed.transition;
        let groups = state.and_then(|state| state.scheme(committed.scheme.id().as_str()));
        let mut dissolved: Vec<_> = transition.retired.iter().copied().collect();
        let mut declared = Vec::with_capacity(transition.assignments.len());
        for assignment in &transition.assignments {
            let members = match &assignment.change {
                AggregateGroupChange::Kept(delta) => {
                    dissolved.push(assignment.identity);
                    let prior = groups
                        .and_then(|groups| groups.group(&assignment.identity))
                        .ok_or_else(|| {
                            let detail =
                                format!("kept group {:?} is not live", assignment.identity);
                            replay_failure(envelope, detail)
                        })?;
                    let mut members = prior.members;
                    for name in delta.removed() {
                        members.remove(name);
                    }
                    for (name, member) in delta.upserted() {
                        members.remove(name);
                        let _ = members.insert(*name, member.carrier.clone(), member.role);
                    }
                    members
                }
                AggregateGroupChange::Born(members) | AggregateGroupChange::Revived(members) => {
                    members.clone()
                }
            };
            declared.push(members.iter().map(|(name, m)| (*name, m.role)).collect());
        }
        dissolved.sort_unstable();
        claims.push(GroupPartitionClaim {
            scheme: committed.scheme,
            dissolved,
            groups: declared,
        });
    }
    Ok(claims)
}

fn replay_failure(envelope: &CanonicalCommitEnvelope, detail: String) -> DurabilityError {
    DurabilityError::new(
        RecoveryFailureClass::ReplayFailure,
        format!(
            "commit {} group lineage: {detail}",
            envelope.commit.commit_id.0
        ),
    )
}
