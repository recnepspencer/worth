use serde::{Deserialize, Serialize};

use crate::aspects::{AspectKey, CanonicalFieldPath};
use crate::values::InternedString;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CanonicalBasisLocus {
    Root,
    EntryOrdinal(u32),
    Aspect(AspectKey),
    AspectField {
        aspect: AspectKey,
        #[serde(deserialize_with = "deserialize_nonempty_path")]
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

fn deserialize_nonempty_path<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<CanonicalFieldPath, D::Error> {
    let path = CanonicalFieldPath::deserialize(deserializer)?;
    if path.fields().is_empty() {
        return Err(serde::de::Error::custom(
            "diagnostic field path cannot be empty",
        ));
    }
    Ok(path)
}
