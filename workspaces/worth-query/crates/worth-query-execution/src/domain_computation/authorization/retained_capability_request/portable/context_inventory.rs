//! Count the existing request context sequence using the same serde parser.
use serde::de::{Deserialize, Deserializer, IgnoredAny, SeqAccess, Visitor};
pub(super) struct ContextCount(pub(super) usize);
impl<'de> Deserialize<'de> for ContextCount {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Count;
        impl<'de> Visitor<'de> for Count {
            type Value = ContextCount;
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("the native request context sequence")
            }
            fn visit_seq<S: SeqAccess<'de>>(
                self,
                mut sequence: S,
            ) -> Result<Self::Value, S::Error> {
                let mut count = 0usize;
                while sequence.next_element::<IgnoredAny>()?.is_some() {
                    count = count
                        .checked_add(1)
                        .ok_or_else(|| serde::de::Error::custom("context count exceeds host"))?;
                }
                Ok(ContextCount(count))
            }
        }
        deserializer.deserialize_seq(Count)
    }
}
