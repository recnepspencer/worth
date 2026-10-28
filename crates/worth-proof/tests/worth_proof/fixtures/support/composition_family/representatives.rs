use worth_proof::{
    AssumptionBasis, AuthoritativeFamilyMember, CompositionFamilySymbol, CurrentValidity,
    ExecutionReadyRecipe, FamilyLifecycleAction, FreshnessScopedBasis, JoinInputs2,
    LoweredFamilyProgram2,
};

pub type LeftBasis = FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u8>>;
pub type RightBasis = FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u16>>;

pub type LeftReadyRecipe = ExecutionReadyRecipe<u64, LeftBasis>;
pub type RightReadyRecipe = ExecutionReadyRecipe<u16, RightBasis>;
pub type JoinedReadyRecipe =
    ExecutionReadyRecipe<JoinInputs2<u64, u16>, JoinInputs2<LeftBasis, RightBasis>>;

pub type RepresentativeSymbol = CompositionFamilySymbol<u8>;
pub type RepresentativeMember = AuthoritativeFamilyMember<u16>;
pub type RepresentativeFamilyAction = FamilyLifecycleAction<u8, u16, &'static str>;
pub type RepresentativeLoweredFamilyProgram = LoweredFamilyProgram2<u8, u16, &'static str>;
