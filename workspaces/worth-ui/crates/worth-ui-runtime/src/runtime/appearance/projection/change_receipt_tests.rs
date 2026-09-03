use worth_ui_dsl::{UiAppearanceAspect, UiThemeColor, UiThemeSlotIdentity, UiThemeValue};

use super::{UiAppearanceChangeOutcome, UiAppearanceChangeReceipt};

#[test]
fn receipt_exposes_each_gate_one_change_outcome_without_host_work() {
    super::super::tests::run_on_appearance_fixture_stack(|| {
        let (mut session, binding, _target, vector, theme) = super::super::tests::inputs();
        let resolver = super::super::UiAppearanceResolver::new();
        let unchanged_mount = crate::mounting::unchanged_test_work();
        let changed_mount = crate::mounting::changed_test_work();
        let base = resolver
            .resolve_node(
                session.graph().snapshot(),
                session.capabilities(),
                &binding,
                &vector,
                &theme,
            )
            .expect("sealed appearance inputs should resolve");
        let original = &base.aspects()[0];

        let candidate = crate::runtime::tests::appearance_component_session_test_support::
        attached_appearance_candidate_submission(
            &session,
            "appearance-receipt-evidence",
            "workspace.component.active_session_current",
        );
        let mut turn = session.begin_observation_turn().unwrap();
        turn.admit_source(candidate).unwrap();
        let observations = turn.seal().unwrap();
        session.classify_observations(observations).unwrap();
        let later_snapshot = session.appearance_owner_snapshot_for_test().unwrap();
        let later_vector = super::super::super::state::UiAppearanceStateVector::seal_for_binding(
            later_snapshot,
            session.graph().snapshot(),
            session.capabilities(),
            &binding,
        )
        .unwrap();
        let evidence_successor = resolver
            .resolve_node(
                session.graph().snapshot(),
                session.capabilities(),
                &binding,
                &later_vector,
                &theme,
            )
            .expect("the later sealed observation remains resolvable");

        let semantic_successor = projection_with_aspect(
            &base,
            binding.role(),
            &theme,
            aspect_with(
                original,
                original.value(),
                UiThemeSlotIdentity::new("theme.alternate").unwrap(),
                original.provenance().source(),
                11,
            ),
        );
        let value_successor = projection_with_aspect(
            &base,
            binding.role(),
            &theme,
            aspect_with(
                original,
                UiThemeValue::Color(UiThemeColor::from_channels([9, 8, 7, 255])),
                original.provenance().selected_slot().clone(),
                original.provenance().source(),
                12,
            ),
        );
        let equal_output_successor = projection_with_aspect(
            &base,
            binding.role(),
            &theme,
            aspect_with(
                original,
                original.value(),
                original.provenance().selected_slot().clone(),
                "theme.equal-output-evidence",
                13,
            ),
        );

        assert_receipt(
            UiAppearanceChangeReceipt::compare(
                Some(&base),
                Some(&evidence_successor),
                &unchanged_mount,
            ),
            UiAppearanceChangeOutcome::InputEvidenceChanged,
            [true, false, false, false, false, false],
        );
        assert_receipt(
            UiAppearanceChangeReceipt::compare(
                Some(&base),
                Some(&semantic_successor),
                &unchanged_mount,
            ),
            UiAppearanceChangeOutcome::SemanticProjectionChanged,
            [false, true, false, false, false, false],
        );
        assert_receipt(
            UiAppearanceChangeReceipt::compare(
                Some(&base),
                Some(&value_successor),
                &unchanged_mount,
            ),
            UiAppearanceChangeOutcome::ResolvedAspectValueChanged,
            [false, true, true, false, false, false],
        );
        assert_receipt(
            UiAppearanceChangeReceipt::compare(Some(&base), Some(&base), &changed_mount),
            UiAppearanceChangeOutcome::MountedMechanicalOutputChanged,
            [false, false, false, true, false, false],
        );
        assert_receipt(
            UiAppearanceChangeReceipt::compare(
                Some(&base),
                Some(&equal_output_successor),
                &unchanged_mount,
            ),
            UiAppearanceChangeOutcome::EqualOutputSuppressed,
            [false, true, false, false, true, false],
        );
        assert_receipt(
            UiAppearanceChangeReceipt::denied(),
            UiAppearanceChangeOutcome::DeniedBeforeEffects,
            [false, false, false, false, false, true],
        );
        assert_inspection(
            &mut session,
            &evidence_successor,
            UiAppearanceChangeReceipt::compare(
                Some(&base),
                Some(&evidence_successor),
                &unchanged_mount,
            ),
            worth_ui_inspection::UiAppearanceInspectionInvalidationCause::InputEvidenceChanged,
            worth_ui_inspection::UiAppearanceInspectionMountedMechanic::Unchanged,
            worth_ui_inspection::UiAppearanceInspectionPhysicalSuppression::NotSuppressed,
        );
        assert_inspection(
            &mut session,
            &semantic_successor,
            UiAppearanceChangeReceipt::compare(
                Some(&base),
                Some(&semantic_successor),
                &unchanged_mount,
            ),
            worth_ui_inspection::UiAppearanceInspectionInvalidationCause::SemanticProjectionChanged,
            worth_ui_inspection::UiAppearanceInspectionMountedMechanic::Unchanged,
            worth_ui_inspection::UiAppearanceInspectionPhysicalSuppression::NotSuppressed,
        );
        assert_inspection(
            &mut session,
            &value_successor,
            UiAppearanceChangeReceipt::compare(Some(&base), Some(&value_successor), &unchanged_mount),
            worth_ui_inspection::UiAppearanceInspectionInvalidationCause::ResolvedAspectValueChanged,
            worth_ui_inspection::UiAppearanceInspectionMountedMechanic::Unchanged,
            worth_ui_inspection::UiAppearanceInspectionPhysicalSuppression::NotSuppressed,
        );
        assert_inspection(
            &mut session,
            &base,
            UiAppearanceChangeReceipt::compare(Some(&base), Some(&base), &changed_mount),
            worth_ui_inspection::UiAppearanceInspectionInvalidationCause::MountedMechanicalOutputChanged,
            worth_ui_inspection::UiAppearanceInspectionMountedMechanic::Changed,
            worth_ui_inspection::UiAppearanceInspectionPhysicalSuppression::NotSuppressed,
        );
        assert_inspection(
            &mut session,
            &equal_output_successor,
            UiAppearanceChangeReceipt::compare(
                Some(&base),
                Some(&equal_output_successor),
                &unchanged_mount,
            ),
            worth_ui_inspection::UiAppearanceInspectionInvalidationCause::EqualOutputSuppressed,
            worth_ui_inspection::UiAppearanceInspectionMountedMechanic::Unchanged,
            worth_ui_inspection::UiAppearanceInspectionPhysicalSuppression::Suppressed,
        );
        assert_inspection(
            &mut session,
            &base,
            UiAppearanceChangeReceipt::denied(),
            worth_ui_inspection::UiAppearanceInspectionInvalidationCause::DeniedBeforeEffects,
            worth_ui_inspection::UiAppearanceInspectionMountedMechanic::NotAttempted,
            worth_ui_inspection::UiAppearanceInspectionPhysicalSuppression::NotAttempted,
        );
        assert!(
            session
                .inspect_mounted_identity()
                .frame_receipts()
                .is_empty(),
            "receipt evidence must not assemble or publish a host frame"
        );
        let _ = session.shutdown();
    });
}

