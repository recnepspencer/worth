use std::sync::Arc;

use worth_ui_host_contract::UiQualifiedTextLayoutIdentity;

use super::{UiMountedQualifiedSemanticText, UiMountedSemanticMechanicRows};
use crate::runtime::persistent_index::UiPersistentOrdMap;

/// The layouts retained rows hold, counted by the rows that hold them, by
/// layout identity and by the reflow key of the request that shaped them.
///
/// Font collections that admitted the same profile shape equal requests into
/// equal identities, and one request fits different lines at different
/// widths, so one identity or reflow key can be held as several layouts at
/// once. Each entry is keyed by its digest and the exact layout it
/// holds, counts only that layout's holders, and leaves with the last one.
#[derive(Clone, Default)]
pub(super) struct UiMountedQualifiedLayoutIndex {
    by_identity: UiPersistentOrdMap<UiMountedHeldLayoutKey, UiMountedQualifiedLayoutEntry>,
    by_reflow: UiPersistentOrdMap<UiMountedReflowLayoutKey, UiMountedQualifiedLayoutEntry>,
}

/// A digest and the address of the held layout. The entry keeps that layout
/// alive, so while the key is present its address names exactly one layout.
type UiMountedHeldLayoutKey = ([u8; 32], usize);

/// A reflow key's digest, the least width the held layout stands for, and
/// its address. The layouts of one request against one collection admit
/// disjoint widths, so ordering by least width puts the only one that can
/// admit a width nearest at or below it.
type UiMountedReflowLayoutKey = ([u8; 32], u32, usize);

#[derive(Clone)]
struct UiMountedQualifiedLayoutEntry {
    layout: Arc<worth_ui_text::UiQualifiedTextLayout>,
    owners: usize,
}

impl UiMountedQualifiedLayoutIndex {
    pub(super) fn len(&self) -> usize {
        self.by_identity.len()
    }

    pub(super) fn insert_rows(&mut self, rows: &UiMountedSemanticMechanicRows) {
        for row in rows.iter() {
            self.insert_row(row);
        }
    }

    pub(super) fn remove_rows(&mut self, rows: &UiMountedSemanticMechanicRows) {
        for row in rows.iter() {
            self.remove_row(row);
        }
    }

    pub(super) fn replace_row(
        &mut self,
        predecessor: &UiMountedQualifiedSemanticText,
        successor: &UiMountedQualifiedSemanticText,
    ) {
        self.remove_row(predecessor);
        self.insert_row(successor);
    }

    fn insert_row(&mut self, row: &UiMountedQualifiedSemanticText) {
        let Some(layout) = row.qualified_layout() else {
            return;
        };
        retain(&mut self.by_identity, identity_key(row, layout), layout);
        retain(&mut self.by_reflow, reflow_key(layout), layout);
    }

    fn remove_row(&mut self, row: &UiMountedQualifiedSemanticText) {
        let Some(layout) = row.qualified_layout() else {
            return;
        };
        release(&mut self.by_identity, identity_key(row, layout));
        release(&mut self.by_reflow, reflow_key(layout));
    }

    /// A held layout with `identity`. Every layout held under one identity
    /// has the same qualified output.
    pub(super) fn get(
        &self,
        identity: UiQualifiedTextLayoutIdentity,
    ) -> Option<&Arc<worth_ui_text::UiQualifiedTextLayout>> {
        held(&self.by_identity, identity.digest()).next()
    }

    /// A held layout the request with `reflow` shapes at `width_millipoints`
    /// against exactly `fonts`. A layout pinned to another collection
    /// instance is never offered, even when that collection admitted the
    /// same profile. Walking down from the width passes only layouts of
    /// other collections before reaching the one candidate of `fonts`.
    pub(super) fn for_request(
        &self,
        reflow: worth_ui_text::UiQualifiedTextReflowKey,
        width_millipoints: u32,
        fonts: &Arc<worth_ui_text::UiGlobalFontCollection>,
    ) -> Option<&Arc<worth_ui_text::UiQualifiedTextLayout>> {
        let digest = reflow.digest();
        std::iter::successors(
            self.by_reflow
                .predecessor(&(digest, width_millipoints, usize::MAX)),
            |(key, _)| self.by_reflow.predecessor(key),
        )
        .take_while(|(key, _)| key.0 == digest)
        .map(|(_, entry)| &entry.layout)
        .find(|layout| Arc::ptr_eq(layout.pinned_font_collection(), fonts))
        .filter(|layout| layout.admits_width(width_millipoints))
    }

    pub(super) fn retained_structural_bytes(&self) -> Option<usize> {
        std::mem::size_of::<Self>()
            .checked_add(self.by_identity.retained_structural_bytes()?)?
            .checked_add(self.by_reflow.retained_structural_bytes()?)
    }
}

