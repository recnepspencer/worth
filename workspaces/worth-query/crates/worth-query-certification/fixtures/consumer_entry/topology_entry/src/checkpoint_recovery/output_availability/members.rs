//! Native population membership and publication fields.
use super::*;
use worth_query_decl::facade::application_schema::fragment::{self, *};
mod records;
pub use records::*;
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Kind {
    Geometry,
    Quantity,
}
impl Kind {
    fn code(&self) -> u64 {
        match self {
            Self::Geometry => 1,
            Self::Quantity => 2,
        }
    }
    fn decode(code: u64) -> Option<Self> {
        match code {
            1 => Some(Self::Geometry),
            2 => Some(Self::Quantity),
            _ => None,
        }
    }
}
impl CanonicalBits for Kind {
    fn canonical_len(&self) -> Option<usize> {
        self.code().canonical_len()
    }
    fn visit_canonical_bits(&self, visitor: &mut dyn FnMut(&[u8]) -> bool) -> bool {
        self.code().visit_canonical_bits(visitor)
    }
}
worth_query_value_binding!(pub KindBinding for Kind {identity:"availability-member-family",scalar:UInt64,encode:Kind::code,decode:Kind::decode,});

pub struct PopulationMember;
impl Relation for PopulationMember {
    const NAME: &'static str = "AvailabilityPopulationMember";
    type To = Body;
}
impl PopulationMember {
    pub(super) fn reference() -> ApplicationRelationRef<Schema, Self, Body, Body> {
        relation(crate::TopologyRecords, Body, Self)
    }
}

pub(super) fn descriptor<Entity>(
    identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
) -> (u32, (u64, u32)) {
    let id = identity.entity_id();
    (
        id.partition_value(),
        (id.local_slot_value(), id.generation_value()),
    )
}
#[derive(serde::Serialize)]
pub(super) struct Member {
    pub(super) number: u64,
    pub(super) entity: WorthQueryInvariantEntityIdentity<Schema, Body>,
}
impl ApplicationComputationPartition for Member {
    const IDENTITY: &'static str = "availability-native-member";
}
impl ChargedBytes for Member {
    fn additional_charged_bytes(&self) -> u64 {
        self.entity.additional_charged_bytes()
    }
}
