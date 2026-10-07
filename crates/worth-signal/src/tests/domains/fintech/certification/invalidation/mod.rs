mod canonical_identity;
#[cfg(test)]
mod claim_adversarial_tests;
mod cost_slope;
mod courtroom;
mod fresh_recompute;
mod locality_canonical_identity;
mod locality_completion;
mod locality_contract;
mod locality_expectation;
mod locality_fresh_recompute;
mod locality_receipt;
mod locality_run;
mod necessity_manifest;
mod performed_work_validation;
mod scenario_completion;
mod sealed_run;
mod strategy_decision;

pub(in crate::tests::domains::fintech) use canonical_identity::{
    FinancialCanonicalCaseIdentity, FinancialCanonicalReportIdentity,
};
pub(in crate::tests::domains::fintech) use cost_slope::{
    certify_ordinary_cost_slopes_from_cases, certify_scheduled_cost_slopes_from_cases,
    InvalidationCostSlopeReport,
};
pub(in crate::tests::domains::fintech) use courtroom::run_financial_causality_courtroom;
pub(crate) use fresh_recompute::FreshFinancialRecompute;
pub(in crate::tests::domains::fintech) use locality_canonical_identity::verified_locality_case_identity;
pub(in crate::tests::domains::fintech) use locality_expectation::{
    ExpectedLocalityCounterRow, FinancialLocalityExpectationManifest,
};
pub(in crate::tests::domains::fintech) use locality_fresh_recompute::FreshFinancialLocalityRecompute;
pub(crate) use locality_receipt::verify_locality_case_with_policy;
pub(crate) use locality_receipt::{verify_locality_case, FinancialLocalityCaseEvidence};
pub(in crate::tests::domains::fintech) use necessity_manifest::{
    FinancialNecessityEvidence, FinancialNecessityManifest,
};
pub(in crate::tests::domains::fintech) use scenario_completion::FinancialScenarioCompletion;
pub(in crate::tests::domains::fintech) use sealed_run::{
    FinancialAspectCausalityCertificationRun, FinancialCertificationPolicy,
    FinancialScenarioCertificationClaim,
};
pub(in crate::tests::domains::fintech) use strategy_decision::certify_current_strategy;
pub(in crate::tests::domains::fintech) use strategy_decision::{
    InvalidationStrategyReport, TraversalStrategyDecision,
};
