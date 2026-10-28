use worth_foundational::{
    admit_requested_foundational_profile, derive_foundational_profile_identity,
    foundational_profile_progression_authority, performance, performance_api, profiles,
    request_foundational_profile_set, AdmissionReadinessProfile, CanonicalizationRuleVersion,
    CertificationPostureProfile, CompatibilityPostureProfile, DiagnosticRichnessProfile,
    FoundationalObservationDisposition, FoundationalPerformanceAccessPatternPosture,
    FoundationalPerformanceAttachmentTargetKind, FoundationalPerformanceBoundary,
    FoundationalPerformanceBreadthLocalityPosture, FoundationalPerformanceClaimConstructionDenial,
    FoundationalPerformanceEvidenceStrength, FoundationalPerformanceExecutionTemperature,
    FoundationalPerformanceFallbackDebtPosture, FoundationalPerformanceFreshnessRetentionPosture,
    FoundationalPerformanceMismatch, FoundationalPerformanceObservationContext,
    FoundationalPerformanceReportRequest, FoundationalPerformanceWorkClass,
    ObservationActivationProfile, RetentionDeliveryProfile, SupportPostureProfile,
};
use worth_proof::TransitionOutcome;

fn claim_with_work(
    work_class: FoundationalPerformanceWorkClass,
    context: Option<FoundationalPerformanceObservationContext>,
) -> Result<
    worth_foundational::FoundationalAuthoritativePerformanceClaim,
    FoundationalPerformanceClaimConstructionDenial,
> {
    let mut builder = performance()
        .claim()
        .authoritative_execution()
        .boundary(FoundationalPerformanceBoundary::AuthoritativeExecution)
        .evidence_strength(FoundationalPerformanceEvidenceStrength::CounterBackedExecutionReceipt)
        .breadth_locality(FoundationalPerformanceBreadthLocalityPosture::PointLocal)
        .access_pattern(FoundationalPerformanceAccessPatternPosture::PointLookup)
        .execution_temperature(FoundationalPerformanceExecutionTemperature::HotPath)
        .freshness_retention(FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent)
        .fallback_debt(FoundationalPerformanceFallbackDebtPosture::Verified)
        .include_work(work_class)
        .exclude_work(FoundationalPerformanceWorkClass::ReplayReconstruction)
        .exclude_work(FoundationalPerformanceWorkClass::SupportReportAssembly)
        .exclude_work(FoundationalPerformanceWorkClass::ForensicParity);
    if let Some(context) = context {
        builder = builder.observation_context(context);
    }
    builder.finish()
}

fn claim_with_excluded_work(
    work_class: FoundationalPerformanceWorkClass,
) -> Result<
    worth_foundational::FoundationalAuthoritativePerformanceClaim,
    FoundationalPerformanceClaimConstructionDenial,
> {
    performance()
        .claim()
        .authoritative_execution()
        .boundary(FoundationalPerformanceBoundary::AuthoritativeExecution)
        .evidence_strength(FoundationalPerformanceEvidenceStrength::CounterBackedExecutionReceipt)
        .breadth_locality(FoundationalPerformanceBreadthLocalityPosture::PointLocal)
        .access_pattern(FoundationalPerformanceAccessPatternPosture::PointLookup)
        .execution_temperature(FoundationalPerformanceExecutionTemperature::HotPath)
        .freshness_retention(FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent)
        .fallback_debt(FoundationalPerformanceFallbackDebtPosture::Verified)
        .include_work(FoundationalPerformanceWorkClass::AuthoritativeMutation)
        .exclude_work(work_class)
        .exclude_work(FoundationalPerformanceWorkClass::ReplayReconstruction)
        .exclude_work(FoundationalPerformanceWorkClass::SupportReportAssembly)
        .exclude_work(FoundationalPerformanceWorkClass::ForensicParity)
        .finish()
}

fn active_context(
    disposition: FoundationalObservationDisposition,
) -> FoundationalPerformanceObservationContext {
    let profile = profiles()
        .set()
        .diagnostic_richness(DiagnosticRichnessProfile::Standard)
        .support_posture(SupportPostureProfile::SupportReady)
        .compatibility_posture(CompatibilityPostureProfile::NativeOnly)
        .admission_readiness(AdmissionReadinessProfile::Admitted)
        .retention_delivery(RetentionDeliveryProfile::Retained)
        .certification_posture(CertificationPostureProfile::Uncertified)
        .execution_objective(worth_foundational::ExecutionObjectiveProfile::Balanced)
        .observation_activation(ObservationActivationProfile::Continuous)
        .compose()
        .expect("context profile");
    let admitted = match admit_requested_foundational_profile(
        request_foundational_profile_set(profile),
        profile,
        None,
        foundational_profile_progression_authority(),
    ) {
        TransitionOutcome::Success(admitted) => admitted,
        other => panic!("expected admitted profile, got {other:?}"),
    };
    let identity = match derive_foundational_profile_identity(
        CanonicalizationRuleVersion::new("m10.observation").expect("version"),
        &admitted,
    ) {
        TransitionOutcome::Success(identity) => identity,
        other => panic!("expected identity, got {other:?}"),
    };
    FoundationalPerformanceObservationContext::new(identity, disposition)
}

