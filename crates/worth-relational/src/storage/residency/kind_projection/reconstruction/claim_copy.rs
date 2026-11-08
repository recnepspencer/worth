//! Interruptible copies and equality of projection claims.
use super::*;

pub(super) fn clone_claim(
    entry: &RegionRelationProjection,
    checkpoint: &mut ProjectionCheckpoint<'_>,
) -> Result<RegionRelationProjection, Cause> {
    checkpoint(size_of::<RegionRelationProjection>() as u64, 4096)?;
    let mut answer = RegionRelationProjection {
        relation: entry.relation,
        kind: entry.kind,
        outgoing: BTreeSet::new(),
        incoming: BTreeSet::new(),
    };
    for (source, target) in [
        (&entry.outgoing, &mut answer.outgoing),
        (&entry.incoming, &mut answer.incoming),
    ] {
        for &slot in source {
            checkpoint(
                ordered_lookup_work(target.len())? + size_of::<usize>() as u64,
                256,
            )?;
            target.insert(slot);
        }
    }
    Ok(answer)
}

pub(super) fn claims_match(
    expected: &BTreeMap<RelationId, RegionRelationProjection>,
    supplied: &[RegionRelationProjection],
    checkpoint: &mut ProjectionCheckpoint<'_>,
) -> Result<bool, Cause> {
    if expected.len() != supplied.len() {
        return Ok(false);
    }
    for (left, right) in expected.values().zip(supplied) {
        checkpoint(1, 0)?;
        if left.relation != right.relation || left.kind != right.kind {
            return Ok(false);
        }
        for (left, right) in [
            (&left.outgoing, &right.outgoing),
            (&left.incoming, &right.incoming),
        ] {
            if left.len() != right.len() {
                return Ok(false);
            }
            for (left, right) in left.iter().zip(right) {
                checkpoint(1, 0)?;
                if left != right {
                    return Ok(false);
                }
            }
        }
    }
    Ok(true)
}
