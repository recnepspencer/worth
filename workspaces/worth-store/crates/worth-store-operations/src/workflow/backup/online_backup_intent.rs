use sha2::{Digest, Sha256};
use worth_store_authority::StoreCurrentAuthorityWitness;
use worth_store_physical_isolation::{
    AdmittedBackupCut, BackupCutAdmissionAuthority, BackupCutAdmissionDenial,
    BackupCutAdmissionRequest, BackupCutCoordinates, BackupCutManifest, BackupCutStoragePosture,
    BackupReachabilityLeaseRegistry, BackupReachabilityLeaseRegistryDenial,
};

use crate::{
    BackupExportCustodyReadiness, OperationalControlAppendDenial, OperationalControlRecord,
    OperationalControlStorePort, OperationalOperationId, OperationalTransitionId,
};

use super::AdmittedOnlineBackup;

#[derive(Debug)]
pub struct OnlineBackupIntent {
    operation_id: OperationalOperationId,
    coordinates: BackupCutCoordinates,
    manifest: BackupCutManifest,
    custody: BackupExportCustodyReadiness,
    storage_posture: BackupCutStoragePosture,
}

#[derive(Debug)]
pub enum OnlineBackupAdmissionDenial {
    LeasePersistence(BackupLeasePersistenceDenial),
    Cut(BackupCutAdmissionDenial),
    SourceVerification(BackupSourceVerificationDenial),
}

#[derive(Debug)]
pub struct UnpersistedBackupReachabilityLease {
    operation_id: OperationalOperationId,
    cut: AdmittedBackupCut,
}

#[derive(Debug)]
pub struct BackupSourceVerificationDenial {
    unverified: UnpersistedBackupReachabilityLease,
    source: worth_store_offline_verifier::BackupCutSourceVerificationDenial,
}

#[derive(Debug)]
pub struct BackupLeasePersistenceDenial {
    unpersisted: UnpersistedBackupReachabilityLease,
    source: BackupLeasePersistenceFailure,
}

#[derive(Debug)]
pub enum BackupLeasePersistenceFailure {
    Control(OperationalControlAppendDenial),
    Registry(BackupReachabilityLeaseRegistryDenial),
    Recovery(worth_store_physical_isolation::BackupCutRecoveryDenial),
}

impl OnlineBackupIntent {
    #[cfg(any(test, feature = "certification-test-authority"))]
    pub fn new(
        operation_id: OperationalOperationId,
        coordinates: BackupCutCoordinates,
        manifest: BackupCutManifest,
        custody: BackupExportCustodyReadiness,
    ) -> Self {
        Self::new_with_storage_posture(
            operation_id,
            coordinates,
            manifest,
            custody,
            BackupCutStoragePosture::for_certification_test(),
        )
    }

    pub fn new_with_storage_posture(
        operation_id: OperationalOperationId,
        coordinates: BackupCutCoordinates,
        manifest: BackupCutManifest,
        custody: BackupExportCustodyReadiness,
        storage_posture: BackupCutStoragePosture,
    ) -> Self {
        Self {
            operation_id,
            coordinates,
            manifest,
            custody,
            storage_posture,
        }
    }

    pub fn admit_cut(
        self,
        current_authority: &StoreCurrentAuthorityWitness,
        control: &impl OperationalControlStorePort,
        leases: &BackupReachabilityLeaseRegistry,
    ) -> Result<AdmittedOnlineBackup, OnlineBackupAdmissionDenial> {
        let budget = default_source_verification_budget(&self.manifest);
        self.admit_cut_with_verification(
            current_authority,
            control,
            leases,
            budget,
            worth_store_offline_verifier::OfflineInspectionCancellation::new(),
        )
    }

    pub fn admit_cut_with_verification(
        self,
        current_authority: &StoreCurrentAuthorityWitness,
        control: &impl OperationalControlStorePort,
        leases: &BackupReachabilityLeaseRegistry,
        budget: worth_store_offline_verifier::OfflineInspectionBudget,
        cancellation: worth_store_offline_verifier::OfflineInspectionCancellation,
    ) -> Result<AdmittedOnlineBackup, OnlineBackupAdmissionDenial> {
        self.admit_cut_with_source_verification(
            current_authority,
            control,
            leases,
            move |manifest| {
                worth_store_offline_verifier::verify_backup_cut_sources_with_cancellation(
                    manifest,
                    budget,
                    cancellation,
                )
            },
        )
    }

    pub(crate) fn admit_cut_with_source_verification(
        self,
        current_authority: &StoreCurrentAuthorityWitness,
        control: &impl OperationalControlStorePort,
        leases: &BackupReachabilityLeaseRegistry,
        verify: impl FnOnce(
            &BackupCutManifest,
        ) -> Result<
            worth_store_offline_verifier::BackupCutSourceVerificationReport,
            worth_store_offline_verifier::BackupCutSourceVerificationDenial,
        >,
    ) -> Result<AdmittedOnlineBackup, OnlineBackupAdmissionDenial> {
        let request = BackupCutAdmissionRequest::new(
            self.custody.authority_bound_receipt(),
            self.coordinates,
            self.manifest,
            self.storage_posture,
        );
        let cut = BackupCutAdmissionAuthority::for_current_store(current_authority)
            .admit(request)
            .map_err(OnlineBackupAdmissionDenial::Cut)?;
        UnpersistedBackupReachabilityLease {
            operation_id: self.operation_id,
            cut,
        }
        .persist_with_source_verification(control, leases, verify)
    }
}

impl UnpersistedBackupReachabilityLease {
    pub fn persist(
        self,
        control: &impl OperationalControlStorePort,
        leases: &BackupReachabilityLeaseRegistry,
    ) -> Result<AdmittedOnlineBackup, OnlineBackupAdmissionDenial> {
        let budget = default_source_verification_budget(self.cut.manifest());
        self.persist_with_verification(
            control,
            leases,
            budget,
            worth_store_offline_verifier::OfflineInspectionCancellation::new(),
        )
    }

