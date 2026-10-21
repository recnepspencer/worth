//! Result meaning is independent of Query permission and scheduling packets.
use super::*;
use serde::{Deserialize, Serialize};
use worth_query_host::facade::domain::InternedString;
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(super) struct Contributor {
    pub(super) kind: members::Kind,
    pub(super) measure: Measure,
    pub(super) source: (u32, (u64, u32)),
    pub(super) output: Option<(u32, (u64, u32))>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", content = "value")]
pub(super) enum Measure {
    Pending,
    Complete(u64),
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(super) struct Snapshot {
    pub(super) count: u64,
    pub(super) measure: Measure,
    pub(super) contributors: Vec<Contributor>,
}
impl Snapshot {
    pub(super) fn empty() -> Self {
        Self {
            count: 0,
            measure: Measure::Complete(0),
            contributors: Vec::new(),
        }
    }
}
impl CanonicalBits for Contributor {
    fn canonical_len(&self) -> Option<usize> {
        (
            (&self.source, &self.output),
            (&self.kind, self.measure.value()),
        )
            .canonical_len()
    }
    fn visit_canonical_bits(&self, visit: &mut dyn FnMut(&[u8]) -> bool) -> bool {
        (
            (&self.source, &self.output),
            (&self.kind, self.measure.value()),
        )
            .visit_canonical_bits(visit)
    }
}
impl CanonicalBits for Snapshot {
    fn canonical_len(&self) -> Option<usize> {
        (&self.count, (self.value(), &self.contributors)).canonical_len()
    }
    fn visit_canonical_bits(&self, visit: &mut dyn FnMut(&[u8]) -> bool) -> bool {
        (&self.count, (self.value(), &self.contributors)).visit_canonical_bits(visit)
    }
}
impl Measure {
    fn value(&self) -> Option<u64> {
        match self {
            Self::Pending => None,
            Self::Complete(value) => Some(*value),
        }
    }
}
impl Snapshot {
    fn value(&self) -> Option<u64> {
        match self.measure {
            Measure::Pending => None,
            Measure::Complete(value) => Some(value),
        }
    }
}
impl ChargedBytes for Snapshot {
    fn additional_charged_bytes(&self) -> u64 {
        self.contributors.additional_charged_bytes()
    }
}
impl ChargedBytes for Contributor {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}
fn encode(value: &Snapshot) -> InternedString {
    InternedString::Raw(toml::to_string(value).expect("finite typed snapshot encodes"))
}
fn decode(value: InternedString) -> Option<Snapshot> {
    let InternedString::Raw(value) = value else {
        return None;
    };
    toml::from_str(&value).ok()
}
worth_query_value_binding! { pub(super) SnapshotBinding for Snapshot {
    identity: "worth.query.certification.availability-snapshot.v1", scalar: String,
    encode: encode, decode: decode,
}}
worth_query_structured_value_binding!(pub(super) SnapshotResultBinding for Snapshot { identity: "worth.query.certification.availability-result.v1" });
