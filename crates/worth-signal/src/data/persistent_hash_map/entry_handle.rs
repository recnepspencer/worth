use std::borrow::Borrow;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct SharedKey<K>(Arc<K>);

impl<K> SharedKey<K> {
    pub(super) fn new(key: K) -> Self {
        Self(Arc::new(key))
    }

    pub(super) fn as_key(&self) -> &K {
        self.0.as_ref()
    }
}

impl<K> Clone for SharedKey<K> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<K> Borrow<K> for SharedKey<K> {
    fn borrow(&self) -> &K {
        self.as_key()
    }
}

impl<K: Hash> Hash for SharedKey<K> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_key().hash(state);
    }
}
