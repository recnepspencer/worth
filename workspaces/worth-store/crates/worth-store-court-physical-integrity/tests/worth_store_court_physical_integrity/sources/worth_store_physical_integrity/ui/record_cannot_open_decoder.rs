use worth_store_physical_format::DurableRootSelector;
use worth_store_physical_integrity::PhysicalIntegrityValidationRecord;

fn decode(record: PhysicalIntegrityValidationRecord) {
    let _ = DurableRootSelector::decode(record);
}

fn main() {}
