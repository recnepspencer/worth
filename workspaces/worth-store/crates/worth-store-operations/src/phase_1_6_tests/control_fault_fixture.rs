use std::cell::Cell;

use worth_store_physical_backend::{
    ControlMediaFault, ControlRecoveryObjectHandle, PhysicalControlAppendReceipt,
};

use super::support::{
    reclaim_reference, BackupArtifactFamily, BackupReachabilityLeaseRegistry,
    ExecutedReachabilityEvidence, HazardLeaseTable, HazardLeaseTableCapacity, ReclaimDenial,
    ReclaimEligibilityProof,
};
use crate::{
    OperationalControlAppendDenial, OperationalControlRecord, OperationalControlStore,
    OperationalControlStorePort,
};

pub(super) struct FailingControlStore;

impl OperationalControlStorePort for FailingControlStore {
    fn publish_recovery_object(
        &self,
        content: &[u8],
    ) -> Result<ControlRecoveryObjectHandle, OperationalControlAppendDenial> {
        Ok(ControlRecoveryObjectHandle::for_content(content))
    }

    fn append(
        &self,
        _record: &OperationalControlRecord,
    ) -> Result<PhysicalControlAppendReceipt, OperationalControlAppendDenial> {
        Err(OperationalControlAppendDenial::Media(
            ControlMediaFault::Io(std::io::Error::other("injected receipt-media failure")),
        ))
    }

    fn compare_exchange_authorization_consumption(
        &self,
        _expected: Option<worth_store_authority::ControlStoreGeneration>,
        record: &OperationalControlRecord,
    ) -> Result<PhysicalControlAppendReceipt, OperationalControlAppendDenial> {
        self.append(record)
    }
}

pub(super) struct ObserveReservedLeaseThenFail<'a> {
    pub(super) delegate: &'a OperationalControlStore,
    pub(super) leases: &'a BackupReachabilityLeaseRegistry,
    pub(super) calls: Cell<usize>,
}

impl OperationalControlStorePort for ObserveReservedLeaseThenFail<'_> {
    fn publish_recovery_object(
        &self,
        content: &[u8],
    ) -> Result<ControlRecoveryObjectHandle, OperationalControlAppendDenial> {
        self.delegate.publish_recovery_object(content)
    }

    fn append(
        &self,
        record: &OperationalControlRecord,
    ) -> Result<PhysicalControlAppendReceipt, OperationalControlAppendDenial> {
        let call = self.calls.get();
        self.calls.set(call + 1);
        if call == 0 {
            let protected = reclaim_reference(BackupArtifactFamily::Page, 4);
            let evidence = ExecutedReachabilityEvidence::for_certification_reference(protected);
            let hazards = HazardLeaseTable::with_capacity(
                HazardLeaseTableCapacity::bounded_slots(1).expect("capacity"),
            )
            .live_index_snapshot();
            let proof = ReclaimEligibilityProof::admit(
                evidence,
                hazards,
                self.leases
                    .live_index_snapshot()
                    .expect("reserved lease remains readable"),
            )
            .expect("reclaim proof");
            assert!(matches!(
                proof.try_reclaim(),
                Err(ReclaimDenial::BlockedByBackupCut { .. })
            ));
            return FailingControlStore.append(record);
        }
        self.delegate.append(record)
    }

    fn compare_exchange_authorization_consumption(
        &self,
        expected: Option<worth_store_authority::ControlStoreGeneration>,
        record: &OperationalControlRecord,
    ) -> Result<PhysicalControlAppendReceipt, OperationalControlAppendDenial> {
        self.delegate
            .compare_exchange_authorization_consumption(expected, record)
    }
}
