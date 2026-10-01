use std::collections::BTreeMap;

mod portfolio_compilation;

use crate::data::aspect::AspectVersion;
use crate::facade::SignalRuntimePolicy;

pub(super) use portfolio_compilation::source_result;
pub(crate) use portfolio_compilation::{
    compile_financial_world, compile_financial_world_with_policy,
};

use super::super::{
    FinancialEconomicSnapshot, FinancialSemanticProjection, FinancialWorldDefinition,
    SemanticOutputKey,
};
use super::locality_execution::CompiledFinancialLocalityWorld;
use super::locality_execution::FinancialPerformanceBatchReport;
use super::{FinancialEvaluationLedger, FinancialRuntime, FinancialSemanticHandles};

pub(crate) struct CompiledPortfolioFinancialWorld {
    pub(super) runtime: FinancialRuntime,
    pub(super) definition: FinancialWorldDefinition,
    pub(super) economic_snapshot: FinancialEconomicSnapshot,
    pub(super) projection: FinancialSemanticProjection,
    pub(super) handles: FinancialSemanticHandles,
    pub(super) ledger: FinancialEvaluationLedger,
    pub(super) baseline_dependency_revisions: BTreeMap<SemanticOutputKey, u64>,
    pub(super) baseline_aspect_versions: BTreeMap<SemanticOutputKey, AspectVersion>,
}

pub(super) enum CompiledFinancialWorldKind {
    Portfolio(CompiledPortfolioFinancialWorld),
    Locality(CompiledFinancialLocalityWorld),
}

pub(crate) struct CompiledFinancialWorld {
    pub(super) kind: CompiledFinancialWorldKind,
}

impl std::ops::Deref for CompiledFinancialWorld {
    type Target = CompiledPortfolioFinancialWorld;

    fn deref(&self) -> &Self::Target {
        match &self.kind {
            CompiledFinancialWorldKind::Portfolio(portfolio) => portfolio,
            CompiledFinancialWorldKind::Locality(_) => {
                panic!("portfolio operation used with compiled locality world")
            }
        }
    }
}

impl std::ops::DerefMut for CompiledFinancialWorld {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match &mut self.kind {
            CompiledFinancialWorldKind::Portfolio(portfolio) => portfolio,
            CompiledFinancialWorldKind::Locality(_) => {
                panic!("portfolio mutation used with compiled locality world")
            }
        }
    }
}

impl CompiledFinancialWorld {
    pub(crate) fn run_locality_performance_sequence(
        &mut self,
        batch_count: usize,
        workers: usize,
        observe: bool,
    ) -> Result<FinancialPerformanceBatchReport, crate::data::error::SignalError> {
        match &mut self.kind {
            CompiledFinancialWorldKind::Locality(locality) => {
                locality.run_performance_sequence(batch_count, workers, observe)
            }
            CompiledFinancialWorldKind::Portfolio(_) => {
                Err(crate::data::error::SignalError::invalid_input(
                    "locality performance sequence used with portfolio world",
                ))
            }
        }
    }

    pub(crate) fn locality_operational_digest_without_observation_work(
        &self,
    ) -> Result<worth_foundational::facade::CanonicalDigestId, crate::data::error::SignalError>
    {
        match &self.kind {
            CompiledFinancialWorldKind::Locality(locality) => {
                locality.operational_digest_without_observation_work()
            }
            CompiledFinancialWorldKind::Portfolio(_) => {
                Err(crate::data::error::SignalError::invalid_input(
                    "locality digest used with portfolio world",
                ))
            }
        }
    }

    pub(crate) fn certify_restore_lifecycle_for_performance(
        &mut self,
    ) -> Result<(), crate::data::error::SignalError> {
        match &mut self.kind {
            CompiledFinancialWorldKind::Locality(locality) => {
                locality.certify_restore_lifecycle().map(|_| ())
            }
            CompiledFinancialWorldKind::Portfolio(_) => {
                Err(crate::data::error::SignalError::invalid_input(
                    "locality restore lifecycle used with portfolio world",
                ))
            }
        }
    }

    pub(crate) fn runtime_policy(&self) -> SignalRuntimePolicy {
        match &self.kind {
            CompiledFinancialWorldKind::Portfolio(portfolio) => {
                portfolio.runtime.graph().runtime_policy()
            }
            CompiledFinancialWorldKind::Locality(locality) => locality.runtime_policy(),
        }
    }
}
