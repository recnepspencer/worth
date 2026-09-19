//! Temporary candidate storage, never dependency or invalidation authority.
use crate::data::error::SignalError;
use crate::data::handle::NodeId;
use crate::logic::evaluation::EvaluationWork;

pub(crate) fn reserve_additional(
    candidates: &mut Vec<NodeId>,
    additional: usize,
    work: &mut EvaluationWork<'_>,
) -> Result<(), SignalError> {
    let count = candidates.len().checked_add(additional);
    work.reserve(count.and_then(|count| {
        count
            .checked_mul(std::mem::size_of::<NodeId>())
            .filter(|bytes| *bytes <= isize::MAX as usize)?;
        // Existing elements may move during growth; each new element is visited
        // and copied. Admission precedes even the capacity reservation.
        count.checked_add(additional)?.checked_add(1)
    }))?;
    candidates.reserve_exact(additional);
    Ok(())
}

pub(crate) fn append(
    candidates: &mut Vec<NodeId>,
    incoming: Vec<NodeId>,
    work: &mut EvaluationWork<'_>,
) -> Result<(), SignalError> {
    reserve_additional(candidates, incoming.len(), work)?;
    candidates.extend(incoming);
    Ok(())
}

pub(crate) fn normalize(
    candidates: &mut Vec<NodeId>,
    work: &mut EvaluationWork<'_>,
) -> Result<(), SignalError> {
    let count = candidates.len();
    let height = usize::BITS as usize - count.leading_zeros() as usize;
    // Heap build/extraction: <= n+n/2 descents, each <= height, with
    // two fixed NodeId comparisons and a swap. Include extraction and dedup.
    work.reserve(
        count
            .checked_add(count / 2)
            .and_then(|n| n.checked_mul(height))
            .and_then(|n| n.checked_mul(8))
            .and_then(|n| n.checked_add(count.checked_mul(4)?)),
    )?;
    for root in (0..count / 2).rev() {
        sift_down(candidates, root);
    }
    for end in (1..count).rev() {
        candidates.swap(0, end);
        sift_down(&mut candidates[..end], 0);
    }
    candidates.dedup();
    Ok(())
}

fn sift_down(candidates: &mut [NodeId], mut root: usize) {
    while root < candidates.len() / 2 {
        let mut child = root * 2 + 1;
        if child + 1 < candidates.len() && candidates[child] < candidates[child + 1] {
            child += 1;
        }
        if candidates[root] >= candidates[child] {
            break;
        }
        candidates.swap(root, child);
        root = child;
    }
}

#[cfg(test)]
mod tests;
