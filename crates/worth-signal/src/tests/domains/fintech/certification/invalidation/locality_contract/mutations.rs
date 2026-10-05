use crate::tests::domains::fintech::world::FinancialLocalityScenario;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum LocalityRedMutation {
    RestoreTransitiveSubscriberClosure,
    QueryProducerWideSubscriberList,
    FilterAspectAfterMutation,
    OmitRejectedCandidatesFromCounters,
    ReplaceOwnedFanoutWithDisconnectedPadding,
    ScanAllRegions,
    FlattenAspectScopeCorrelation,
    EnumerateUnrelatedDetailBuckets,
    RetainOnlyLastChangedDetail,
    RejectScopeAfterMutationOrEnqueue,
    TrustDriftedDerivedIndex,
    DeduplicateByNodeOrAspectSlotOnly,
    ReplaceCanonicalCauseSetWithLastCause,
    EvaluateConvergedTargetTwice,
    ReconstructProvenanceAfterMerge,
    MakeCanonicalizationSeedOrderDependent,
    UseSparseOnlyDenseShortcut,
    CapDenseWork,
    DropNecessaryDenseWork,
    RelabelFullGraphWalkAsEvaluation,
    OmitQueueWidthOrAllocationRows,
    CompareStrategiesOnDifferentStreams,
    ExecuteStalePreRewireWork,
    CompareEdgeShapeWithoutRevision,
    AdvanceRevisionOnRejectedCycle,
    ReuseOldTopologyOrder,
    OmitRevalidationOrChurnRows,
    LetRejectedTopologyMutationEnqueueWork,
    SerializeAndExecuteReadyAuthority,
    ReuseOldGraphRevisionOrEpochBinding,
    SubstitutePendingCauseAtEqualCardinality,
    CountRecoveryAsOrdinaryHotWork,
    CompareReplayByKindsOrFinalValuesOnly,
}

impl LocalityRedMutation {
    const ALL: [Self; 33] = [
        Self::RestoreTransitiveSubscriberClosure,
        Self::QueryProducerWideSubscriberList,
        Self::FilterAspectAfterMutation,
        Self::OmitRejectedCandidatesFromCounters,
        Self::ReplaceOwnedFanoutWithDisconnectedPadding,
        Self::ScanAllRegions,
        Self::FlattenAspectScopeCorrelation,
        Self::EnumerateUnrelatedDetailBuckets,
        Self::RetainOnlyLastChangedDetail,
        Self::RejectScopeAfterMutationOrEnqueue,
        Self::TrustDriftedDerivedIndex,
        Self::DeduplicateByNodeOrAspectSlotOnly,
        Self::ReplaceCanonicalCauseSetWithLastCause,
        Self::EvaluateConvergedTargetTwice,
        Self::ReconstructProvenanceAfterMerge,
        Self::MakeCanonicalizationSeedOrderDependent,
        Self::UseSparseOnlyDenseShortcut,
        Self::CapDenseWork,
        Self::DropNecessaryDenseWork,
        Self::RelabelFullGraphWalkAsEvaluation,
        Self::OmitQueueWidthOrAllocationRows,
        Self::CompareStrategiesOnDifferentStreams,
        Self::ExecuteStalePreRewireWork,
        Self::CompareEdgeShapeWithoutRevision,
        Self::AdvanceRevisionOnRejectedCycle,
        Self::ReuseOldTopologyOrder,
        Self::OmitRevalidationOrChurnRows,
        Self::LetRejectedTopologyMutationEnqueueWork,
        Self::SerializeAndExecuteReadyAuthority,
        Self::ReuseOldGraphRevisionOrEpochBinding,
        Self::SubstitutePendingCauseAtEqualCardinality,
        Self::CountRecoveryAsOrdinaryHotWork,
        Self::CompareReplayByKindsOrFinalValuesOnly,
    ];
}

