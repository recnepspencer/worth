use worth_ui_runtime::facade::mounted::{
    UiHostPresentationReconciliation, UiHostSurfacePresentationMode, UiMountedFrameOutcome,
    UiMountedFrameRequest, UiMountedFrameReuse, UiMountedIdentityDenial,
    UiMountedPresentationAdmissionDenial, UiPresentationDeadline,
};
use worth_ui_test_support::WorthUiMountedIdentityCertificationExt;
use worth_ui_test_support::WorthUiMountedPublicationCertificationExt;

use super::mounted_application_lifecycle::in_flight_presentation_world::{
    mounted_session, prepared,
};
use super::mounted_application_lifecycle::known_empty_surface_world::profile;
use super::mounted_host_protocol::scripted_host::ScriptedPresentationHost;

#[path = "mounted_presentation_reconciliation/support.rs"]
mod support;

use support::{classify_reuse, expect_published, prepared_frame, prepared_with_request};

#[test]
fn published_predecessor_survives_indeterminacy_and_requires_exact_re_presentation() {
    let host = ScriptedPresentationHost::default();
    let (mut session, bindings) =
        mounted_session(host.clone(), "published-current-reconciliation", 1);
    let request = UiMountedFrameRequest::all_bound_surfaces();
    let affected_binding = bindings[0];
    host.push_presented();
    let predecessor_frame = prepared_with_request(&mut session, &request);
    let predecessor = expect_published(session.present_prepared_mounted_frame(
        predecessor_frame,
        UiPresentationDeadline::at_tick(10),
        0,
    ));

    host.push_presentation(
        worth_ui_host_contract::UiHostSurfacePresentationOutcome::PresentationIndeterminate,
    );
    let failed_candidate = prepared_frame(&mut session);
    assert!(matches!(
        session.present_prepared_mounted_frame(
            failed_candidate,
            UiPresentationDeadline::at_tick(10),
            1,
        ),
        UiMountedFrameOutcome::PresentationIndeterminate(_)
    ));
    assert_eq!(session.current_mounted_publication(), Some(&predecessor));

    let replacement = session
        .rebind_host_surface(
            affected_binding,
            UiHostSurfacePresentationMode::RecordOnly,
            profile(2),
        )
        .unwrap();
    assert_eq!(
        session.current_mounted_publication(),
        Some(&predecessor),
        "mechanical rebind cannot erase predecessor runtime truth"
    );
    assert!(matches!(
        classify_reuse(&mut session, &request),
        UiMountedFrameReuse::ComparisonRequired(_)
    ));
    assert!(!session.reconcile_mounted_presentation(
        UiHostPresentationReconciliation::KnownEmptyBaseline {
            affected_binding,
            replacement,
        }
    ));

    let calls_before_blocked_attempt = host.presentation_calls();
    let blocked = prepared_frame(&mut session);
    assert!(matches!(
        session.present_prepared_mounted_frame(
            blocked,
            UiPresentationDeadline::at_tick(10),
            2,
        ),
        UiMountedFrameOutcome::AdmissionDenied(rejection)
            if matches!(
                rejection.denial(),
                UiMountedPresentationAdmissionDenial::BindingRequiresReconciliation(_)
            )
    ));
    assert_eq!(host.presentation_calls(), calls_before_blocked_attempt);

    host.push_presented();
    let reconciled = session
        .present_current_mounted_frame_for_reconciliation(
            &[
                worth_ui_runtime::facade::mounted::UiMountedSurfaceReconciliationBinding::new(
                    affected_binding,
                    replacement.binding_generation(),
                ),
            ],
            UiPresentationDeadline::at_tick(10),
            3,
        )
        .unwrap();
    let UiMountedFrameOutcome::Reconciled(reconciled) = reconciled else {
        panic!("exact current-frame re-presentation must reconcile without minting a frame");
    };
    assert_eq!(reconciled.frame(), predecessor.frame());
    assert_eq!(reconciled.bindings(), &[replacement.binding_generation()]);
    assert_eq!(session.current_mounted_publication(), Some(&reconciled));
    let UiMountedFrameReuse::Exact(witness) = classify_reuse(&mut session, &request) else {
        panic!("reconciled current truth must restore exact reuse authority");
    };
    assert_eq!(witness.frame(), predecessor.frame());

    host.push_presented();
    let successor_frame = prepared_frame(&mut session);
    assert!(matches!(
        session.present_prepared_mounted_frame(
            successor_frame,
            UiPresentationDeadline::at_tick(20),
            4,
        ),
        UiMountedFrameOutcome::Published(_)
    ));
}

