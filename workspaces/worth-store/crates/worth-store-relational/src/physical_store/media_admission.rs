//! Normal media qualification only after the lease holder chose initialization.
use super::*;
use worth_store::physical_runtime::{
    AdmittedPhysicalRuntime, MediaAdmissionOutcome, QualifiedRecoveryFilesystemMedia,
};

pub(super) fn admit_for_initialization(
    root: &Path,
    held: QualifiedRecoveryFilesystemMedia,
    admission: FilesystemMediaAdmission,
) -> Result<MediaOwnedPhysicalRuntime, Refusal> {
    finish(runtime(root)?.try_admit_initializing_home_media(held, admission))
}

#[cfg(test)]
pub(super) fn admit_fresh(
    root: &Path,
    admission: FilesystemMediaAdmission,
) -> Result<MediaOwnedPhysicalRuntime, Refusal> {
    finish(runtime(root)?.try_admit_filesystem_media(admission))
}

fn runtime(root: &Path) -> Result<AdmittedPhysicalRuntime, Refusal> {
    let admission = PhysicalRuntimeAdmission::new(root)
        .map_err(|error| Refusal::new(Stage::RuntimeAdmission, format!("{error:?}")))?;
    PhysicalStore::admit(admission)
        .map_err(|error| Refusal::new(Stage::RuntimeAdmission, format!("{error:?}")))
}

fn finish(outcome: MediaAdmissionOutcome) -> Result<MediaOwnedPhysicalRuntime, Refusal> {
    match outcome.into_raw() {
        TransitionOutcome::Success(media) => Ok(media),
        TransitionOutcome::Denied(denial) => {
            let reason = format!("{:?}", denial.reason());
            denial.into_runtime().close();
            Err(Refusal::new(Stage::MediaAdmission, reason))
        }
        TransitionOutcome::Deferred(deferred) => {
            let detail = format!("{:?}", deferred.reason());
            deferred.into_runtime().close();
            let mut refusal = Refusal::new(Stage::MediaAdmission, detail);
            refusal.kind = RelationalStorePhysicalRefusalKind::MutationOwnerContended;
            Err(refusal)
        }
        TransitionOutcome::Failed(failure) => {
            let mut refusal = Refusal::new(Stage::MediaAdmission, format!("{:?}", failure.cause()));
            if failure.mutation_owner_contended() {
                refusal.kind = RelationalStorePhysicalRefusalKind::MutationOwnerContended;
            }
            Err(refusal)
        }
        other => Err(Refusal::new(
            Stage::MediaAdmission,
            unfinished_outcome(&other),
        )),
    }
}
