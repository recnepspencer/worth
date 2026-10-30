#[allow(unused_imports)]
use crate::certification::ordinary_context::performance::observation_work_disclosure::*;
use {
    worth_foundational::FoundationalPerformanceObservationContext,
    worth_foundational::FoundationalPerformanceReportRequest,
};

#[test]
fn optional_observation_work_requires_active_profile_context() {
    assert_eq!(
        claim_with_work(
            worth_foundational::FoundationalPerformanceWorkClass::DiagnosticFactCapture,
            None
        ),
        Err(worth_foundational::FoundationalPerformanceClaimConstructionDenial::MissingObservationContext)
    );
    assert_eq!(
        claim_with_work(
            worth_foundational::FoundationalPerformanceWorkClass::DiagnosticFactCapture,
            Some(active_context(worth_foundational::FoundationalObservationDisposition::Inactive)),
        ),
        Err(worth_foundational::FoundationalPerformanceClaimConstructionDenial::ObservationWorkRequiresActiveDisposition)
    );
    let expected_context =
        active_context(worth_foundational::FoundationalObservationDisposition::Continuous);
    let claim = claim_with_work(
        worth_foundational::FoundationalPerformanceWorkClass::DiagnosticFactCapture,
        Some(expected_context.clone()),
    )
    .expect("active optional observation should build");
    assert_eq!(claim.observation_context(), Some(&expected_context));
    assert!(claim
        .included_work()
        .contains(&worth_foundational::FoundationalPerformanceWorkClass::DiagnosticFactCapture));
}

#[test]
fn every_optional_work_class_is_independently_gated_and_disclosed() {
    let optional = [
        worth_foundational::FoundationalPerformanceWorkClass::StructuralCounterCapture,
        worth_foundational::FoundationalPerformanceWorkClass::DiagnosticFactCapture,
        worth_foundational::FoundationalPerformanceWorkClass::DescriptiveLineageRecordMaintenance,
        worth_foundational::FoundationalPerformanceWorkClass::ProvenanceFactCapture,
        worth_foundational::FoundationalPerformanceWorkClass::ReplaySidecarMaintenance,
    ];
    for work_class in optional {
        assert_eq!(
            claim_with_work(work_class, None),
            Err(worth_foundational::FoundationalPerformanceClaimConstructionDenial::MissingObservationContext)
        );
        assert_eq!(
            claim_with_work(
                work_class,
                Some(active_context(worth_foundational::FoundationalObservationDisposition::Inactive)),
            ),
            Err(
                worth_foundational::FoundationalPerformanceClaimConstructionDenial::ObservationWorkRequiresActiveDisposition
            )
        );
        let continuous =
            active_context(worth_foundational::FoundationalObservationDisposition::Continuous);
        let claim = claim_with_work(work_class, Some(continuous.clone()))
            .expect("continuous observation should admit each optional class");
        assert_eq!(claim.observation_context(), Some(&continuous));
        assert_eq!(claim.included_work(), &[work_class]);

        let explicit =
            worth_foundational::FoundationalObservationDisposition::ExplicitlyActivated {
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
            .contains(&worth_foundational::FoundationalPerformanceWorkClass::ReplayReconstruction));

        let excluded = claim_with_excluded_work(work_class)
            .expect("each optional class should be disclosable as excluded work");
        assert!(excluded.excluded_work().contains(&work_class));
        assert!(!excluded.included_work().contains(&work_class));
        assert!(excluded.observation_context().is_none());
    }

    let control = claim_with_work(
        worth_foundational::FoundationalPerformanceWorkClass::AuthoritativeMutation,
        None,
    )
    .expect("non-observation work does not require observation context");
    assert!(control.observation_context().is_none());
}

#[test]
fn observation_context_changes_comparison_and_canonical_basis() {
    let continuous =
        active_context(worth_foundational::FoundationalObservationDisposition::Continuous);
    let explicit = active_context(
        worth_foundational::FoundationalObservationDisposition::ExplicitlyActivated {
            scope: worth_foundational::FoundationalObservationActivationScope::Batch,
            session: worth_foundational::BoundaryHandle::new(8),
            observed_epoch: worth_foundational::BoundaryEpoch::new(2),
        },
    );
    let left = worth_foundational::performance_api::lower_lane::basis::performance_bundle(
        claim_with_work(
            worth_foundational::FoundationalPerformanceWorkClass::DiagnosticFactCapture,
            Some(continuous),
        )
        .expect("continuous claim"),
    )
    .finish()
    .expect("left bundle");
    let right = worth_foundational::performance_api::lower_lane::basis::performance_bundle(
        claim_with_work(
            worth_foundational::FoundationalPerformanceWorkClass::DiagnosticFactCapture,
            Some(explicit),
        )
        .expect("explicit claim"),
    )
    .finish()
    .expect("right bundle");

    let comparison =
        worth_foundational::performance_api::lower_lane::basis::compare_performance_bundles(
            &left, &right,
        );
    assert!(comparison.mismatches().iter().any(|mismatch| matches!(
        mismatch,
        worth_foundational::FoundationalPerformanceMismatch::ObservationContext { .. }
    )));
    let left_basis =
        worth_foundational::performance_api::lower_lane::basis::prepare_performance_bundle_for_canonical_basis(
            worth_foundational::performance_api::lower_lane::basis::performance_basis_rule_version(),
            &left,
        );
    let right_basis =
        worth_foundational::performance_api::lower_lane::basis::prepare_performance_bundle_for_canonical_basis(
            worth_foundational::performance_api::lower_lane::basis::performance_basis_rule_version(),
            &right,
        );
    let (
        worth_proof::TransitionOutcome::Success(left_basis),
        worth_proof::TransitionOutcome::Success(right_basis),
    ) = (left_basis, right_basis)
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
    let continuous =
        active_context(worth_foundational::FoundationalObservationDisposition::Continuous);
    let explicit = active_context(
        worth_foundational::FoundationalObservationDisposition::ExplicitlyActivated {
            scope: worth_foundational::FoundationalObservationActivationScope::Operation,
            session: worth_foundational::BoundaryHandle::new(9),
            observed_epoch: worth_foundational::BoundaryEpoch::new(6),
        },
    );
    let materialize = |context: FoundationalPerformanceObservationContext| {
        let claim = claim_with_work(
            worth_foundational::FoundationalPerformanceWorkClass::DiagnosticFactCapture,
            Some(context),
        )
        .expect("report claim");
        let bundle =
            worth_foundational::performance_api::lower_lane::basis::performance_bundle(claim)
                .finish()
                .expect("report bundle");
        let source =
            worth_foundational::performance_api::lower_lane::reports::attach_performance_bundle(
                worth_foundational::FoundationalPerformanceAttachmentTargetKind::BoundaryArtifact,
                bundle,
            )
            .expect("report source");
        worth_foundational::performance_api::lower_lane::reports::plan_performance_report(
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
        match worth_foundational::performance_api::lower_lane::basis::prepare_materialized_performance_report_for_canonical_basis(
            worth_foundational::performance_api::lower_lane::basis::performance_basis_rule_version(),
            report,
        ) {
            worth_proof::TransitionOutcome::Success(basis) => basis,
            other => panic!("report basis should prepare: {other:?}"),
        }
    };
    assert_ne!(
        basis(&continuous_report).payload().entries(),
        basis(&explicit_report).payload().entries()
    );
}
