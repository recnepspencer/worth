use worth_store_recovery_runtime::{
    PhysicalRecoveryOpenRequest, PhysicalRecoveryOutcome, WorthStoreRecovery,
};

pub(super) fn execute(
    request: PhysicalRecoveryOpenRequest,
    profile: super::arguments::BoundedProfile,
    yieldpoint: Option<worth_store::physical_runtime::PhysicalRecoveryProcessYieldpoint>,
) -> Result<PhysicalRecoveryOutcome, String> {
    // The two non-default profiles below are certification-only terminal
    // constructors. The fresh-process crash lanes use a named bounded release
    // profile; these profiles are never evidence of process death.
    match profile {
        super::arguments::BoundedProfile::PhaseTwoAdmission
        | super::arguments::BoundedProfile::FateCoverage => Ok(match yieldpoint {
            Some(yieldpoint) => {
                WorthStoreRecovery::recover_with_process_yieldpoint(request, yieldpoint)
            }
            None => WorthStoreRecovery::recover(request),
        }),
        super::arguments::BoundedProfile::Refused => {
            reject_yieldpoint(yieldpoint)?;
            refused(request)
        }
        super::arguments::BoundedProfile::PublicationIndeterminate => {
            reject_yieldpoint(yieldpoint)?;
            publication_indeterminate(request)
        }
    }
}

fn reject_yieldpoint(
    yieldpoint: Option<worth_store::physical_runtime::PhysicalRecoveryProcessYieldpoint>,
) -> Result<(), String> {
    if yieldpoint.is_some() {
        return Err("recovery yieldpoints require an ordinary recovery profile".to_owned());
    }
    Ok(())
}

fn refused(request: PhysicalRecoveryOpenRequest) -> Result<PhysicalRecoveryOutcome, String> {
    let admitted = request
        .admit()
        .map_err(|refusal| format!("refusal profile admission failed: {refusal:?}"))?;
    let discovered = admitted
        .discover()
        .map_err(|outcome| format!("refusal profile discovery failed: {outcome:?}"))?;
    let selected = discovered
        .select()
        .map_err(|outcome| format!("refusal profile selection failed: {outcome:?}"))?;
    Ok(selected.cancel_before_reconstruction())
}

#[cfg(feature = "certification-test-authority")]
fn publication_indeterminate(
    request: PhysicalRecoveryOpenRequest,
) -> Result<PhysicalRecoveryOutcome, String> {
    use worth_store::physical_runtime::PhysicalRecoveryPublicationCommandStage;

    let admitted = request
        .admit()
        .map_err(|refusal| format!("publication profile admission failed: {refusal:?}"))?;
    let discovered = admitted
        .discover()
        .map_err(|outcome| format!("publication profile discovery failed: {outcome:?}"))?;
    let selected = discovered
        .select()
        .map_err(|outcome| format!("publication profile selection failed: {outcome:?}"))?;
    let planned = selected
        .plan()
        .map_err(|outcome| format!("publication profile planning failed: {outcome:?}"))?;
    let staged = planned
        .stage()
        .map_err(|outcome| format!("publication profile staging failed: {outcome:?}"))?;
    staged.certification_fail_publication_scheduler_settlement_at(
        PhysicalRecoveryPublicationCommandStage::CandidateSynchronization,
    );
    match staged.publish() {
        Err(outcome) => Ok(outcome),
        Ok(_durable) => Err("publication profile unexpectedly completed".to_owned()),
    }
}

#[cfg(not(feature = "certification-test-authority"))]
fn publication_indeterminate(
    _request: PhysicalRecoveryOpenRequest,
) -> Result<PhysicalRecoveryOutcome, String> {
    Err("publication-indeterminate profile requires certification-test-authority".to_owned())
}
