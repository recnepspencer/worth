//! An authored `wheel` clause lowers to the runtime's canonical wheel contract,
//! and a horizon the runtime cannot honour is refused at service admission,
//! before any policy is normalized.
use std::path::PathBuf;

use worth_ui_dsl::{WorthUiAuthoredSourceInput, WorthUiDslCompiler};

use crate::declaration::{
    UiNormalizedServicePolicyPlan, UiScrollAnchorBehavior, UiScrollWheelBehaviorDenial,
    UiServicePolicyDefaults, UiServicePolicyNormalizationDenial,
    UI_SCROLL_WHEEL_SETTLE_TICK_CEILING,
};
use crate::facade::WorthUi;

use super::{
    prepare_semantic_handoff, WorthUiSemanticHandoffPreparationStop,
    WorthUiServiceDeclarationAdmissionCause,
};

fn app() -> crate::facade::entry::WorthUiHostNeutralApp {
    WorthUi::app()
        .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
        .freeze()
        .expect("empty capability app freezes")
}

fn package(source: &str) -> worth_ui_dsl::WorthUiSealedSemanticPackage {
    WorthUiDslCompiler::compile_source(
        WorthUiAuthoredSourceInput::rooted_at(PathBuf::from("workspace"))
            .with_module("app/main.wui", source.to_owned()),
    )
    .expect("scroll wheel source seals")
}

const MOTION_OWNER: &str = "motion m { reduced system_respecting }";

#[test]
fn authored_smooth_wheel_lowers_to_the_settle_horizon_it_declared() {
    let source = format!(
        "scroll activity_list {{ nested anchor stable_key wheel smooth 120 }} {MOTION_OWNER}"
    );
    let material = prepare_semantic_handoff(package(&source), app().capabilities())
        .expect("an admissible smooth wheel admits");
    let (_, _, evidence) = material.into_parts();
    let plan = UiNormalizedServicePolicyPlan::normalize(
        UiServicePolicyDefaults::default(),
        evidence.authored_service_policy_defaults(),
        evidence.runtime_service_support(),
    )
    .expect("the motion owner the smooth wheel needs is declared");
    let scroll = plan.scroll().expect("the scroll owner is installed");
    assert_eq!(scroll.wheel_behavior().settle_ticks(), Some(120));
    assert!(scroll.bubbles_remainder());
    assert_eq!(
        scroll.anchor_behavior(),
        UiScrollAnchorBehavior::RebaseStableAnchor
    );
}

#[test]
fn unstated_and_declared_immediate_wheels_lower_to_the_same_contract() {
    for wheel in ["", "wheel immediate"] {
        let source = format!("scroll activity_list {{ anchor clamp {wheel} }}");
        let material = prepare_semantic_handoff(package(&source), app().capabilities())
            .expect("an immediate wheel admits");
        let (_, _, evidence) = material.into_parts();
        let plan = UiNormalizedServicePolicyPlan::normalize(
            UiServicePolicyDefaults::default(),
            evidence.authored_service_policy_defaults(),
            evidence.runtime_service_support(),
        )
        .expect("an immediate wheel needs no motion owner");
        let scroll = plan.scroll().expect("the scroll owner is installed");
        assert_eq!(scroll.wheel_behavior().settle_ticks(), None);
    }
}

#[test]
fn smooth_wheel_beyond_the_runtime_ceiling_is_refused_at_admission() {
    let horizon = UI_SCROLL_WHEEL_SETTLE_TICK_CEILING + 1;
    let source =
        format!("scroll activity_list {{ anchor clamp wheel smooth {horizon} }} {MOTION_OWNER}");
    let denial = match prepare_semantic_handoff(package(&source), app().capabilities()) {
        Ok(_) => panic!("a horizon the runtime cannot honour is refused"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.stop(),
        WorthUiSemanticHandoffPreparationStop::ServiceDeclaration {
            declaration_index: 0,
            cause: WorthUiServiceDeclarationAdmissionCause::ScrollWheelHorizon(
                UiScrollWheelBehaviorDenial::SettleHorizonExceedsCeiling
            ),
        }
    );
}

#[test]
fn smooth_wheel_without_a_motion_owner_is_refused_at_normalization() {
    let material = prepare_semantic_handoff(
        package("scroll activity_list { anchor clamp wheel smooth 120 }"),
        app().capabilities(),
    )
    .expect("admission judges the horizon, not the owner closure");
    let (_, _, evidence) = material.into_parts();
    let denial = UiNormalizedServicePolicyPlan::normalize(
        UiServicePolicyDefaults::default(),
        evidence.authored_service_policy_defaults(),
        evidence.runtime_service_support(),
    )
    .expect_err("a smooth wheel needs the motion owner");
    assert_eq!(
        denial,
        UiServicePolicyNormalizationDenial::SmoothWheelWithoutMotionOwner { settle_ticks: 120 }
    );
}
