//! Fresh ordinary admission consumes the poisoned root protocol itself.
use super::{artifact_inventory::ArtifactInventory, artifact_process::ArtifactRequest};
use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    PhysicalRootProtocolRoute as Route, RecordBootstrapDenial as Denial,
};

pub(super) fn require(request: &ArtifactRequest, inventory: &ArtifactInventory) {
    let family = request.poison.map(|index| inventory.granules[index].family);
    let expected = match family {
        Some("bootstrap_catalog") => Some(Denial::CatalogDamaged),
        Some("root_manifest") => Some(Denial::CurrentRootDamaged),
        Some("free_space_header") => Some(Denial::FreeSpaceManifestDamaged),
        // Ordinary open follows the bootstrap's root, not C8's selector files.
        Some("current_root_selector" | "previous_root_selector") | None => None,
        _ => unreachable!("ordinary-open families selected before dispatch"),
    };
    match super::production_open::outcome(&request.root, request.profile)
        .unwrap()
        .into_raw()
    {
        TransitionOutcome::Success(serving) => {
            assert!(
                expected.is_none(),
                "ordinary open admitted poisoned {family:?}"
            );
            assert_eq!(
                serving
                    .root_protocol_counters()
                    .selector_entries(Route::OrdinaryOpen),
                0
            );
            assert_eq!(
                serving
                    .root_protocol_counters()
                    .root_entries(Route::OrdinaryOpen),
                2
            );
            serving.abort();
        }
        TransitionOutcome::Denied(denial) => {
            assert!(
                !(request.operator == super::artifact_edit::ArtifactOperator::ScopeSubstitution
                    && matches!(family, Some("free_space_header" | "bootstrap_catalog")))
            );
            assert_eq!(Some(denial.reason()), expected, "ordinary open {family:?}");
            let runtime = denial.into_runtime();
            assert_eq!(
                runtime
                    .root_protocol_counters()
                    .selector_entries(Route::OrdinaryOpen),
                0
            );
            assert_eq!(
                runtime
                    .root_protocol_counters()
                    .root_entries(Route::OrdinaryOpen),
                if family == Some("free_space_header") {
                    2
                } else {
                    0
                },
                "only admitted prerequisite roots may enter their owner projection"
            );
            runtime.abort();
        }
        TransitionOutcome::Stale(denial) => {
            assert_eq!(family, Some("free_space_header"));
            assert_eq!(
                request.operator,
                super::artifact_edit::ArtifactOperator::ScopeSubstitution
            );
            assert_eq!(denial.reason(), worth_store::physical_runtime::RecordServingStaleReason::FreeSpaceGenerationMismatch);
            let runtime = denial.into_runtime();
            assert_eq!(
                runtime
                    .root_protocol_counters()
                    .root_entries(Route::OrdinaryOpen),
                2
            );
            runtime.abort();
        }
        TransitionOutcome::RebindRequired(denial) => {
            assert_eq!(family, Some("bootstrap_catalog"));
            assert_eq!(
                request.operator,
                super::artifact_edit::ArtifactOperator::ScopeSubstitution
            );
            assert_eq!(
                denial.reason(),
                worth_store::physical_runtime::RecordServingRebindReason::StoreIdentityMismatch
            );
            let runtime = denial.into_runtime();
            assert_eq!(
                runtime
                    .root_protocol_counters()
                    .root_entries(Route::OrdinaryOpen),
                0
            );
            runtime.abort();
        }
        _ => panic!(
            "ordinary open must return its exact domain denial, not fail admission elsewhere"
        ),
    }
}
