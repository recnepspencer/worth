use crate::aspects::{AspectKey, CanonicalFieldPath};
use crate::values::InternedString;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CanonicalBasisLocus {
    Root,
    EntryOrdinal(u32),
    Aspect(AspectKey),
    AspectField {
        aspect: AspectKey,
        path: CanonicalFieldPath,
    },
    Named(InternedString),
}

impl CanonicalBasisLocus {
    pub fn owned_allocation_capacity_bytes(&self) -> usize {
        match self {
            Self::Root | Self::EntryOrdinal(_) => 0,
            Self::Aspect(aspect) => aspect.owned_allocation_capacity_bytes(),
            Self::AspectField { aspect, path } => aspect
                .owned_allocation_capacity_bytes()
                .saturating_add(path.owned_allocation_capacity_bytes()),
            Self::Named(name) => name.owned_allocation_capacity_bytes(),
        }
    }
}
