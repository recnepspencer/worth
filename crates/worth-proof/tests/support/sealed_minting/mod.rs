mod cases;

use std::mem::{align_of, needs_drop};

use worth_proof::{
    Admitted, AssumptionBasis, AuthorityMarker, AuthorityWitness, CanonicalOrder, CanonicalVec,
    CapabilityMarker, CapabilityWitness, DisjointPair, Lowered, Proof, Recipe, Resolved,
    StructuralProofAuthority, UniqueVec,
};

use super::type_shapes::{CodegenHonestyReport, CodegenShapeCheck};

pub fn assert_fixture_completeness() {
    cases::assert_fixture_completeness();
}

pub fn compile_fail_bundle() -> super::compile_fail::CompileFailBundle {
    cases::compile_fail_bundle()
}

pub(crate) struct RepresentativeAuthority;
impl AuthorityMarker for RepresentativeAuthority {}

pub(crate) struct RepresentativeCapability;
impl CapabilityMarker for RepresentativeCapability {}

pub fn codegen_honesty_report() -> CodegenHonestyReport {
    CodegenHonestyReport::size_layout_and_drop_certified(
        cases::SUITE,
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

fn shape<Actual, Expected>(lane: &'static str) -> CodegenShapeCheck {
    CodegenShapeCheck::new(
        lane,
        align_of::<Actual>(),
        align_of::<Expected>(),
        needs_drop::<Actual>(),
        needs_drop::<Expected>(),
    )
}
