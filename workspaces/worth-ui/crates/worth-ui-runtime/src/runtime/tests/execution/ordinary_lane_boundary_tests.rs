use super::ordinary_lane_test_support::{
    ordinary_lane_denial_for_missing_family, ordinary_lane_fixture,
    ordinary_lane_fixture_with_unrelated_diagnostics,
};
use crate::capability::{ThemeTokenValue, UiThemeColor};
use crate::runtime::execution::ordinary_lane::WorthUiOrdinaryLaneFrameExecutor;
use crate::runtime::planning::execution_plan_input::WorthUiPlanOrdinaryMeaning;
use crate::runtime::{
    WorthUiComponentHandle, WorthUiExecutionLane, WorthUiHandleSlotGeneration,
    WorthUiOrdinaryFrameTarget, WorthUiOrdinaryLaneFrameDenialReason,
    WorthUiOrdinaryLanePlanDenialReason, WorthUiPlanNodeInputFamily,
};

#[test]
fn executable_rows_retain_exact_admitted_command_and_native_token_meaning() {
    let (_, plan, _) = ordinary_fixture();
    let command = plan
        .first_row_for_family(WorthUiPlanNodeInputFamily::Command)
        .1
        .expect("the ordinary executable plan should contain a command row");
    let token = plan
        .first_row_for_family(WorthUiPlanNodeInputFamily::TokenStyle)
        .1
        .expect("the ordinary executable plan should contain a token row");

    match command
        .ordinary_meaning()
        .expect("the executable command row retains admitted meaning")
    {
        WorthUiPlanOrdinaryMeaning::Command(command) => {
            assert_eq!(command.reference().descriptor().label(), "Save");
            assert_eq!(
                command.reference().command().id().as_str(),
                "workspace.command.save"
            );
        }
        meaning => panic!("command row retained the wrong meaning: {meaning:?}"),
    }

    let expected_color =
        ThemeTokenValue::color(UiThemeColor::parse("#101820").expect("valid test color"));
    match token
        .ordinary_meaning()
        .expect("the executable token row retains admitted native meaning")
    {
        WorthUiPlanOrdinaryMeaning::Token(token) => {
            assert_eq!(token.entry().descriptor().value(), Some(&expected_color));
            assert_eq!(
                token
                    .semantics()
                    .resolved_target_entry()
                    .descriptor()
                    .value(),
                Some(&expected_color)
            );
        }
        meaning => panic!("token row retained the wrong meaning: {meaning:?}"),
    }
}

#[test]
fn equivalent_ordinary_lane_plans_execute_with_equivalent_traversal_counters() {
    let (_left_runtime, left_plan, left_allocation) = ordinary_fixture();
    let (_right_runtime, right_plan, right_allocation) = ordinary_fixture();
    let left_handle = left_allocation
        .component_handles()
        .next()
        .expect("fixture has component handle");
    let right_handle = right_allocation
        .component_handles()
        .next()
        .expect("fixture has component handle");

    let left_receipt = WorthUiOrdinaryLaneFrameExecutor::execute(
        &left_plan,
        WorthUiOrdinaryFrameTarget::component(left_handle),
    )
    .expect("runtime frame execution succeeds");
    let right_receipt = WorthUiOrdinaryLaneFrameExecutor::execute(
        &right_plan,
        WorthUiOrdinaryFrameTarget::component(right_handle),
    )
    .expect("runtime frame execution succeeds");

    assert_eq!(
        left_receipt.certification().ordinary_plan_digest(),
        right_receipt.certification().ordinary_plan_digest()
    );
    assert_eq!(left_receipt.counters(), right_receipt.counters());
    assert_eq!(left_receipt.counters().ordinary_frame_row_touch_count(), 1);
    assert_eq!(left_receipt.counters().full_plan_scan_count(), 0);
    let evidence = left_receipt
        .resolution_evidence()
        .expect("indexed execution carries compact resolution evidence");
    assert_eq!(
        evidence.outcome(),
        crate::runtime::WorthUiHandleResolutionOutcome::Resolved
    );
    assert_eq!(
        evidence.expected_family(),
        WorthUiPlanNodeInputFamily::ComponentInvocation
    );
    assert_eq!(evidence.direct_index_lookup_count(), 1);
    assert_eq!(evidence.registry_lookup_count(), 0);
    assert_eq!(evidence.string_resolution_count(), 0);

    let denial = WorthUiOrdinaryLaneFrameExecutor::execute(
        &right_plan,
        WorthUiOrdinaryFrameTarget::component(left_handle),
    )
    .expect_err("semantic equivalence cannot transfer exact handle authority");
    assert_eq!(
        denial.reason(),
        WorthUiOrdinaryLaneFrameDenialReason::TargetArenaMismatch
    );
    assert_eq!(denial.counters().ordinary_frame_row_touch_count(), 0);
}

