pub use worth_foundational::facade::FoundationalPerformanceCounterName;
pub use worth_query::facade::domain;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvidenceScenario {
    Honest,
    OmitSidecars,
    MissingRequiredCounter,
    BackwardCounter,
    ImpossibleAggregate,
    SearchOverclaim,
    SearchNotApplicable,
    SearchNoFeasibleSelected,
    SearchAllFeasibleRejected,
    LossMismatch,
    OutputOccurrenceMismatch,
    MalformedSidecars,
    MalformedTransformationSidecar,
    MalformedOptionalCounter,
    LedgerRegression,
    ReplayCoreDrift,
    MixedDecisionGovernance,
    UndeclaredCounter,
    DuplicateCounter,
    ProviderCertificationMissing,
    MissingDecisionSummary,
    DuplicateDecisionSummary,
    UndeclaredDecisionSummary,
}

pub fn evidence_material(
    output_occurrence_identity: &str,
    scenario: EvidenceScenario,
) -> domain::WorthQueryDomainEvidenceMaterial {
    let candidate = if scenario == EvidenceScenario::BackwardCounter {
        CounterWindow::new(7, 6)
    } else {
        CounterWindow::new(0, 6)
    };
    evidence_material_with_window(
        claimed_output(output_occurrence_identity, scenario),
        scenario,
        candidate,
        false,
    )
}

pub fn workflow_evidence_material(
    output_occurrence_identity: &str,
    scenario: EvidenceScenario,
    stage_identity: &str,
) -> domain::WorthQueryDomainEvidenceMaterial {
    let candidate = match (stage_identity, scenario) {
        ("start", _) => CounterWindow::new(0, 6),
        ("left", EvidenceScenario::LedgerRegression) => CounterWindow::new(0, 5),
        ("left", _) => CounterWindow::new(6, 12),
        _ => panic!("only evidence-declaring workflow stages may attach evidence"),
    };
    evidence_material_with_window(
        claimed_output(output_occurrence_identity, scenario),
        scenario,
        candidate,
        stage_identity == "left",
    )
}

pub fn claimed_output(output_occurrence_identity: &str, scenario: EvidenceScenario) -> &str {
    if scenario == EvidenceScenario::OutputOccurrenceMismatch {
        "foreign-output-occurrence"
    } else {
        output_occurrence_identity
    }
}

#[derive(Clone, Copy)]
pub struct CounterWindow {
    pub initial: u64,
    pub observed: u64,
}

impl CounterWindow {
    pub const fn new(initial: u64, observed: u64) -> Self {
        Self { initial, observed }
    }
}

pub fn evidence_material_with_window(
    claimed_output: &str,
    scenario: EvidenceScenario,
    candidate: CounterWindow,
    workflow_left: bool,
) -> domain::WorthQueryDomainEvidenceMaterial {
    let bytes_observed = if scenario == EvidenceScenario::ReplayCoreDrift && workflow_left {
        129
    } else {
        128
    };
    let work = CounterWindow::new(
        candidate.initial,
        candidate.observed + 4 - u64::from(scenario == EvidenceScenario::ImpossibleAggregate),
    );
    let work_components = CounterWindow::new(candidate.initial, candidate.observed + 4);
    let mut material = domain::WorthQueryDomainEvidenceMaterial::new()
        .counter(observation("bytes", 0, bytes_observed))
        .counter(observation("elements", 0, 4))
        .counter(observation(
            "candidate-comparisons",
            candidate.initial,
            candidate.observed,
        ))
        .counter(observation(
            "operation-work-components",
            work_components.initial,
            work_components.observed,
        ))
        .candidate_search(candidate_summary(scenario))
        .transformation(transformation_summary(claimed_output, scenario));
    if scenario != EvidenceScenario::MissingDecisionSummary {
        material = material.decision(decision_summary(decision_kind()));
    }
    if scenario == EvidenceScenario::DuplicateDecisionSummary {
        material = material.decision(decision_summary(decision_kind()));
    }
    if scenario == EvidenceScenario::UndeclaredDecisionSummary {
        material = material.decision(decision_summary(
            domain::WorthQueryDecisionKind::new("undeclared-decision").unwrap(),
        ));
    }
    if scenario == EvidenceScenario::MixedDecisionGovernance {
        material = material.decision(domain::WorthQueryDecisionSummary::new(
            selected_decision_kind(),
            domain::WorthQueryDecisionSummaryCounts::new(1, 1, 1, 0),
        ));
    }
    if scenario != EvidenceScenario::MissingRequiredCounter {
        material = material.counter(observation("work", work.initial, work.observed));
    }
    if scenario == EvidenceScenario::UndeclaredCounter {
        material = material.counter(observation("undeclared-counter", 0, 1));
    }
    if scenario == EvidenceScenario::DuplicateCounter {
        material = material.counter(observation("bytes", 0, bytes_observed));
    }
    if scenario != EvidenceScenario::OmitSidecars {
        let optional = if scenario == EvidenceScenario::MalformedOptionalCounter {
            CounterWindow::new(9, 8)
        } else {
            CounterWindow::new(0, 9)
        };
        let optional = observation("trace-events", optional.initial, optional.observed);
        let optional = if scenario == EvidenceScenario::ProviderCertificationMissing {
            optional
        } else {
            optional.with_provider_certification("domain-evidence-provider-v1")
        };
        material = material
            .counter(optional)
            .with_sidecar(sidecars(claimed_output, scenario));
    }
    material
}

