use std::sync::Arc;

use worth_ui_host_contract::{UiQualifiedTextLayoutIdentity, UiQualifiedTextLayoutRequestIdentity};

use super::{UiMountedQualifiedSemanticText, UiMountedSemanticMechanicRows};
use crate::runtime::persistent_index::UiPersistentOrdMap;

/// The layouts retained rows hold, counted by the rows that hold them, by
/// layout identity and by the request that shaped them.
///
/// Font collections that admitted the same profile shape equal requests into
/// equal identities, so one identity or request can be held as several
/// layouts at once. Each entry is keyed by its digest and the exact layout it
/// holds, counts only that layout's holders, and leaves with the last one.
#[derive(Clone, Default)]
pub(super) struct UiMountedQualifiedLayoutIndex {
    by_identity: UiPersistentOrdMap<UiMountedHeldLayoutKey, UiMountedQualifiedLayoutEntry>,
    by_request: UiPersistentOrdMap<UiMountedHeldLayoutKey, UiMountedQualifiedLayoutEntry>,
}

/// A digest and the address of the held layout. The entry keeps that layout
/// alive, so while the key is present its address names exactly one layout.
type UiMountedHeldLayoutKey = ([u8; 32], usize);

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
        retain(&mut self.by_request, request_key(row, layout), layout);
    }

    fn remove_row(&mut self, row: &UiMountedQualifiedSemanticText) {
        let Some(layout) = row.qualified_layout() else {
            return;
        };
        release(&mut self.by_identity, identity_key(row, layout));
        release(&mut self.by_request, request_key(row, layout));
    }

    /// A held layout with `identity`. Every layout held under one identity
    /// has the same qualified output.
    pub(super) fn get(
        &self,
        identity: UiQualifiedTextLayoutIdentity,
    ) -> Option<&Arc<worth_ui_text::UiQualifiedTextLayout>> {
        held(&self.by_identity, identity.digest()).next()
    }

    /// The held layout shaped for `request` against exactly `fonts`. A
    /// layout pinned to another collection instance is never offered, even
    /// when that collection admitted the same profile.
    pub(super) fn for_request(
        &self,
        request: UiQualifiedTextLayoutRequestIdentity,
        fonts: &Arc<worth_ui_text::UiGlobalFontCollection>,
    ) -> Option<&Arc<worth_ui_text::UiQualifiedTextLayout>> {
        held(&self.by_request, request.digest())
            .find(|layout| Arc::ptr_eq(layout.pinned_font_collection(), fonts))
    }

    pub(super) fn retained_structural_bytes(&self) -> Option<usize> {
        std::mem::size_of::<Self>()
            .checked_add(self.by_identity.retained_structural_bytes()?)?
            .checked_add(self.by_request.retained_structural_bytes()?)
    }
}

fn identity_key(
    row: &UiMountedQualifiedSemanticText,
    layout: &Arc<worth_ui_text::UiQualifiedTextLayout>,
) -> UiMountedHeldLayoutKey {
    (row.qualified_layout_identity().digest(), address(layout))
}

fn request_key(
    row: &UiMountedQualifiedSemanticText,
    layout: &Arc<worth_ui_text::UiQualifiedTextLayout>,
) -> UiMountedHeldLayoutKey {
    (
        row.mechanic().qualified_layout_request().digest(),
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

fn retain(
    index: &mut UiPersistentOrdMap<UiMountedHeldLayoutKey, UiMountedQualifiedLayoutEntry>,
    key: UiMountedHeldLayoutKey,
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

fn release(
    index: &mut UiPersistentOrdMap<UiMountedHeldLayoutKey, UiMountedQualifiedLayoutEntry>,
    key: UiMountedHeldLayoutKey,
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
