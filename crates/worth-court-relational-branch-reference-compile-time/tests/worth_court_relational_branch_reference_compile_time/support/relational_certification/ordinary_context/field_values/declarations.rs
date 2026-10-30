pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::BookingStatus, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::HazardClass, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::InspectionResult, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::OperatingPosture, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::Region, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::VesselClass, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::VoyageStatus};

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
        (worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Port(value), worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Name) => Some(crate::relational_certification::ordinary_context::field_values::FieldValue::Text(value.name.clone())),
        (worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Port(value), worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::PortCode) => Some(crate::relational_certification::ordinary_context::field_values::FieldValue::U16(value.code)),
        (worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Port(value), worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Region) => Some(crate::relational_certification::ordinary_context::field_values::FieldValue::Region(value.region)),
        (worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Port(value), worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Posture) => Some(crate::relational_certification::ordinary_context::field_values::FieldValue::Posture(value.posture)),
        (worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Terminal(value), worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Name) => {
            Some(crate::relational_certification::ordinary_context::field_values::FieldValue::Text(value.name.clone()))
        }
        (worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Terminal(value), worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Capacity) => {
            Some(crate::relational_certification::ordinary_context::field_values::FieldValue::U32(value.capacity.0))
        }
        (worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Terminal(value), worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Posture) => {
            Some(crate::relational_certification::ordinary_context::field_values::FieldValue::Posture(value.posture))
        }
        (worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Berth(value), worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Name) => Some(crate::relational_certification::ordinary_context::field_values::FieldValue::Text(value.name.clone())),
        (worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Berth(value), worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Depth) => Some(crate::relational_certification::ordinary_context::field_values::FieldValue::U16(value.depth.0)),
        (worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Berth(value), worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Capacity) => Some(crate::relational_certification::ordinary_context::field_values::FieldValue::U32(value.capacity.0)),
        (worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Berth(value), worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Posture) => Some(crate::relational_certification::ordinary_context::field_values::FieldValue::Posture(value.posture)),
        (worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Vessel(value), worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::CallSign) => {
            Some(crate::relational_certification::ordinary_context::field_values::FieldValue::Text(value.call_sign.clone()))
        }
        (worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Vessel(value), worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Class) => {
            Some(crate::relational_certification::ordinary_context::field_values::FieldValue::VesselClass(value.class))
        }
        (worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Vessel(value), worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Capacity) => {
            Some(crate::relational_certification::ordinary_context::field_values::FieldValue::U32(value.capacity.0))
        }
        (worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Vessel(value), worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Posture) => {
            Some(crate::relational_certification::ordinary_context::field_values::FieldValue::Posture(value.posture))
        }
        (worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Voyage(value), worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Status) => {
            Some(crate::relational_certification::ordinary_context::field_values::FieldValue::VoyageStatus(value.status))
        }
        (worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Voyage(value), worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::DepartureMinute) => {
            Some(crate::relational_certification::ordinary_context::field_values::FieldValue::U32(value.departure.0))
        }
        (worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Voyage(value), worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::ArrivalMinute) => {
            Some(crate::relational_certification::ordinary_context::field_values::FieldValue::U32(value.arrival.0))
        }
        (worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Voyage(value), worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Revision) => Some(crate::relational_certification::ordinary_context::field_values::FieldValue::U16(value.revision)),
        (worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::PortCall(value), worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Sequence) => {
            Some(crate::relational_certification::ordinary_context::field_values::FieldValue::U16(value.sequence))
        }
        (worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::PortCall(value), worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Revision) => {
            Some(crate::relational_certification::ordinary_context::field_values::FieldValue::U16(value.revision))
        }
        (worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::CargoLot(value), worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Mass) => Some(crate::relational_certification::ordinary_context::field_values::FieldValue::U32(value.mass.0)),
        (worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::CargoLot(value), worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::CustomerCode) => {
            Some(crate::relational_certification::ordinary_context::field_values::FieldValue::Text(value.customer_code.0.clone()))
        }
        (worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::CargoLot(value), worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::HazardClass) => {
            Some(crate::relational_certification::ordinary_context::field_values::FieldValue::HazardClass(value.hazard))
        }
        (worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::CargoLot(value), worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::BookingStatus) => {
            Some(crate::relational_certification::ordinary_context::field_values::FieldValue::BookingStatus(value.booking))
        }
        (worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Inspection(value), worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::InspectionResult) => {
            Some(crate::relational_certification::ordinary_context::field_values::FieldValue::InspectionResult(value.result))
        }
        (worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Inspection(value), worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::InspectionMinute) => {
            Some(crate::relational_certification::ordinary_context::field_values::FieldValue::U32(value.minute.0))
        }
        _ => None,
    }
}
