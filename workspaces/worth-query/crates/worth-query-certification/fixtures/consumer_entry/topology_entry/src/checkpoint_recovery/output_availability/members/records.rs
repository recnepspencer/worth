//! This installed application extends the body with population facts.
use super::*;
pub struct AvailabilityRecords;
impl Fragment for AvailabilityRecords {
    const NAME: &'static str = "availability_records";
    type Bindings = TopologyRecordsBindings;
    type Members = (crate::TopologyRecords, Quantity);
}
pub use crate::{Body, BodyKey, Length, PlanarMutationScope, PositionX, PositionY};
pub struct Quantity;
impl Entity for Quantity {
    const NAME: &'static str = "Quantity";
    type Aspects = (QuantityFacts,);
    type Relations = ();
}
impl Quantity {
    pub fn reference<S: TopologySchemaBinding>() -> ApplicationEntityRef<S, Self> {
        entity(AvailabilityRecords, Self)
    }
}
pub struct QuantityFacts;
impl Aspect for QuantityFacts {
    const NAME: &'static str = "QuantityFacts";
    type Row = QuantityValue;
}
pub struct QuantityValue {
    key: String,
    snapshot: Snapshot,
}
impl fragment::Record<QuantityValue> for TopologyRecordsBindings {
    fn record() -> impl Shape<QuantityValue, Self> {
        field("QuantityKey", |row: &QuantityValue| row.key.clone())
            .and("QuantitySnapshot", |row: &QuantityValue| {
                row.snapshot.clone()
            })
            .build(|key, snapshot, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(QuantityValue { key, snapshot })
            })
    }
}
pub struct PopulationFacts;
impl Aspect for PopulationFacts {
    const NAME: &'static str = "PopulationFacts";
    type Row = PopulationValue;
}
pub struct PopulationValue {
    number: u64,
    demand: String,
    kind: Kind,
}
impl fragment::Record<PopulationValue> for TopologyRecordsBindings {
    fn record() -> impl Shape<PopulationValue, Self> {
        field("MemberNumber", |row: &PopulationValue| row.number)
            .and("DemandKey", |row: &PopulationValue| row.demand.clone())
            .and("MemberKind", |row: &PopulationValue| row.kind)
            .build(|number, demand, kind, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(PopulationValue {
                    number,
                    demand,
                    kind,
                })
            })
    }
}
impl Scalar<Snapshot> for TopologyRecordsBindings {
    type Binding = SnapshotBinding;
    const EQUALITY_QUERYABLE: bool = true;
}
impl Scalar<Kind> for TopologyRecordsBindings {
    type Binding = KindBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
pub type QuantityKey = FieldSelection<AvailabilityRecords, Quantity, QuantityFacts, String>;
pub type QuantitySnapshot = FieldSelection<AvailabilityRecords, Quantity, QuantityFacts, Snapshot>;
pub type MemberNumber = FieldSelection<crate::TopologyRecords, Body, PopulationFacts, u64>;
pub type DemandKey = FieldSelection<crate::TopologyRecords, Body, PopulationFacts, String>;
pub type MemberKind = FieldSelection<crate::TopologyRecords, Body, PopulationFacts, Kind>;
