use super::*;
use worth_ui_test_support::WorthUiMountedIdentityCertificationExt;

#[test]
fn ordinary_publication_preserves_predecessor_and_exact_reuse_skips_the_adapter() {
    let host = ScriptedPresentationHost::default();
    let (mut session, _) = mounted_session(host.clone(), "mounted-publication-ordinary", 1);
    let request = UiMountedFrameRequest::all_bound_surfaces();
    host.push_presented();
    let first = published(
        session
            .execute_mounted_frame(
                request.clone(),
                UiPresentationDeadline::at_tick(10),
                0,
                |_| {},
            )
            .unwrap_or_else(|_| panic!("empty source turn publishes a mounted frame")),
    );
    assert_eq!(
        session.inspect_mounted_identity().current_frame(),
        Some(first.frame())
    );
    assert_eq!(first.predecessor(), None);
    assert_eq!(first.cost_report().named().retained(), 1);

    let calls_before_reuse = host.presentation_calls();
    let outcome = session
        .execute_mounted_frame(
            request.clone(),
            UiPresentationDeadline::at_tick(11),
            1,
            |_| {},
        )
        .unwrap_or_else(|_| panic!("identical framework turn carries exact reuse authority"));
    assert_constant_unchanged_cost(outcome.cost_report().unwrap());
    let unchanged = match outcome {
        UiMountedFrameOutcome::Unchanged(receipt) => receipt,
        _ => panic!("exact reuse is classified as unchanged"),
    };
    assert_eq!(unchanged, first);
    assert_eq!(host.presentation_calls(), calls_before_reuse);

    host.push_rejected();
    let rejected = session
        .execute_mounted_frame(
            UiMountedFrameRequest::all_bound_surfaces(),
            UiPresentationDeadline::at_tick(20),
            2,
            |_| {},
        )
        .unwrap_or_else(|_| panic!("a fresh request reaches host presentation"));
    let rejected_frame = match &rejected {
        UiMountedFrameOutcome::RejectedBeforeEffects(frame) => {
            frame.frame().canonical_core().frame()
        }
        _ => panic!("scripted rejection remains typed before effects"),
    };
    assert_ne!(rejected_frame, first.frame());
    assert_eq!(
        session.current_mounted_publication(),
        Some(&first),
        "effect-free rejection preserves predecessor publication"
    );

    drop(rejected);
    host.push_presented();
    let recovered = published(
        session
            .execute_mounted_frame(
                UiMountedFrameRequest::all_bound_surfaces(),
                UiPresentationDeadline::at_tick(30),
                3,
                |_| {},
            )
            .unwrap_or_else(|_| panic!("publication recovers after effect-free rejection")),
    );
    assert_eq!(recovered.predecessor(), Some(first.frame()));
    assert_eq!(session.current_mounted_publication(), Some(&recovered));
}

fn assert_constant_unchanged_cost(cost: worth_ui_runtime::facade::mounted::UiMountCostReport) {
    assert_eq!(
        cost.work_class(),
        worth_ui_runtime::facade::mounted::UiMountWorkClass::UnchangedReuse
    );
    assert_eq!(cost.initial_mounted_instances(), 0);
    assert_eq!(cost.changed_mounted_instances(), 0);
    assert_eq!(cost.index_entries_touched(), 0);
    assert_eq!(cost.replaced_batch_rows(), 0);
    assert_eq!(cost.replaced_batch_bytes(), 0);
    assert_eq!(cost.surface_instance_pairs(), 0);
    assert_eq!(cost.changed_binding_generations(), 0);
    assert_eq!(cost.adapter().presented_surfaces(), 0);
    assert_eq!(cost.named().considered(), 0);
    assert_eq!(cost.named().minted(), 0);
    assert_eq!(cost.named().reused(), 1);
}
