use std::collections::BTreeMap;

pub(crate) struct UiPortalStackOrderIndex {
    by_ordinal: BTreeMap<super::super::UiPortalStackOrdinal, super::super::UiPortalIdentity>,
}

impl UiPortalStackOrderIndex {
    pub(super) const fn new() -> Self {
        Self {
            by_ordinal: BTreeMap::new(),
        }
    }

    pub(crate) fn can_insert(
        &self,
        ordinal: super::super::UiPortalStackOrdinal,
        portal: super::super::UiPortalIdentity,
    ) -> bool {
        self.by_ordinal
            .get(&ordinal)
            .is_none_or(|current| *current == portal)
    }

    pub(crate) fn insert(
        &mut self,
        ordinal: super::super::UiPortalStackOrdinal,
        portal: super::super::UiPortalIdentity,
    ) {
        assert!(
            self.can_insert(ordinal, portal),
            "Portal stack ordinals are unique to one live portal"
        );
        self.by_ordinal.insert(ordinal, portal);
    }

    pub(crate) fn remove(
        &mut self,
        ordinal: super::super::UiPortalStackOrdinal,
        portal: super::super::UiPortalIdentity,
    ) {
        assert_eq!(
            self.by_ordinal.get(&ordinal),
            Some(&portal),
            "Portal stack removal must name the exact indexed row"
        );
        self.by_ordinal.remove(&ordinal);
    }

    pub(crate) fn iter(
        &self,
    ) -> impl DoubleEndedIterator<
        Item = (
            &super::super::UiPortalStackOrdinal,
            &super::super::UiPortalIdentity,
        ),
    > {
        self.by_ordinal.iter()
    }

    pub(crate) fn topmost(&self) -> Option<super::super::UiPortalIdentity> {
        self.by_ordinal.values().next_back().copied()
    }

    pub(crate) fn clear(&mut self) {
        self.by_ordinal.clear();
    }

    pub(crate) fn len_for_snapshot(&self) -> usize {
        self.by_ordinal.len()
    }

    #[cfg(test)]
    pub(super) fn rebuild(
        records: &BTreeMap<super::super::UiPortalIdentity, super::UiPortalRecord>,
    ) -> Self {
        let mut index = Self::new();
        for (portal, record) in records {
            index.insert(record.stack_ordinal, *portal);
        }
        index
    }
}
