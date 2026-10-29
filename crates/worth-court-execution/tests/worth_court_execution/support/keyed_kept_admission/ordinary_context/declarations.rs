// A retained subset pays before cloning and uses a stable two-pass predicate.
pub(crate) use {std::cell::Cell, std::cmp::Ordering, std::rc::Rc};
pub(crate) use {worth_execution::ChargedBytes, worth_execution::KeyedEditDenial, worth_execution::KeyedItem, worth_execution::KeyedPartitioner, worth_execution::PartitionItemId};
pub(crate) use {worth_foundational::PartitionIdentity};

pub(crate) struct Key(pub(crate) u8, pub(crate) Rc<Cell<usize>>, pub(crate) Vec<u8>);
impl Clone for Key {
    fn clone(&self) -> Self {
        self.1.set(self.1.get() + 1);
        Self(self.0, std::rc::Rc::clone(&self.1), self.2.clone())
    }
}
impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl Eq for Key {}
impl PartialOrd for Key {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Key {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}
impl ChargedBytes for Key {
    fn additional_charged_bytes(&self) -> u64 {
        self.2.len() as u64
    }
}