#[test]
fn multi_surface_reconciliation_re_presents_one_complete_current_frame() {
    let host = ScriptedPresentationHost::default();
    let (mut session, affected) =
        mounted_session(host.clone(), "multi-surface-current-reconciliation", 2);
    host.push_presented();
    host.push_presented();
    let predecessor_frame = prepared_frame(&mut session);
    let predecessor = expect_published(session.present_prepared_mounted_frame(
        predecessor_frame,
        UiPresentationDeadline::at_tick(10),
        0,
    ));

    host.push_presented();
    host.push_presentation(
        worth_ui_host_contract::UiHostSurfacePresentationOutcome::PresentationIndeterminate,
    );
    let failed = prepared_frame(&mut session);
    assert!(matches!(
        session.present_prepared_mounted_frame(failed, UiPresentationDeadline::at_tick(10), 1),
        UiMountedFrameOutcome::PresentationIndeterminate(_)
    ));

    let mut replacements = Vec::new();
    for (epoch, affected_binding) in affected.iter().enumerate() {
        let replacement = session
            .rebind_host_surface(
                *affected_binding,
                UiHostSurfacePresentationMode::RecordOnly,
                profile(u64::try_from(epoch + 10).unwrap()),
            )
            .unwrap();
        replacements.push(
            worth_ui_runtime::facade::mounted::UiMountedSurfaceReconciliationBinding::new(
                *affected_binding,
                replacement.binding_generation(),
            ),
        );
    }
    host.push_presented();
    host.push_presented();
    let outcome = session
        .present_current_mounted_frame_for_reconciliation(
            &replacements,
            UiPresentationDeadline::at_tick(20),
            2,
        )
        .unwrap();
    let UiMountedFrameOutcome::Reconciled(reconciled) = outcome else {
        panic!("all affected bindings must reconcile through one complete predecessor frame");
    };
    assert_eq!(reconciled.frame(), predecessor.frame());
    assert_eq!(reconciled.bindings().len(), 2);
    assert_eq!(host.presentation_calls(), 6);
}

#[test]
fn incomplete_duplicate_and_cross_surface_reconciliation_sets_deny_before_effects() {
    let host = ScriptedPresentationHost::default();
    let (mut session, affected) =
        mounted_session(host.clone(), "invalid-current-reconciliation-set", 2);
    host.push_presented();
    host.push_presented();
    let predecessor_frame = prepared_frame(&mut session);
    expect_published(session.present_prepared_mounted_frame(
        predecessor_frame,
        UiPresentationDeadline::at_tick(10),
        0,
    ));

    host.push_presented();
    host.push_presentation(
        worth_ui_host_contract::UiHostSurfacePresentationOutcome::PresentationIndeterminate,
    );
    let failed = prepared_frame(&mut session);
    assert!(matches!(
        session.present_prepared_mounted_frame(failed, UiPresentationDeadline::at_tick(10), 1),
        UiMountedFrameOutcome::PresentationIndeterminate(_)
    ));

    let replacements = affected
        .iter()
        .enumerate()
        .map(|(epoch, binding)| {
            let replacement = session
                .rebind_host_surface(
                    *binding,
                    UiHostSurfacePresentationMode::RecordOnly,
                    profile(u64::try_from(epoch + 30).unwrap()),
                )
                .unwrap();
            worth_ui_runtime::facade::mounted::UiMountedSurfaceReconciliationBinding::new(
                *binding,
                replacement.binding_generation(),
            )
        })
        .collect::<Vec<_>>();
    let calls_before_denials = host.presentation_calls();

    assert!(matches!(
        session
            .present_current_mounted_frame_for_reconciliation(
                &replacements[..1],
                UiPresentationDeadline::at_tick(20),
                2,
            )
            .unwrap(),
        UiMountedFrameOutcome::AdmissionDenied(rejection)
            if rejection.denial()
                == UiMountedPresentationAdmissionDenial::ReconciliationBasisMismatch
    ));
    assert!(matches!(
        session.present_current_mounted_frame_for_reconciliation(
            &[replacements[0], replacements[0]],
            UiPresentationDeadline::at_tick(20),
            2,
        ),
        Err(UiMountedIdentityDenial::ReconciliationBasisMismatch)
    ));
    assert!(matches!(
        session.present_current_mounted_frame_for_reconciliation(
            &[
                worth_ui_runtime::facade::mounted::UiMountedSurfaceReconciliationBinding::new(
                    affected[0],
                    replacements[1].replacement(),
                )
            ],
            UiPresentationDeadline::at_tick(20),
            2,
        ),
        Err(UiMountedIdentityDenial::ReconciliationBasisMismatch)
    ));
    assert_eq!(host.presentation_calls(), calls_before_denials);

    host.push_presented();
    host.push_presented();
    assert!(matches!(
        session
            .present_current_mounted_frame_for_reconciliation(
                &replacements,
                UiPresentationDeadline::at_tick(20),
                3,
            )
            .unwrap(),
        UiMountedFrameOutcome::Reconciled(_)
    ));
}