pub fn candidate_summary(scenario: EvidenceScenario) -> domain::WorthQueryCandidateSearchSummary {
    let completeness = if scenario == EvidenceScenario::SearchOverclaim {
        domain::WorthQueryCandidateSearchPosture::Exhaustive
    } else {
        domain::WorthQueryCandidateSearchPosture::Sampled {
            sample_identity: "sample-v1".into(),
        }
    };
    let (feasibility, rejected_count, incumbent) = match scenario {
        EvidenceScenario::SearchNotApplicable => (
            domain::WorthQueryCandidateFeasibilityClass::NotApplicable,
            1,
            domain::WorthQueryCandidateIncumbentDisposition::Selected,
        ),
        EvidenceScenario::SearchNoFeasibleSelected => (
            domain::WorthQueryCandidateFeasibilityClass::NoFeasibleCandidate,
            1,
            domain::WorthQueryCandidateIncumbentDisposition::Selected,
        ),
        EvidenceScenario::SearchAllFeasibleRejected => (
            domain::WorthQueryCandidateFeasibilityClass::AllConsideredFeasible,
            1,
            domain::WorthQueryCandidateIncumbentDisposition::Selected,
        ),
        _ => (
            domain::WorthQueryCandidateFeasibilityClass::FeasibleCandidateFound,
            1,
            domain::WorthQueryCandidateIncumbentDisposition::Selected,
        ),
    };
    domain::WorthQueryCandidateSearchSummary::from_parts(
        domain::WorthQueryCandidateSearchSummaryParts {
            universe: domain::WorthQueryDomainEvidenceValue::new("candidate-universe", "sample-v1"),
            considered_count: 2,
            termination_family: "candidate-termination".into(),
            termination: domain::WorthQueryCandidateTerminationClass::SampleCompleted,
            completeness,
            feasibility_family: "candidate-feasibility".into(),
            feasibility,
            comparison_authority: domain::WorthQueryDomainEvidenceValue::new(
                "candidate-comparison",
                "score-v1",
            ),
            optimality: domain::WorthQueryCandidateOptimalityPosture::BestInDeclaredSample {
                sample_identity: "sample-v1".into(),
            },
            rejected_count,
            incumbent_family: "candidate-incumbent".into(),
            incumbent,
        },
    )
}