    pub fn persist_with_verification(
        self,
        control: &impl OperationalControlStorePort,
        leases: &BackupReachabilityLeaseRegistry,
        budget: worth_store_offline_verifier::OfflineInspectionBudget,
        cancellation: worth_store_offline_verifier::OfflineInspectionCancellation,
    ) -> Result<AdmittedOnlineBackup, OnlineBackupAdmissionDenial> {
        self.persist_with_source_verification(control, leases, move |manifest| {
            worth_store_offline_verifier::verify_backup_cut_sources_with_cancellation(
                manifest,
                budget,
                cancellation,
            )
        })
    }

    fn persist_with_source_verification(
        self,
        control: &impl OperationalControlStorePort,
        leases: &BackupReachabilityLeaseRegistry,
        verify: impl FnOnce(
            &BackupCutManifest,
        ) -> Result<
            worth_store_offline_verifier::BackupCutSourceVerificationReport,
            worth_store_offline_verifier::BackupCutSourceVerificationDenial,
        >,
    ) -> Result<AdmittedOnlineBackup, OnlineBackupAdmissionDenial> {
        let lease_transition = transition(&self.operation_id, "source-lease");
        let recovery = match self.cut.recovery_record() {
            Ok(record) => record,
            Err(source) => {
                return Err(OnlineBackupAdmissionDenial::LeasePersistence(
                    BackupLeasePersistenceDenial {
                        unpersisted: self,
                        source: BackupLeasePersistenceFailure::Recovery(source),
                    },
                ))
            }
        };
        let lease_record = match recovery.lease_persistence_record() {
            Ok(record) => record,
            Err(source) => {
                return Err(OnlineBackupAdmissionDenial::LeasePersistence(
                    BackupLeasePersistenceDenial {
                        unpersisted: self,
                        source: BackupLeasePersistenceFailure::Recovery(source),
                    },
                ))
            }
        };
        let holder = crate::control_store::backup_lease_holder_id(&self.operation_id);
        let lease_reservation = match leases.reserve_admission(holder, lease_record) {
            Ok(reservation) => reservation,
            Err(source) => {
                return Err(OnlineBackupAdmissionDenial::LeasePersistence(
                    BackupLeasePersistenceDenial {
                        unpersisted: self,
                        source: BackupLeasePersistenceFailure::Registry(source),
                    },
                ))
            }
        };
        let source_verification = match verify(self.cut.manifest()) {
            Ok(report) => report,
            Err(source) => {
                drop(lease_reservation);
                return Err(OnlineBackupAdmissionDenial::SourceVerification(
                    BackupSourceVerificationDenial {
                        unverified: self,
                        source,
                    },
                ));
            }
        };
        let recovery_object = match control.publish_recovery_object(recovery.recovery_bytes()) {
            Ok(handle) => handle,
            Err(source) => {
                return Err(OnlineBackupAdmissionDenial::LeasePersistence(
                    BackupLeasePersistenceDenial {
                        unpersisted: self,
                        source: BackupLeasePersistenceFailure::Control(source),
                    },
                ))
            }
        };
        let receipt = match control.append(&OperationalControlRecord::source_lease_persisted(
            self.cut.authority_identity(),
            self.operation_id.clone(),
            lease_transition,
            recovery,
            recovery_object,
        )) {
            Ok(receipt) => receipt,
            Err(source) => {
                return Err(OnlineBackupAdmissionDenial::LeasePersistence(
                    BackupLeasePersistenceDenial {
                        unpersisted: self,
                        source: BackupLeasePersistenceFailure::Control(source),
                    },
                ))
            }
        };
        if let Err(source) = lease_reservation.acknowledge_durable_persistence(receipt) {
            return Err(OnlineBackupAdmissionDenial::LeasePersistence(
                BackupLeasePersistenceDenial {
                    unpersisted: self,
                    source: BackupLeasePersistenceFailure::Registry(source),
                },
            ));
        }
        Ok(AdmittedOnlineBackup::new(
            self.operation_id,
            self.cut,
            source_verification,
        ))
    }
}

fn default_source_verification_budget(
    manifest: &BackupCutManifest,
) -> worth_store_offline_verifier::OfflineInspectionBudget {
    worth_store_offline_verifier::OfflineInspectionBudget::bounded(
        64 * 1024,
        manifest.total_bytes(),
    )
    .expect("an admitted backup cut has nonempty physical media")
}

impl BackupSourceVerificationDenial {
    pub const fn source(&self) -> &worth_store_offline_verifier::BackupCutSourceVerificationDenial {
        &self.source
    }

    pub fn into_retry(
        self,
    ) -> (
        UnpersistedBackupReachabilityLease,
        worth_store_offline_verifier::BackupCutSourceVerificationDenial,
    ) {
        (self.unverified, self.source)
    }
}

impl BackupLeasePersistenceDenial {
    pub fn into_retry(
        self,
    ) -> (
        UnpersistedBackupReachabilityLease,
        BackupLeasePersistenceFailure,
    ) {
        (self.unpersisted, self.source)
    }
}

pub(crate) fn transition(
    operation: &OperationalOperationId,
    suffix: &str,
) -> OperationalTransitionId {
    let readable = format!("{}:{suffix}", operation.as_str());
    if let Ok(identity) = OperationalTransitionId::new(readable) {
        return identity;
    }
    let digest = Sha256::digest(operation.as_str().as_bytes());
    OperationalTransitionId::new(format!("op-{}:{suffix}", hex(&digest[..16])))
        .expect("fixed-length transition identity")
}

pub(crate) fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
