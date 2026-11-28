//! The rule name retains its tuple debug and named newtype wire representation.
use super::*;
use serde::{Deserializer, Serializer};

impl std::fmt::Debug for CustomInvariantRuleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("CustomInvariantRuleId")
            .field(&self.as_str())
            .finish()
    }
}
impl Serialize for CustomInvariantRuleId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_newtype_struct("CustomInvariantRuleId", self.as_str())
    }
}
impl<'de> Deserialize<'de> for CustomInvariantRuleId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Arc::<str>::deserialize(deserializer).map(Self::new)
    }
}
