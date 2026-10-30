use worth_store_buffer_pool::PhysicalFrameLease;
use worth_store_physical_integrity::PhysicalIntegrityValidationRecord;

#[expect(
    dead_code,
    reason = "Compile-only positive consumer witness; its signature and body are checked without fabricating admitted runtime inputs."
)]
fn observe(lease: &PhysicalFrameLease) {
    let _: Option<PhysicalIntegrityValidationRecord> = lease.integrity_validation();
}

pub(crate) fn main() {}
