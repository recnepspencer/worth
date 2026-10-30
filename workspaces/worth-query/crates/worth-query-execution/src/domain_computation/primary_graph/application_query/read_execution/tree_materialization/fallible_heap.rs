//! One in-place heap ordering; callers own comparisons, swaps and their controls.
use std::cmp::Ordering;
pub(in crate::domain_computation::primary_graph::application_query) trait Heap {
    type Error;
    fn len(&self) -> usize;
    fn compare(&mut self, left: usize, right: usize) -> Result<Ordering, Self::Error>;
    fn swap(&mut self, left: usize, right: usize) -> Result<(), Self::Error>;
}
pub(in crate::domain_computation::primary_graph::application_query) fn sort<H: Heap>(
    heap: &mut H,
) -> Result<(), H::Error> {
    let len = heap.len();
    for root in (0..len / 2).rev() {
        sift(heap, root, len)?;
    }
    for end in (1..len).rev() {
        heap.swap(0, end)?;
        sift(heap, 0, end)?;
    }
    Ok(())
}
fn sift<H: Heap>(heap: &mut H, mut root: usize, end: usize) -> Result<(), H::Error> {
    loop {
        let left = root.saturating_mul(2).saturating_add(1);
        if left >= end {
            return Ok(());
        }
        let right = left + 1;
        let greater = if right < end && heap.compare(left, right)? == Ordering::Less {
            right
        } else {
            left
        };
        if heap.compare(root, greater)? != Ordering::Less {
            return Ok(());
        }
        heap.swap(root, greater)?;
        root = greater;
    }
}
pub(in crate::domain_computation::primary_graph::application_query) fn sort_by<T, E>(
    values: &mut [T],
    compare: impl FnMut(&T, &T) -> Result<Ordering, E>,
) -> Result<(), E> {
    struct Slice<'a, T, F> {
        values: &'a mut [T],
        compare: F,
    }
    impl<T, E, F: FnMut(&T, &T) -> Result<Ordering, E>> Heap for Slice<'_, T, F> {
        type Error = E;
        fn len(&self) -> usize {
            self.values.len()
        }
        fn compare(&mut self, l: usize, r: usize) -> Result<Ordering, E> {
            (self.compare)(&self.values[l], &self.values[r])
        }
        fn swap(&mut self, l: usize, r: usize) -> Result<(), E> {
            self.values.swap(l, r);
            Ok(())
        }
    }
    sort(&mut Slice { values, compare })
}
