

pub(crate) use {crate::shared_support::compile_fail::bundle::CompileFailBundle, crate::shared_support::compile_fail::bundle::CompileFailCase};
pub(crate) use {crate::shared_support::proof_shapes::digest::BasisDigest, crate::shared_support::proof_shapes::digest::ProofShapeDigest};
pub(crate) use {crate::shared_support::type_shapes::codegen::CodegenHonestyReport, crate::shared_support::type_shapes::codegen::CodegenShapeCheck, crate::shared_support::type_shapes::debt::DebtInventory, crate::shared_support::type_shapes::debt::DebtItem, crate::shared_support::type_shapes::type_shape::TypeShapeCheck, crate::shared_support::type_shapes::type_shape::TypeShapeReport};
pub(crate) use {std::any::type_name};
pub(crate) use {std::mem::align_of, std::mem::needs_drop, std::mem::size_of};

pub(crate) use {crate::shared_support::core_artifact::representatives::RawPhase};
pub(crate) use {worth_proof::Artifact, worth_proof::ArtifactParts, worth_proof::ArtifactView, worth_proof::AssumptionBasis, worth_proof::CanonicalOrder, worth_proof::CanonicalVec, worth_proof::DisjointPair, worth_proof::ExactlyOne, worth_proof::NoAssumptionBasis, worth_proof::NoProofs, worth_proof::NonEmpty, worth_proof::Pair, worth_proof::Proof, worth_proof::StructuralProofAuthority, worth_proof::UniqueVec};

pub(crate) fn compile_fail_bundle() -> CompileFailBundle {
    let cases = vec![
        crate::shared_support::compile_fail::bundle::CompileFailCase::new(
            "phase_boundaries",
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_proof/sources/worth_proof/ui/milestone1/raw_artifact_cannot_satisfy_validated_api.rs"),
        ),
        crate::shared_support::compile_fail::bundle::CompileFailCase::new(
            "proven_collection_boundaries",
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_proof/sources/worth_proof/ui/milestone1/raw_collections_cannot_satisfy_proven_apis.rs"),
        ),
        crate::shared_support::compile_fail::bundle::CompileFailCase::new(
            "fixed_shape_boundaries",
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_proof/sources/worth_proof/ui/milestone1/raw_fixed_shapes_cannot_satisfy_fixed_shape_apis.rs"),
        ),
    ];
    crate::shared_support::compile_fail::bundle::CompileFailBundle::new("core_artifact_and_proof_substrate", cases)
}

pub(crate) fn type_shape_report(checks: Vec<TypeShapeCheck>) -> TypeShapeReport {
    crate::shared_support::type_shapes::type_shape::TypeShapeReport::new("core_artifact_and_proof_substrate", checks)
}

