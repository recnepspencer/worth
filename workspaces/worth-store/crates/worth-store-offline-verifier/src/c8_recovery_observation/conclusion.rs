#[path = "conclusion/artifacts.rs"]
mod artifacts;
#[path = "conclusion/evidence.rs"]
mod evidence;
#[path = "conclusion/model.rs"]
mod model;
#[path = "conclusion/reducer.rs"]
mod reducer;
#[path = "conclusion/selectors.rs"]
mod selectors;

pub(super) use model::RecoveryObserverConclusion;

pub(super) fn conclude(
    artifacts: &[super::artifact_walk::ObservedRecoveryArtifact],
) -> Result<RecoveryObserverConclusion, super::RecoveryObserverWalTopologyDenial> {
    reducer::conclude(artifacts)
}
