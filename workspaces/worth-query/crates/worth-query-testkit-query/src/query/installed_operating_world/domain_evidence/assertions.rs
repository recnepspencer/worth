pub use std::collections::BTreeMap;

pub use worth_foundational::facade::RetentionDeliveryProfile;
pub use worth_query::facade::domain;

pub fn assert_optional_counter_sidecar(counters: &[domain::WorthQueryAdmittedStructuralCounter]) {
    assert_eq!(counters.len(), 1);
    let counter = &counters[0];
    assert_eq!(counter.schema().name().as_str(), "trace-events");
    assert_eq!(
        counter.provider_certification(),
        Some("domain-evidence-provider-v1")
    );
    assert_eq!(counter.initial(), 0);
    assert_eq!(counter.observed(), 9);
    assert_eq!(
        counter.schema().requiredness(),
        domain::WorthQueryStructuralCounterRequiredness::OptionalSidecar
    );
}

pub fn assert_governance(governance: &domain::WorthQueryDomainEvidenceGovernance) {
    assert_eq!(governance.audiences(), &["audit", "support"]);
    assert_eq!(
        governance.classification(),
        domain::WorthQueryArtifactClassification::Restricted
    );
    assert_eq!(
        governance.redaction(),
        domain::WorthQueryArtifactRedactionPosture::NotRequired
    );
    assert_eq!(governance.retention(), RetentionDeliveryProfile::Durable);
    assert_eq!(
        governance.deletion(),
        domain::WorthQueryArtifactDeletionPosture::DomainControlled
    );
    assert_eq!(
        governance.legal_hold(),
        domain::WorthQueryArtifactLegalHoldPosture::RequiredWhenDirected
    );
}

pub fn assert_mandatory_core(core: &domain::WorthQueryDomainEvidenceCore) {
    assert_required_counters(core.counters());
    assert_decision_summary(core.decisions());
    assert_search_summary(core.candidate_search().unwrap());
    assert_transformation_summary(core.transformation().unwrap());
}

pub fn assert_required_counters(counters: &[domain::WorthQueryAdmittedStructuralCounter]) {
    let counters = counters
        .iter()
        .map(|counter| {
            (
                counter.schema().name().as_str(),
                (counter.initial(), counter.observed()),
            )
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(
        counters,
        BTreeMap::from([
            ("bytes", (0, 128)),
            ("candidate-comparisons", (0, 6)),
            ("elements", (0, 4)),
            ("operation-work-components", (0, 10)),
            ("work", (0, 10)),
        ])
    );
}

pub fn assert_decision_summary(decisions: &[domain::WorthQueryAdmittedDecisionSummary]) {
    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0].schema().kind().as_str(), "candidate-rejected");
    assert_eq!(
        decisions[0].schema().reason_family().as_str(),
        "search-reason"
    );
    assert_eq!(
        decisions[0]
            .schema()
            .affected_artifact_key_family()
            .as_str(),
        "candidate"
    );
    assert_eq!(
        decisions[0].counts(),
        domain::WorthQueryDecisionSummaryCounts::new(1, 1, 1, 0)
    );
}

pub fn assert_search_summary(search: &domain::WorthQueryCandidateSearchSummary) {
    let search = search.parts();
    assert_eq!(search.universe.family(), "candidate-universe");
    assert_eq!(search.universe.value(), "sample-v1");
    assert_eq!(search.considered_count, 2);
    assert_eq!(
        search.termination,
        domain::WorthQueryCandidateTerminationClass::SampleCompleted
    );
    assert_eq!(
        search.completeness,
        domain::WorthQueryCandidateSearchPosture::Sampled {
            sample_identity: "sample-v1".into()
        }
    );
    assert_eq!(
        search.feasibility,
        domain::WorthQueryCandidateFeasibilityClass::FeasibleCandidateFound
    );
    assert_eq!(
        search.optimality,
        domain::WorthQueryCandidateOptimalityPosture::BestInDeclaredSample {
            sample_identity: "sample-v1".into()
        }
    );
    assert_eq!(search.rejected_count, 1);
    assert_eq!(
        search.incumbent,
        domain::WorthQueryCandidateIncumbentDisposition::Selected
    );
}

pub fn assert_transformation_summary(transformation: &domain::WorthQueryTransformationSummary) {
    let transformation = transformation.parts();
    assert_eq!(
        transformation.source_occurrence.family(),
        "source-occurrence"
    );
    assert_eq!(transformation.source_occurrence.value(), "source-1");
    assert_eq!(transformation.transformation_family, "normalize-candidates");
    assert_eq!(transformation.transformation_version, 1);
    assert_eq!(
        transformation.correspondence,
        domain::WorthQuerySourceOutputCorrespondence::OneToMany
    );
    assert_eq!(
        transformation.disposition,
        domain::WorthQueryTransformationDisposition::Normalized
    );
    assert_eq!(
        transformation.error,
        domain::WorthQueryTransformationErrorPosture::Bounded
    );
    assert_eq!(
        transformation.loss,
        domain::WorthQueryTransformationLossPosture::DeclaredLossy
    );
}
