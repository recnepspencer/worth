mod authoring_values;
mod batch_lanes;
mod crud_lanes;
mod receipt_digest;

use crate::harness::certification::{
    CanonicalCertificationRow, HostileExpectation, ParityAnchor, RejectionCertificationRow,
};

use self::batch_lanes::{
    authoritative_batch_lane, mutation_surface_contract_lane, preview_batch_lane,
};
use self::crud_lanes::{authoritative_crud_lane, clear_lane};
use super::rejections::{duplicate_aspect_authoring_rejection, unsupported_intent_rejection};
use super::{
    description_value_touch, title_value_touch, AspectApiFinalizationCertificationBundle,
    AspectApiFinalizationPerturbationClass, AspectApiFinalizationRejectionBundle,
};

pub(super) fn canonical_rows(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Vec<
    CanonicalCertificationRow<
        AspectApiFinalizationPerturbationClass,
        AspectApiFinalizationCertificationBundle,
    >,
> {
    vec![
        CanonicalCertificationRow {
            row_name: "authoritative-insert-update-delete-surface",
            perturbation_class: AspectApiFinalizationPerturbationClass::AuthoritativeCrudSurface,
            hostile_expectation: HostileExpectation::EquivalentToControl,
            parity_anchor: ParityAnchor::Control,
            control_lane: authoritative_crud_lane(
                request_execution,
                "Buy milk",
                "Buy oat milk",
                "task-1",
            ),
            hostile_lane: authoritative_crud_lane(
                request_execution,
                "Read docs",
                "Read better docs",
                "task-2",
            ),
            parity_lane: authoritative_crud_lane(
                request_execution,
                "Pay bills",
                "Pay rent",
                "task-3",
            ),
        },
        CanonicalCertificationRow {
            row_name: "typed-clear-narrows-by-touched-meaning",
            perturbation_class: AspectApiFinalizationPerturbationClass::TypedClearNarrowing,
            hostile_expectation: HostileExpectation::DistinctFromControl,
            parity_anchor: ParityAnchor::Hostile,
            control_lane: clear_lane(request_execution, description_value_touch()),
            hostile_lane: clear_lane(request_execution, title_value_touch()),
            parity_lane: clear_lane(request_execution, title_value_touch()),
        },
        CanonicalCertificationRow {
            row_name: "preview-batch-lane-isolation",
            perturbation_class: AspectApiFinalizationPerturbationClass::PreviewBatchIsolation,
            hostile_expectation: HostileExpectation::DistinctFromControl,
            parity_anchor: ParityAnchor::Hostile,
            control_lane: authoritative_batch_lane(request_execution),
            hostile_lane: preview_batch_lane(request_execution),
            parity_lane: preview_batch_lane(request_execution),
        },
        CanonicalCertificationRow {
            row_name: "mutation-surface-closeout-contract-sync",
            perturbation_class: AspectApiFinalizationPerturbationClass::MutationSurfaceCloseoutSync,
            hostile_expectation: HostileExpectation::EquivalentToControl,
            parity_anchor: ParityAnchor::Control,
            control_lane: mutation_surface_contract_lane(request_execution),
            hostile_lane: mutation_surface_contract_lane(request_execution),
            parity_lane: mutation_surface_contract_lane(request_execution),
        },
    ]
}

pub(super) fn rejection_rows(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Vec<
    RejectionCertificationRow<
        AspectApiFinalizationPerturbationClass,
        AspectApiFinalizationCertificationBundle,
        AspectApiFinalizationRejectionBundle,
    >,
> {
    vec![
        RejectionCertificationRow {
            row_name: "unsupported-intent-family-fails-typed-and-early",
            perturbation_class:
                AspectApiFinalizationPerturbationClass::UnsupportedIntentFamilyDenied,
            control_lane: mutation_surface_contract_lane(request_execution),
            hostile_lane: unsupported_intent_rejection(request_execution),
            parity_lane: mutation_surface_contract_lane(request_execution),
        },
        RejectionCertificationRow {
            row_name: "duplicate-clear-and-set-denied-before-routing",
            perturbation_class:
                AspectApiFinalizationPerturbationClass::DuplicateAspectAuthoringDenied,
            control_lane: mutation_surface_contract_lane(request_execution),
            hostile_lane: duplicate_aspect_authoring_rejection(request_execution),
            parity_lane: mutation_surface_contract_lane(request_execution),
        },
    ]
}