#[test]
fn ordinary_frame_path_does_not_parse_or_resolve_source() {
    let (_runtime, ordinary_plan, _allocation) = ordinary_fixture();
    let command_handle = crate::runtime::WorthUiCommandHandle::from_runtime_handle(
        ordinary_plan
            .first_runtime_handle_for_family(WorthUiPlanNodeInputFamily::Command)
            .expect("fixture has command handle"),
    );
    let token_handle = crate::runtime::WorthUiTokenHandle::from_runtime_handle(
        ordinary_plan
            .first_runtime_handle_for_family(WorthUiPlanNodeInputFamily::TokenStyle)
            .expect("fixture has token handle"),
    );

    let command_receipt = WorthUiOrdinaryLaneFrameExecutor::execute(
        &ordinary_plan,
        WorthUiOrdinaryFrameTarget::command(command_handle),
    )
    .expect("runtime frame execution succeeds");
    let token_receipt = WorthUiOrdinaryLaneFrameExecutor::execute(
        &ordinary_plan,
        WorthUiOrdinaryFrameTarget::token_support(token_handle),
    )
    .expect("runtime frame execution succeeds");

    assert_path_is_source_free(command_receipt.counters());
    assert_path_is_source_free(token_receipt.counters());
    assert_eq!(command_receipt.counters().command_surface_touch_count(), 1);
    assert_eq!(token_receipt.counters().token_support_touch_count(), 1);
}

#[test]
fn ordinary_plan_requires_admitted_support_for_each_included_surface() {
    assert_mismatched_admission_denies(
        WorthUiPlanNodeInputFamily::Command,
        WorthUiExecutionLane::CommandSurface,
        WorthUiOrdinaryLanePlanDenialReason::LaneAdmissionMissingCommandSurfaceSupport,
    );
    assert_mismatched_admission_denies(
        WorthUiPlanNodeInputFamily::TokenStyle,
        WorthUiExecutionLane::StyleToken,
        WorthUiOrdinaryLanePlanDenialReason::LaneAdmissionMissingStyleTokenSupport,
    );
}

fn assert_mismatched_admission_denies(
    removed_family: WorthUiPlanNodeInputFamily,
    removed_lane: WorthUiExecutionLane,
    expected_reason: WorthUiOrdinaryLanePlanDenialReason,
) {
    let denial = ordinary_lane_denial_for_missing_family(removed_family, removed_lane);

    assert_eq!(denial.reason(), expected_reason);
    assert_eq!(denial.counters().denial_count(), 1);
}

#[test]
fn ordinary_frame_rejects_stale_typed_handle_generation() {
    let (_runtime, ordinary_plan, allocation) = ordinary_fixture();
    let fresh_handle = allocation
        .component_handles()
        .next()
        .expect("fixture has component handle");
    let stale_generation =
        WorthUiHandleSlotGeneration::new(fresh_handle.slot_generation().as_u64() + 1);
    let stale_handle = WorthUiComponentHandle::new(
        fresh_handle.plan_index(),
        stale_generation,
        fresh_handle.arena_identity(),
    );

    let denial = WorthUiOrdinaryLaneFrameExecutor::execute(
        &ordinary_plan,
        WorthUiOrdinaryFrameTarget::component(stale_handle),
    )
    .expect_err("runtime frame execution denies");

    assert_eq!(
        denial.reason(),
        WorthUiOrdinaryLaneFrameDenialReason::TargetSlotGenerationMismatch
    );
    assert_eq!(denial.plan_index(), Some(fresh_handle.plan_index()));
    assert_eq!(denial.counters().certification_failure_count(), 1);
    let evidence = denial
        .resolution_evidence()
        .expect("stale denial carries compact resolution evidence");
    assert_eq!(
        evidence.outcome(),
        crate::runtime::WorthUiHandleResolutionOutcome::StaleSlotGeneration
    );
    assert_eq!(evidence.target(), stale_handle.locator());
    assert_eq!(evidence.direct_index_lookup_count(), 1);
}

