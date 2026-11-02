//! Canonical memo identity and its actual semantic-comparison work.
use super::{Denial, Evaluation, Handle, Member, SourceRead};
use std::collections::BTreeSet;
mod storage;
pub(super) use storage::MemoValues;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum DefinitionScope {
    Root,
    Named,
}

#[derive(Clone, Debug)]
pub(super) struct Key {
    pub(super) definition: Handle,
    pub(super) definition_scope: DefinitionScope,
    pub(super) node: usize,
    pub(super) anchor: Member,
    pub(super) reverse: bool,
}
impl Key {
    /// The caller has admitted this exact root before opening its local map.
    pub(super) fn root(definition: Handle, node: usize, anchor: Member, reverse: bool) -> Self {
        Self {
            definition,
            definition_scope: DefinitionScope::Root,
            node,
            anchor,
            reverse,
        }
    }
    fn comparison_bytes(&self) -> Option<usize> {
        // Quote compared fields, not the retained Handle's storage representation.
        let mut bytes = size_of::<DefinitionScope>()
            .checked_add(size_of::<usize>())?
            .checked_add(self.anchor_comparison_bytes()?)?
            .checked_add(size_of::<bool>())?;
        if self.definition_scope == DefinitionScope::Named {
            let identity = &self.definition.parts().identity;
            bytes = bytes
                .checked_add(size_of_val(&identity.version))?
                .checked_add(identity.schema.0.len())?
                .checked_add(identity.name.len())?;
        }
        Some(bytes)
    }
    fn anchor_comparison_bytes(&self) -> Option<usize> {
        Self::anchor_bytes(&self.anchor)
    }
    fn anchor_bytes(anchor: &Member) -> Option<usize> {
        let bytes = size_of::<Member>().checked_add(anchor.name_bytes()?)?;
        if let Member::Projected(_) = anchor {
            bytes.checked_add(size_of::<crate::graph_member::ProjectedMemberCoordinate>())
        } else {
            Some(bytes)
        }
    }
    fn ordering_key(
        &self,
    ) -> (
        Option<&crate::derived_relation::DerivedRelationIdentity>,
        usize,
        &Member,
        bool,
    ) {
        (
            match self.definition_scope {
                DefinitionScope::Root => None,
                DefinitionScope::Named => Some(&self.definition.parts().identity),
            },
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
    /// Only actual same-registry navigation changes the retained definition.
    pub(super) fn target_key(&mut self, parent: &Key, target: Handle) -> Result<Key, Denial> {
        self.read.checkpoint(1)?;
        let definition_scope = if std::sync::Arc::ptr_eq(&target.entry, &self.root_definition.entry)
        {
            DefinitionScope::Root
        } else {
            DefinitionScope::Named
        };
        let node = target.parts().nodes.len() - 1;
        self.read.checkpoint(0)?;
        Ok(Key {
            definition: target,
            definition_scope,
            node,
            ..parent.clone()
        })
    }
    pub(super) fn save(&mut self, key: Key, values: BTreeSet<Member>) -> Result<(), Denial> {
        self.values.insert(key, values, self.read).map(|_| ())
    }
}

#[cfg(test)]
mod tests;