fn assert_inspection(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    projection: &super::super::UiAppearanceProjection,
    receipt: UiAppearanceChangeReceipt,
    cause: worth_ui_inspection::UiAppearanceInspectionInvalidationCause,
    mounted: worth_ui_inspection::UiAppearanceInspectionMountedMechanic,
    physical: worth_ui_inspection::UiAppearanceInspectionPhysicalSuppression,
) {
    let world = worth_ui_inspection::UiAppearanceInspectionWorld::new(
        projection.state().basis().session().as_u64(),
        projection
            .state()
            .basis()
            .generation()
            .prepared_generation()
            .semantic_package_identity()
            .narrowing_fingerprint(),
        projection.state().basis().surface().diagnostic_value(),
    );
    session.record_appearance_projection_for_inspection(projection, 1, receipt);
    let worth_ui_inspection::UiAppearanceInspectionOutcome::Found(explanation) = session
        .why_appearance(worth_ui_inspection::UiAppearanceInspectionQuery::new(
            world,
            projection.target().graph_node().digest(),
            projection.aspects()[0].aspect(),
        ))
    else {
        panic!("production why_appearance facade should expose recorded receipt");
    };
    assert_eq!(explanation.invalidation_cause(), cause);
    assert_eq!(explanation.mounted_mechanic(), mounted);
    assert_eq!(explanation.physical_suppression(), physical);
}

fn projection_with_aspect(
    base: &super::super::UiAppearanceProjection,
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
    theme: &super::super::super::theme::UiThemeResolutionView,
    aspect: super::super::UiResolvedAppearanceAspect,
) -> super::super::UiAppearanceProjection {
    super::super::UiAppearanceProjection::seal(
        &base.target(),
        role,
        base.state().clone(),
        theme,
        [aspect].into_iter().collect(),
    )
}

fn aspect_with(
    original: &super::super::UiResolvedAppearanceAspect,
    value: UiThemeValue,
    selected_slot: UiThemeSlotIdentity,
    source: &str,
    digest: u64,
) -> super::super::UiResolvedAppearanceAspect {
    let terminal = original.provenance().terminal_slot().clone();
    super::super::UiResolvedAppearanceAspect::new(
        UiAppearanceAspect::Background,
        original.state_classes().to_vec().into_boxed_slice(),
        original.decision_cell_ordinal(),
        value,
        super::super::UiAppearanceProvenance::new(selected_slot, terminal, source, 0),
        original.support(),
        digest,
        original.decision_cells_visited(),
        original.theme_slots_compared(),
    )
}

fn assert_receipt(
    receipt: UiAppearanceChangeReceipt,
    outcome: UiAppearanceChangeOutcome,
    flags: [bool; 6],
) {
    assert_eq!(receipt.outcome(), Some(outcome));
    assert_eq!(receipt.input_evidence_changed(), flags[0]);
    assert_eq!(receipt.semantic_projection_changed(), flags[1]);
    assert_eq!(receipt.resolved_aspect_value_changed(), flags[2]);
    assert_eq!(receipt.mounted_mechanical_output_changed(), flags[3]);
    assert_eq!(receipt.equal_output_suppressed(), flags[4]);
    assert_eq!(receipt.denied_before_effects(), flags[5]);
}
