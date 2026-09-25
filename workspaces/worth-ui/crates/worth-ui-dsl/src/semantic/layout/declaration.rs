use crate::UiDslComponentReference;

use super::UiLayoutGrid;

/// An authored interval of logical viewport widths, `[min, max)`, or
/// unbounded above. Whether it is empty or overlaps another is judged where
/// the layout lowers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiLayoutWidthInterval {
    min: u16,
    max: Option<u16>,
}

impl UiLayoutWidthInterval {
    pub const fn at_least(min: u16) -> Self {
        Self { min, max: None }
    }

    pub const fn between(min: u16, max: u16) -> Self {
        Self {
            min,
            max: Some(max),
        }
    }

    pub const fn min(self) -> u16 {
        self.min
    }

    pub const fn max(self) -> Option<u16> {
        self.max
    }
}

/// A container component's authored layout: the fallback grid, and the grid
/// each viewport-width interval selects instead.
///
/// It restates the layout the container registered in the Mosaic capability
/// vocabulary. The runtime lowers it through that vocabulary's constructors
/// and admits it only where the two are one layout.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiLayoutDeclaration {
    container: UiDslComponentReference,
    fallback: UiLayoutGrid,
    variants: Vec<(UiLayoutWidthInterval, UiLayoutGrid)>,
}

impl UiLayoutDeclaration {
    pub fn new(container: UiDslComponentReference, fallback: UiLayoutGrid) -> Self {
        Self {
            container,
            fallback,
            variants: Vec::new(),
        }
    }

    pub fn with_variant(mut self, interval: UiLayoutWidthInterval, grid: UiLayoutGrid) -> Self {
        self.variants.push((interval, grid));
        self
    }

    pub fn container(&self) -> &UiDslComponentReference {
        &self.container
    }

    pub fn fallback(&self) -> &UiLayoutGrid {
        &self.fallback
    }

    /// The variants in the order they were authored.
    pub fn variants(&self) -> impl Iterator<Item = (UiLayoutWidthInterval, &UiLayoutGrid)> {
        self.variants
            .iter()
            .map(|(interval, grid)| (*interval, grid))
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = b"worth-ui:layout:v1".to_vec();
        text(&mut bytes, self.container.as_str());
        self.fallback.encode(&mut bytes);
        count(&mut bytes, self.variants.len());
        for (interval, grid) in &self.variants {
            bytes.extend_from_slice(&interval.min.to_le_bytes());
            optional(&mut bytes, interval.max);
            grid.encode(&mut bytes);
        }
        bytes
    }
}

pub(super) fn count(bytes: &mut Vec<u8>, count: usize) {
    bytes.extend_from_slice(&(count as u64).to_le_bytes());
}

pub(super) fn text(bytes: &mut Vec<u8>, value: &str) {
    count(bytes, value.len());
    bytes.extend_from_slice(value.as_bytes());
}

pub(super) fn optional(bytes: &mut Vec<u8>, value: Option<u16>) {
    match value {
        None => bytes.push(0),
        Some(value) => {
            bytes.push(1);
            bytes.extend_from_slice(&value.to_le_bytes());
        }
    }
}
