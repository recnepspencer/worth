#[path = "representatives.rs"]
pub(crate) mod representatives;

use super::compile_fail::{CompileFailBundle, CompileFailCase};
use super::proof_shapes::{BasisDigest, ProofShapeDigest};
use super::type_shapes::{
    CodegenHonestyReport, CodegenShapeCheck, DebtInventory, DebtItem, TypeShapeCheck,
    TypeShapeReport,
};
use std::any::type_name;
use std::mem::{align_of, needs_drop, size_of};

pub use representatives::RawPhase;
use worth_proof::{
    Artifact, ArtifactParts, ArtifactView, AssumptionBasis, CanonicalOrder, CanonicalVec,
    DisjointPair, ExactlyOne, NoAssumptionBasis, NoProofs, NonEmpty, Pair, Proof,
    StructuralProofAuthority, UniqueVec,
};

pub fn compile_fail_bundle() -> CompileFailBundle {
    let cases = vec![
        CompileFailCase::new(
            "phase_boundaries",
            "tests/ui/milestone1/raw_artifact_cannot_satisfy_validated_api.rs",
        ),
        CompileFailCase::new(
            "proven_collection_boundaries",
            "tests/ui/milestone1/raw_collections_cannot_satisfy_proven_apis.rs",
        ),
        CompileFailCase::new(
            "fixed_shape_boundaries",
            "tests/ui/milestone1/raw_fixed_shapes_cannot_satisfy_fixed_shape_apis.rs",
        ),
    ];
    CompileFailBundle::new("core_artifact_and_proof_substrate", cases)
}

pub fn type_shape_report(checks: Vec<TypeShapeCheck>) -> TypeShapeReport {
    TypeShapeReport::new("core_artifact_and_proof_substrate", checks)
}

pub fn derive_type_shape_report() -> TypeShapeReport {
    type_shape_report(vec![
        TypeShapeCheck::new(
            "payload_only_artifact",
            size_of::<Artifact<RawPhase, u64, NoProofs, NoAssumptionBasis>>(),
            size_of::<u64>(),
        ),
        TypeShapeCheck::new(
            "zero_sized_proof_artifact",
            size_of::<
                Artifact<
                    RawPhase,
                    u64,
                    Proof<CanonicalOrder, StructuralProofAuthority>,
                    NoAssumptionBasis,
                >,
            >(),
            size_of::<u64>(),
        ),
        TypeShapeCheck::new(
            "assumption_bearing_artifact",
            size_of::<
                Artifact<
                    RawPhase,
                    u64,
                    Proof<CanonicalOrder, StructuralProofAuthority>,
                    AssumptionBasis<u32>,
                >,
            >(),
            size_of::<(u64, AssumptionBasis<u32>)>(),
        ),
        TypeShapeCheck::new(
            "artifact_view",
            size_of::<
                ArtifactView<
                    'static,
                    RawPhase,
                    u64,
                    Proof<CanonicalOrder, StructuralProofAuthority>,
                    AssumptionBasis<u32>,
                >,
            >(),
            size_of::<(
                &'static u64,
                &'static Proof<CanonicalOrder, StructuralProofAuthority>,
                &'static AssumptionBasis<u32>,
            )>(),
        ),
        TypeShapeCheck::new(
            "artifact_parts",
            size_of::<
                ArtifactParts<
                    u64,
                    Proof<CanonicalOrder, StructuralProofAuthority>,
                    AssumptionBasis<u32>,
                >,
            >(),
            size_of::<(
                u64,
                Proof<CanonicalOrder, StructuralProofAuthority>,
                AssumptionBasis<u32>,
            )>(),
        ),
        TypeShapeCheck::new(
            "canonical_vec",
            size_of::<CanonicalVec<u64>>(),
            size_of::<Vec<u64>>(),
        ),
        TypeShapeCheck::new(
            "unique_vec",
            size_of::<UniqueVec<u64>>(),
            size_of::<Vec<u64>>(),
        ),
        TypeShapeCheck::new(
            "disjoint_pair",
            size_of::<DisjointPair<u64>>(),
            size_of::<Pair<u64>>(),
        ),
    ])
}

pub fn proof_shape_digest() -> ProofShapeDigest {
    ProofShapeDigest::new(
        "core_artifact_and_proof_substrate",
        vec![
            type_name::<Artifact<RawPhase, u64, NoProofs, NoAssumptionBasis>>(),
            type_name::<
                Artifact<
                    RawPhase,
                    u64,
                    Proof<CanonicalOrder, StructuralProofAuthority>,
                    AssumptionBasis<u32>,
                >,
            >(),
            type_name::<
                ArtifactView<
                    'static,
                    RawPhase,
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
        ],
    )
}