pub(super) fn red_mutations(scenario: FinancialLocalityScenario) -> &'static [LocalityRedMutation] {
    use LocalityRedMutation as M;
    match scenario {
        FinancialLocalityScenario::SparseBookFanout => &[
            M::RestoreTransitiveSubscriberClosure,
            M::QueryProducerWideSubscriberList,
            M::FilterAspectAfterMutation,
            M::OmitRejectedCandidatesFromCounters,
            M::ReplaceOwnedFanoutWithDisconnectedPadding,
        ],
        FinancialLocalityScenario::PartitionedCurveUniverse => &[
            M::ScanAllRegions,
            M::FlattenAspectScopeCorrelation,
            M::EnumerateUnrelatedDetailBuckets,
            M::RetainOnlyLastChangedDetail,
            M::RejectScopeAfterMutationOrEnqueue,
            M::TrustDriftedDerivedIndex,
        ],
        FinancialLocalityScenario::ConvergentFactorBatch => &[
            M::DeduplicateByNodeOrAspectSlotOnly,
            M::ReplaceCanonicalCauseSetWithLastCause,
            M::EvaluateConvergedTargetTwice,
            M::ReconstructProvenanceAfterMerge,
            M::MakeCanonicalizationSeedOrderDependent,
        ],
        FinancialLocalityScenario::DenseMarketClose => &[
            M::UseSparseOnlyDenseShortcut,
            M::CapDenseWork,
            M::DropNecessaryDenseWork,
            M::RelabelFullGraphWalkAsEvaluation,
            M::OmitQueueWidthOrAllocationRows,
            M::CompareStrategiesOnDifferentStreams,
        ],
        FinancialLocalityScenario::PortfolioDependencyChurn => &[
            M::ExecuteStalePreRewireWork,
            M::CompareEdgeShapeWithoutRevision,
            M::AdvanceRevisionOnRejectedCycle,
            M::ReuseOldTopologyOrder,
            M::OmitRevalidationOrChurnRows,
            M::LetRejectedTopologyMutationEnqueueWork,
        ],
        FinancialLocalityScenario::BranchRestoreLocalityReplay => &[
            M::SerializeAndExecuteReadyAuthority,
            M::ReuseOldGraphRevisionOrEpochBinding,
            M::SubstitutePendingCauseAtEqualCardinality,
            M::CountRecoveryAsOrdinaryHotWork,
            M::CompareReplayByKindsOrFinalValuesOnly,
        ],
    }
}

#[test]
fn assigned_red_mutation_sets_are_exact_exhaustive_and_scenario_owned() {
    let expected_counts = [5, 6, 5, 6, 6, 5];
    let mut assigned = std::collections::BTreeSet::new();
    for (scenario, expected_count) in FinancialLocalityScenario::ALL
        .into_iter()
        .zip(expected_counts)
    {
        let mutations = red_mutations(scenario);
        assert_eq!(mutations.len(), expected_count);
        for mutation in mutations {
            assert!(
                assigned.insert(*mutation),
                "a red mutation must belong to exactly one scenario"
            );
        }
    }
    assert_eq!(assigned, LocalityRedMutation::ALL.into());
}

#[derive(Clone, Copy)]
enum MutationObservation {
    Counter(crate::data::telemetry::InvalidationPerformedCounter),
    EvaluatedWork,
    CausalBinding,
}

