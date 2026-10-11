//! One aggregate of the comparison widths owned by demand keys.
use super::WorthQueryOutputDemandKey;

pub(super) struct KeyComparisonWork {
    pub(super) maximum: usize,
    pub(super) total: usize,
}

pub(super) fn comparison_work<'a>(
    keys: impl IntoIterator<Item = &'a WorthQueryOutputDemandKey>,
) -> Option<KeyComparisonWork> {
    keys.into_iter().try_fold(
        KeyComparisonWork {
            maximum: 0,
            total: 0,
        },
        |work, key| {
            let width = key.comparison_work()?;
            Some(KeyComparisonWork {
                maximum: work.maximum.max(width),
                total: work.total.checked_add(width)?,
            })
        },
    )
}

impl KeyComparisonWork {
    pub(super) fn sort_bound(&self, count: usize) -> Option<usize> {
        count
            .checked_mul(usize::BITS as usize - count.max(1).leading_zeros() as usize + 1)?
            .checked_mul(self.maximum)
    }
}

#[cfg(test)]
pub(super) mod fixtures;
#[cfg(test)]
mod tests;
