use super::bootstrap::BootstrapTransitionFailure;
use crate::physical_runtime::{
    PhysicalRecordFormatMismatch, RecordBootstrapDenial, RecordServingRebindReason,
    UnsupportedPhysicalRecordFormat,
};

pub(super) fn classify_catalog(
    denial: crate::physical_runtime::integrity::resident_admission::denial::ResidentIntegrityAdmissionDenial,
) -> BootstrapTransitionFailure {
    use crate::physical_runtime::integrity::resident_admission::denial::ResidentIntegrityAdmissionDenial;

    match denial {
        ResidentIntegrityAdmissionDenial::BootstrapScopeMismatch(mismatch)
            if mismatch.observed_store() != mismatch.rejection().scope().store_identity() =>
        {
            BootstrapTransitionFailure::RebindRequired(
                RecordServingRebindReason::StoreIdentityMismatch,
            )
        }
        ResidentIntegrityAdmissionDenial::BootstrapScopeMismatch(mismatch) => {
            BootstrapTransitionFailure::Denied(RecordBootstrapDenial::PhysicalRecordFormatMismatch(
                PhysicalRecordFormatMismatch::new(
                    mismatch.expected_format(),
                    mismatch.observed_format(),
                ),
            ))
        }
        ResidentIntegrityAdmissionDenial::BootstrapUnsupportedFormat(unsupported) => {
            BootstrapTransitionFailure::Denied(
                RecordBootstrapDenial::UnsupportedPhysicalRecordFormat(
                    UnsupportedPhysicalRecordFormat::new(unsupported.reason()),
                ),
            )
        }
        ResidentIntegrityAdmissionDenial::Validation(
            worth_store_physical_integrity::PhysicalIntegrityRejection::Unsupported(unsupported),
        ) if unsupported.axis()
            == worth_store_physical_integrity::PhysicalIntegrityVersionAxis::PhysicalFormat =>
        {
            unsupported_format(unsupported.observed())
        }
        _ => BootstrapTransitionFailure::Denied(RecordBootstrapDenial::CatalogDamaged),
    }
}

pub(super) fn classify_root(
    denial: crate::physical_runtime::RootProtocolAdmissionDenial,
) -> BootstrapTransitionFailure {
    use worth_store_physical_integrity::{
        PhysicalIntegrityRejection, PhysicalIntegrityVersionAxis,
    };

    match denial {
        crate::physical_runtime::RootProtocolAdmissionDenial::Validation(
            PhysicalIntegrityRejection::Unsupported(unsupported),
        ) if unsupported.axis() == PhysicalIntegrityVersionAxis::PhysicalFormat => {
            unsupported_format(unsupported.observed())
        }
        _ => BootstrapTransitionFailure::Denied(RecordBootstrapDenial::CurrentRootDamaged),
    }
}

pub(super) fn classify_free_space(
    denial: crate::physical_runtime::integrity::resident_admission::denial::ResidentIntegrityAdmissionDenial,
) -> BootstrapTransitionFailure {
    match denial {
        crate::physical_runtime::integrity::resident_admission::denial::ResidentIntegrityAdmissionDenial::Validation(
            worth_store_physical_integrity::PhysicalIntegrityRejection::Damaged(localization),
        ) if localization.cause()
            == worth_store_physical_integrity::PhysicalDamageCause::PhysicalGenerationMismatch =>
        {
            BootstrapTransitionFailure::Stale(
                crate::physical_runtime::RecordServingStaleReason::FreeSpaceGenerationMismatch,
            )
        }
        crate::physical_runtime::integrity::resident_admission::denial::ResidentIntegrityAdmissionDenial::Validation(
            worth_store_physical_integrity::PhysicalIntegrityRejection::Unsupported(unsupported),
        ) if unsupported.axis()
            == worth_store_physical_integrity::PhysicalIntegrityVersionAxis::PhysicalFormat =>
        {
            unsupported_format(unsupported.observed())
        }
        _ => BootstrapTransitionFailure::Denied(RecordBootstrapDenial::FreeSpaceManifestDamaged),
    }
}

fn unsupported_format(observed: u32) -> BootstrapTransitionFailure {
    BootstrapTransitionFailure::Denied(RecordBootstrapDenial::UnsupportedPhysicalRecordFormat(
        UnsupportedPhysicalRecordFormat::new(
            worth_store_physical_format::PhysicalRecordFormatDenial::UnsupportedVersion(
                observed as u16,
            ),
        ),
    ))
}
