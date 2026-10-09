use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use super::*;
use crate::domain_computation::{
    WorthQueryDecisionFactAdmission, WorthQueryDecisionFactComparisonAdmission,
    WorthQueryDecisionFactComparisonEvidence, WorthQueryDecisionFactEvidence,
    WorthQueryDecisionFactEvidenceView, WorthQueryDecisionFactLocator,
    WorthQueryDecisionFactProvider, WorthQueryDecisionFactRequest,
    WorthQueryDecisionFactRequestView, WorthQueryDecisionReadSetFailure,
    WorthQueryDecisionReadSetFreshnessOutcome, WorthQueryProviderExecutionPlanView,
    WorthQueryProviderSessionFailure, WorthQueryProviderSessionLifecycle,
    WorthQueryProviderSessionToken, WorthQueryProviderSessionTokenAdmission,
    WorthQueryProviderSessionView,
};
use worth_query_installation::facade::{WorthQueryDecisionFactFamily, WorthQueryDecisionFactKind};

use super::decision_read_set_fixture::{cleanup, managed_decision_graph_run_with_provider, staged};

pub(super) const OBSERVE_CALLS: &str = "__observe-calls";

pub(super) struct DecisionProvider {
    pub(super) versions: Arc<Mutex<BTreeMap<String, u64>>>,
}

pub(super) struct UnusedDecisionExecution;

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
        crate::domain_computation::provider_session::execution_resource_support(
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
        crate::domain_computation::WorthQueryProviderTerminalDescription,
        crate::domain_computation::WorthQueryProviderSessionCommitStop,
    > {
        Ok(
            crate::domain_computation::WorthQueryProviderTerminalDescription::new("unused commit")
                .expect("fixture description is valid"),
        )
    }

    fn abort_provider_session(
        &self,
        _session: &WorthQueryProviderSessionView<'_>,
    ) -> Result<
        crate::domain_computation::WorthQueryProviderTerminalDescription,
        WorthQueryProviderSessionFailure,
    > {
        Ok(
            crate::domain_computation::WorthQueryProviderTerminalDescription::new("decision abort")
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

#[test]
fn all_fact_families_capture_canonically_and_compare_without_false_conflicts() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let kinds = all_fact_kinds();
        let families = families(&kinds);
        let versions = Arc::new(Mutex::new(version_map(kinds.len())));
        let (mut running, graph) = managed_decision_graph_run_with_provider(
            DecisionProvider {
                versions: Arc::clone(&versions),
            },
            families,
        );
        let staged = staged(execution, &mut running, &graph);
        {
            let reads = staged.read_authority();
            let requests = requests(&kinds);
            let first = reads
                .capture_decision_read_set(
                    requests.clone(),
                    crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                    None,
                )
                .expect("all installed fact families should capture");
            let second = reads
                .capture_decision_read_set(
                    requests.into_iter().rev(),
                    crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                    None,
                )
                .expect("discovery order must not affect capture");
            assert_eq!(first.identity(), second.identity());
            assert_eq!(first.fact_count(), kinds.len());
            let outcome = reads
                .compare_decision_read_set(first)
                .expect("unchanged provider facts should compare");
            let WorthQueryDecisionReadSetFreshnessOutcome::Fresh(fresh) = outcome else {
                panic!("unchanged facts must remain fresh");
            };
            assert_eq!(fresh.counters().compared_facts(), kinds.len());
            assert_eq!(fresh.counters().false_conflicts(), 0);
        }
        staged.abort();
        cleanup(running);
    });
}

#[test]
fn every_relevant_family_stales_independently_while_unrelated_axes_remain_fresh() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let kinds = all_fact_kinds();
        let versions = Arc::new(Mutex::new(version_map(kinds.len())));
        let (mut running, graph) = managed_decision_graph_run_with_provider(
            DecisionProvider {
                versions: Arc::clone(&versions),
            },
            families(&kinds),
        );
        let staged = staged(execution, &mut running, &graph);
        {
            let reads = staged.read_authority();
            for changed_index in 0..kinds.len() {
                let unchanged = reads
                    .capture_decision_read_set(
                        requests(&kinds),
                        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                        None,
                    )
                    .expect("complete decision facts should capture");
                {
                    let mut versions = versions.lock().unwrap();
                    for unrelated in [
                        "unrelated-entity",
                        "unrelated-aspect",
                        "unrelated-partition",
                        "unrelated-artifact-family",
                    ] {
                        versions.insert(unrelated.to_owned(), changed_index as u64 + 2);
                    }
                }
                let WorthQueryDecisionReadSetFreshnessOutcome::Fresh(fresh) =
                    reads.compare_decision_read_set(unchanged).unwrap()
                else {
                    panic!("unrelated semantic drift must remain fresh");
                };
                assert_eq!(fresh.counters().false_conflicts(), 0);

                let changed = reads
                    .capture_decision_read_set(
                        requests(&kinds),
                        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                        None,
                    )
                    .expect("complete decision facts should recapture");
                versions
                    .lock()
                    .unwrap()
                    .insert(format!("locator-{changed_index}"), 2);
                let WorthQueryDecisionReadSetFreshnessOutcome::Stale(stale) =
                    reads.compare_decision_read_set(changed).unwrap()
                else {
                    panic!("each relevant family must independently stale the receipt");
                };
                assert_eq!(stale.stale_fact_count(), 1);
                versions
                    .lock()
                    .unwrap()
                    .insert(format!("locator-{changed_index}"), 1);
            }
        }
        staged.abort();
        cleanup(running);
    });
}

fn all_fact_kinds() -> Vec<WorthQueryDecisionFactKind> {
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

fn families(kinds: &[WorthQueryDecisionFactKind]) -> Vec<WorthQueryDecisionFactFamily> {
    kinds
        .iter()
        .enumerate()
        .map(|(index, kind)| {
            WorthQueryDecisionFactFamily::new(format!("family-{index}"), kind.clone()).unwrap()
        })
        .collect()
}

fn requests(kinds: &[WorthQueryDecisionFactKind]) -> Vec<WorthQueryDecisionFactRequest> {
    kinds
        .iter()
        .enumerate()
        .map(|(index, kind)| request(index, kind))
        .collect()
}

pub(super) fn request(
    index: usize,
    kind: &WorthQueryDecisionFactKind,
) -> WorthQueryDecisionFactRequest {
    WorthQueryDecisionFactRequest::new(format!("family-{index}"), locator(index, kind)).unwrap()
}

fn locator(index: usize, kind: &WorthQueryDecisionFactKind) -> WorthQueryDecisionFactLocator {
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

pub(super) fn version_map(count: usize) -> BTreeMap<String, u64> {
    (0..count)
        .map(|index| (format!("locator-{index}"), 1))
        .collect()
}
