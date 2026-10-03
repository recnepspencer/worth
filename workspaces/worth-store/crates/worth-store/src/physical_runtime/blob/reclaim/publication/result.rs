use worth_store_physical_format::PersistedRecordIdentity;

use crate::physical_runtime::{BlobAppendFailure, CompletedPhysicalMutation};

use super::{BlobReclaimFailure, BlobReclaimPublicationStage, ReclaimPublication};

impl ReclaimPublication<'_, '_> {
    pub(super) fn reservation_failure(
        &self,
        manifest_record: PersistedRecordIdentity,
        cause: BlobAppendFailure,
    ) -> BlobReclaimFailure {
        let safe_manifest_only = match &cause {
            BlobAppendFailure::Idempotency(_) | BlobAppendFailure::Preparation(_) => {
                self.admitted.attempt().settle_manifest_without_drop()
            }
            BlobAppendFailure::ProvenNoEffect(fate) => {
                self.admitted.attempt().prove_reservation_no_effect(fate)
                    || self.admitted.attempt().settle_manifest_without_drop()
            }
            _ => false,
        };
        if safe_manifest_only {
            BlobReclaimFailure::ManifestRetained {
                manifest_record,
                cause,
            }
        } else {
            failure(
                BlobReclaimPublicationStage::Reservation,
                Some(manifest_record),
                cause,
            )
        }
    }

    pub(super) fn drop_failure(
        &self,
        manifest_record: PersistedRecordIdentity,
        cause: BlobAppendFailure,
    ) -> BlobReclaimFailure {
        let safe_manifest_only = match &cause {
            BlobAppendFailure::Idempotency(_) | BlobAppendFailure::Preparation(_) => {
                self.admitted.attempt().settle_reservation_without_drop()
            }
            BlobAppendFailure::ProvenNoEffect(fate) => {
                self.admitted.attempt().prove_drop_no_effect(fate)
            }
            BlobAppendFailure::Indeterminate(_)
            | BlobAppendFailure::MissingRecordIdentity
            | BlobAppendFailure::ExtraRecordIdentities => false,
        };
        if safe_manifest_only {
            BlobReclaimFailure::ManifestRetained {
                manifest_record,
                cause,
            }
        } else {
            failure(
                BlobReclaimPublicationStage::Drop,
                Some(manifest_record),
                cause,
            )
        }
    }
}

pub(super) fn one_record(
    completed: &CompletedPhysicalMutation,
) -> Result<PersistedRecordIdentity, BlobAppendFailure> {
    match completed.persisted_records() {
        [record] => Ok(*record),
        [] => Err(BlobAppendFailure::MissingRecordIdentity),
        _ => Err(BlobAppendFailure::ExtraRecordIdentities),
    }
}

pub(super) fn released_descriptor_record(
    completed: &CompletedPhysicalMutation,
    has_directory_replacement: bool,
) -> Result<PersistedRecordIdentity, BlobAppendFailure> {
    match (completed.persisted_records(), has_directory_replacement) {
        ([descriptor], false) | ([descriptor, _], true) => Ok(*descriptor),
        ([], _) => Err(BlobAppendFailure::MissingRecordIdentity),
        _ => Err(BlobAppendFailure::ExtraRecordIdentities),
    }
}

pub(super) fn failure(
    stage: BlobReclaimPublicationStage,
    manifest_record: Option<PersistedRecordIdentity>,
    cause: BlobAppendFailure,
) -> BlobReclaimFailure {
    BlobReclaimFailure::Publication {
        stage,
        manifest_record,
        cause,
    }
}
