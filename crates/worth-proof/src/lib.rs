//! Compile-time proof-bearing progression substrate for WORTH.

mod artifact;
mod assumption;
mod band;
mod binding;
mod brand;
mod collections;
mod composition;
pub mod contracts;
mod dx;
mod effect;
mod linear;
mod phase;
pub mod prelude;
mod proof;
pub mod raw;
mod recipe;
mod source_observation;
mod transition;
mod type_level;

pub(crate) use type_level::type_level_traits;

pub use crate::dx::{
    compose_ready, create, family_pair, gate_ready, join_ready, member, non_empty, pair,
    proof_flow, ready_now, recipe, retire, rewrite, supersede, sym, AdmittedBridgedRecipeDxExt,
    BasisPostureDxExt, BasisPostureKind, CheckedExecutionReadyRecipeDxExt,
    CheckedLoweredRecipeDxExt, CheckedProofOutcomeExecuteExt, CheckedProofOutcomeLowerExt,
    CheckedProofOutcomeReadyExt, CheckedProofOutcomeToAdmitExt, CheckedResolvedRecipeDxExt,
    CheckedUnresolvedRecipeDxExt, ExecutionReadyRecipeDxExt, FamilyActionDxExt, FamilyActionKind,
    FamilyPairDxExt, LoweredBridgedRecipeDxExt, LoweredFamilyProgramDxExt, LoweredRecipeDxExt,
    ProofFlow, ProofOutcome, ProofOutcomeKind, ReadyJoinRecipeDxExt, ReadyJoinSummary,
    RecipeStageDxExt, RecipeStageKind, ResolvedBridgedRecipeDxExt, ResolvedRecipeDxExt,
    UnresolvedRecipeDxExt,
};
pub use raw::PhaseMarker;
pub use raw::{
    admit_ready_and_execute_recipe, apply_contextual_transition, apply_transition,
    checked_admit_ready_and_execute_recipe, checked_readmit_ready_and_execute_recipe,
    compose_join_success_transition, compose_join_transition_outcome, compose_success_transition,
    compose_transition_outcome, readmit_ready_and_execute_recipe,
    resolve_checked_lower_and_admit_recipe, resolve_lower_and_admit_recipe,
    AdmitExecutionReadyRecipeTransition, AdmitRecipeTransition,
    CheckedAdmitExecutionReadyRecipeTransition, CheckedAdmitRecipeTransition,
    CheckedLowerRecipeTransition, CheckedReadmitLoweredForExecutionReadyTransition,
    CheckedResolveRecipeTransition, ContextualTransition, DeferredTransitionOutcome,
    DenialTransitionOutcome, ExecuteReadyRecipeTransition, ExecutionReadinessContext,
    ExecutionReadyAdmissionReadiness, FreshnessTransitionOutcome, LowerRecipeTransition,
    LoweredReadmissionContext, LoweredReadmissionReadiness, PreConstructionGate,
    ReadmitLoweredForExecutionReadyTransition, RecipeAdmissionReadiness, RecipeLoweringReadiness,
    RecipeResolutionContext, RecipeResolutionGate, ResolveRecipeTransition,
    SuccessfulTransitionOutcome, Transition, TransitionOutcome, TransitionReadiness,
};
pub use raw::{
    compose_join_ready_recipe_pair, fork_artifact_pair, join_artifact_pair, join_ready_recipe_pair,
    lower_deterministic_family_pair, resolve_family_symbol, AuthoritativeFamilyMember,
    CompositionFamilySymbol, FamilyLifecycleAction, FamilyResolvedReference, ForkOutputs2,
    JoinInputs2, LoweredFamilyProgram2,
};
pub use raw::{
    evaluate_freshness, take_sample, AssumptionBasis, AuthorityRevalidationRequired,
    AuthorityRevalidationRequiredBasis, BoundaryBridged,
    BoundaryBridgedAuthorityRevalidationRequiredBasis, BoundaryBridgedRebindRequiredBasis,
    BoundaryBridgedStaleReadableBasis, CurrentValidity, EvaluatedFreshness, FreshnessClass,
    FreshnessEvaluation, FreshnessPolicy, FreshnessSample, FreshnessScopedBasis, FreshnessSource,
    FreshnessVerdict, NoAssumptionBasis, RebindRequired, RebindRequiredBasis, StaleReadable,
    StaleReadableBasis,
};
pub use raw::{
    prove_derivation, prove_inversion, ActionMarker, DerivedFrom, InverseOf, Inverts, Performed,
};
pub use raw::{with_brand, Brand, Branded};
pub use raw::{
    Admitted, ExecutedRecipe, ExecutionReadyRecipe, Lowered, Recipe, RecipeStageMarker, Resolved,
    Unresolved,
};
pub use raw::{Artifact, ArtifactParts, ArtifactView};
pub use raw::{
    AuthorityMarker, AuthorityProves, AuthorityWitness, CanonicalOrder, CapabilityMarker,
    CapabilityWitness, Disjointness, NoProofs, Normalization, Proof, ProofMarker, ProofSet,
    ProofSetAuthorizedBy, ProofSetCons, StructuralProofAuthority, Uniqueness,
};
pub use raw::{Binding, BindingAxes};
pub use raw::{CanonicalVec, DisjointPair, ExactlyOne, NonEmpty, Pair, UniqueVec};
pub use raw::{LinearResource, TerminalReceipt, TerminalState};
pub use source_observation::{
    AdmittedConditionalSourceObservation, ConditionalEvaluationSource,
    ConditionalSourceObservationAuthority, ConditionalSourceObservationOwner,
};

#[doc(hidden)]
pub use band::__band_guard_package_matches_any_prefix;
