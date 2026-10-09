use super::ordered::Run;
use std::cmp::Ordering;

/// Stack cursor per binary level, with no allocated lookup/result collection.
/// Full ordering across runs preserves native canonical iteration semantics.
pub(crate) struct OrderedIter<'a, T> {
    runs: &'a [Option<Run<T>>],
    positions: [usize; usize::BITS as usize],
    ends: [usize; usize::BITS as usize],
    remaining: usize,
}
impl<'a, T: Ord> OrderedIter<'a, T> {
    pub(super) fn new(runs: &'a [Option<Run<T>>], compare: impl Fn(&T) -> Ordering) -> Self {
        let mut out = Self {
            runs,
            positions: [0; usize::BITS as usize],
            ends: [0; usize::BITS as usize],
            remaining: 0,
        };
        for (index, run) in runs.iter().enumerate() {
            let Some(run) = run else { continue };
            let start = run.partition_point(|row| compare(row.value()) == Ordering::Less);
            let end = run.partition_point(|row| compare(row.value()) != Ordering::Greater);
            out.positions[index] = start;
            out.ends[index] = end;
            out.remaining += end - start;
        }
        out
    }
}
impl<'a, T: Ord> Iterator for OrderedIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        let mut selected: Option<usize> = None;
        for (index, run) in self.runs.iter().enumerate() {
            if self.positions[index] == self.ends[index] {
                continue;
            }
            let row = &run.as_ref().expect("nonempty level")[self.positions[index]];
            if selected.is_none_or(|prior| {
                row < &self.runs[prior].as_ref().expect("selected level")[self.positions[prior]]
            }) {
                selected = Some(index);
            }
        }
        let index = selected?;
        let row = self.runs[index].as_ref().expect("selected level")[self.positions[index]].value();
        self.positions[index] += 1;
        self.remaining -= 1;
        Some(row)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}
impl<T: Ord> ExactSizeIterator for OrderedIter<'_, T> {}