pub fn basis_digest() -> BasisDigest {
    BasisDigest::new(
        "core_artifact_and_proof_substrate",
        vec![
            type_name::<NoAssumptionBasis>(),
            type_name::<AssumptionBasis<u32>>(),
        ],
    )
}

pub fn codegen_honesty_report() -> CodegenHonestyReport {
    CodegenHonestyReport::size_layout_and_drop_certified(
        "core_artifact_and_proof_substrate",
        vec![
            CodegenShapeCheck::new(
                "artifact",
                align_of::<Artifact<RawPhase, u64, NoProofs, NoAssumptionBasis>>(),
                align_of::<u64>(),
                needs_drop::<Artifact<RawPhase, u64, NoProofs, NoAssumptionBasis>>(),
                needs_drop::<u64>(),
            ),
            CodegenShapeCheck::new(
                "artifact_with_basis",
                align_of::<Artifact<RawPhase, u64, Proof<CanonicalOrder, StructuralProofAuthority>, AssumptionBasis<u32>>>(),
                align_of::<(u64, AssumptionBasis<u32>)>(),
                needs_drop::<
                    Artifact<RawPhase, u64, Proof<CanonicalOrder, StructuralProofAuthority>, AssumptionBasis<u32>>,
                >(),
                needs_drop::<(u64, AssumptionBasis<u32>)>(),
            ),
            CodegenShapeCheck::new(
                "artifact_view",
                align_of::<
                    ArtifactView<'static, RawPhase, u64, Proof<CanonicalOrder, StructuralProofAuthority>, AssumptionBasis<u32>>,
                >(),
                align_of::<(
                    &'static u64,
                    &'static Proof<CanonicalOrder, StructuralProofAuthority>,
                    &'static AssumptionBasis<u32>,
                )>(),
                needs_drop::<
                    ArtifactView<'static, RawPhase, u64, Proof<CanonicalOrder, StructuralProofAuthority>, AssumptionBasis<u32>>,
                >(),
                needs_drop::<(
                    &'static u64,
                    &'static Proof<CanonicalOrder, StructuralProofAuthority>,
                    &'static AssumptionBasis<u32>,
                )>(),
            ),
            CodegenShapeCheck::new(
                "artifact_parts",
                align_of::<ArtifactParts<u64, Proof<CanonicalOrder, StructuralProofAuthority>, AssumptionBasis<u32>>>(),
                align_of::<(u64, Proof<CanonicalOrder, StructuralProofAuthority>, AssumptionBasis<u32>)>(),
                needs_drop::<ArtifactParts<u64, Proof<CanonicalOrder, StructuralProofAuthority>, AssumptionBasis<u32>>>(),
                needs_drop::<(u64, Proof<CanonicalOrder, StructuralProofAuthority>, AssumptionBasis<u32>)>(),
            ),
            CodegenShapeCheck::new(
                "canonical_vec",
                align_of::<CanonicalVec<u64>>(),
                align_of::<Vec<u64>>(),
                needs_drop::<CanonicalVec<u64>>(),
                needs_drop::<Vec<u64>>(),
            ),
            CodegenShapeCheck::new(
                "unique_vec",
                align_of::<UniqueVec<u64>>(),
                align_of::<Vec<u64>>(),
                needs_drop::<UniqueVec<u64>>(),
                needs_drop::<Vec<u64>>(),
            ),
            CodegenShapeCheck::new(
                "disjoint_pair",
                align_of::<DisjointPair<u64>>(),
                align_of::<Pair<u64>>(),
                needs_drop::<DisjointPair<u64>>(),
                needs_drop::<Pair<u64>>(),
            ),
            CodegenShapeCheck::new(
                "non_empty",
                align_of::<NonEmpty<u64>>(),
                align_of::<Vec<u64>>(),
                needs_drop::<NonEmpty<u64>>(),
                needs_drop::<Vec<u64>>(),
            ),
            CodegenShapeCheck::new(
                "exactly_one",
                align_of::<ExactlyOne<u64>>(),
                align_of::<u64>(),
                needs_drop::<ExactlyOne<u64>>(),
                needs_drop::<u64>(),
            ),
        ],
        "Milestone 1 certifies representative size/layout/drop honesty only; no MIR or ASM baseline diff is shipped yet.",
    )
}

pub fn debt_inventory() -> DebtInventory {
    DebtInventory::new(
        "core_artifact_and_proof_substrate",
        vec![
            DebtItem::new(
                "codegen_honesty",
                "Representative size/layout lanes are certified, but no handwritten baseline diff report exists yet.",
            ),
            DebtItem::new(
                "freshness_topology",
                "`assumption/freshness.rs` remains deferred until Milestone 3 makes the seam load-bearing.",
            ),
        ],
    )
}
