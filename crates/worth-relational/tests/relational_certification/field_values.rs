use crate::relational_certification::world::supply_chain::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum FieldValue {
    Text(String),
    U16(u16),
    U32(u32),
    Region(Region),
    Posture(OperatingPosture),
    VesselClass(VesselClass),
    VoyageStatus(VoyageStatus),
    HazardClass(HazardClass),
    BookingStatus(BookingStatus),
    InspectionResult(InspectionResult),
}

pub(crate) fn field_value(record: &EntityRecord, field: FieldKey) -> Option<FieldValue> {
    match (record, field) {
        (EntityRecord::Port(value), FieldKey::Name) => Some(FieldValue::Text(value.name.clone())),
        (EntityRecord::Port(value), FieldKey::PortCode) => Some(FieldValue::U16(value.code)),
        (EntityRecord::Port(value), FieldKey::Region) => Some(FieldValue::Region(value.region)),
        (EntityRecord::Port(value), FieldKey::Posture) => Some(FieldValue::Posture(value.posture)),
        (EntityRecord::Terminal(value), FieldKey::Name) => {
            Some(FieldValue::Text(value.name.clone()))
        }
        (EntityRecord::Terminal(value), FieldKey::Capacity) => {
            Some(FieldValue::U32(value.capacity.0))
        }
        (EntityRecord::Terminal(value), FieldKey::Posture) => {
            Some(FieldValue::Posture(value.posture))
        }
        (EntityRecord::Berth(value), FieldKey::Name) => Some(FieldValue::Text(value.name.clone())),
        (EntityRecord::Berth(value), FieldKey::Depth) => Some(FieldValue::U16(value.depth.0)),
        (EntityRecord::Berth(value), FieldKey::Capacity) => Some(FieldValue::U32(value.capacity.0)),
        (EntityRecord::Berth(value), FieldKey::Posture) => Some(FieldValue::Posture(value.posture)),
        (EntityRecord::Vessel(value), FieldKey::CallSign) => {
            Some(FieldValue::Text(value.call_sign.clone()))
        }
        (EntityRecord::Vessel(value), FieldKey::Class) => {
            Some(FieldValue::VesselClass(value.class))
        }
        (EntityRecord::Vessel(value), FieldKey::Capacity) => {
            Some(FieldValue::U32(value.capacity.0))
        }
        (EntityRecord::Vessel(value), FieldKey::Posture) => {
            Some(FieldValue::Posture(value.posture))
        }
        (EntityRecord::Voyage(value), FieldKey::Status) => {
            Some(FieldValue::VoyageStatus(value.status))
        }
        (EntityRecord::Voyage(value), FieldKey::DepartureMinute) => {
            Some(FieldValue::U32(value.departure.0))
        }
        (EntityRecord::Voyage(value), FieldKey::ArrivalMinute) => {
            Some(FieldValue::U32(value.arrival.0))
        }
        (EntityRecord::Voyage(value), FieldKey::Revision) => Some(FieldValue::U16(value.revision)),
        (EntityRecord::PortCall(value), FieldKey::Sequence) => {
            Some(FieldValue::U16(value.sequence))
        }
        (EntityRecord::PortCall(value), FieldKey::Revision) => {
            Some(FieldValue::U16(value.revision))
        }
        (EntityRecord::CargoLot(value), FieldKey::Mass) => Some(FieldValue::U32(value.mass.0)),
        (EntityRecord::CargoLot(value), FieldKey::CustomerCode) => {
            Some(FieldValue::Text(value.customer_code.0.clone()))
        }
        (EntityRecord::CargoLot(value), FieldKey::HazardClass) => {
            Some(FieldValue::HazardClass(value.hazard))
        }
        (EntityRecord::CargoLot(value), FieldKey::BookingStatus) => {
            Some(FieldValue::BookingStatus(value.booking))
        }
        (EntityRecord::Inspection(value), FieldKey::InspectionResult) => {
            Some(FieldValue::InspectionResult(value.result))
        }
        (EntityRecord::Inspection(value), FieldKey::InspectionMinute) => {
            Some(FieldValue::U32(value.minute.0))
        }
        _ => None,
    }
}
