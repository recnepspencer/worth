//! Readonly observation of the real connection's computed child demands.
use std::{cell::RefCell, collections::BTreeSet};
thread_local! {
    static DISCOVERED: RefCell<Option<BTreeSet<String>>> = const {RefCell::new(None)};
}
pub(crate) struct Observation;
impl Observation {
    pub(crate) fn begin() -> Self {
        DISCOVERED.with(|keys| assert!(keys.replace(Some(BTreeSet::new())).is_none()));
        Self
    }
    pub(crate) fn saw(&self, key: &str) -> bool {
        DISCOVERED.with(|keys| {
            keys.borrow()
                .as_ref()
                .unwrap()
                .contains(&format!("first:{key}"))
        })
    }
}
impl Drop for Observation {
    fn drop(&mut self) {
        DISCOVERED.with(|keys| {
            keys.replace(None);
        });
    }
}
pub(super) fn record(key: &str) {
    DISCOVERED.with(|keys| {
        if let Some(keys) = &mut *keys.borrow_mut() {
            keys.insert(key.into());
        }
    });
}
