use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The request body's declared wire version. The server admits exactly one
/// version and refuses every other as `UnsupportedProtocol`, so a client
/// never receives a shape it cannot decode.
///
/// `v4` carries owning money custody commands. Earlier versions are refused before effects.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BankHttpProtocolVersion {
    V4,
    Unsupported(String),
}

impl Serialize for BankHttpProtocolVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(match self {
            Self::V4 => "v4",
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
        Ok(if value == "v4" {
            Self::V4
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
}

impl BankHttpRequestControls {
    pub const fn new(deadline_milliseconds: u64, maximum_results: usize) -> Self {
        Self {
            deadline_milliseconds,
            maximum_results,
        }
    }
}

/// Effect commands that do not select retained history carry only their deadline.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BankHttpEffectControls {
    pub deadline_milliseconds: u64,
}
