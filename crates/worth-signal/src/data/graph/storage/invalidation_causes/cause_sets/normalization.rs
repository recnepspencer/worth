//! Canonical cause ordering, preserving the first input for duplicate keys.
use crate::data::error::SignalError;
use crate::data::proof::invalidation::binding::ResolvedDependencyCause as Cause;
use crate::logic::evaluation::EvaluationWork;

#[derive(Debug)]
pub(crate) struct NormalizedCauseSet(Vec<Cause>);
impl std::ops::Deref for NormalizedCauseSet {
    type Target = [Cause];
    fn deref(&self) -> &[Cause] {
        &self.0
    }
}
impl NormalizedCauseSet {
    pub(crate) fn prepare(
        causes: Vec<Cause>,
        work: &mut EvaluationWork<'_>,
    ) -> Result<Self, SignalError> {
        work.reserve(Some(causes.len()))?;
        let mut largest = 0usize;
        for cause in &causes {
            if let Some(scope) = &cause.key.edge_scope {
                let bytes = scope
                    .partition
                    .0
                    .len()
                    .checked_add(scope.detail.as_ref().map_or(0, String::len));
                work.reserve(Some(1))?;
                largest = largest
                    .max(bytes.ok_or_else(|| SignalError::internal("cause key size overflow"))?);
            }
        }
        let comparison = largest.checked_mul(2).and_then(|n| n.checked_add(64));
        work.reserve(comparison.and_then(|n| n.checked_mul(causes.len())))?;
        if causes.windows(2).all(|pair| pair[0].key < pair[1].key) {
            return Ok(Self(causes));
        }
        for cause in &causes {
            work.reserve(
                cause
                    .changed_scopes
                    .as_slice()
                    .len()
                    .checked_mul(4)
                    .and_then(|n| n.checked_add(8)),
            )?;
        }
        let len = causes.len();
        let levels = usize::BITS as usize - len.leading_zeros() as usize + 1;
        work.reserve(
            comparison
                .and_then(|n| n.checked_mul(len))
                .and_then(|n| n.checked_mul(levels))
                .and_then(|n| n.checked_mul(8)),
        )?;
        work.reserve(
            len.checked_mul(std::mem::size_of::<(usize, Cause)>() + std::mem::size_of::<Cause>())
                .filter(|n| *n <= isize::MAX as usize),
        )?;
        let mut indexed: Vec<_> = causes.into_iter().enumerate().collect();
        for root in (0..len / 2).rev() {
            sift(&mut indexed, root, len);
        }
        for end in (1..len).rev() {
            indexed.swap(0, end);
            sift(&mut indexed, 0, end);
        }
        indexed.dedup_by(|right, left| right.1.key == left.1.key);
        Ok(Self(indexed.into_iter().map(|(_, cause)| cause).collect()))
    }
    pub(crate) fn into_vec(self) -> Vec<Cause> {
        self.0
    }
}
fn less(left: &(usize, Cause), right: &(usize, Cause)) -> bool {
    left.1
        .key
        .cmp(&right.1.key)
        .then(left.0.cmp(&right.0))
        .is_lt()
}
fn sift(values: &mut [(usize, Cause)], mut root: usize, end: usize) {
    while let Some(left) = root
        .checked_mul(2)
        .and_then(|n| n.checked_add(1))
        .filter(|n| *n < end)
    {
        let child = if left + 1 < end && less(&values[left], &values[left + 1]) {
            left + 1
        } else {
            left
        };
        if !less(&values[root], &values[child]) {
            break;
        }
        values.swap(root, child);
        root = child;
    }
}

#[cfg(test)]
mod tests;
