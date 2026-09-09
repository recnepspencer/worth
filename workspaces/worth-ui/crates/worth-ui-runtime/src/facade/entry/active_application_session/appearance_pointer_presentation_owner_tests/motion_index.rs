use super::*;
use crate::mounting::UiMountedMotionSampleSettlement;
use crate::runtime::motion::{UiMotionCommitReceipt, UiMotionDeclaration, UiMotionTargetIdentity};

#[test]
fn presented_hit_index_changes_only_after_host_settlement_and_preserves_retired_geometry() {
    let role = super::super::fixture::role();
    let (mut session, host) = super::super::fixture::session(&role);
    let (surface, graph) = super::super::mounting_fixture::mount(&mut session, 1_000);
    super::super::close_source_turn(&mut session, &role, "indexed-motion-initial");
    publish(&mut session, &host, 1);
    let basis = presentation(&session, surface);
    let row = *session
        .mounted
        .interaction_hit_test_basis(basis)
        .unwrap()
        .rows()
        .iter()
        .find(|row| {
            session
                .mounted
                .current_mounted_identity_basis(row.mounted_instance())
                .unwrap()
                .graph_node_identity()
                == graph
        })
        .unwrap();
    let bounds = row.bounds();
    let point = [
        f64::from(bounds.x() + bounds.width() / 2.0),
        f64::from(bounds.y() + bounds.height() / 2.0),
    ];
    let target = UiMotionTargetIdentity::from_family_owner(surface, row.mounted_instance(), 99);
    // Only the semantic track receipt is a sampling fixture. Mounted completion,
    // work production, host acceptance, retention, and point lookup are production paths.
    let receipt = UiMotionCommitReceipt::for_sampling_test_transition(
        401,
        target,
        basis,
        Some([
            bounds.x(),
            bounds.y() - bounds.height() * 2.0,
            bounds.width(),
            bounds.height(),
        ]),
        true,
        Some([bounds.x(), bounds.y(), bounds.width(), bounds.height()]),
        true,
        UiMotionDeclaration::portal_entrance(),
        None,
    );
    session.mounted.set_reduced_motion_posture(crate::mounting::presentation::motion_sampling::UiPresentationReducedMotionPosture::NoPreference);
    session.mounted.install_motion_commit(receipt).unwrap();
    assert!(contains(&session, basis, point, row.mounted_instance()));
    host.push_rejected();
    let prepared = session.mounted.prepare_motion_tick(1, basis).unwrap();
    assert!(matches!(
        session
            .mounted
            .present_prepared_motion_tick(&session.host_session, prepared, basis),
        UiMountedMotionSampleSettlement::Discarded
    ));
    assert!(contains(&session, basis, point, row.mounted_instance()));
    host.push_native_display_presented();
    let prepared = session.mounted.prepare_motion_tick(1, basis).unwrap();
    let committed =
        match session
            .mounted
            .present_prepared_motion_tick(&session.host_session, prepared, basis)
        {
            UiMountedMotionSampleSettlement::Committed(receipt) => receipt,
            _ => panic!("scripted host must commit the sample"),
        };
    let sampled = committed.samples()[0]
        .geometry()
        .unwrap()
        .presentation_basis();
    assert!(
        !contains(&session, sampled, point, row.mounted_instance()),
        "sample={:?}; original={:?}; full={:?}",
        committed.samples()[0].geometry(),
        bounds,
        session
            .mounted
            .interaction_hit_test_basis(sampled)
            .unwrap()
            .rows()
    );
    let exit = UiMotionCommitReceipt::for_sampling_test_transition(
        402,
        target,
        sampled,
        Some([bounds.x(), bounds.y(), bounds.width(), bounds.height()]),
        true,
        Some([bounds.x(), bounds.y(), bounds.width(), bounds.height()]),
        false,
        UiMotionDeclaration::portal_exit(),
        None,
    );
    let track = exit.track().identity();
    session.mounted.install_motion_commit(exit).unwrap();
    for tick in [2, 200] {
        host.push_native_display_presented();
        let prepared = session.mounted.prepare_motion_tick(tick, sampled).unwrap();
        assert!(matches!(
            session
                .mounted
                .present_prepared_motion_tick(&session.host_session, prepared, sampled),
            UiMountedMotionSampleSettlement::Committed(_)
        ));
    }
    assert!(session.mounted.retire_terminal_motion_sample(track));
    assert!(
        !contains(&session, sampled, point, row.mounted_instance()),
        "retiring bookkeeping cannot replace presented geometry"
    );
    super::super::close_source_turn(&mut session, &role, "indexed-motion-successor");
    publish(&mut session, &host, 2);
    let successor = presentation(&session, surface);
    assert!(
        contains(&session, successor, point, row.mounted_instance()),
        "mounted successor replaces the prior sample basis"
    );
}

fn contains(
    session: &crate::facade::WorthUiActiveApplicationSession,
    presentation: UiHostObservationPresentationBasis,
    point: [f64; 2],
    instance: UiMountedInstanceIdentity,
) -> bool {
    session
        .mounted
        .interaction_hit_test_candidates(presentation, point)
        .unwrap_or_else(|_| panic!("presented candidate basis must remain available"))
        .rows()
        .iter()
        .any(|row| row.mounted_instance() == instance)
}
