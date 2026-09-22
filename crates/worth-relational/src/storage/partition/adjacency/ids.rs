use super::RelationSet;
use crate::identity::data::RelationId;
use crate::storage::substrate::SharedMapIter;

/// Borrowed ordered membership; constructing this view never expands fanout.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct AdjacencyIds<'a> {
    relations: Option<&'a RelationSet>,
}

impl<'a> AdjacencyIds<'a> {
    pub(super) fn new(relations: Option<&'a RelationSet>) -> Self {
        Self { relations }
    }
    pub(crate) fn iter(self) -> AdjacencyIdsIter<'a> {
        AdjacencyIdsIter {
            entries: self.relations.map(RelationSet::iter),
        }
    }
    #[cfg(test)]
    pub(crate) fn is_empty(self) -> bool {
        self.relations.is_none_or(RelationSet::is_empty)
    }
    pub(crate) fn len(self) -> usize {
        self.relations.map_or(0, RelationSet::len)
    }
    pub(crate) fn to_vec(self) -> Vec<RelationId> {
        self.iter().copied().collect()
    }
}

pub(crate) struct AdjacencyIdsIter<'a> {
    entries: Option<SharedMapIter<'a, RelationId, ()>>,
}
impl<'a> Iterator for AdjacencyIdsIter<'a> {
    type Item = &'a RelationId;
    fn next(&mut self) -> Option<Self::Item> {
        self.entries.as_mut()?.next().map(|(id, _)| id)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.entries.as_ref().map_or(0, ExactSizeIterator::len);
        (len, Some(len))
    }
}
impl ExactSizeIterator for AdjacencyIdsIter<'_> {}
impl<'a> IntoIterator for AdjacencyIds<'a> {
    type Item = &'a RelationId;
    type IntoIter = AdjacencyIdsIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl serde::Serialize for AdjacencyIds<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(self.iter())
    }
}
#[cfg(test)]
impl<const N: usize> PartialEq<[RelationId; N]> for AdjacencyIds<'_> {
    fn eq(&self, other: &[RelationId; N]) -> bool {
        self.iter().eq(other.iter())
    }
}
