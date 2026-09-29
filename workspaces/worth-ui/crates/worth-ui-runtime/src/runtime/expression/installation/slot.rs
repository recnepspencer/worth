use super::UiExpressionCatalogPreparationDenial;

const MAXIMUM_EXPRESSIONS: usize = 65_536;

/// A dense expression index. The slot number is the expression's rank in the
/// topological order, so every operand expression has a lower slot than its
/// readers.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UiExpressionSlot(u32);

impl UiExpressionSlot {
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// How many expressions one catalog installs. It is the only source of
/// slots, and it exists only for a count that fits `MAXIMUM_EXPRESSIONS`, so
/// no slot number can be out of range of `u32`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiExpressionSlotCount(u32);

impl UiExpressionSlotCount {
    pub(super) fn of(expressions: usize) -> Result<Self, UiExpressionCatalogPreparationDenial> {
        u32::try_from(expressions)
            .ok()
            .filter(|_| expressions <= MAXIMUM_EXPRESSIONS)
            .map(Self)
            .ok_or(UiExpressionCatalogPreparationDenial::CapacityExceeded {
                observed: expressions,
                maximum: MAXIMUM_EXPRESSIONS,
            })
    }

    /// Every slot of the catalog, in rank order.
    pub(crate) fn slots(self) -> impl Iterator<Item = UiExpressionSlot> {
        (0..self.0).map(UiExpressionSlot)
    }
}
