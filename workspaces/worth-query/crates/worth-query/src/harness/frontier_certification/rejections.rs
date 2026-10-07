use crate::harness::fixtures::execution_preflights::{
    alternate_basis_ordered_collection_preflight, direct_runtime_preflight,
    ordered_collection_without_traversal_preflight,
};
use crate::live::promote_preflight_bundle_to_live;
use crate::planning::{
    lower_frontier_planning_bundle, FrontierPlanningError, FrontierPlanningInput,
};

use super::{FrontierCertificationRejection, FrontierFailureClass};

pub(super) fn unsupported_frontier_family_rejection() -> FrontierCertificationRejection {
    let error =
        crate::planning::lower_execution_preflight_to_frontier_plan(&direct_runtime_preflight())
            .expect_err("detail preflight must reject frontier admission");
    assert_eq!(error, FrontierPlanningError::UnsupportedFrontierFamily);
    FrontierCertificationRejection {
        failure_class: FrontierFailureClass::UnsupportedFrontierFamily,
        failure_digest: format!("unsupported_frontier_family:{error:?}"),
    }
}

pub(super) fn unsupported_bundle_composition_rejection() -> FrontierCertificationRejection {
    let preflight = ordered_collection_without_traversal_preflight();
    let live = promote_preflight_bundle_to_live(&preflight).expect("live promotion");
    let error = lower_frontier_planning_bundle(&[
        FrontierPlanningInput::from(preflight),
        FrontierPlanningInput::from(live),
    ])
    .expect_err("mixed preflight/live bundle must reject");
    assert_eq!(error, FrontierPlanningError::UnsupportedBundleComposition);
    FrontierCertificationRejection {
        failure_class: FrontierFailureClass::UnsupportedBundleComposition,
        failure_digest: format!("unsupported_bundle_composition:{error:?}"),
    }
}

pub(super) fn mixed_basis_bundle_rejection() -> FrontierCertificationRejection {
    let first = ordered_collection_without_traversal_preflight();
    let second = alternate_basis_ordered_collection_preflight();
    let expected_basis_digest = first.basis().proof().digest().as_str().to_string();
    let found_basis_digest = second.basis().proof().digest().as_str().to_string();
    let error = lower_frontier_planning_bundle(&[
        FrontierPlanningInput::from(first),
        FrontierPlanningInput::from(second),
    ])
    .expect_err("mixed-basis bundle must reject");
    match &error {
        FrontierPlanningError::MixedBasisBundle {
            expected_basis_digest: expected,
            found_basis_digest: found,
        } => {
            assert_eq!(expected.as_str(), expected_basis_digest);
            assert_eq!(found.as_str(), found_basis_digest);
            assert_ne!(expected, found);
        }
        other => panic!("expected mixed-basis denial, got {other:?}"),
    }
    FrontierCertificationRejection {
        failure_class: FrontierFailureClass::MixedBasisBundleDenied,
        failure_digest: format!("mixed_basis_bundle:{error:?}"),
    }
}
