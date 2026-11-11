//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::basis::inline_type_budget::collect(census);
    super::domain_computation::inline_type_budget::collect(census);
    let _owner = census.owner::<worth_runtime_world::facade::ProductBranchObservation>(
        "crates/worth-runtime-world/src/branch/observation.rs",
        "ProductBranchObservation",
        128 * 1024,
    );
    let _owner = census.owner::<worth_runtime_world::facade::ProductBranchReferenceSnapshot>(
        "crates/worth-runtime-world/src/branch/reference_snapshot.rs",
        "ProductBranchReferenceSnapshot",
        128 * 1024,
    );
    let _owner = census.owner::<worth_runtime_world::facade::CompositeCommitIdentity>(
        "crates/worth-runtime-world/src/identity/commit.rs",
        "CompositeCommitIdentity",
        128 * 1024,
    );
    let _owner = census.owner::<worth_runtime_world::facade::RuntimeWorldOwnerUnavailable>(
        "crates/worth-runtime-world/src/lifecycle/ports.rs",
        "RuntimeWorldOwnerUnavailable",
        128 * 1024,
    );
    let _owner = census.owner::<worth_runtime_world::facade::RuntimeWorldCancellationToken>(
        "crates/worth-runtime-world/src/publication/cancellation.rs",
        "RuntimeWorldCancellationToken",
        128 * 1024,
    );
    let _owner = census.owner::<worth_runtime_world::facade::SignalComponentPlanPosture>(
        "crates/worth-runtime-world/src/publication/component_plan/signal.rs",
        "SignalComponentPlanPosture",
        128 * 1024,
    );
    let _owner = census.owner::<worth_runtime_world::facade::NoEffectCause>(
        "crates/worth-runtime-world/src/publication/no_effect.rs",
        "NoEffectCause",
        128 * 1024,
    );
    let _owner = census.owner::<worth_runtime_world::facade::SignalAttemptProgress>(
        "crates/worth-runtime-world/src/publication/progress.rs",
        "SignalAttemptProgress",
        128 * 1024,
    );
    let _owner = census.owner::<worth_runtime_world::facade::CompositeAttemptProgress>(
        "crates/worth-runtime-world/src/publication/progress.rs",
        "CompositeAttemptProgress",
        128 * 1024,
    );
    let _owner = census.owner::<worth_runtime_world::facade::ProductUnpublishedCause>(
        "crates/worth-runtime-world/src/recovery/product_unpublished/cause.rs",
        "ProductUnpublishedCause",
        128 * 1024,
    );
}