fn identity_key(
    row: &UiMountedQualifiedSemanticText,
    layout: &Arc<worth_ui_text::UiQualifiedTextLayout>,
) -> UiMountedHeldLayoutKey {
    (row.qualified_layout_identity().digest(), address(layout))
}

fn reflow_key(layout: &Arc<worth_ui_text::UiQualifiedTextLayout>) -> UiMountedReflowLayoutKey {
    (
        layout.reflow_key().digest(),
        layout.least_width_millipoints(),
        address(layout),
    )
}

fn address(layout: &Arc<worth_ui_text::UiQualifiedTextLayout>) -> usize {
    Arc::as_ptr(layout) as usize
}

/// The layouts held under `digest`, in address order. No layout lives at
/// address zero, so every held key follows `(digest, 0)`.
fn held(
    index: &UiPersistentOrdMap<UiMountedHeldLayoutKey, UiMountedQualifiedLayoutEntry>,
    digest: [u8; 32],
) -> impl Iterator<Item = &Arc<worth_ui_text::UiQualifiedTextLayout>> {
    std::iter::successors(index.successor(&(digest, 0)), |(key, _)| {
        index.successor(key)
    })
    .take_while(move |(key, _)| key.0 == digest)
    .map(|(_, entry)| &entry.layout)
}

fn retain<K: Ord + Clone>(
    index: &mut UiPersistentOrdMap<K, UiMountedQualifiedLayoutEntry>,
    key: K,
    layout: &Arc<worth_ui_text::UiQualifiedTextLayout>,
) {
    let entry = index.get(&key).cloned().map_or_else(
        || UiMountedQualifiedLayoutEntry {
            layout: Arc::clone(layout),
            owners: 1,
        },
        |mut entry| {
            entry.owners = entry.owners.saturating_add(1);
            entry
        },
    );
    index.insert(key, entry);
}

fn release<K: Ord + Clone>(
    index: &mut UiPersistentOrdMap<K, UiMountedQualifiedLayoutEntry>,
    key: K,
) {
    let Some(mut entry) = index.get(&key).cloned() else {
        return;
    };
    if entry.owners == 1 {
        index.remove(&key);
    } else {
        entry.owners -= 1;
        index.insert(key, entry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mounting::projection::semantic_text::request_for_test;
    use worth_ui_text::{UiGlobalFontCollection, UiQualifiedTextLayout};

    /// Two collections that admitted one profile hold layouts of one request
    /// at interleaved least widths. A width finds only the layout of its own
    /// collection that admits it: a nearer layout of the other collection is
    /// passed over, and a width between its collection's layouts misses.
    #[test]
    fn a_width_finds_only_its_own_collections_layout_that_admits_it() {
        let admit = || Arc::new(UiGlobalFontCollection::admit_qualified_profile().unwrap().0);
        let (ours, theirs) = (admit(), admit());
        let our_narrow = qualified(40_000, &ours);
        let our_wide = qualified(400_000, &ours);
        let between = (40_000..400_000)
            .step_by(1_000)
            .find(|width| !our_narrow.admits_width(*width) && !our_wide.admits_width(*width))
            .unwrap();
        let their_between = qualified(between, &theirs);
        let their_narrow = qualified(40_000, &theirs);
        let reflow = our_narrow.reflow_key();
        assert!(reflow == their_between.reflow_key());
        assert!(their_between.least_width_millipoints() > our_narrow.least_width_millipoints());

        let mut index = UiMountedQualifiedLayoutIndex::default();
        for layout in [&our_narrow, &our_wide, &their_between, &their_narrow] {
            retain(&mut index.by_reflow, reflow_key(layout), layout);
        }
        let found = |width: u32, fonts: &Arc<UiGlobalFontCollection>| {
            index.for_request(reflow, width, fonts).map(Arc::clone)
        };
        let is = |found: Option<Arc<UiQualifiedTextLayout>>, layout| {
            found.is_some_and(|found| Arc::ptr_eq(&found, layout))
        };

        assert!(found(between, &ours).is_none());
        assert!(is(found(between, &theirs), &their_between));
        assert!(is(found(40_000, &ours), &our_narrow));
        assert!(is(found(40_000, &theirs), &their_narrow));
        assert!(is(found(600_000, &ours), &our_wide));
        assert!(found(600_000, &theirs).is_none());
    }

    fn qualified(
        width_millipoints: u32,
        fonts: &Arc<UiGlobalFontCollection>,
    ) -> Arc<UiQualifiedTextLayout> {
        let request = request_for_test(
            "office hours and more",
            width_millipoints,
            8,
            Arc::clone(fonts),
        );
        Arc::new(request.qualify().unwrap())
    }
}
