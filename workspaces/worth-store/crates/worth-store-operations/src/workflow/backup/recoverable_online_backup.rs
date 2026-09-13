use worth_store_authority::StoreCurrentAuthorityWitness;
use worth_store_physical_isolation::BackupCutRecoveryRecord;

use crate::{
    ActiveBackupRecoveryHandle, BackupExportCustodyReadiness, OperationalOperationId,
    SelectedOperationalControlState,
};

use super::AdmittedOnlineBackup;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoverableOnlineBackup {
    operation_id: OperationalOperationId,
    recovery: BackupCutRecoveryRecord,
    materialization_plan: Option<crate::BackupMaterializationRecoveryPlan>,
}

#[derive(Debug)]
pub struct OnlineBackupReadmissionDenial {
    recoverable: RecoverableOnlineBackup,
    source: OnlineBackupReadmissionFailure,
}

#[derive(Debug)]
pub enum OnlineBackupReadmissionFailure {
    InvalidObservationBudget,
    Cut(worth_store_physical_isolation::BackupCutReadmissionDenial),
    SourceVerification(worth_store_offline_verifier::BackupCutSourceVerificationDenial),
}

impl OnlineBackupReadmissionDenial {
    pub const fn source(&self) -> &OnlineBackupReadmissionFailure {
        &self.source
    }

    pub fn into_retry(self) -> (RecoverableOnlineBackup, OnlineBackupReadmissionFailure) {
        (self.recoverable, self.source)
    }
}

impl RecoverableOnlineBackup {
    fn from_control_handle(handle: ActiveBackupRecoveryHandle) -> Self {
        let (operation_id, recovery, materialization_plan) = handle.into_parts();
        Self {
            operation_id,
            recovery,
            materialization_plan,
        }
    }

    pub const fn operation_id(&self) -> &OperationalOperationId {
        &self.operation_id
    }

    pub const fn cut_identity(&self) -> [u8; 32] {
        self.recovery.cut_identity()
    }

    pub const fn materialization_plan(&self) -> Option<&crate::BackupMaterializationRecoveryPlan> {
        self.materialization_plan.as_ref()
    }

    #[cfg(any(test, feature = "certification-test-authority"))]
    pub fn readmit(
        self,
        current_authority: &StoreCurrentAuthorityWitness,
        custody: &BackupExportCustodyReadiness,
        observation_buffer_bytes: usize,
    ) -> Result<AdmittedOnlineBackup, OnlineBackupReadmissionDenial> {
        self.readmit_with_storage_posture(
            current_authority,
            custody,
            worth_store_physical_isolation::BackupCutStoragePosture::for_certification_test(),
            observation_buffer_bytes,
        )
    }

    pub fn readmit_with_storage_posture(
        self,
        current_authority: &StoreCurrentAuthorityWitness,
        custody: &BackupExportCustodyReadiness,
        storage_posture: worth_store_physical_isolation::BackupCutStoragePosture,
        observation_buffer_bytes: usize,
    ) -> Result<AdmittedOnlineBackup, OnlineBackupReadmissionDenial> {
        let Some(inspection_budget) =
            worth_store_offline_verifier::OfflineInspectionBudget::bounded(
                observation_buffer_bytes,
                u64::MAX,
            )
        else {
            return Err(OnlineBackupReadmissionDenial {
                recoverable: self,
                source: OnlineBackupReadmissionFailure::InvalidObservationBudget,
            });
        };
        let cut = match self.recovery.readmit(
            current_authority,
            custody.authority_bound_receipt(),
            storage_posture,
            observation_buffer_bytes,
        ) {
            Ok(cut) => cut,
            Err(source) => {
                return Err(OnlineBackupReadmissionDenial {
                    recoverable: self,
                    source: OnlineBackupReadmissionFailure::Cut(source),
                })
            }
        };
        let source_verification = match worth_store_offline_verifier::verify_backup_cut_sources(
            cut.manifest(),
            inspection_budget,
        ) {
            Ok(report) => report,
            Err(source) => {
                return Err(OnlineBackupReadmissionDenial {
                    recoverable: self,
                    source: OnlineBackupReadmissionFailure::SourceVerification(source),
                })
            }
        };
        Ok(AdmittedOnlineBackup::new(
            self.operation_id,
            cut,
            source_verification,
        ))
    }
}

pub fn recover_online_backups(
    selected: SelectedOperationalControlState,
) -> impl ExactSizeIterator<Item = RecoverableOnlineBackup> {
    selected
        .into_active_backup_recovery_handles()
        .into_iter()
        .map(RecoverableOnlineBackup::from_control_handle)
}