fn report_profile() -> worth_foundational::FoundationalProfileSet {
    profiles()
        .set()
        .diagnostic_richness(DiagnosticRichnessProfile::Standard)
        .support_posture(SupportPostureProfile::SupportReady)
        .compatibility_posture(CompatibilityPostureProfile::NativeOnly)
        .admission_readiness(AdmissionReadinessProfile::Admitted)
        .retention_delivery(RetentionDeliveryProfile::Retained)
        .certification_posture(CertificationPostureProfile::Uncertified)
        .execution_objective(worth_foundational::ExecutionObjectiveProfile::Balanced)
        .observation_activation(ObservationActivationProfile::Continuous)
        .compose()
        .expect("report profile")
}

#[test]
fn optional_observation_work_requires_active_profile_context() {
    assert_eq!(
        claim_with_work(
            FoundationalPerformanceWorkClass::DiagnosticFactCapture,
            None
        ),
        Err(FoundationalPerformanceClaimConstructionDenial::MissingObservationContext)
    );
    assert_eq!(
        claim_with_work(
            FoundationalPerformanceWorkClass::DiagnosticFactCapture,
            Some(active_context(FoundationalObservationDisposition::Inactive)),
        ),
        Err(FoundationalPerformanceClaimConstructionDenial::ObservationWorkRequiresActiveDisposition)
    );
    let expected_context = active_context(FoundationalObservationDisposition::Continuous);
    let claim = claim_with_work(
        FoundationalPerformanceWorkClass::DiagnosticFactCapture,
        Some(expected_context.clone()),
    )
    .expect("active optional observation should build");
    assert_eq!(claim.observation_context(), Some(&expected_context));
    assert!(claim
        .included_work()
        .contains(&FoundationalPerformanceWorkClass::DiagnosticFactCapture));
}

#[test]
fn every_optional_work_class_is_independently_gated_and_disclosed() {
    let optional = [
        FoundationalPerformanceWorkClass::StructuralCounterCapture,
        FoundationalPerformanceWorkClass::DiagnosticFactCapture,
        FoundationalPerformanceWorkClass::DescriptiveLineageRecordMaintenance,
        FoundationalPerformanceWorkClass::ProvenanceFactCapture,
        FoundationalPerformanceWorkClass::ReplaySidecarMaintenance,
    ];
    for work_class in optional {
        assert_eq!(
            claim_with_work(work_class, None),
            Err(FoundationalPerformanceClaimConstructionDenial::MissingObservationContext)
        );
        assert_eq!(
            claim_with_work(
                work_class,
                Some(active_context(FoundationalObservationDisposition::Inactive)),
            ),
            Err(
                FoundationalPerformanceClaimConstructionDenial::
                    ObservationWorkRequiresActiveDisposition
            )
        );
        let continuous = active_context(FoundationalObservationDisposition::Continuous);
        let claim = claim_with_work(work_class, Some(continuous.clone()))
            .expect("continuous observation should admit each optional class");
        assert_eq!(claim.observation_context(), Some(&continuous));
        assert_eq!(claim.included_work(), &[work_class]);

        let explicit = FoundationalObservationDisposition::ExplicitlyActivated {
            scope: worth_foundational::FoundationalObservationActivationScope::Operation,
            session: worth_foundational::BoundaryHandle::new(7),
            observed_epoch: worth_foundational::BoundaryEpoch::new(3),
        };
        let explicit_context = active_context(explicit);
        let claim = claim_with_work(work_class, Some(explicit_context.clone()))
            .expect("explicit observation should admit each optional class");
        assert_eq!(claim.observation_context(), Some(&explicit_context));
        assert_eq!(claim.included_work(), &[work_class]);
        assert!(claim
            .excluded_work()
            .contains(&FoundationalPerformanceWorkClass::ReplayReconstruction));

        let excluded = claim_with_excluded_work(work_class)
            .expect("each optional class should be disclosable as excluded work");
        assert!(excluded.excluded_work().contains(&work_class));
        assert!(!excluded.included_work().contains(&work_class));
        assert!(excluded.observation_context().is_none());
    }

    let control = claim_with_work(
        FoundationalPerformanceWorkClass::AuthoritativeMutation,
        None,
    )
    .expect("non-observation work does not require observation context");
    assert!(control.observation_context().is_none());
}