pub fn transformation_summary(
    output_occurrence_identity: &str,
    scenario: EvidenceScenario,
) -> domain::WorthQueryTransformationSummary {
    let loss = if scenario == EvidenceScenario::LossMismatch {
        domain::WorthQueryTransformationLossPosture::Lossless
    } else {
        domain::WorthQueryTransformationLossPosture::DeclaredLossy
    };
    domain::WorthQueryTransformationSummary::from_parts(
        domain::WorthQueryTransformationSummaryParts {
            source_occurrence: domain::WorthQueryDomainEvidenceValue::new(
                "source-occurrence",
                "source-1",
            ),
            output_occurrence_identity: output_occurrence_identity.into(),
            transformation_family: "normalize-candidates".into(),
            transformation_version: 1,
            correspondence: domain::WorthQuerySourceOutputCorrespondence::OneToMany,
            disposition: domain::WorthQueryTransformationDisposition::Normalized,
            error: domain::WorthQueryTransformationErrorPosture::Bounded,
            loss,
        },
    )
}

pub fn sidecars(
    output_occurrence_identity: &str,
    scenario: EvidenceScenario,
) -> domain::WorthQueryDomainEvidenceSidecar {
    let candidate_records = if scenario == EvidenceScenario::MalformedSidecars {
        vec![domain::WorthQueryCandidateRecord::new(
            "candidate-1",
            domain::WorthQueryCandidateRecordDisposition::Rejected,
        )]
    } else {
        vec![
            domain::WorthQueryCandidateRecord::new(
                "candidate-1",
                domain::WorthQueryCandidateRecordDisposition::Rejected,
            ),
            domain::WorthQueryCandidateRecord::new(
                "candidate-2",
                domain::WorthQueryCandidateRecordDisposition::Incumbent,
            ),
        ]
    };
    let transformation_outputs = if scenario == EvidenceScenario::MalformedTransformationSidecar {
        vec![output_occurrence_identity.to_owned()]
    } else {
        vec![
            output_occurrence_identity.to_owned(),
            "output-secondary".to_owned(),
        ]
    };
    let mut decision_records = vec![domain::WorthQueryDecisionRecord::from_parts(
        domain::WorthQueryDecisionRecordParts {
            kind: decision_kind(),
            reason_family: "search-reason".into(),
            artifact_key_family: "candidate".into(),
            artifact_key: "candidate-1".into(),
            causal_parent: domain::WorthQueryDecisionCausalParent::Single("candidate-root".into()),
            payload_version: 1,
            payload: "rejected-by-score".into(),
            recovery_relevant: false,
        },
    )];
    if scenario == EvidenceScenario::MixedDecisionGovernance {
        decision_records.push(domain::WorthQueryDecisionRecord::from_parts(
            domain::WorthQueryDecisionRecordParts {
                kind: selected_decision_kind(),
                reason_family: "selection-reason".into(),
                artifact_key_family: "candidate".into(),
                artifact_key: "candidate-2".into(),
                causal_parent: domain::WorthQueryDecisionCausalParent::Single(
                    "candidate-root".into(),
                ),
                payload_version: 1,
                payload: "selected-by-score".into(),
                recovery_relevant: false,
            },
        ));
    }
    domain::WorthQueryDomainEvidenceSidecar::new()
        .decision_records(decision_records)
        .candidate_records(candidate_records)
        .transformation_records([domain::WorthQueryTransformationRecord::new(
            "source-1",
            transformation_outputs,
            domain::WorthQueryTransformationDisposition::Normalized,
            domain::WorthQueryTransformationErrorPosture::Bounded,
        )])
}

pub fn observation(
    name: &str,
    initial: u64,
    observed: u64,
) -> domain::WorthQueryStructuralCounterObservation {
    domain::WorthQueryStructuralCounterObservation::new(counter(name), initial, observed)
}

pub fn decision_kind() -> domain::WorthQueryDecisionKind {
    domain::WorthQueryDecisionKind::new("candidate-rejected").unwrap()
}

pub fn decision_summary(kind: domain::WorthQueryDecisionKind) -> domain::WorthQueryDecisionSummary {
    domain::WorthQueryDecisionSummary::new(
        kind,
        domain::WorthQueryDecisionSummaryCounts::new(1, 1, 1, 0),
    )
}

pub fn selected_decision_kind() -> domain::WorthQueryDecisionKind {
    domain::WorthQueryDecisionKind::new("candidate-selected").unwrap()
}

pub fn counter(name: &str) -> FoundationalPerformanceCounterName {
    FoundationalPerformanceCounterName::new(name).unwrap()
}
