

pub(crate) use {std::mem::align_of, std::mem::needs_drop};

pub(crate) use {worth_proof::Admitted, worth_proof::AssumptionBasis, worth_proof::AuthorityMarker, worth_proof::AuthorityWitness, worth_proof::CanonicalOrder, worth_proof::CanonicalVec, worth_proof::CapabilityMarker, worth_proof::CapabilityWitness, worth_proof::DisjointPair, worth_proof::Lowered, worth_proof::Proof, worth_proof::Recipe, worth_proof::Resolved, worth_proof::StructuralProofAuthority, worth_proof::UniqueVec};

pub(crate) use {crate::shared_support::type_shapes::codegen::CodegenHonestyReport, crate::shared_support::type_shapes::codegen::CodegenShapeCheck};

pub(crate) fn assert_fixture_completeness() {
    crate::shared_support::sealed_minting::cases::assert_fixture_completeness();
}

pub(crate) fn compile_fail_bundle() -> crate::shared_support::compile_fail::bundle::CompileFailBundle {
    crate::shared_support::sealed_minting::cases::compile_fail_bundle()
}

pub(crate) struct RepresentativeAuthority;
impl AuthorityMarker for RepresentativeAuthority {}

pub(crate) struct RepresentativeCapability;
impl CapabilityMarker for RepresentativeCapability {}

pub(crate) fn codegen_honesty_report() -> CodegenHonestyReport {
    crate::shared_support::type_shapes::codegen::CodegenHonestyReport::size_layout_and_drop_certified(
        crate::shared_support::sealed_minting::cases::SUITE,
        vec![
            shape::<Proof<CanonicalOrder, StructuralProofAuthority>, ()>("proof"),
            shape::<AuthorityWitness<RepresentativeAuthority>, ()>("authority_witness"),
            shape::<CapabilityWitness<RepresentativeCapability>, ()>("capability_witness"),
            shape::<CanonicalVec<u64>, Vec<u64>>("canonical_vec"),
            shape::<UniqueVec<u64>, Vec<u64>>("unique_vec"),
            shape::<DisjointPair<u64>, (u64, u64)>("disjoint_pair"),
            shape::<Recipe<Resolved, u64, AssumptionBasis<u8>>, (u64, AssumptionBasis<u8>)>(
                "resolved_recipe",
            ),
            shape::<Recipe<Lowered, u64, AssumptionBasis<u8>>, (u64, AssumptionBasis<u8>)>(
                "lowered_recipe",
            ),
            shape::<Recipe<Admitted, u64, AssumptionBasis<u8>>, (u64, AssumptionBasis<u8>)>(
                "admitted_recipe",
            ),
        ],
        "Representative size/layout/drop evidence is scoped to sealed and witness-bearing forms; no handwritten MIR or ASM diff is claimed.",
    )
}

pub(crate) fn shape<Actual, Expected>(lane: &'static str) -> CodegenShapeCheck {
    crate::shared_support::type_shapes::codegen::CodegenShapeCheck::new(
        lane,
        align_of::<Actual>(),
        align_of::<Expected>(),
        needs_drop::<Actual>(),
        needs_drop::<Expected>(),
    )
}