#[test]
fn observation_context_changes_comparison_and_canonical_basis() {
    let continuous = active_context(FoundationalObservationDisposition::Continuous);
    let explicit = active_context(FoundationalObservationDisposition::ExplicitlyActivated {
        scope: worth_foundational::FoundationalObservationActivationScope::Batch,
        session: worth_foundational::BoundaryHandle::new(8),
        observed_epoch: worth_foundational::BoundaryEpoch::new(2),
    });
    let left = performance_api::lower_lane::basis::performance_bundle(
        claim_with_work(
            FoundationalPerformanceWorkClass::DiagnosticFactCapture,
            Some(continuous),
        )
        .expect("continuous claim"),
    )
    .finish()
    .expect("left bundle");
    let right = performance_api::lower_lane::basis::performance_bundle(
        claim_with_work(
            FoundationalPerformanceWorkClass::DiagnosticFactCapture,
            Some(explicit),
        )
        .expect("explicit claim"),
    )
    .finish()
    .expect("right bundle");

    let comparison = performance_api::lower_lane::basis::compare_performance_bundles(&left, &right);
    assert!(comparison.mismatches().iter().any(|mismatch| matches!(
        mismatch,
        FoundationalPerformanceMismatch::ObservationContext { .. }
    )));
    let left_basis =
        performance_api::lower_lane::basis::prepare_performance_bundle_for_canonical_basis(
            performance_api::lower_lane::basis::performance_basis_rule_version(),
            &left,
        );
    let right_basis =
        performance_api::lower_lane::basis::prepare_performance_bundle_for_canonical_basis(
            performance_api::lower_lane::basis::performance_basis_rule_version(),
            &right,
        );
    let (TransitionOutcome::Success(left_basis), TransitionOutcome::Success(right_basis)) =
        (left_basis, right_basis)
    else {
        panic!("contextual bundle bases should prepare");
    };
    assert_ne!(
        left_basis.payload().entries(),
        right_basis.payload().entries()
    );
}

#[test]
fn observation_context_survives_report_materialization_and_canonicalization() {
    let continuous = active_context(FoundationalObservationDisposition::Continuous);
    let explicit = active_context(FoundationalObservationDisposition::ExplicitlyActivated {
        scope: worth_foundational::FoundationalObservationActivationScope::Operation,
        session: worth_foundational::BoundaryHandle::new(9),
        observed_epoch: worth_foundational::BoundaryEpoch::new(6),
    });
    let materialize = |context: FoundationalPerformanceObservationContext| {
        let claim = claim_with_work(
            FoundationalPerformanceWorkClass::DiagnosticFactCapture,
            Some(context),
        )
        .expect("report claim");
        let bundle = performance_api::lower_lane::basis::performance_bundle(claim)
            .finish()
            .expect("report bundle");
        let source = performance_api::lower_lane::reports::attach_performance_bundle(
            FoundationalPerformanceAttachmentTargetKind::BoundaryArtifact,
            bundle,
        )
        .expect("report source");
        performance_api::lower_lane::reports::plan_performance_report(
            FoundationalPerformanceReportRequest {
                source,
                profile: report_profile(),
                include_layout_intent: false,
                include_contract_names: false,
                include_counter_specs: false,
                include_counter_rows: false,
                include_supporting_evidence_rows: false,
                include_budget_decisions: false,
                include_denied_work: false,
                include_widened_work: false,
            },
        )
        .materialize()
    };
    let continuous_report = materialize(continuous.clone());
    let explicit_report = materialize(explicit.clone());
    assert_eq!(continuous_report.observation_context(), Some(&continuous));
    assert_eq!(explicit_report.observation_context(), Some(&explicit));
    let basis = |report| {
        match performance_api::lower_lane::basis::prepare_materialized_performance_report_for_canonical_basis(
            performance_api::lower_lane::basis::performance_basis_rule_version(),
            report,
        ) {
            TransitionOutcome::Success(basis) => basis,
            other => panic!("report basis should prepare: {other:?}"),
        }
    };
    assert_ne!(
        basis(&continuous_report).payload().entries(),
        basis(&explicit_report).payload().entries()
    );
}
