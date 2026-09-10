use worth_query::facade::domain;

use super::installed_operation_fixture::{
    configured_runtime, support_dimension_workspace, workspace, GeometryDomain, ReadFamily,
    ReadVertex,
};

#[test]
fn bound_projection_mints_one_query_owned_support_contract() {
    let workspace = workspace("consumer-support", false).unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, ReadVertex)
        .unwrap();
    let contract = bound.consumer_projection_contract().unwrap();
    assert_eq!(contract.binding_identity(), bound.binding_identity());
    assert_eq!(contract.basis_identity(), bound.basis_identity());
    assert_eq!(
        contract.canonical_operation_identity(),
        bound.definition().canonical_identity()
    );
    assert_eq!(
        contract.canonical_projection(),
        &bound.definition().semantics().canonical_query
    );
    assert_eq!(
        contract.collection(),
        &bound.definition().semantics().collection
    );
    assert_eq!(contract.replay(), &bound.definition().semantics().replay);
    assert_eq!(
        contract.aftermath(),
        bound.definition().semantics().aftermath.as_ref()
    );
    assert_eq!(contract.lineage(), bound.definition().semantics().lineage);
    assert_eq!(
        contract.promotion(),
        &bound.definition().semantics().promotion
    );
    assert_eq!(
        contract.requirement(domain::WorthQueryConsumerSupportDimension::ProjectionConsumption),
        domain::WorthQuerySupportRequirement::Required
    );
    assert_eq!(
        contract.support_posture(domain::WorthQueryConsumerSupportDimension::ProjectionConsumption),
        domain::WorthQueryConsumerSupportPosture::Supported
    );
    assert_eq!(contract.counters().dimensions_evaluated, 15);
    assert_eq!(contract.counters().installation_generation_checks, 1);
    assert_eq!(contract.counters().mint_guard_checks, 1);
    assert_eq!(contract.counters().reporting_digest_comparisons, 0);
    assert_eq!(contract.counters().downstream_hook_inspections, 0);
    let denial = match bound.consumer_projection_contract() {
        Ok(_) => panic!("a bound capability must not mint a second consumer contract"),
        Err(denial) => denial,
    };
    assert!(matches!(
        denial,
        domain::WorthQueryConsumerProjectionContractDenial::AlreadyMinted { .. }
    ));
    assert_eq!(denial.counters().installation_generation_checks, 1);
    assert_eq!(denial.counters().mint_guard_checks, 1);
    assert_eq!(denial.counters().dimensions_evaluated, 0);
    let boundary =
        contract.with_downstream_requirements(domain::WorthQueryConsumerBoundaryRequirements {
            presentation: domain::WorthQueryConsumerPresentationPosture::Interactive,
            allocation: domain::WorthQueryConsumerAllocationPosture::Owned,
        });
    assert_eq!(
        boundary
            .query_contract()
            .requirement(domain::WorthQueryConsumerSupportDimension::ProjectionConsumption),
        domain::WorthQuerySupportRequirement::Required
    );
    assert_eq!(
        boundary.downstream_requirements().presentation,
        domain::WorthQueryConsumerPresentationPosture::Interactive
    );
}

#[test]
fn runtime_support_truth_drifts_to_a_dimension_specific_query_denial() {
    for dimension in domain::WorthQueryConsumerSupportDimension::ALL {
        let workspace = support_dimension_workspace(
            &format!("consumer-support-drift-{dimension:?}"),
            dimension,
            domain::WorthQueryConsumerSupportPosture::Unsupported,
        )
        .unwrap();
        let installed_domain = workspace.domain(GeometryDomain).unwrap();
        let bound = workspace
            .observe_operating_world(workspace.current_world())
            .unwrap()
            .family(ReadFamily)
            .bind(&installed_domain, ReadVertex)
            .unwrap();
        let denial = match bound.consumer_projection_contract() {
            Ok(_) => panic!("required unsupported {dimension:?} must deny"),
            Err(domain::WorthQueryConsumerProjectionContractDenial::Compatibility(denial)) => {
                denial
            }
            Err(other) => panic!("unexpected contract denial: {other:?}"),
        };
        assert_eq!(denial.dimension(), dimension);
        assert_eq!(
            denial.runtime_posture(),
            domain::WorthQueryConsumerSupportPosture::Unsupported
        );
        assert_eq!(denial.counters().reporting_digest_comparisons, 0);
        assert_eq!(denial.counters().downstream_hook_inspections, 0);
    }
}

