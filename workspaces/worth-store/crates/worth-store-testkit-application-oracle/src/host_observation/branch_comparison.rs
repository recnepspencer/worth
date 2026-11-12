//! Branch roles supply semantic pairings, never issuer or digest order.
use super::{
    comparison::{self, HostDivergence, HostOracleComparison, ProjectionPart},
    projection::StateProjection,
};
use crate::facade::BranchLabel;
pub fn compare_branches(
    comparison: &mut HostOracleComparison,
    h: &[(BranchLabel, &StateProjection)],
    r: &[(BranchLabel, &StateProjection)],
) -> Result<(), HostDivergence> {
    if !h.iter().any(|(label, _)| *label == BranchLabel::Main)
        || !r.iter().any(|(label, _)| *label == BranchLabel::Main)
        || h.len() != r.len()
    {
        return Err(HostDivergence::Meaning {
            part: ProjectionPart::BranchRoster,
            index: 0,
        });
    }
    for label in [BranchLabel::Main, BranchLabel::Left, BranchLabel::Right] {
        let home: Vec<_> = h.iter().filter(|(l, _)| *l == label).collect();
        let reference: Vec<_> = r.iter().filter(|(l, _)| *l == label).collect();
        if home.len() != reference.len() || home.len() > 1 {
            return Err(HostDivergence::Meaning {
                part: ProjectionPart::BranchRoster,
                index: 0,
            });
        }
        if let (Some(h), Some(r)) = (home.first(), reference.first()) {
            comparison::compare(comparison, h.1, r.1)?;
        }
    }
    Ok(())
}