#[test]
fn ordinary_frame_rejects_foreign_session_handle_before_touch() {
    let (_runtime, ordinary_plan, allocation) = ordinary_fixture();
    let (_, _, foreign_allocation) = ordinary_fixture();
    let fresh = allocation
        .component_handles()
        .next()
        .expect("fixture has a component handle");
    let foreign_arena = foreign_allocation.receipt().arena_identity();
    assert_ne!(fresh.arena_identity(), foreign_arena);
    let foreign =
        WorthUiComponentHandle::new(fresh.plan_index(), fresh.slot_generation(), foreign_arena);

    let denial = WorthUiOrdinaryLaneFrameExecutor::execute(
        &ordinary_plan,
        WorthUiOrdinaryFrameTarget::component(foreign),
    )
    .expect_err("a foreign session locator must deny");

    assert_eq!(
        denial.reason(),
        WorthUiOrdinaryLaneFrameDenialReason::TargetArenaMismatch
    );
    assert_eq!(denial.counters().ordinary_frame_row_touch_count(), 0);
    let evidence = denial
        .resolution_evidence()
        .expect("foreign-session denial carries compact resolution evidence");
    assert_eq!(
        evidence.outcome(),
        crate::runtime::WorthUiHandleResolutionOutcome::ForeignSessionArena
    );
    assert_eq!(evidence.direct_index_lookup_count(), 0);
    assert_eq!(evidence.registry_lookup_count(), 0);
    assert_eq!(evidence.string_resolution_count(), 0);
}

#[test]
fn ordinary_frame_rejects_wrong_typed_family_before_touch() {
    let (_runtime, ordinary_plan, allocation) = ordinary_fixture();
    let command = allocation
        .command_handles()
        .next()
        .expect("fixture has a command handle");
    let forged_component = WorthUiComponentHandle::new(
        command.plan_index(),
        command.slot_generation(),
        command.arena_identity(),
    );

    let denial = WorthUiOrdinaryLaneFrameExecutor::execute(
        &ordinary_plan,
        WorthUiOrdinaryFrameTarget::component(forged_component),
    )
    .expect_err("a command slot cannot execute as a component");

    assert_eq!(
        denial.reason(),
        WorthUiOrdinaryLaneFrameDenialReason::TargetFamilyMismatch
    );
    assert_eq!(denial.counters().ordinary_frame_row_touch_count(), 0);
    let evidence = denial
        .resolution_evidence()
        .expect("wrong-family denial carries compact resolution evidence");
    assert_eq!(
        evidence.outcome(),
        crate::runtime::WorthUiHandleResolutionOutcome::WrongFamily
    );
    assert_eq!(
        evidence.resolved_family(),
        Some(crate::runtime::WorthUiPlanNodeInputFamily::Command)
    );
    assert_eq!(evidence.direct_index_lookup_count(), 1);
}

#[test]
fn unrelated_family_scale_does_not_widen_handle_resolution() {
    let (_narrow_runtime, narrow_plan, narrow_allocation) =
        ordinary_lane_fixture_with_unrelated_diagnostics(0);
    let (_wide_runtime, wide_plan, wide_allocation) =
        ordinary_lane_fixture_with_unrelated_diagnostics(256);
    let narrow_handle = narrow_allocation.component_handles().next().unwrap();
    let wide_handle = wide_allocation.component_handles().next().unwrap();

    let narrow = WorthUiOrdinaryLaneFrameExecutor::execute(
        &narrow_plan,
        WorthUiOrdinaryFrameTarget::component(narrow_handle),
    )
    .expect("narrow plan resolves component")
    .resolution_evidence()
    .expect("narrow resolution emits evidence");
    let wide = WorthUiOrdinaryLaneFrameExecutor::execute(
        &wide_plan,
        WorthUiOrdinaryFrameTarget::component(wide_handle),
    )
    .expect("wide plan resolves component")
    .resolution_evidence()
    .expect("wide resolution emits evidence");

    assert!(
        wide_plan.counters().skipped_nonordinary_plan_row_count()
            > narrow_plan.counters().skipped_nonordinary_plan_row_count()
    );
    assert_eq!(narrow.direct_index_lookup_count(), 1);
    assert_eq!(wide.direct_index_lookup_count(), 1);
    assert_eq!(wide.registry_lookup_count(), 0);
    assert_eq!(wide.string_resolution_count(), 0);
}

fn assert_path_is_source_free(counters: crate::runtime::WorthUiOrdinaryLaneCounters) {
    assert_eq!(counters.source_parse_count(), 0);
    assert_eq!(counters.registry_lookup_count(), 0);
    assert_eq!(counters.artifact_tree_scan_count(), 0);
    assert_eq!(counters.component_string_resolution_count(), 0);
    assert_eq!(counters.command_string_resolution_count(), 0);
    assert_eq!(counters.full_plan_scan_count(), 0);
}

fn ordinary_fixture() -> (
    crate::runtime::WorthUiRuntimeFrameworkLoop,
    crate::runtime::WorthUiOrdinaryLanePlan,
    crate::runtime::WorthUiRuntimeHandleAllocation,
) {
    ordinary_lane_fixture()
}
