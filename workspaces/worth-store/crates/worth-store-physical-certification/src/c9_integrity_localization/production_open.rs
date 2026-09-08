use super::production_profile::ProductionWorldProfile;
use std::path::Path;
use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, PhysicalRecordAccessPolicy, PhysicalRecordFormatDeclaration,
    PhysicalRecordOpen, PhysicalRuntimeAdmission, PhysicalStore, ServingPhysicalRuntime,
};

pub(super) fn open(
    root: &Path,
    profile: ProductionWorldProfile,
) -> Result<ServingPhysicalRuntime, String> {
    let runtime = PhysicalStore::admit(
        PhysicalRuntimeAdmission::new(root)
            .map_err(|error| format!("open runtime declaration: {error:?}"))?,
    )
    .map_err(|error| format!("open runtime: {error:?}"))?;
    let media = super::production_store::admit_media(runtime)?;
    let durability = super::production_store::admit_durability(&media)?;
    let format = AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder()
            .page_size(profile.page_size())
            .admit()
            .unwrap(),
    );
    let access = PhysicalRecordAccessPolicy::builder().admit(format).unwrap();
    let request = PhysicalRecordOpen::new(format, access, durability)
        .with_residency_policy(profile.residency(format));
    match media.open_record_store(request).into_raw() {
        TransitionOutcome::Success(serving) => Ok(serving),
        TransitionOutcome::Denied(denial) => Err(format!(
            "ordinary clean Store reopen denied: {:?}",
            denial.reason()
        )),
        TransitionOutcome::Stale(denial) => Err(format!(
            "ordinary clean Store reopen stale: {:?}",
            denial.reason()
        )),
        TransitionOutcome::RebindRequired(denial) => Err(format!(
            "ordinary clean Store reopen rebind: {:?}",
            denial.reason()
        )),
        TransitionOutcome::Failed(denial) => Err(format!(
            "ordinary clean Store reopen inspection: {:?}",
            denial.cause()
        )),
        TransitionOutcome::Deferred(never) => match never {},
    }
}
