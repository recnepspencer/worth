use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::runtime::selection) struct UiSelectionOwnerRecord {
    pub(in crate::runtime::selection) revision: u64,
    pub(in crate::runtime::selection) incarnation: super::super::UiSelectionOwnerIncarnation,
    pub(in crate::runtime::selection) policy: super::super::UiSelectionPolicy,
    pub(in crate::runtime::selection) catalog: std::sync::Arc<[super::super::UiSelectionStableKey]>,
    pub(in crate::runtime::selection) catalog_positions:
        std::sync::Arc<BTreeMap<super::super::UiSelectionStableKey, usize>>,
    pub(in crate::runtime::selection) catalog_posture: super::super::UiSelectionCatalogPosture,
    pub(in crate::runtime::selection) catalog_revision: u64,
    pub(in crate::runtime::selection) catalog_available: bool,
    pub(in crate::runtime::selection) selected:
        crate::runtime::persistent_index::UiPersistentOrdSet<super::super::UiSelectionStableKey>,
    pub(in crate::runtime::selection) anchor: Option<super::super::UiSelectionStableKey>,
    pub(in crate::runtime::selection) cursor: Option<super::super::UiSelectionStableKey>,
}

impl UiSelectionOwnerRecord {
    pub(in crate::runtime::selection) const fn positions(
        &self,
    ) -> super::super::UiSelectionPositions {
        super::super::UiSelectionPositions {
            anchor: self.anchor,
            cursor: self.cursor,
        }
    }
}

pub(super) fn empty_record(
    registration: &super::super::UiSelectionRegistration,
) -> UiSelectionOwnerRecord {
    UiSelectionOwnerRecord {
        revision: 0,
        incarnation: registration.incarnation(),
        policy: registration.policy(),
        catalog: std::sync::Arc::from([]),
        catalog_positions: std::sync::Arc::new(BTreeMap::new()),
        catalog_posture: registration.catalog_posture(),
        catalog_revision: registration.catalog_revision(),
        catalog_available: true,
        selected: Default::default(),
        anchor: None,
        cursor: None,
    }
}

pub(in crate::runtime::selection) fn validate_catalog(
    owner: super::super::UiSelectionOwnerIdentity,
    catalog: &[super::super::UiSelectionStableKey],
) -> Result<
    BTreeMap<super::super::UiSelectionStableKey, usize>,
    super::super::UiSelectionRequestDenial,
> {
    if catalog.len() > super::super::model::UI_SELECTION_CATALOG_LIMIT {
        return Err(super::super::UiSelectionRequestDenial::CatalogCapacityExceeded);
    }
    let mut positions = BTreeMap::new();
    for (index, key) in catalog.iter().copied().enumerate() {
        if key.family() != owner.item_key_family() {
            return Err(super::super::UiSelectionRequestDenial::ForeignItemKeyFamily);
        }
        if positions.insert(key, index).is_some() {
            return Err(super::super::UiSelectionRequestDenial::DuplicateCatalogKey);
        }
    }
    Ok(positions)
}
