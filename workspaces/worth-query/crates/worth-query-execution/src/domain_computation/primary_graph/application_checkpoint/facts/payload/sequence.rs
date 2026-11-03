//! Ordered access without a contiguous-slice or implicit materialization door.
use super::{Fact, FactPayload};

pub(in crate::domain_computation::primary_graph) trait FactSequence {
    fn len(&self) -> usize;
    fn get(&self, ordinal: usize) -> Option<&Fact>;
    fn iter(&self) -> impl DoubleEndedIterator<Item = &Fact> + ExactSizeIterator + Clone;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl FactSequence for FactPayload {
    fn len(&self) -> usize {
        self.len()
    }

    fn get(&self, ordinal: usize) -> Option<&Fact> {
        self.get(ordinal)
    }
    fn iter(&self) -> impl DoubleEndedIterator<Item = &Fact> + ExactSizeIterator + Clone {
        self.iter()
    }
}

macro_rules! slice_sequence {
    ($($ty:ty),* $(,)?) => { $(impl FactSequence for $ty {
        fn len(&self) -> usize { AsRef::<[Fact]>::as_ref(self).len() }
        fn get(&self, ordinal: usize) -> Option<&Fact> { AsRef::<[Fact]>::as_ref(self).get(ordinal) }
        fn iter(&self) -> impl DoubleEndedIterator<Item = &Fact> + ExactSizeIterator + Clone {
            AsRef::<[Fact]>::as_ref(self).iter()
        }
    })* };
}
slice_sequence!([Fact], Vec<Fact>, std::sync::Arc<[Fact]>);
impl<const N: usize> FactSequence for [Fact; N] {
    fn len(&self) -> usize {
        N
    }

    fn get(&self, ordinal: usize) -> Option<&Fact> {
        self.as_slice().get(ordinal)
    }
    fn iter(&self) -> impl DoubleEndedIterator<Item = &Fact> + ExactSizeIterator + Clone {
        self.as_slice().iter()
    }
}
impl FactSequence for worth_execution::ExecutionArray<Fact> {
    fn len(&self) -> usize {
        self.elements().len()
    }

    fn get(&self, ordinal: usize) -> Option<&Fact> {
        self.elements().get(ordinal)
    }
    fn iter(&self) -> impl DoubleEndedIterator<Item = &Fact> + ExactSizeIterator + Clone {
        self.elements().iter()
    }
}
