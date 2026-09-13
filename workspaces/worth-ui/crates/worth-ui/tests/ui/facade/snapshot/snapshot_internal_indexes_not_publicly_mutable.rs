use worth_ui::facade::app::WorthUi;

fn main() {
    let app = WorthUi::app()
        .with_change_profile(worth_ui_runtime::facade::rebind::UiChangeProfile::platform_pulse())
        .freeze()
        .expect("application preparation should succeed");
    let mut index = app.capabilities().index();

    index.commands = index.commands;
}

// facade construction denials share one compiler process.
mod covered_001 {
    include!("../command_projection/construction/projection_command_meaning_methods_are_not_available.rs");
}
mod covered_002 {
    include!("../topology/facade_reexports/facade_reexport_does_not_expose_internal_topology.rs");
}
mod covered_003 {
    include!("../construction/sealed_lifecycle/direct_builder_construction_fails.rs");
}
mod covered_005 {
    include!("../construction/sealed_lifecycle/direct_app_construction_fails.rs");
}
mod covered_006 {
    include!("../lifecycle/freeze/register_after_snapshot_freeze_fails.rs");
}
mod covered_007 {
    include!("../lifecycle/snapshot_authority/registered_set/registered_capability_set_not_publicly_mutable.rs");
}
mod covered_008 {
    include!(
        "../identity/family_interchange/same_text_different_id_families_are_not_interchangeable.rs"
    );
}
mod covered_009 {
    include!("../identity/construction/raw_text_cannot_replace_validated_id.rs");
}
mod covered_010 {
    include!("../support/construction/classified_posture_cannot_replace_admitted_witness.rs");
}
mod covered_011 {
    include!("../support/construction/raw_text_cannot_define_support_posture.rs");
}
mod covered_012 {
    include!("../support/construction/external_support_id_impl_is_forbidden.rs");
}
mod covered_013 {
    include!("../diagnostics/construction/raw_strings_cannot_replace_diagnostic_codes.rs");
}
mod covered_014 {
    include!("../command/construction/command_readiness_cannot_be_flattened_to_bool.rs");
}
mod covered_015 {
    include!("../../facade_visibility/topology/facade_does_not_reexport_registry_types.rs");
}
mod covered_016 {
    include!("../icon/construction/raw_asset_path_cannot_replace_icon_dependency.rs");
}
mod covered_017 {
    include!("../mosaic_sizing/construction/raw_number_cannot_replace_named_mosaic_sizing_measurement.rs");
}
mod covered_018 {
    include!("../mosaic_state/construction/raw_text_cannot_replace_mosaic_state_owner_scope_id.rs");
}
mod covered_019 {
    include!("../native_capability/construction/ambient_host_check_cannot_replace_native_capability_posture.rs");
}
mod covered_020 {
    include!("../plugin_slot/construction/plugin_global_mutation_hook_is_diagnostic_only.rs");
}
mod covered_021 {
    include!("../runtime_outcome_projection/construction/local_status_enum_cannot_replace_runtime_outcome_reference.rs");
}
mod covered_022 {
    include!("../settings/construction/raw_map_cannot_replace_setting_descriptor.rs");
}
mod covered_023 {
    include!("../task_presentation/construction/task_runtime_handle_cannot_replace_task_presentation_descriptor.rs");
}
mod covered_024 {
    include!("../theme_token/construction/raw_color_cannot_replace_theme_token_dependency.rs");
}
mod covered_025 {
    include!("../query_binding/construction/query_registration_requires_installed_view.rs");
}
mod covered_026 {
    include!("../query_binding/construction/direct_view_binding_registration_is_not_public.rs");
}
mod covered_027 {
    include!("../query_binding/construction/detached_view_binding_descriptor_constructor_is_not_public.rs");
}
