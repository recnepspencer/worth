//! The caller drains every child outside Native and keeps Box backing funded.
use super::*;

impl Drop for PredicatePreparationFactory {
    fn drop(&mut self) {
        let mut children = self
            .children
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        while let Some(mut link) = children {
            children = link.next.take();
            let funding = link.allocation.clone();
            drop(link);
            drop(funding);
        }
    }
}
