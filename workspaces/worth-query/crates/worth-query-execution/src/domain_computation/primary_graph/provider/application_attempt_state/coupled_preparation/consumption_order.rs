//! Members finish in consumption order (R3): a member's lineage slot records
//! after every member whose completion one of its consumptions settles
//! against, because R3 may settle a consumer to a replacement appended after
//! it. Final support ranks the group in that order before the World commit;
//! publication finishes members by rank and never computes an order itself.
use super::CoupledApplicationMember;

/// Rank `members` in consumption order, in position order among members
/// that are ready together. A cycle (a later round that read its own settled
/// consumer) has no order: the members it leaves unranked are returned, and
/// the group must refuse before the World commit.
pub(in crate::domain_computation::primary_graph::provider) fn rank_consumption(
    members: &mut [CoupledApplicationMember],
) -> Result<(), Vec<usize>> {
    let completions = members
        .iter()
        .map(|member| {
            let attempt = &member.owned.as_ref()?.attempt;
            Some(attempt.coupled_preparation()?.completion.clone())
        })
        .collect::<Vec<_>>();
    let parents = members
        .iter()
        .map(|member| {
            let Some(owned) = member.owned.as_ref() else {
                return Vec::new();
            };
            owned
                .attempt
                .consumed_outputs()
                .iter()
                .filter_map(|consumed| consumed.applied())
                .flat_map(|link| {
                    let settling = link.settling_completion();
                    completions
                        .iter()
                        .enumerate()
                        .filter(move |(_, parent)| {
                            parent
                                .as_ref()
                                .is_some_and(|parent| parent.same_writer(&settling))
                        })
                        .map(|(index, _)| index)
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let mut done = vec![false; members.len()];
    for rank in 0..members.len() {
        let ready = |index: usize| {
            !done[index]
                && parents[index]
                    .iter()
                    .all(|&parent| parent == index || done[parent])
        };
        let Some(next) = (0..members.len()).find(|&index| ready(index)) else {
            return Err((0..members.len()).filter(|&index| !done[index]).collect());
        };
        done[next] = true;
        members[next].consumption_rank = rank;
    }
    Ok(())
}
