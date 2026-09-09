use crate::mounting::{UiMountedSemanticContentInput, UiPreparedMountedFrame};
use crate::runtime::appearance::{UiAppearanceInspectionDenial, UiAppearanceInspectionRecord};
use worth_ui_dsl::{UiAppearanceAxisClass, UiAppearanceStateAxis};
use worth_ui_host_contract::UiMountedInstanceIdentity;

pub(super) fn collection_candidate(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    transition: worth_ui_query_binding::UiProjectionInputFactTransition,
) -> UiPreparedMountedFrame {
    let mut content = UiMountedSemanticContentInput::empty();
    content.merge_projection_inputs(transition.revision().slot().index() + 1);
    content
        .insert_projection_input_transition(transition)
        .unwrap();
    prepare(session, content)
}

pub(super) fn empty_replacement(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    capacity: usize,
    merge: bool,
) -> UiPreparedMountedFrame {
    let mut content = UiMountedSemanticContentInput::empty();
    if merge {
        content.merge_projection_inputs(capacity);
    } else {
        content.replace_projection_inputs(capacity);
    }
    prepare(session, content)
}

fn prepare(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    content: UiMountedSemanticContentInput,
) -> UiPreparedMountedFrame {
    let completion = session.execute_framework_turn(|_| {}).unwrap();
    let mut execution = completion
        .into_execution()
        .unwrap_or_else(|_| panic!("candidate framework turn executes"));
    let theme = execution.presentation.theme_values_source();
    execution
        .prepare_mounted_frame_with_content_internal(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            content,
            theme,
        )
        .unwrap_or_else(|denial| panic!("candidate collection frame prepares: {denial:?}"))
}

pub(super) fn assert_candidate(
    frame: &mut UiPreparedMountedFrame,
    expected: &[(UiMountedInstanceIdentity, Option<UiAppearanceAxisClass>)],
    selection_revision: u64,
) {
    assert_eq!(
        frame
            .appearance_selection_cost_report()
            .selected_instance_count(),
        expected.len()
    );
    assert_eq!(
        frame
            .appearance_selection_cost_report()
            .materialized_context_count(),
        expected.len()
    );
    let attempt =
        worth_ui_host_contract::UiMountedPresentationAttemptIdentity::mint_unbound().unwrap();
    let (_, records) = frame.lower_appearance(attempt, None).into_parts();
    let aggregate_denied = expected.iter().any(|(_, class)| class.is_none());
    let mut observed = std::collections::BTreeMap::new();
    for record in records {
        let (item, class) = match record {
            UiAppearanceInspectionRecord::Projection { projection, .. } => {
                assert!(
                    !aggregate_denied,
                    "a denied member rejects the whole appearance output"
                );
                assert_eq!(
                    projection.state().basis().owner_revisions()[3],
                    selection_revision,
                    "surviving keys retain the sealed owner's actual revision"
                );
                (
                    projection.target().mounted_instance(),
                    projection.state().class(UiAppearanceStateAxis::Selection),
                )
            }
            UiAppearanceInspectionRecord::Denial {
                context,
                denial,
                receipt,
            } => {
                assert!(
                    aggregate_denied,
                    "a surviving collection must resolve successfully"
                );
                let expected_class = expected
                    .iter()
                    .find(|(item, _)| *item == context.target().mounted_instance())
                    .unwrap()
                    .1;
                assert_eq!(
                    denial,
                    if expected_class.is_none() {
                        UiAppearanceInspectionDenial::Basis
                    } else {
                        // The valid member is rejected by the atomic aggregate,
                        // not admitted as a successor beside a denied member.
                        UiAppearanceInspectionDenial::MountLowering
                    }
                );
                assert!(receipt.denied_before_effects());
                (context.target().mounted_instance(), None)
            }
        };
        assert!(observed.insert(item, class).is_none());
    }
    assert_eq!(
        observed,
        expected
            .iter()
            .map(|(item, class)| (*item, if aggregate_denied { None } else { *class }))
            .collect()
    );
}