#[test]
fn basis_support_drift_denies_before_other_dimension_admission() {
    let workspace = configured_runtime()
        .consumer_support_posture(
            domain::WorthQueryConsumerSupportDimension::Basis,
            domain::WorthQueryConsumerSupportPosture::Unsupported,
        )
        .workspace("consumer-basis-support-drift")
        .unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, ReadVertex)
        .unwrap();
    let denial = match bound.consumer_projection_contract() {
        Ok(_) => panic!("unsupported exact basis support must deny"),
        Err(domain::WorthQueryConsumerProjectionContractDenial::Compatibility(denial)) => denial,
        Err(other) => panic!("unexpected contract denial: {other:?}"),
    };

    assert_eq!(
        denial.dimension(),
        domain::WorthQueryConsumerSupportDimension::Basis
    );
    assert_eq!(denial.counters().dimensions_evaluated, 1);
    assert_eq!(denial.counters().downstream_hook_inspections, 0);
}

#[test]
fn equivalent_runtime_paths_and_rebuilt_indexes_preserve_support_truth() {
    let direct = workspace("consumer-support-convergence-direct", false).unwrap();
    let rebuilt = workspace("consumer-support-convergence-rebuilt", true).unwrap();
    assert!(rebuilt
        .verify_domain_execution_index_rebuild()
        .is_equivalent());
    let direct_domain = direct.domain(GeometryDomain).unwrap();
    let rebuilt_domain = rebuilt.domain(GeometryDomain).unwrap();
    let direct_bound = direct
        .observe_operating_world(direct.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&direct_domain, ReadVertex)
        .unwrap();
    let rebuilt_bound = rebuilt
        .observe_operating_world(rebuilt.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&rebuilt_domain, ReadVertex)
        .unwrap();
    let direct_contract = direct_bound.consumer_projection_contract().unwrap();
    let rebuilt_contract = rebuilt_bound.consumer_projection_contract().unwrap();
    assert_eq!(
        direct_contract.operation_identity(),
        rebuilt_contract.operation_identity()
    );
    assert_eq!(
        direct_contract.native_projection(),
        rebuilt_contract.native_projection()
    );
    assert_eq!(
        direct_contract.publication(),
        rebuilt_contract.publication()
    );
    assert_eq!(direct_contract.terminal(), rebuilt_contract.terminal());
    assert_eq!(direct_contract.counters(), rebuilt_contract.counters());
    for dimension in domain::WorthQueryConsumerSupportDimension::ALL {
        assert_eq!(
            direct_contract.requirement(dimension),
            rebuilt_contract.requirement(dimension)
        );
        assert_eq!(
            direct_contract.support_posture(dimension),
            rebuilt_contract.support_posture(dimension)
        );
    }
}

#[test]
fn foundational_support_projection_preserves_exact_descriptive_support() {
    let workspace = configured_runtime()
        .consumer_support_posture(
            domain::WorthQueryConsumerSupportDimension::ConditionalEvaluation,
            domain::WorthQueryConsumerSupportPosture::Deferred,
        )
        .workspace("consumer-foundational-support-projection")
        .unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, ReadVertex)
        .unwrap();
    let contract = bound.consumer_projection_contract().unwrap();
    let artifact = contract.foundational_support_projection().unwrap();
    let projection = artifact.payload().payload();

    assert_eq!(projection.binding_identity(), contract.binding_identity());
    assert_eq!(projection.basis_identity(), contract.basis_identity());
    assert_eq!(
        projection.freshness(),
        domain::WorthQueryConsumerSupportBoundaryFreshness::Current
    );
    assert_eq!(
        projection.availability(),
        domain::WorthQueryConsumerSupportBoundaryAvailability::RequiredDimensionsAvailable
    );
    assert_eq!(
        projection.degradation(),
        domain::WorthQueryConsumerSupportBoundaryDegradation::UnrequiredDimensionsDeferred
    );
    assert!(projection.rows().iter().all(|row| {
        row.requirement == contract.requirement(row.dimension)
            && row.posture == contract.support_posture(row.dimension)
    }));
    assert_eq!(
        artifact
            .payload()
            .profile()
            .materialized()
            .support_posture(),
        worth_foundational::facade::SupportPostureProfile::SupportReady
    );
}
