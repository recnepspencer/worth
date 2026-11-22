//! Count boxed lineage nodes using the current serde enum, without a lineage heap.
use serde::{de::IgnoredAny, Deserialize, Deserializer};

pub(super) struct LineageCount(pub(super) usize);

impl<'de> Deserialize<'de> for LineageCount {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        enum Lineage {
            Unbound,
            Root {
                #[serde(rename = "status_revision")]
                _revision: IgnoredAny,
            },
            Delegated {
                #[serde(rename = "grantor")]
                _grantor: IgnoredAny,
                #[serde(rename = "parent_grant")]
                _parent_grant: IgnoredAny,
                #[serde(rename = "status_revision")]
                _revision: IgnoredAny,
                parent: LineageCount,
            },
        }
        Ok(Self(match Lineage::deserialize(deserializer)? {
            Lineage::Delegated { parent, .. } => 1 + parent.0,
            _ => 0,
        }))
    }
}
