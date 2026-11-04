use crate::domain_computation::managed_run::tests::ManagedGraph;
use worth_query_execution::facade::domain_computation::WorthQueryCooperativeGraphProviderExecution;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderCall;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderExecutionStart;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderFailure;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderStep;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderStepDisposition;

use crate::domain_computation::managed_run::tests::decision_read_set_fixture::cleanup;
use crate::domain_computation::managed_run::tests::decision_read_set_fixture::managed_decision_graph_run_with_provider;
use crate::domain_computation::managed_run::tests::decision_read_set_fixture::staged;
use crate::domain_computation::managed_run::tests::*;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::Mutex;
use worth_query_execution::facade::domain_computation::WorthQueryDecisionFactAdmission;
use worth_query_execution::facade::domain_computation::WorthQueryDecisionFactComparisonAdmission;
use worth_query_execution::facade::domain_computation::WorthQueryDecisionFactComparisonEvidence;
use worth_query_execution::facade::domain_computation::WorthQueryDecisionFactEvidence;
use worth_query_execution::facade::domain_computation::WorthQueryDecisionFactEvidenceView;
use worth_query_execution::facade::domain_computation::WorthQueryDecisionFactLocator;
use worth_query_execution::facade::domain_computation::WorthQueryDecisionFactProvider;
use worth_query_execution::facade::domain_computation::WorthQueryDecisionFactRequest;
use worth_query_execution::facade::domain_computation::WorthQueryDecisionFactRequestView;
use worth_query_execution::facade::domain_computation::WorthQueryDecisionReadSetFailure;
use worth_query_execution::facade::domain_computation::WorthQueryDecisionReadSetFreshnessOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryProviderExecutionPlanView;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionFailure;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionLifecycle;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionToken;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionTokenAdmission;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionView;
use worth_query_installation::facade::WorthQueryDecisionFactFamily;
use worth_query_installation::facade::WorthQueryDecisionFactKind;
pub const OBSERVE_CALLS: &str = "__observe-calls";
pub struct DecisionProvider {
    pub versions: Arc<Mutex<BTreeMap<String, u64>>>,
}
pub struct UnusedDecisionExecution;
impl WorthQueryGraphProviderExecution for UnusedDecisionExecution {
    fn advance(
        &mut self,
        _step: &mut WorthQueryGraphProviderStep,
    ) -> Result<WorthQueryGraphProviderStepDisposition, WorthQueryGraphProviderFailure> {
        unreachable!("decision-read tests use only the sealed provider session")
    }

    fn dispose(&mut self) -> Result<(), WorthQueryGraphProviderFailure> {
        Ok(())
    }
}
impl WorthQueryGraphParticipationProvider<ManagedGraph> for DecisionProvider {
    type Execution = UnusedDecisionExecution;

    fn execution_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport {
        crate::domain_computation::provider_session::tests::execution_resource_support(
            "decision-provider",
            8,
        )
    }

    fn begin(
        &self,
        _call: &WorthQueryGraphProviderCall,
        _start: &mut WorthQueryGraphProviderExecutionStart,
    ) -> Result<
        WorthQueryCooperativeGraphProviderExecution<Self::Execution>,
        WorthQueryGraphProviderFailure,
    > {
        unreachable!("decision-read tests use only the sealed provider session")
    }
}
impl WorthQueryProviderSessionLifecycle for DecisionProvider {
    fn readmit_provider_plan(
        &self,
        _plan: &WorthQueryProviderExecutionPlanView<'_>,
        admission: WorthQueryProviderSessionTokenAdmission,
    ) -> Result<WorthQueryProviderSessionToken, WorthQueryProviderSessionFailure> {
        admission.admit("decision-session")
    }

    fn prepare_provider_session(
        &self,
        _session: &WorthQueryProviderSessionView<'_>,
    ) -> Result<(), WorthQueryProviderSessionFailure> {
        Ok(())
    }

    fn prepare_staged_session(
        &self,
        _session: &WorthQueryProviderSessionView<'_>,
    ) -> Result<(), WorthQueryProviderSessionFailure> {
        Ok(())
    }

