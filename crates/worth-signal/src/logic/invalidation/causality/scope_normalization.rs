use super::preparation_work::scope_comparison;
use crate::data::error::SignalError;
use crate::data::output::PartitionSubscription;
use crate::data::proof::PartitionScopeSet;
use crate::logic::evaluation::EvaluationWork;

pub(crate) fn normalize(
    mut scopes: Vec<PartitionSubscription>,
    work: &mut EvaluationWork<'_>,
) -> Result<PartitionScopeSet, SignalError> {
    work.reserve(Some(scopes.len()))?;
    let mut comparison = 16;
    for scope in &scopes {
        let bound = scope_comparison(Some(scope));
        work.reserve(bound.map(|_| 0))?;
        comparison = comparison.max(bound.expect("checked scope size"));
    }
    let count = scopes.len();
    let height = usize::BITS as usize - count.leading_zeros() as usize;
    // Heap comparisons and moves, dedup, canonical validation and SmallVec
    // transfer. Strings move without deep copying during normalization.
    work.reserve((|| {
        count
            .checked_add(count / 2)?
            .checked_mul(height)?
            .checked_mul(comparison.checked_mul(2)?.checked_add(4)?)?
            .checked_add(count.checked_mul(comparison.checked_mul(2)?.checked_add(4)?)?)
    })())?;
    for root in (0..count / 2).rev() {
        sift_down(&mut scopes, root);
    }
    for end in (1..count).rev() {
        scopes.swap(0, end);
        sift_down(&mut scopes[..end], 0);
    }
    scopes.dedup();
    PartitionScopeSet::from_canonical_scopes(scopes)
        .ok_or_else(|| SignalError::internal("scope normalization produced a noncanonical set"))
}

fn sift_down(scopes: &mut [PartitionSubscription], mut root: usize) {
    while root < scopes.len() / 2 {
        let mut child = root * 2 + 1;
        if child + 1 < scopes.len() && scopes[child] < scopes[child + 1] {
            child += 1;
        }
        if scopes[root] >= scopes[child] {
            break;
        }
        scopes.swap(root, child);
        root = child;
    }
}