pub(crate) fn derive_type_shape_report() -> TypeShapeReport {
    type_shape_report(vec![
        crate::shared_support::type_shapes::type_shape::TypeShapeCheck::new(
            "payload_only_artifact",
            size_of::<Artifact<RawPhase, u64, NoProofs, NoAssumptionBasis>>(),
            size_of::<u64>(),
        ),
        crate::shared_support::type_shapes::type_shape::TypeShapeCheck::new(
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
        crate::shared_support::type_shapes::type_shape::TypeShapeCheck::new(
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
        crate::shared_support::type_shapes::type_shape::TypeShapeCheck::new(
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
        crate::shared_support::type_shapes::type_shape::TypeShapeCheck::new(
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
        crate::shared_support::type_shapes::type_shape::TypeShapeCheck::new(
            "canonical_vec",
            size_of::<CanonicalVec<u64>>(),
            size_of::<Vec<u64>>(),
        ),
        crate::shared_support::type_shapes::type_shape::TypeShapeCheck::new(
            "unique_vec",
            size_of::<UniqueVec<u64>>(),
            size_of::<Vec<u64>>(),
        ),
        crate::shared_support::type_shapes::type_shape::TypeShapeCheck::new(
            "disjoint_pair",
            size_of::<DisjointPair<u64>>(),
            size_of::<Pair<u64>>(),
        ),
    ])
}

pub(crate) fn proof_shape_digest() -> ProofShapeDigest {
    crate::shared_support::proof_shapes::digest::ProofShapeDigest::new(
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

pub(crate) fn basis_digest() -> BasisDigest {
    crate::shared_support::proof_shapes::digest::BasisDigest::new(
        "core_artifact_and_proof_substrate",
        vec![
            type_name::<NoAssumptionBasis>(),
            type_name::<AssumptionBasis<u32>>(),
        ],
    )
}

pub(crate) fn codegen_honesty_report() -> CodegenHonestyReport {
    crate::shared_support::type_shapes::codegen::CodegenHonestyReport::size_layout_and_drop_certified(
        "core_artifact_and_proof_substrate",
        vec![
            crate::shared_support::type_shapes::codegen::CodegenShapeCheck::new(
                "artifact",
                align_of::<Artifact<RawPhase, u64, NoProofs, NoAssumptionBasis>>(),
                align_of::<u64>(),
                needs_drop::<Artifact<RawPhase, u64, NoProofs, NoAssumptionBasis>>(),
                needs_drop::<u64>(),
            ),
            crate::shared_support::type_shapes::codegen::CodegenShapeCheck::new(
                "artifact_with_basis",
                align_of::<Artifact<RawPhase, u64, Proof<CanonicalOrder, StructuralProofAuthority>, AssumptionBasis<u32>>>(),
                align_of::<(u64, AssumptionBasis<u32>)>(),
                needs_drop::<
                    Artifact<RawPhase, u64, Proof<CanonicalOrder, StructuralProofAuthority>, AssumptionBasis<u32>>,
                >(),
                needs_drop::<(u64, AssumptionBasis<u32>)>(),
            ),
            crate::shared_support::type_shapes::codegen::CodegenShapeCheck::new(
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
            crate::shared_support::type_shapes::codegen::CodegenShapeCheck::new(
                "artifact_parts",
                align_of::<ArtifactParts<u64, Proof<CanonicalOrder, StructuralProofAuthority>, AssumptionBasis<u32>>>(),
                align_of::<(u64, Proof<CanonicalOrder, StructuralProofAuthority>, AssumptionBasis<u32>)>(),
                needs_drop::<ArtifactParts<u64, Proof<CanonicalOrder, StructuralProofAuthority>, AssumptionBasis<u32>>>(),
                needs_drop::<(u64, Proof<CanonicalOrder, StructuralProofAuthority>, AssumptionBasis<u32>)>(),
            ),
            crate::shared_support::type_shapes::codegen::CodegenShapeCheck::new(
                "canonical_vec",
                align_of::<CanonicalVec<u64>>(),
                align_of::<Vec<u64>>(),
                needs_drop::<CanonicalVec<u64>>(),
                needs_drop::<Vec<u64>>(),
            ),
            crate::shared_support::type_shapes::codegen::CodegenShapeCheck::new(
                "unique_vec",
                align_of::<UniqueVec<u64>>(),
                align_of::<Vec<u64>>(),
                needs_drop::<UniqueVec<u64>>(),
                needs_drop::<Vec<u64>>(),
            ),
            crate::shared_support::type_shapes::codegen::CodegenShapeCheck::new(
                "disjoint_pair",
                align_of::<DisjointPair<u64>>(),
                align_of::<Pair<u64>>(),
                needs_drop::<DisjointPair<u64>>(),
                needs_drop::<Pair<u64>>(),
            ),
            crate::shared_support::type_shapes::codegen::CodegenShapeCheck::new(
                "non_empty",
                align_of::<NonEmpty<u64>>(),
                align_of::<Vec<u64>>(),
                needs_drop::<NonEmpty<u64>>(),
                needs_drop::<Vec<u64>>(),
            ),
            crate::shared_support::type_shapes::codegen::CodegenShapeCheck::new(
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

pub(crate) fn debt_inventory() -> DebtInventory {
    crate::shared_support::type_shapes::debt::DebtInventory::new(
        "core_artifact_and_proof_substrate",
        vec![
            crate::shared_support::type_shapes::debt::DebtItem::new(
                "codegen_honesty",
                "Representative size/layout lanes are certified, but no handwritten baseline diff report exists yet.",
            ),
            crate::shared_support::type_shapes::debt::DebtItem::new(
                "freshness_topology",
                "`assumption/freshness.rs` remains deferred until Milestone 3 makes the seam load-bearing.",
            ),
        ],
    )
}