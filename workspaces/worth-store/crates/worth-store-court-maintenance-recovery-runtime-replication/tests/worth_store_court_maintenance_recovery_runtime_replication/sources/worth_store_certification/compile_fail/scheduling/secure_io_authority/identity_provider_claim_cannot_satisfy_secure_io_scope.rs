use worth_store_io_scheduler::{
    IoSchedulerBackendCapabilityAdmission, SecureIoOperation, SecureIoPreservationRequest,
};
use worth_store_security::StoreJwtSubjectClaim;

fn main() {
    let backend: IoSchedulerBackendCapabilityAdmission = todo!();
    let claim = StoreJwtSubjectClaim::raw("sub-123");
    let _ = SecureIoPreservationRequest::new(SecureIoOperation::ReadAhead, &claim, &backend);
}
