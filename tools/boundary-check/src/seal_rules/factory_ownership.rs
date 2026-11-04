//! Named factory reach follows its semantic owner and exact caller class.
pub(super) const FACTORIES: &[&str] = &[
    "install_home_checkpoint_maintenance",
    "begin_fresh_home_initialization",
    "prepare_fresh_home_schema_installation",
    "from_home_original_bytes",
    "copy_home_input_for_request",
    "from_absent_home_read",
    "from_original_home_read",
    "home_fresh_schema_initialization",
    "registered_authoritative_source",
    "with_authoritative_source",
    "from_owner_publication",
    "from_owner_partition_publication",
    "certification_query",
    "readmit_world_home",
    "admit_fresh_world_home",
    "admit_fresh_home_source",
    "admit_fresh_home_from_source",
    "validate_fresh_source_runtime",
    "validate_fresh_initial_schema_installation",
    "world_fresh_source_admission",
    "home_fresh_query_finalization_parts",
    "home_recovered_query_finalization_parts",
    "fresh_query_finalization_parts",
    "recovered_query_finalization_parts",
    "readmit_prepared_home",
    "begin_after_world_seal",
    "retire_home_maintenance",
    "with_query_retirement_owners",
    "prepare_initial_empty_home_stop",
    "world_composition_runtime",
    "world_recovery_authority",
    "discover_recovered_associated_effects",
    "capture_complete_checkpoint",
    "complete_checkpoint_service",
    "from_native_checkpoint_source",
    "from_home_checkpoint_selection",
    "from_protected_home_source",
    "from_protected_world_journal",
    "query_runtime_handle",
    "prepare_home_program",
    "initialize_home_native_schema",
    "prepare_home_native_receiver_schema",
    "finish_fresh_home_native_readmission",
    "finish_existing_home_native_readmission",
    "attach_existing_home_registered_source",
    "attach_existing_home_world",
    "attach_home_registered_source",
    "attach_home_world",
    "prepare_native_schema_for_home",
    "query_finalization_parts",
    "initialize_new_home",
    "query_composition_parts",
    "attach_fresh_home",
    "attach_prepared_home",
    "home_schema_installation_runtime",
    "home_query_finalization_parts",
    "query_publication_home_binding",
    "prepare_query_publication_without_signal",
    "bind_query_native_proposal",
    "complete_query_native_preparation",
    "readmit_no_pending_owners",
    "into_no_pending_home",
];

