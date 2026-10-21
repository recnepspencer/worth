//! Native population membership and publication fields.
use super::*;
worth_query_entity!(pub(super) Quantity for Schema: TopologySchemaBinding);
worth_query_aspect!(pub(super) QuantityFacts for Schema: TopologySchemaBinding, Quantity;
    identity = AspectIdentity(0x9174_2731), revision = AspectContractRevision(1),);
worth_query_field!(pub(super) QuantityKey for Schema: TopologySchemaBinding, Quantity, QuantityFacts:
    String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub(super) QuantitySnapshot for Schema: TopologySchemaBinding, Quantity, QuantityFacts:
    Snapshot => SnapshotBinding, read_write, equality);
worth_query_aspect!(pub(super) PopulationFacts for Schema: TopologySchemaBinding, Body;
    identity = AspectIdentity(0x9174_2732), revision = AspectContractRevision(1),);
worth_query_field!(pub(super) MemberNumber for Schema: TopologySchemaBinding, Body, PopulationFacts:
    u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub(super) DemandKey for Schema: TopologySchemaBinding, Body, PopulationFacts:
 String => StringApplicationValueBinding, read_only, equality);
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub(super) enum Kind {
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
worth_query_value_binding!(pub(super) KindBinding for Kind {identity:"availability-member-family",scalar:UInt64,encode:Kind::code,decode:Kind::decode,});
worth_query_field!(pub(super) MemberKind for Schema: TopologySchemaBinding, Body, PopulationFacts: Kind => KindBinding, read_only, equality);
pub(super) struct PopulationMember;
impl PopulationMember {
    pub(super) fn reference() -> ApplicationRelationRef<Schema, Self, Body, Body> {
        ApplicationRelationRef::from_schema_identifiers(
            "AvailabilityPopulationMember",
            "Body",
            "Body",
            ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        )
    }
}
pub(super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .entity(Quantity::reference())
        .aspect(Quantity::reference(), QuantityFacts::reference())
        .field(Quantity::reference(), QuantityKey::reference())
        .field(Quantity::reference(), QuantitySnapshot::reference())
        .aspect(Body::reference(), PopulationFacts::reference())
        .field(Body::reference(), MemberNumber::reference())
        .field(Body::reference(), DemandKey::reference())
        .field(Body::reference(), MemberKind::reference())
        .relation(
            PopulationMember::reference(),
            Body::reference(),
            Body::reference(),
        )
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
