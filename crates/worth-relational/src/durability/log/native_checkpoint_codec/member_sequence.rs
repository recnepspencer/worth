//! One exact borrowed wire table; its original admission precedes Serde growth.
use super::checkpoint_wire::ByteString;
use crate::durability::medium::RelationalDurableMemberLocator as Locator;
use serde::Deserializer;
pub(super) fn decode<'de: 'a, 'a, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<(Locator, ByteString<'a>)>, D::Error> {
    struct Entries;
    impl<'de> serde::de::Visitor<'de> for Entries {
        type Value = Vec<(Locator, ByteString<'de>)>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("an exact native member byte table")
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut input: A,
        ) -> Result<Self::Value, A::Error> {
            let count = input
                .size_hint()
                .ok_or_else(|| serde::de::Error::custom("native member count is required"))?;
            let mut rows = Vec::with_capacity(count);
            while let Some(row) = input.next_element()? {
                if rows.len() == count {
                    return Err(serde::de::Error::custom(
                        "native member table exceeds its length",
                    ));
                }
                rows.push(row);
            }
            if rows.len() != count {
                return Err(serde::de::Error::custom("native member table is truncated"));
            }
            Ok(rows)
        }
    }
    deserializer.deserialize_seq(Entries)
}
pub(super) fn reserve(
    payload: &[u8],
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<
    Option<worth_execution::ExecutionMemoryReservation>,
    crate::durability::data::RelationalRecoveryStop,
> {
    use crate::durability::{
        canonical_commit_artifact::{
            borrowed_payload::BorrowedPayload, RelationalCanonicalCommitArtifactRefusal as Refusal,
        },
        data::RelationalRecoveryStop,
    };
    let mut count = None;
    BorrowedPayload(payload)
        .fields(request, |name, value| {
            if name == "members" {
                if count.is_some() {
                    return Err(Refusal::PayloadDecode {
                        detail: "duplicate native member table".into(),
                    });
                }
                count = Some(value.elements(request, |_| Ok(()))?);
            }
            Ok(())
        })
        .map_err(|cause| match cause {
            Refusal::Request(cause) => RelationalRecoveryStop::Request(cause),
            _ => super::manifest_capture::corrupt("invalid borrowed member table").into(),
        })?;
    let count =
        count.ok_or_else(|| super::manifest_capture::corrupt("member table is required"))?;
    let bytes = std::alloc::Layout::array::<(Locator, ByteString<'_>)>(count)
        .map_err(|_| super::manifest_capture::corrupt("member table layout overflow"))?
        .size();
    request
        .reserve_retained_memory(bytes as u64)
        .map_err(|cause| RelationalRecoveryStop::Request(cause.into()))
}

pub(super) fn empty_envelopes<'de: 'a, 'a, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<ByteString<'a>>, D::Error> {
    struct Empty;
    impl<'de> serde::de::Visitor<'de> for Empty {
        type Value = Vec<ByteString<'de>>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("the empty CHK22 inline envelope field")
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut input: A,
        ) -> Result<Self::Value, A::Error> {
            if input.size_hint() != Some(0)
                || input.next_element::<serde::de::IgnoredAny>()?.is_some()
            {
                return Err(serde::de::Error::custom(
                    "CHK22 canonical artifacts belong only to members",
                ));
            }
            Ok(Vec::new())
        }
    }
    deserializer.deserialize_seq(Empty)
}
