use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The request body's declared wire version. The server admits exactly one
/// version and refuses every other as `UnsupportedProtocol`, so a client
/// never receives a shape it cannot decode.
///
/// `v3` adds the historical `previously_committed` mutation outcome without
/// live receipt evidence. Older decoders cannot interpret that outcome, so
/// `v1` and `v2` requests are refused before effects.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BankHttpProtocolVersion {
    V3,
    Unsupported(String),
}

impl Serialize for BankHttpProtocolVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(match self {
            Self::V3 => "v3",
            Self::Unsupported(value) => value,
        })
    }
}

impl<'de> Deserialize<'de> for BankHttpProtocolVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(if value == "v3" {
            Self::V3
        } else {
            Self::Unsupported(value)
        })
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BankHttpRequestControls {
    pub deadline_milliseconds: u64,
    pub maximum_results: usize,
    pub maximum_work: usize,
}

impl BankHttpRequestControls {
    pub const fn new(
        deadline_milliseconds: u64,
        maximum_results: usize,
        maximum_work: usize,
    ) -> Self {
        Self {
            deadline_milliseconds,
            maximum_results,
            maximum_work,
        }
    }
}
