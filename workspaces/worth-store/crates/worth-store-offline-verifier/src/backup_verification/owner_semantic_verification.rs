#[path = "owner_semantic_verification/admission.rs"]
mod admission;
#[path = "owner_semantic_verification/counters.rs"]
mod counters;
#[path = "owner_semantic_verification/outcome.rs"]
mod outcome;
#[path = "owner_semantic_verification/result_accumulation.rs"]
mod result_accumulation;
#[path = "owner_semantic_verification/result_closure.rs"]
mod result_closure;
#[path = "owner_semantic_verification/row_verification.rs"]
mod row_verification;

use std::path::Path;

use super::owner_media_read::OwnerMediaReadSession;
use super::BackupVerificationDefect;
use worth_store_physical_format::BackupBundleManifest;

pub(super) use counters::OwnerSemanticVerificationCounters;
pub(super) use outcome::{OwnerSemanticVerificationDenial, OwnerSemanticVerificationResult};

pub(super) fn verify_owner_semantics(
    root: &Path,
    manifest: &BackupBundleManifest,
    max_buffer_bytes: usize,
    maximum_owned_allocation_bytes: u64,
    defects: &mut Vec<BackupVerificationDefect>,
    media: &mut OwnerMediaReadSession,
) -> Result<OwnerSemanticVerificationResult, OwnerSemanticVerificationDenial> {
    let admission = admission::admit(
        root,
        manifest,
        max_buffer_bytes,
        maximum_owned_allocation_bytes,
    )?;
    let admission::OwnerAdmission {
        mut counters,
        mut recovery_candidates,
        mut owner_bindings,
        mut expected_root,
    } = admission;
    for row in manifest.artifacts() {
        media
            .reject_interruption()
            .map_err(OwnerSemanticVerificationDenial::Inspection)?;
        let Some(attempted) = counters.record_attempt() else {
            defects.push(BackupVerificationDefect::VerificationCounterOverflow);
            break;
        };
        counters = attempted;
        let path = root.join(row.output_name());
        let mut reader = match media.reader(&path) {
            Ok(reader) => reader,
            Err(worth_store_physical_backend::OfflineMediaReadDenial::InvalidFileIndex)
                if !path.exists() =>
            {
                // Structural comparison already recorded the absent component.
                continue;
            }
            Err(denial) => return Err(OwnerSemanticVerificationDenial::Media(denial)),
        };
        let actual_bytes = reader.length();
        let verification = row_verification::verify(
            &mut reader,
            actual_bytes,
            expected_root,
            root,
            row,
            max_buffer_bytes,
        );
        let bytes_read = reader
            .finish()
            .map_err(OwnerSemanticVerificationDenial::Inspection)?;
        counters =
            counters
                .record_read(bytes_read)
                .ok_or(OwnerSemanticVerificationDenial::Inspection(
                    crate::OfflineInspectionDenial::CounterOverflow,
                ))?;
        match verification {
            Ok(verified) => {
                counters = result_accumulation::record(
                    counters,
                    verified,
                    defects,
                    &mut recovery_candidates,
                    &mut owner_bindings,
                    &mut expected_root,
                );
            }
            Err(kind) => defects.push(BackupVerificationDefect::OwnerSemanticMismatch {
                output_name: row.output_name().to_owned(),
                format: row.format(),
                kind,
            }),
        }
    }
    result_closure::close(
        root,
        manifest,
        max_buffer_bytes,
        maximum_owned_allocation_bytes,
        counters,
        recovery_candidates,
        owner_bindings,
    )
}
