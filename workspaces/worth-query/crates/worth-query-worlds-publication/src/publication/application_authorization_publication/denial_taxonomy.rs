pub use worth_foundational::facade::{
    AdmissionReadinessProfile, CertificationPostureProfile, CompatibilityPostureProfile,
    DiagnosticRichnessProfile, ExecutionObjectiveProfile, FoundationalBoundaryArtifactCategory,
    FoundationalBoundaryEvidenceCloseoutDisposition, FoundationalBoundaryEvidenceExecutionPosture,
    FoundationalBoundaryEvidenceFreshnessPosture, FoundationalBoundaryEvidenceLocality,
    FoundationalBoundaryEvidenceReceiptKind, FoundationalDiagnosticDenialClass,
    FoundationalDiagnosticOutcomeKind, FoundationalDiagnosticRow,
    FoundationalProfileAttachmentTargetKind, FoundationalProfileSet, FoundationalProfileSetInput,
    ObservationActivationProfile, RetentionDeliveryProfile, SupportPostureProfile,
};
pub use worth_query_execution::facade::primary_graph::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
pub use worth_query_execution::facade::primary_graph::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind as Kind,
};
pub use worth_query_publication::facade::domain_computation::{
    publish_application_authorization_denial, WorthQueryApplicationAuthorizationPublicationProfile,
    WorthQueryPublishedApplicationAuthorizationDenial,
    WorthQueryPublishedApplicationAuthorizationDenialCause as Cause,
};

pub use super::installed_composition::{real_denial, CompositionScenario};

#[derive(Clone, Copy)]
pub struct ExpectedDenialPublication {
    pub cause: Cause,
    pub diagnostic_code: &'static str,
}

impl ExpectedDenialPublication {
    pub const MISSING: Self = Self {
        cause: Cause::MissingCapability,
        diagnostic_code: "worth.query.authorization.missing-capability",
    };

    pub const EXPLICIT: Self = Self {
        cause: Cause::ExplicitPolicyDenial,
        diagnostic_code: "worth.query.authorization.explicit-policy-denial",
    };
}

pub fn assert_publication(
    denial: &WorthQueryOperationAuthorizationDenial,
    expected: ExpectedDenialPublication,
) -> WorthQueryPublishedApplicationAuthorizationDenial {
    let profile = exact_profile();
    let published = publish_application_authorization_denial(
        denial,
        WorthQueryApplicationAuthorizationPublicationProfile::exact(profile),
    )
    .unwrap();
    assert_closed_denial(&published, denial, expected.cause);
    assert_boundary_profile(&published, &profile);
    assert_denial_diagnostic(&published, expected.diagnostic_code);
    assert_current_provenance(&published);
    assert_denied_closeout(&published);
    assert_publication_receipt(&published);
    published
}

pub fn assert_closed_denial(
    published: &WorthQueryPublishedApplicationAuthorizationDenial,
    denial: &WorthQueryOperationAuthorizationDenial,
    expected_cause: Cause,
) {
    assert_eq!(published.artifact().cause(), expected_cause);
    assert_eq!(
        published.artifact().contributing_cause_count(),
        denial.causes().len()
    );
}

pub fn assert_boundary_profile(
    published: &WorthQueryPublishedApplicationAuthorizationDenial,
    profile: &FoundationalProfileSet,
) {
    assert_eq!(
        published.boundary_category(),
        FoundationalBoundaryArtifactCategory::Artifact
    );
    assert_eq!(
        published.boundary().payload().target_kind(),
        FoundationalProfileAttachmentTargetKind::BoundaryArtifact
    );
    let progression = published.boundary().payload().profile();
    assert_eq!(progression.requested(), profile);
    assert_eq!(progression.admitted(), profile);
    assert_eq!(progression.materialized(), profile);
}

pub fn assert_denial_diagnostic(
    published: &WorthQueryPublishedApplicationAuthorizationDenial,
    expected_code: &str,
) {
    assert_eq!(
        published.explanation().outcome_kind(),
        FoundationalDiagnosticOutcomeKind::Denied
    );
    assert_eq!(published.explanation().rows().len(), 1);
    assert_eq!(
        published.explanation().rows()[0].code().as_str(),
        expected_code
    );
    let FoundationalDiagnosticRow::Decision(row) = &published.explanation().rows()[0] else {
        panic!("authorization publication must contain one decision row");
    };
    assert_eq!(
        row.denial_class(),
        Some(FoundationalDiagnosticDenialClass::PolicyDenied)
    );
}

pub fn assert_current_provenance(published: &WorthQueryPublishedApplicationAuthorizationDenial) {
    assert_eq!(
        published.provenance().locality(),
        FoundationalBoundaryEvidenceLocality::Current
    );
    assert_eq!(
        published.provenance().freshness_posture(),
        FoundationalBoundaryEvidenceFreshnessPosture::FreshRetained
    );
    assert_ne!(publication_identity(published), 0);
}

pub fn assert_denied_closeout(published: &WorthQueryPublishedApplicationAuthorizationDenial) {
    let closeout = published.denied_closeout_receipt();
    assert_eq!(
        closeout.receipt_kind(),
        FoundationalBoundaryEvidenceReceiptKind::Closeout
    );
    assert_eq!(
        closeout.closeout_disposition(),
        Some(FoundationalBoundaryEvidenceCloseoutDisposition::Denied)
    );
    assert_eq!(
        closeout.execution_posture(),
        FoundationalBoundaryEvidenceExecutionPosture::NotExecuted
    );
    assert!(!closeout.did_execute());
}

pub fn assert_publication_receipt(published: &WorthQueryPublishedApplicationAuthorizationDenial) {
    let publication = published.publication_receipt();
    assert_eq!(
        publication.receipt_kind(),
        FoundationalBoundaryEvidenceReceiptKind::Publication
    );
    assert_eq!(
        publication.execution_posture(),
        FoundationalBoundaryEvidenceExecutionPosture::Executed
    );
    assert!(publication.did_execute());
}

pub fn publication_identity(
    publication: &WorthQueryPublishedApplicationAuthorizationDenial,
) -> u64 {
    publication
        .provenance()
        .source_basis()
        .boundary_artifact_locator()
        .unwrap()
        .artifact_id()
        .get()
}

pub fn exact_profile() -> FoundationalProfileSet {
    FoundationalProfileSet::new(FoundationalProfileSetInput {
        diagnostic_richness: DiagnosticRichnessProfile::Standard,
        support_posture: SupportPostureProfile::SupportReady,
        compatibility_posture: CompatibilityPostureProfile::CompatibilityLowered,
        execution_objective: ExecutionObjectiveProfile::Balanced,
        observation_activation: ObservationActivationProfile::Continuous,
        admission_readiness: AdmissionReadinessProfile::Admitted,
        retention_delivery: RetentionDeliveryProfile::Retained,
        certification_posture: CertificationPostureProfile::EvidenceBacked,
    })
    .unwrap()
}