    fn commit_prepared_session(
        &self,
        _session: &WorthQueryProviderSessionView<'_>,
        _allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        worth_query_execution::facade::domain_computation::WorthQueryProviderTerminalDescription,
        worth_query_execution::facade::domain_computation::WorthQueryProviderSessionCommitStop,
    > {
        Ok(
            worth_query_execution::facade::domain_computation::WorthQueryProviderTerminalDescription::new("unused commit")
                .expect("fixture description is valid"),
        )
    }

    fn abort_provider_session(
        &self,
        _session: &WorthQueryProviderSessionView<'_>,
    ) -> Result<
        worth_query_execution::facade::domain_computation::WorthQueryProviderTerminalDescription,
        WorthQueryProviderSessionFailure,
    > {
        Ok(
            worth_query_execution::facade::domain_computation::WorthQueryProviderTerminalDescription::new("decision abort")
                .expect("fixture description is valid"),
        )
    }
}
impl WorthQueryDecisionFactProvider for DecisionProvider {
    fn observe_decision_fact(
        &self,
        _session: WorthQueryProviderSessionView<'_>,
        request: WorthQueryDecisionFactRequestView<'_>,
        admission: WorthQueryDecisionFactAdmission,
    ) -> Result<WorthQueryDecisionFactEvidence, WorthQueryDecisionReadSetFailure> {
        let mut versions = self
            .versions
            .lock()
            .expect("decision versions should not be poisoned");
        *versions.entry(OBSERVE_CALLS.to_owned()).or_default() += 1;
        let version = versions
            .get(request.locator().identity())
            .copied()
            .unwrap_or_default();
        admission.observe(version.to_string())
    }

    fn compare_decision_fact(
        &self,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
        _session: WorthQueryProviderSessionView<'_>,
        evidence: WorthQueryDecisionFactEvidenceView<'_>,
        admission: WorthQueryDecisionFactComparisonAdmission,
    ) -> Result<WorthQueryDecisionFactComparisonEvidence, WorthQueryDecisionReadSetFailure> {
        let version = self
            .versions
            .lock()
            .expect("decision versions should not be poisoned")
            .get(evidence.locator().identity())
            .copied()
            .unwrap_or_default();
        admission.observe_current_version(version.to_string())
    }
}
pub fn all_fact_kinds() -> Vec<WorthQueryDecisionFactKind> {
    vec![
        WorthQueryDecisionFactKind::ObservedValue,
        WorthQueryDecisionFactKind::AbsenceOrNonMembership,
        WorthQueryDecisionFactKind::PredicateOrComparison,
        WorthQueryDecisionFactKind::OrderingOrExtremum,
        WorthQueryDecisionFactKind::CardinalityUniquenessOrOwnership,
        WorthQueryDecisionFactKind::TraversalFrontierOrPath,
        WorthQueryDecisionFactKind::AccessProductCoverageOrMembership,
        WorthQueryDecisionFactKind::ArtifactSemanticProjection,
        WorthQueryDecisionFactKind::DomainStructuralProof,
    ]
}
pub fn families(kinds: &[WorthQueryDecisionFactKind]) -> Vec<WorthQueryDecisionFactFamily> {
    kinds
        .iter()
        .enumerate()
        .map(|(index, kind)| {
            WorthQueryDecisionFactFamily::new(format!("family-{index}"), kind.clone()).unwrap()
        })
        .collect()
}
pub fn requests(kinds: &[WorthQueryDecisionFactKind]) -> Vec<WorthQueryDecisionFactRequest> {
    kinds
        .iter()
        .enumerate()
        .map(|(index, kind)| request(index, kind))
        .collect()
}
pub fn request(index: usize, kind: &WorthQueryDecisionFactKind) -> WorthQueryDecisionFactRequest {
    WorthQueryDecisionFactRequest::new(format!("family-{index}"), locator(index, kind)).unwrap()
}
pub fn locator(index: usize, kind: &WorthQueryDecisionFactKind) -> WorthQueryDecisionFactLocator {
    let identity = format!("locator-{index}");
    match kind {
        WorthQueryDecisionFactKind::ObservedValue => {
            WorthQueryDecisionFactLocator::observed(identity)
        }
        WorthQueryDecisionFactKind::AbsenceOrNonMembership => {
            WorthQueryDecisionFactLocator::absence(identity)
        }
        WorthQueryDecisionFactKind::PredicateOrComparison => {
            WorthQueryDecisionFactLocator::predicate(identity)
        }
        WorthQueryDecisionFactKind::OrderingOrExtremum => {
            WorthQueryDecisionFactLocator::ordering(identity)
        }
        WorthQueryDecisionFactKind::CardinalityUniquenessOrOwnership => {
            WorthQueryDecisionFactLocator::cardinality(identity)
        }
        WorthQueryDecisionFactKind::TraversalFrontierOrPath => {
            WorthQueryDecisionFactLocator::traversal(identity)
        }
        WorthQueryDecisionFactKind::AccessProductCoverageOrMembership => {
            WorthQueryDecisionFactLocator::access_product(identity)
        }
        WorthQueryDecisionFactKind::ArtifactSemanticProjection => {
            WorthQueryDecisionFactLocator::artifact_projection(identity)
        }
        WorthQueryDecisionFactKind::DomainStructuralProof => {
            WorthQueryDecisionFactLocator::structural_proof(identity)
        }
    }
    .unwrap()
}
pub fn version_map(count: usize) -> BTreeMap<String, u64> {
    (0..count)
        .map(|index| (format!("locator-{index}"), 1))
        .collect()
}
