mod baseline;
mod compiler;
mod definition;
mod fixture_compiler;
mod fixture_projection;
#[cfg(test)]
mod fixture_projection_tests;
mod locality_definition;
mod locality_scale;
mod market_inputs;
mod positions;
mod reference_finance;
mod semantic_projection;

pub(super) use baseline::{
    FinancialComparatorProfile, FinancialReproductionTuple, FinancialScaleTuple,
    FinancialScenarioIdentity,
};
pub(super) use compiler::strategy_work_projection;
pub(crate) use compiler::{
    compile_financial_locality_world, compile_financial_locality_world_at_tier,
    compile_financial_locality_world_with_policy, compile_financial_world,
    compile_financial_world_with_policy, CompiledFinancialWorld, FinancialPerformanceBatchReport,
    LocalityOptionalObservationInventory,
};
pub(super) use compiler::{
    FinancialBranchLifecycleCompletion, FinancialDependencyRewireEvidence,
    FinancialEvaluationLedger, FinancialFactorSequenceEvidence, FinancialLocalityRedObservation,
    FinancialPerformedCanonicalWork, FinancialPerformedWorkOrigin,
    FinancialQuoteTranslationEvidence, FinancialRestoreLifecycleEvidence, FinancialSemanticHandles,
};
pub(crate) use definition::FinancialWorldDefinition;
pub(super) use definition::{FinancialConditionPolicy, FinancialConsumerRole};
pub(super) use fixture_compiler::{compile_runtime_fixture, compile_unseeded_runtime_fixture};
pub(super) use fixture_projection::{
    FinancialFixtureProjection, FixtureAggregateState, FixtureMarketPoint, FixtureScenarioShock,
};
pub(super) use locality_definition::{
    FinancialLocalityAction, FinancialLocalityActionTrace, FinancialLocalityAdmissionPolicy,
    FinancialLocalityComparisonPolicy, FinancialLocalityDefinition,
    FinancialLocalityExecutionPolicy, FinancialLocalityFormula, FinancialLocalityMutation,
    FinancialLocalityOutput, FinancialLocalityOutputPolicy, FinancialLocalitySourceObligation,
    FinancialLocalityStagedWork, FinancialLocalitySubscription, FinancialLocalityTopologyChange,
    FinancialLocalityTraceIdentity, FinancialStructuralMutation, LocalityEconomicOwner,
    LocalityScope, LocalitySemanticOutputId,
};
pub(crate) use locality_scale::DensityRatio;
pub(super) use locality_scale::{
    ordinary_locality_cases, retained_locality_benchmark_cases, scheduled_locality_cases,
    FinancialLocalityScenario, LocalityCaseContract, LocalityLane, LocalityScaleTuple,
    RestorePosture, SparseFanoutAxis,
};
use market_inputs::{
    Currency, FinancialMarketInputs, FixedPrice, QuoteId, VolatilityBucket, FIXED_SCALE,
};
pub(super) use market_inputs::{CurveBucket, FxPair, MarketFactorKey};
use positions::FinancialPosition;
pub(super) use positions::PositionKind;
pub(super) use positions::{
    FinancialAspect, FinancialComparatorPolicy, FinancialOutputEquivalencePolicy, InstrumentId,
};
pub(super) use reference_finance::{
    reference_position_result, FinancialAmount, PositionFinancialResult,
};
pub(super) use semantic_projection::{
    FinancialEconomicSnapshot, FinancialSemanticProjection, SemanticOutputKey,
};
