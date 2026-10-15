#[allow(unused_imports)]
use crate::shared_support as support;

use std::any::type_name;

use support::core_artifact;
use support::type_shapes::TypeShapeCheck;
use worth_proof::{
    Artifact, ArtifactParts, ArtifactView, AssumptionBasis, CanonicalOrder, CanonicalVec,
    DisjointPair, NoAssumptionBasis, NoProofs, Proof, StructuralProofAuthority, UniqueVec,
};

#[test]
fn milestone_1_evidence_bundle_is_machine_checkable() {
    let type_shape_report = core_artifact::derive_type_shape_report();
    let compile_fail_bundle = core_artifact::compile_fail_bundle();
    let proof_shape_digest = core_artifact::proof_shape_digest();
    let basis_digest = core_artifact::basis_digest();
    let codegen_honesty_report = core_artifact::codegen_honesty_report();
    let debt_inventory = core_artifact::debt_inventory();

    assert_eq!(
        type_shape_report.suite(),
        "core_artifact_and_proof_substrate"
    );
    assert_eq!(
        type_shape_report
            .checks()
            .iter()
            .map(|check| check.lane())
            .collect::<Vec<_>>(),
        vec![
            "payload_only_artifact",
            "zero_sized_proof_artifact",
            "assumption_bearing_artifact",
            "artifact_view",
            "artifact_parts",
            "canonical_vec",
            "unique_vec",
            "disjoint_pair",
        ]
    );
    assert!(type_shape_report
        .checks()
        .iter()
        .all(TypeShapeCheck::matches));

    assert_eq!(
        compile_fail_bundle.suite(),
        "core_artifact_and_proof_substrate"
    );
    assert_eq!(compile_fail_bundle.cases().len(), 3);
    assert_eq!(
        compile_fail_bundle.families(),
        vec![
            "phase_boundaries",
            "proven_collection_boundaries",
            "fixed_shape_boundaries",
        ]
    );
    assert_eq!(
        compile_fail_bundle
            .cases()
            .iter()
            .map(|case| (case.family(), case.path()))
            .collect::<Vec<_>>(),
        vec![
            (
                "phase_boundaries",
                "tests/ui/milestone1/raw_artifact_cannot_satisfy_validated_api.rs",
            ),
            (
                "proven_collection_boundaries",
                "tests/ui/milestone1/raw_collections_cannot_satisfy_proven_apis.rs",
            ),
            (
                "fixed_shape_boundaries",
                "tests/ui/milestone1/raw_fixed_shapes_cannot_satisfy_fixed_shape_apis.rs",
            ),
        ]
    );

    assert_eq!(
        proof_shape_digest.suite(),
        "core_artifact_and_proof_substrate"
    );
    assert_eq!(
        proof_shape_digest.entries(),
        [
            type_name::<Artifact<core_artifact::RawPhase, u64, NoProofs, NoAssumptionBasis>>(),
            type_name::<
                Artifact<
                    core_artifact::RawPhase,
                    u64,
                    Proof<CanonicalOrder, StructuralProofAuthority>,
                    AssumptionBasis<u32>,
                >,
            >(),
            type_name::<
                ArtifactView<
                    'static,
                    core_artifact::RawPhase,
                    u64,
                    Proof<CanonicalOrder, StructuralProofAuthority>,
                    AssumptionBasis<u32>,
                >,
            >(),
            type_name::<
                ArtifactParts<
                    u64,
                    Proof<CanonicalOrder, StructuralProofAuthority>,
                    AssumptionBasis<u32>,
                >,
            >(),
            type_name::<CanonicalVec<u64>>(),
            type_name::<UniqueVec<u64>>(),
            type_name::<DisjointPair<u64>>(),
        ]
    );

    assert_eq!(basis_digest.suite(), "core_artifact_and_proof_substrate");
    assert_eq!(
        basis_digest.entries(),
        [
            type_name::<NoAssumptionBasis>(),
            type_name::<AssumptionBasis<u32>>()
        ]
    );

    assert_eq!(
        codegen_honesty_report.verified_scope(),
        "size_layout_and_drop_only"
    );
    assert_eq!(
        codegen_honesty_report.suite(),
        "core_artifact_and_proof_substrate"
    );
    assert_eq!(
        codegen_honesty_report
            .checks()
            .iter()
            .map(|check| check.lane())
            .collect::<Vec<_>>(),
        vec![
            "artifact",
            "artifact_with_basis",
            "artifact_view",
            "artifact_parts",
            "canonical_vec",
            "unique_vec",
            "disjoint_pair",
            "non_empty",
            "exactly_one",
        ]
    );
    assert!(codegen_honesty_report
        .checks()
        .iter()
        .all(|check| check.matches()));
    assert!(!codegen_honesty_report.hidden_dynamic_lookup());
    assert!(!codegen_honesty_report.hidden_virtual_dispatch());
    assert!(!codegen_honesty_report.mandatory_allocation_introduced());

    assert_eq!(debt_inventory.suite(), "core_artifact_and_proof_substrate");
    assert_eq!(debt_inventory.items().len(), 2);
    assert_eq!(
        debt_inventory
            .items()
            .iter()
            .map(|item| item.category())
            .collect::<Vec<_>>(),
        vec!["codegen_honesty", "freshness_topology"]
    );
    assert_eq!(
        debt_inventory
            .items()
            .iter()
            .map(|item| item.debt())
            .collect::<Vec<_>>(),
        vec![
            "Representative size/layout lanes are certified, but no handwritten baseline diff report exists yet.",
            "`assumption/freshness.rs` remains deferred until Milestone 3 makes the seam load-bearing.",
        ]
    );
}
