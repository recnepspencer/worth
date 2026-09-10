use serde::{Deserialize, Serialize};

mod retained_charge;

use crate::data::core_profile::AspectMaskBits;

/// Maximum number of independently versioned aspect slots supported per node.
pub const MAX_ASPECTS: usize = crate::data::core_profile::MAX_ASPECTS;

/// Caller-defined aspect key for subscription, invalidation, and version lookup.
///
/// `worth-signal` does not assign semantic meaning to aspect slots. Embedding
/// runtimes define what each slot represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Aspect(u8);

impl Aspect {
    /// Create an aspect key for the given slot index.
    ///
    /// Valid indices are `0..MAX_ASPECTS`.
    ///
    /// Prefer defining aspects as named constants near the graph domain model:
    ///
    /// ```rust
    /// use worth_signal::facade::Aspect;
    ///
    /// const PRICE: Aspect = Aspect::new(0);
    /// const TAX: Aspect = Aspect::new(1);
    /// ```
    pub const fn new(index: u8) -> Self {
        assert!(index < MAX_ASPECTS as u8, "aspect index out of range");
        Self(index)
    }

    /// Fallible aspect constructor for dynamically chosen indices.
    pub fn try_new(index: u8) -> Option<Self> {
        (index < MAX_ASPECTS as u8).then_some(Self(index))
    }

    /// Zero-based slot index for this aspect.
    pub const fn index(self) -> usize {
        self.0 as usize
    }

    /// Raw numeric identifier for this aspect.
    pub const fn id(self) -> u8 {
        self.0
    }

    pub(crate) const fn bit(self) -> AspectMaskBits {
        (1 as AspectMaskBits) << self.0
    }
}
