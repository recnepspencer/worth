//! Canonical memo identity and its actual semantic-comparison work.
use super::{DefinitionDenial, Denial, Evaluation, Handle, Member, SourceRead};
use std::collections::BTreeSet;

#[derive(Clone, Debug)]
pub(super) struct Key {
    pub(super) definition: Handle,
    pub(super) node: usize,
    pub(super) anchor: Member,
    pub(super) reverse: bool,
}
impl Key {
    fn ordering_key(
        &self,
    ) -> (
        &crate::derived_relation::DerivedRelationIdentity,
        usize,
        &Member,
        bool,
    ) {
        (
            &self.definition.parts().identity,
            self.node,
            &self.anchor,
            self.reverse,
        )
    }
}
impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        self.ordering_key() == other.ordering_key()
    }
}
impl Eq for Key {}
impl PartialOrd for Key {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Key {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ordering_key().cmp(&other.ordering_key())
    }
}

impl<Port: SourceRead> Evaluation<'_, Port> {
    pub(super) fn memo_key_work(&mut self, key: &Key) -> Result<(), Denial> {
        let identity = &key.definition.parts().identity;
        let key_bytes = identity
            .schema
            .0
            .len()
            .checked_add(identity.name.len())
            .and_then(|n| n.checked_add(size_of::<Key>()))
            .and_then(|n| n.checked_add(key.anchor.name_bytes()?))
            .ok_or(DefinitionDenial::WorkBudgetExceeded)?;
        let levels = (usize::BITS - self.values.len().max(1).leading_zeros()) as usize;
        let work = key_bytes
            .checked_mul(11)
            .and_then(|n| n.checked_mul(levels))
            .ok_or(DefinitionDenial::WorkBudgetExceeded)?;
        self.read.checkpoint(work)
    }
    pub(super) fn save(&mut self, key: Key, values: BTreeSet<Member>) -> Result<(), Denial> {
        self.memo_key_work(&key)?;
        self.read
            .retain(size_of::<(Key, BTreeSet<Member>)>() + 4 * size_of::<usize>())?;
        self.values.insert(key, values);
        Ok(())
    }
}
