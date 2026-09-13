use worth_store_buffer_pool::PhysicalFrameLease;
use worth_store_physical_integrity::PhysicalIntegrityValidationRecord;

fn observe(lease: &PhysicalFrameLease) {
    let _: Option<PhysicalIntegrityValidationRecord> = lease.integrity_validation();
}

fn main() {}
