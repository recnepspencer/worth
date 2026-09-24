use super::*;
use crate::native::presentation::appearance::cursor::accept_cursor;
use worth_ui_host_contract::UiPointerAffordanceFamily;

#[test]
fn superseded_cursor_survives_unrelated_committed_successor_without_overwriting_departure() {
    for successor_cursor in [None, Some(UiPointerAffordanceFamily::Default)] {
        let world = DrawListWorld::new();
        let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
        let previous = world.rect(
            frame,
            world.first,
            0.0,
            UiMountedRgba8::new(10, 20, 30, 255),
        );
        let initial = world.initial(frame, [previous]);
        let mut retained = UiNativeRetainedDrawList::initial(&initial, &[]).unwrap();
        let replacement_frame = UiMountedFrameIdentity::mint_unbound().unwrap();
        let replacement = world.rect(
            replacement_frame,
            world.first,
            0.0,
            UiMountedRgba8::new(40, 50, 60, 255),
        );
        let (_, undo) = retained
            .stage_delta(
                &replacement_delta(&world, frame, previous, replacement_frame, replacement),
                &[],
            )
            .unwrap();
        let mut state = crate::native::UiNativeHostState::new();
        let predecessor = physical_basis(&world, &initial);
        let successor = predecessor.test_successor();
        let binding = predecessor.binding().diagnostic_value();
        state.retained_draw_lists.insert(binding, retained);
        let mut pending = pending_delta(
            &mut state,
            predecessor,
            undo,
            UiNativePhysicalSignalStatus::Completed,
        );
        assert!(pending.bind_completion_identity(1, Some(UiPointerAffordanceFamily::Activation)));
        state.pending_presentations.push(pending);
        assert_eq!(
            state.accepted_cursor, None,
            "unsettled work cannot change the OS cursor"
        );

        // The successor completes before the predecessor's external poll.
        let owners = reserve_presentation_owners(
            &mut state.resources,
            &mut state.physical_signal,
            successor,
        )
        .unwrap_or_else(|_| panic!("successor admission"));
        assert!(settle_port_result(
            &mut state.resources,
            &mut state.physical_signal,
            owners,
            Ok(crate::native::presentation::UiNativePresentationPortObservation::test())
        )
        .is_ok());
        state.presentation_epochs.insert(
            binding,
            worth_ui_host_contract::UiHostPresentationEpoch::issued_by_host(
                successor.attempt().diagnostic_value(),
            ),
        );
        accept_cursor(&mut state, successor.attempt(), successor_cursor);
        let due = state.physical_signal.next_due_tick().unwrap();
        state.physical_signal.advance_clock_to(due).unwrap();
        assert!(state.progress_one_physical_signal_ready());
        let expected = successor_cursor.unwrap_or(UiPointerAffordanceFamily::Activation);
        assert_eq!(state.accepted_cursor.unwrap().1, expected);
        assert_eq!(
            state.retained_draw_lists[&binding].frame(),
            replacement_frame
        );
        assert!(state.resources.current().is_zero());
        assert_eq!(state.physical_signal.observation().active_requests, 0);
    }
}
