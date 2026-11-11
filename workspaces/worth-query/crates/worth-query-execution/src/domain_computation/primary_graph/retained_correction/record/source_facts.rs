//! Historical exactness is recorded; an unavailable original never becomes empty.
use super::*;
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::domain_computation::primary_graph) enum RecordedSourceFacts {
    Empty,
    Exact { version: u16, bytes: Vec<u8> },
    Unavailable(RecordedSourceFactsUnavailable),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(in crate::domain_computation::primary_graph) enum RecordedSourceFactsUnavailable {
    OriginalRebase,
    UnsupportedEncoding,
}