impl LocalityRedMutation {
    fn observation(self) -> MutationObservation {
        use crate::data::telemetry::InvalidationPerformedCounter as C;
        use LocalityRedMutation as M;
        match self {
            M::RestoreTransitiveSubscriberClosure | M::RelabelFullGraphWalkAsEvaluation => {
                MutationObservation::Counter(C::NonSemanticNodeVisits)
            }
            M::QueryProducerWideSubscriberList => {
                MutationObservation::Counter(C::ReverseIndexCandidatesReturned)
            }
            M::FilterAspectAfterMutation | M::RejectScopeAfterMutationOrEnqueue => {
                MutationObservation::Counter(C::DirectSettlementsProduced)
            }
            M::OmitRejectedCandidatesFromCounters => {
                MutationObservation::Counter(C::CandidatesRejectedByAspectContract)
            }
            M::ScanAllRegions | M::EnumerateUnrelatedDetailBuckets => {
                MutationObservation::Counter(C::ReverseIndexBucketProbes)
            }
            M::FlattenAspectScopeCorrelation | M::RetainOnlyLastChangedDetail => {
                MutationObservation::Counter(C::CandidatesRejectedByScope)
            }
            M::TrustDriftedDerivedIndex => {
                MutationObservation::Counter(C::DirectSubscriberEdgesExamined)
            }
            M::UseSparseOnlyDenseShortcut | M::OmitQueueWidthOrAllocationRows => {
                MutationObservation::Counter(C::PeakBatchMemoryItems)
            }
            M::AdvanceRevisionOnRejectedCycle | M::LetRejectedTopologyMutationEnqueueWork => {
                MutationObservation::Counter(C::RejectedTopologyMutations)
            }
            M::OmitRevalidationOrChurnRows => {
                MutationObservation::Counter(C::TopologyRevisionRevalidations)
            }
            M::CountRecoveryAsOrdinaryHotWork => {
                MutationObservation::Counter(C::RecoveryReconstructionWork)
            }
            M::ReplaceOwnedFanoutWithDisconnectedPadding
            | M::EvaluateConvergedTargetTwice
            | M::CapDenseWork
            | M::DropNecessaryDenseWork => MutationObservation::EvaluatedWork,
            M::DeduplicateByNodeOrAspectSlotOnly
            | M::ReplaceCanonicalCauseSetWithLastCause
            | M::ReconstructProvenanceAfterMerge
            | M::MakeCanonicalizationSeedOrderDependent
            | M::CompareStrategiesOnDifferentStreams
            | M::ExecuteStalePreRewireWork
            | M::CompareEdgeShapeWithoutRevision
            | M::ReuseOldTopologyOrder
            | M::SerializeAndExecuteReadyAuthority
            | M::ReuseOldGraphRevisionOrEpochBinding
            | M::SubstitutePendingCauseAtEqualCardinality
            | M::CompareReplayByKindsOrFinalValuesOnly => MutationObservation::CausalBinding,
        }
    }
}

#[test]
fn every_assigned_red_mutation_breaks_its_scenario_owned_observation() {
    use crate::facade::DiagnosticsTier;
    use crate::tests::domains::fintech::certification::invalidation::{
        locality_receipt::validate_case_results, FinancialLocalityExpectationManifest,
        FreshFinancialLocalityRecompute,
    };
    use crate::tests::domains::fintech::world::{
        compile_financial_locality_world, ordinary_locality_cases, FinancialWorldDefinition,
    };

    for scenario in FinancialLocalityScenario::ALL {
        let case = ordinary_locality_cases()
            .into_iter()
            .find(|case| case.scenario() == scenario)
            .expect("each scenario has an ordinary mutation courtroom");
        let definition = FinancialWorldDefinition::locality_case(41, case);
        let mut compiled = compile_financial_locality_world(definition).unwrap();
        compiled.set_locality_diagnostics_tier(DiagnosticsTier::Operational);
        let manifest = FinancialLocalityExpectationManifest::derive(
            compiled.locality_definition(),
            compiled.locality_graph_instance(),
        );
        let fresh = FreshFinancialLocalityRecompute::run(compiled.locality_definition());
        let (baseline, _) = compiled
            .observe_locality_action_trace_with_workers(0, 1)
            .unwrap();
        validate_case_results(&compiled, &manifest, &fresh, &baseline).unwrap();

        for mutation in red_mutations(scenario) {
            let mut drifted = baseline.clone();
            match mutation.observation() {
                MutationObservation::Counter(counter) => {
                    let mut values = drifted.performed_counters.values();
                    values[counter as usize] = values[counter as usize].saturating_add(1);
                    drifted.performed_counters =
                        crate::data::telemetry::SignalInvalidationRealizedCounters::from_values(
                            values,
                        );
                }
                MutationObservation::EvaluatedWork => {
                    let removed = drifted
                        .evaluated_outputs
                        .iter()
                        .next()
                        .copied()
                        .expect("locality mutation courtroom performs semantic work");
                    drifted.evaluated_outputs.remove(&removed);
                }
                MutationObservation::CausalBinding => {
                    let (identity, count) = drifted
                        .performed_work
                        .iter()
                        .next()
                        .map(|(identity, count)| (identity.clone(), *count))
                        .expect("locality mutation courtroom performs bound work");
                    drifted.performed_work.remove(&identity);
                    drifted
                        .performed_work
                        .insert(identity.with_drifted_origin_for_test(), count);
                }
            }
            assert!(
                validate_case_results(&compiled, &manifest, &fresh, &drifted).is_err(),
                "{scenario:?} mutation {mutation:?} passed for the wrong reason"
            );
        }
    }
}