pub(super) fn ownership(name: &str) -> (&'static str, &'static str) {
    if name == "install_home_checkpoint_maintenance" {
        ("application-home-checkpoint-maintenance", "workspaces/worth-store/crates/worth-store-application/")
    } else if name == "begin_fresh_home_initialization" {
        ("native-fresh-initialization", "crates/worth-relational/")
    } else if name == "prepare_fresh_home_schema_installation" {
        (
            "native-durable-initial-installation",
            "crates/worth-relational/",
        )
    } else if name == "from_home_original_bytes" {
        ("native-home-original-source", "crates/worth-relational/")
    } else if name == "copy_home_input_for_request" {
        (
            "native-home-original-source-copy",
            "crates/worth-relational/",
        )
    } else if name == "from_absent_home_read" {
        (
            "native-source-origin-preparation",
            "workspaces/worth-store/crates/worth-store-relational/",
        )
    } else if name == "from_original_home_read" {
        (
            "native-source-origin-preparation",
            "workspaces/worth-store/crates/worth-store-relational/",
        )
    } else if name == "home_fresh_schema_initialization" {
        (
            "native-home-schema-installation",
            "workspaces/worth-store/crates/worth-store-relational/",
        )
    } else if name == "admit_fresh_home_source" {
        ("native-fresh-home-source", "crates/worth-relational/")
    } else if name == "admit_fresh_home_from_source" {
        (
            "native-fresh-home-readmission",
            "workspaces/worth-store/crates/worth-store-relational/",
        )
    } else if name == "world_fresh_source_admission" {
        (
            "native-world-fresh-source",
            "workspaces/worth-store/crates/worth-store-relational/",
        )
    } else if matches!(
        name,
        "home_fresh_query_finalization_parts" | "home_recovered_query_finalization_parts"
    ) {
        (
            "native-home-source-finalization",
            "workspaces/worth-store/crates/worth-store-relational/",
        )
    } else if name == "fresh_query_finalization_parts" {
        (
            "application-home-fresh-finalization",
            "workspaces/worth-store/crates/worth-store-application/",
        )
    } else if name == "recovered_query_finalization_parts" {
        (
            "application-home-recovered-finalization",
            "workspaces/worth-store/crates/worth-store-application/",
        )
    } else if name == "validate_fresh_source_runtime" {
        ("native-fresh-source-affinity", "crates/worth-relational/")
    } else if name == "validate_fresh_initial_schema_installation" {
        (
            "native-fresh-initial-installation",
            "crates/worth-relational/",
        )
    } else if name == "admit_fresh_world_home" {
        ("world-fresh-home", "crates/worth-runtime-world/")
    } else if name == "world_recovery_authority" {
        (
            "native-world-readmission",
            "workspaces/worth-store/crates/worth-store-relational/",
        )
    } else if name == "prepare_initial_empty_home_stop" {
        (
            "application-home-bridge-stop",
            "crates/worth-runtime-bridge/",
        )
    } else if name == "with_query_retirement_owners" {
        (
            "query-home-retirement",
            "workspaces/worth-store/crates/worth-store-application/",
        )
    } else if matches!(name, "begin_after_world_seal" | "retire_home_maintenance") {
        (
            "query-home-retirement",
            "workspaces/worth-query/crates/worth-query-execution/",
        )
    } else if name == "readmit_prepared_home" {
        (
            "native-selected-home-readmission",
            "workspaces/worth-store/crates/worth-store-relational/",
        )
    } else if matches!(name, "readmit_no_pending_owners" | "into_no_pending_home") {
        (
            "application-home-owner-admission",
            "workspaces/worth-store/crates/worth-store-application/",
        )
    } else if name == "discover_recovered_associated_effects" {
        ("native-recovered-inventory", "crates/worth-relational/")
    } else if name == "from_home_checkpoint_selection" {
        (
            "world-home-checkpoint-selection",
            "crates/worth-runtime-world/",
        )
    } else if ["from_protected_home_source", "from_protected_world_journal"].contains(&name) {
        (
            "world-home-checkpoint-source",
            "crates/worth-runtime-world/",
        )
    } else if name == "from_native_checkpoint_source" {
        (
            "native-checkpoint-read-protection",
            "crates/worth-relational/",
        )
    } else if ["capture_complete_checkpoint", "complete_checkpoint_service"].contains(&name) {
        (
            "native-home-complete-checkpoint",
            "crates/worth-relational/",
        )
    } else if name == "world_composition_runtime" {
        (
            "native-home-composition",
            "workspaces/worth-store/crates/worth-store-relational/",
        )
    } else if name == "query_runtime_handle" {
        ("query-registered-native", "crates/worth-runtime-bridge/")
    } else if matches!(
        name,
        "prepare_home_program"
            | "initialize_home_native_schema"
            | "prepare_home_native_receiver_schema"
            | "finish_fresh_home_native_readmission"
            | "finish_existing_home_native_readmission"
            | "attach_existing_home_registered_source"
            | "attach_existing_home_world"
            | "attach_home_registered_source"
            | "attach_home_world"
    ) {
        (
            "query-home-preparation",
            "workspaces/worth-query/crates/worth-query-execution/",
        )
    } else if matches!(
        name,
        "prepare_native_schema_for_home"
            | "query_finalization_parts"
            | "initialize_new_home"
            | "query_composition_parts"
    ) {
        (
            "native-home-query-preparation",
            "workspaces/worth-store/crates/worth-store-application/",
        )
    } else if matches!(name, "attach_fresh_home" | "attach_prepared_home") {
        (
            "native-home-attachment",
            "workspaces/worth-store/crates/worth-store-relational/",
        )
    } else if name == "home_schema_installation_runtime" {
        (
            "native-home-schema-installation",
            "workspaces/worth-store/crates/worth-store-relational/",
        )
    } else if name == "home_query_finalization_parts" {
        (
            "native-home-query-finalization",
            "workspaces/worth-store/crates/worth-store-relational/",
        )
    } else if matches!(
        name,
        "query_publication_home_binding"
            | "prepare_query_publication_without_signal"
            | "bind_query_native_proposal"
            | "complete_query_native_preparation"
    ) {
        (
            "query-world-commit-preparation",
            "crates/worth-runtime-world/",
        )
    } else if name == "readmit_world_home" {
        ("world-home", "crates/worth-runtime-world/")
    } else if name == "certification_query" {
        (
            "installation",
            "workspaces/worth-query/crates/worth-query-installation/",
        )
    } else {
        ("bridge", "crates/worth-runtime-bridge/")
    }
}
