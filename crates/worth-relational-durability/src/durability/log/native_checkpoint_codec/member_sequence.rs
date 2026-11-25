//! Current checkpoints carry neither inline members nor inline envelopes.
use super::checkpoint_wire::ByteString;
use serde::Deserializer;
use worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableMemberLocator as Locator;

pub(super) fn empty_members<'de: 'a, 'a, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<(Locator, ByteString<'a>)>, D::Error> {
    struct Empty;
    impl<'de> serde::de::Visitor<'de> for Empty {
        type Value = Vec<(Locator, ByteString<'de>)>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("the empty current-checkpoint inline member field")
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut input: A,
        ) -> Result<Self::Value, A::Error> {
            if input.size_hint() != Some(0)
                || input.next_element::<serde::de::IgnoredAny>()?.is_some()
            {
                return Err(serde::de::Error::custom(
                    "embedded checkpoint placement is retired; members belong only to external cold storage",
                ));
            }
            Ok(Vec::new())
        }
    }
    deserializer.deserialize_seq(Empty)
}

pub(super) fn empty_envelopes<'de: 'a, 'a, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<ByteString<'a>>, D::Error> {
    struct Empty;
    impl<'de> serde::de::Visitor<'de> for Empty {
        type Value = Vec<ByteString<'de>>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("the empty current-checkpoint inline envelope field")
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut input: A,
        ) -> Result<Self::Value, A::Error> {
            if input.size_hint() != Some(0)
                || input.next_element::<serde::de::IgnoredAny>()?.is_some()
            {
                return Err(serde::de::Error::custom(
                    "canonical artifacts belong only to checkpoint members",
                ));
            }
            Ok(Vec::new())
        }
    }
    deserializer.deserialize_seq(Empty)
}
