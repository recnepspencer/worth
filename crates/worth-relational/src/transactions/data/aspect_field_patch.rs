use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, AspectValue, CanonicalFieldPath, FieldKey, LocatorAuthority,
};

use crate::aspect_wire::{
    decode_aspect_field_patch_canonical_bytes, encode_aspect_field_patch_canonical_bytes,
};

pub use crate::aspect_wire::AspectFieldPatchCodecError;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AspectFieldPatch {
    locators: Box<[(AspectFieldLocator, AspectValue)]>,
}

pub fn planned_aspect_field_locator(
    aspect_key: AspectKey,
    field_path: CanonicalFieldPath,
) -> AspectFieldLocator {
    AspectFieldLocator::new(LocatorAuthority::Planned, aspect_key, field_path)
}

pub fn planned_single_field_locator(aspect_key: AspectKey, field: FieldKey) -> AspectFieldLocator {
    planned_aspect_field_locator(aspect_key, CanonicalFieldPath::single(field))
}

pub(crate) fn validate_planned_aspect_field_locator(
    locator: AspectFieldLocator,
) -> Result<AspectFieldLocator, AspectFieldPatchCodecError> {
    if locator.aspect().authority() != LocatorAuthority::Planned {
        return Err(AspectFieldPatchCodecError::new(
            "aspect field patch locator must use planned authority",
        ));
    }
    Ok(locator)
}

impl AspectFieldPatch {
    pub fn new(locators: BTreeMap<AspectFieldLocator, AspectValue>) -> Self {
        Self {
            locators: locators.into_iter().collect::<Vec<_>>().into_boxed_slice(),
        }
    }

    pub(crate) fn from_sorted_owned_entries(
        locators: Vec<(AspectFieldLocator, AspectValue)>,
    ) -> Self {
        debug_assert!(locators.windows(2).all(|pair| pair[0].0 < pair[1].0));
        Self {
            locators: locators.into_boxed_slice(),
        }
    }

    pub fn from_locator(locator: AspectFieldLocator, value: AspectValue) -> Self {
        Self {
            locators: Box::new([(locator, value)]),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.locators.is_empty()
    }

    pub fn len(&self) -> usize {
        self.locators.len()
    }

    pub fn locators(&self) -> impl Iterator<Item = &AspectFieldLocator> {
        self.locators.iter().map(|(locator, _)| locator)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&AspectFieldLocator, &AspectValue)> {
        self.locators
            .iter()
            .map(|(locator, value)| (locator, value))
    }

    pub fn get(&self, locator: &AspectFieldLocator) -> Option<&AspectValue> {
        self.locators
            .binary_search_by_key(&locator, |(candidate, _)| candidate)
            .ok()
            .map(|index| &self.locators[index].1)
    }

    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        self.locators.iter().fold(
            std::mem::size_of_val(self.locators.as_ref()) as u64,
            |bytes, (locator, value)| {
                bytes
                    .saturating_add(locator.owned_allocation_capacity_bytes() as u64)
                    .saturating_add(value.owned_allocation_capacity_bytes() as u64)
            },
        )
    }

    pub fn to_canonical_bytes(&self) -> Result<Vec<u8>, AspectFieldPatchCodecError> {
        encode_aspect_field_patch_canonical_bytes(self)
    }

    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, AspectFieldPatchCodecError> {
        decode_aspect_field_patch_canonical_bytes(bytes)
    }
}

impl From<BTreeMap<AspectFieldLocator, AspectValue>> for AspectFieldPatch {
    fn from(locators: BTreeMap<AspectFieldLocator, AspectValue>) -> Self {
        Self::new(locators)
    }
}

impl Serialize for AspectFieldPatch {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes = self
            .to_canonical_bytes()
            .map_err(serde::ser::Error::custom)?;
        serializer.serialize_bytes(&bytes)
    }
}

impl<'de> Deserialize<'de> for AspectFieldPatch {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes = crate::aspect_wire::read_canonical_bytes(deserializer)?;
        Self::from_canonical_bytes(&bytes).map_err(serde::de::Error::custom)
    }
}
