//! A correction of a recorded stage with coupled stages in the same envelope,
//! outside the document (`fixture::coupled`).
use super::super::fixture::coupled::{correct_with_coupled, Coupled};
use super::*;

impl Document<'_> {
    /// Reverse `entity`'s stage in step `source` under model key `key`, with
    /// `coupled` resolved against the branch's live members after it.
    pub fn correct_coupled(&mut self, key: u32, source: u32, entity: usize, coupled: &Stages) -> Coupled {
        let (receipt, undo) = self
            .sources
            .get(&(source, entity))
            .cloned()
            .unwrap_or_else(|| panic!("k{key}: step {source} has no envelope receipt for entity {entity}"));
        let calls = calls(coupled, read_entities(self.host, self.branch).map(|entity| entity.is_some()));
        correct_with_coupled(self.host, self.shape, self.branch, self.key(key), &receipt, undo, &calls)
    }
}
