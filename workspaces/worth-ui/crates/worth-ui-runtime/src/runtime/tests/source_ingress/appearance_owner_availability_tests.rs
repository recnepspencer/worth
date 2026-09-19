use super::active_application_session_test_support::admit_candidate_catalog;
use super::appearance_component_session_test_support::{
    appearance_candidate_submission, source_backed_appearance_role_capable_session,
};
use super::appearance_owner_availability_test_support::{
    focus_background_role, ownerless_focus_consumer_app,
};

#[test]
fn launch_rejects_appearance_demand_without_its_runtime_owner() {
    let denial = match ownerless_focus_consumer_app().launch() {
        Ok(session) => {
            let _ = session.shutdown();
            panic!("Focus appearance demand must not launch without Focus ownership")
        }
        Err(denial) => denial,
    };
    assert!(matches!(
        denial,
        crate::runtime::WorthUiRuntimeLaunchDenial::AppearanceOwnerUnavailable(
            worth_ui_dsl::UiAppearanceStateAxis::Focus
        )
    ));
}

#[test]
fn cutover_rejects_new_ownerless_demand_and_preserves_the_predecessor() {
    let role = focus_background_role();
    let mut session = source_backed_appearance_role_capable_session(&role);
    let predecessor_generation = session.generation_identity().clone();
    let predecessor_runtime = session.inspect_runtime();
    let mut prepared = session
        .prepare_replacement(appearance_candidate_submission(
            &session,
            "ownerless-focus-cutover",
            Some(&role),
        ))
        .unwrap();
    let catalog = admit_candidate_catalog(&mut prepared);
    let lowered = session.lower_prepared_replacement(*prepared).unwrap();
    let pending = session.stage_prepared_replacement(lowered).unwrap();
    let boundary = session
        .execute_framework_turn(|_| {})
        .unwrap()
        .into_completion()
        .into_execution()
        .unwrap()
        .into_activation_boundary();

    let denial = match session.activate_prepared_replacement(pending, catalog, boundary, None) {
        Ok(_) => panic!("ownerless Focus demand must not cut over"),
        Err(denial) => denial,
    };

    assert!(matches!(
        denial,
        crate::facade::WorthUiApplicationCutoverDenial::AppearanceOwnerUnavailable(
            worth_ui_dsl::UiAppearanceStateAxis::Focus
        )
    ));
    assert_eq!(session.generation_identity(), &predecessor_generation);
    assert_eq!(session.inspect_runtime(), predecessor_runtime);
    let _ = session.shutdown();
}
