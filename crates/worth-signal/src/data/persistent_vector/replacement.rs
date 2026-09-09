//! Replace a selected payload without copying the discarded value.
use super::{install_changed_page, PersistentVector, PersistentVectorStorage};
use std::sync::Arc;
impl<T: Clone, const PAGE_LEN: usize> PersistentVector<T, PAGE_LEN> {
    pub(crate) fn replace_discard(&mut self, index: usize, value: T) {
        assert!(
            index < self.len(),
            "persistent vector replacement index out of bounds"
        );
        self.retained_charge = None;
        match &mut self.storage {
            PersistentVectorStorage::Exclusive(values) => values[index] = value,
            PersistentVectorStorage::ForkShared {
                base,
                changed_pages,
                ..
            } => {
                let page_index = index / PAGE_LEN;
                install_changed_page::<T, PAGE_LEN>(base, changed_pages, page_index);
                Arc::make_mut(
                    changed_pages
                        .get_mut(&page_index)
                        .expect("changed page must be installed"),
                )
                .replace(value, index % PAGE_LEN);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Debug)]
    struct Uncloneable(u32);
    impl Clone for Uncloneable {
        fn clone(&self) -> Self {
            panic!("replacement must not clone payload")
        }
    }
    #[test]
    fn replaces_base_and_appended_payloads_without_clone() {
        let mut source = PersistentVector::<Uncloneable>::new();
        source.push_back(Uncloneable(1));
        let mut fork = source.fork_persistent();
        fork.push_back(Uncloneable(2));
        let sibling = fork.fork_persistent();
        fork.replace_discard(0, Uncloneable(3));
        fork.replace_discard(1, Uncloneable(4));
        fork.replace_discard(0, Uncloneable(5));
        assert_eq!(source.get(0).unwrap().0, 1);
        assert_eq!(sibling.get(0).unwrap().0, 1);
        assert_eq!(sibling.get(1).unwrap().0, 2);
        assert_eq!(fork.get(0).unwrap().0, 5);
        assert_eq!(fork.get(1).unwrap().0, 4);
        assert_eq!(fork.len(), 2);
    }
}