#[test]
fn verified_candidate_only_deregistration_closes_its_blocked_generation() {
    let host = ScriptedPresentationHost::default();
    let (mut session, current_bindings) =
        mounted_session(host.clone(), "candidate-only-deregistration", 1);
    host.push_presented();
    let predecessor_frame = prepared_frame(&mut session);
    let predecessor = expect_published(session.present_prepared_mounted_frame(
        predecessor_frame,
        UiPresentationDeadline::at_tick(10),
        0,
    ));
    let node = session
        .inspect_mounted_identity()
        .mounted_instances()
        .first()
        .unwrap()
        .graph_node_identity();
    let node = session.mounted_graph_node(node).unwrap();
    let candidate_surface = session.create_semantic_surface().unwrap();
    let candidate_binding = session
        .register_host_surface(
            candidate_surface,
            UiHostSurfacePresentationMode::RecordOnly,
            profile(20),
        )
        .unwrap()
        .binding_generation();
    let candidate_instance = session.mount_instance(node, candidate_surface).unwrap();

    host.push_presented();
    host.push_presentation(
        worth_ui_host_contract::UiHostSurfacePresentationOutcome::PresentationIndeterminate,
    );
    let failed = prepared_frame(&mut session);
    assert!(matches!(
        session.present_prepared_mounted_frame(failed, UiPresentationDeadline::at_tick(10), 1),
        UiMountedFrameOutcome::PresentationIndeterminate(_)
    ));
    let current_replacement = session
        .rebind_host_surface(
            current_bindings[0],
            UiHostSurfacePresentationMode::RecordOnly,
            profile(21),
        )
        .unwrap();
    let deregistered_candidate = session.deregister_host_surface(candidate_binding).unwrap();
    assert_eq!(deregistered_candidate, candidate_surface);
    assert_eq!(session.current_mounted_publication(), Some(&predecessor));

    host.push_presented();
    assert!(matches!(
        session
            .present_current_mounted_frame_for_reconciliation(
                &[
                    worth_ui_runtime::facade::mounted::UiMountedSurfaceReconciliationBinding::new(
                        current_bindings[0],
                        current_replacement.binding_generation(),
                    )
                ],
                UiPresentationDeadline::at_tick(20),
                2,
            )
            .unwrap(),
        UiMountedFrameOutcome::Reconciled(_)
    ));

    session
        .register_host_surface(
            candidate_surface,
            UiHostSurfacePresentationMode::RecordOnly,
            profile(22),
        )
        .unwrap();
    host.push_presented();
    host.push_presented();
    let successor = prepared_frame(&mut session);
    let candidate_projection = successor
        .surfaces()
        .iter()
        .find(|surface| surface.projection().surface() == candidate_surface)
        .expect("successor must project the restored candidate-only surface");
    assert!(candidate_projection
        .projection()
        .nodes()
        .iter()
        .any(|node| node.mounted_instance() == candidate_instance));
    assert!(matches!(
        session.present_prepared_mounted_frame(successor, UiPresentationDeadline::at_tick(30), 3,),
        UiMountedFrameOutcome::Published(_)
    ));
}
