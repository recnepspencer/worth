use std::mem::{align_of, needs_drop};

use worth_proof::{FamilyResolvedReference, ForkOutputs2, JoinInputs2, Pair};

use super::super::type_shapes::{CodegenHonestyReport, CodegenShapeCheck};
use super::representatives::{
    JoinedReadyRecipe, RepresentativeFamilyAction, RepresentativeLoweredFamilyProgram,
    RepresentativeMember, RepresentativeSymbol,
};

pub fn codegen_honesty_report() -> CodegenHonestyReport {
    CodegenHonestyReport::size_layout_and_drop_certified(
        "static_fork_join_and_composition_family",
        vec![
            CodegenShapeCheck::new(
                "fork_outputs2",
                align_of::<ForkOutputs2<u64, u16>>(),
                align_of::<(u64, u16)>(),
                needs_drop::<ForkOutputs2<u64, u16>>(),
                needs_drop::<(u64, u16)>(),
            ),
            CodegenShapeCheck::new(
                "join_inputs2",
                align_of::<JoinInputs2<u64, u16>>(),
                align_of::<(u64, u16)>(),
                needs_drop::<JoinInputs2<u64, u16>>(),
                needs_drop::<(u64, u16)>(),
            ),
            CodegenShapeCheck::new(
                "joined_ready_recipe",
                align_of::<JoinedReadyRecipe>(),
                align_of::<worth_proof::Recipe<
                    worth_proof::Lowered,
                    JoinInputs2<u64, u16>,
                    JoinInputs2<
                        worth_proof::FreshnessScopedBasis<
                            worth_proof::CurrentValidity,
                            worth_proof::AssumptionBasis<u8>,
                        >,
                        worth_proof::FreshnessScopedBasis<
                            worth_proof::CurrentValidity,
                            worth_proof::AssumptionBasis<u16>,
                        >,
                    >,
                >>(),
                needs_drop::<JoinedReadyRecipe>(),
                needs_drop::<worth_proof::Recipe<
                    worth_proof::Lowered,
                    JoinInputs2<u64, u16>,
                    JoinInputs2<
                        worth_proof::FreshnessScopedBasis<
                            worth_proof::CurrentValidity,
                            worth_proof::AssumptionBasis<u8>,
                        >,
                        worth_proof::FreshnessScopedBasis<
                            worth_proof::CurrentValidity,
                            worth_proof::AssumptionBasis<u16>,
                        >,
                    >,
                >>(),
            ),
            CodegenShapeCheck::new(
                "family_resolved_reference",
                align_of::<FamilyResolvedReference<u8, u16>>(),
                align_of::<(RepresentativeSymbol, RepresentativeMember)>(),
                needs_drop::<FamilyResolvedReference<u8, u16>>(),
                needs_drop::<(RepresentativeSymbol, RepresentativeMember)>(),
            ),
            CodegenShapeCheck::new(
                "lowered_family_program2",
                align_of::<RepresentativeLoweredFamilyProgram>(),
                align_of::<Pair<RepresentativeFamilyAction>>(),
                needs_drop::<RepresentativeLoweredFamilyProgram>(),
                needs_drop::<Pair<RepresentativeFamilyAction>>(),
            ),
        ],
        "Milestone 6 certifies representative size/layout/drop honesty for fixed-arity fork/join carriers, ready-join wrappers, and deterministic family-lowering carriers; it does not yet certify broader N-ary composition or cross-crate migration baselines.",
    )
}
