pub(crate) use {
    worth_proof::AssumptionBasis, worth_proof::AuthoritativeFamilyMember,
    worth_proof::CompositionFamilySymbol, worth_proof::CurrentValidity,
    worth_proof::ExecutionReadyRecipe, worth_proof::FamilyLifecycleAction,
    worth_proof::FreshnessScopedBasis, worth_proof::JoinInputs2,
    worth_proof::LoweredFamilyProgram2,
};

pub(crate) type LeftBasis = FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u8>>;
pub(crate) type RightBasis = FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u16>>;

pub(crate) type LeftReadyRecipe = ExecutionReadyRecipe<u64, LeftBasis>;
pub(crate) type RightReadyRecipe = ExecutionReadyRecipe<u16, RightBasis>;
pub(crate) type JoinedReadyRecipe =
    ExecutionReadyRecipe<JoinInputs2<u64, u16>, JoinInputs2<LeftBasis, RightBasis>>;

pub(crate) type RepresentativeSymbol = CompositionFamilySymbol<u8>;
pub(crate) type RepresentativeMember = AuthoritativeFamilyMember<u16>;
pub(crate) type RepresentativeFamilyAction = FamilyLifecycleAction<u8, u16, &'static str>;
pub(crate) type RepresentativeLoweredFamilyProgram = LoweredFamilyProgram2<u8, u16, &'static str>;
