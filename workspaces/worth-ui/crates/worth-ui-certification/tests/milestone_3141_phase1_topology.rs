use std::collections::BTreeSet;

use super::{repository_document, workspace_source_inventory};

#[path = "milestone_3141_phase1_topology/authority_residue.rs"]
mod authority_residue;
#[path = "milestone_3141_phase1_topology/external_ports.rs"]
mod external_ports;
#[path = "milestone_3141_phase1_topology/font_authority.rs"]
mod font_authority;
#[path = "milestone_3141_phase1_topology/phase_five_destination.rs"]
mod phase_five_destination;
#[path = "milestone_3141_phase1_topology/phase_five_physical_signal.rs"]
mod phase_five_physical_signal;
#[path = "milestone_3141_phase1_topology/phase_five_query_async.rs"]
mod phase_five_query_async;
#[path = "milestone_3141_phase1_topology/phase_five_raster_authority.rs"]
mod phase_five_raster_authority;
#[path = "milestone_3141_phase1_topology/phase_five_raster_order.rs"]
mod phase_five_raster_order;
#[path = "milestone_3141_phase1_topology/phase_three_application.rs"]
mod phase_three_application;
#[path = "milestone_3141_phase1_topology/preparation_call_graph.rs"]
mod preparation_call_graph;
#[path = "milestone_3141_phase1_topology/pulse_text.rs"]
mod pulse_text;

fn assert_current_protocol_rejects_mixed_revision() {
    use worth_ui_host_contract::{
        UiHostMeasurementSchemaVersion, UiHostObservationSchemaVersion, UiHostProtocolContract,
        UiHostProtocolDenial, UiHostProtocolIdentity, UiHostProtocolNegotiation,
        UiHostProtocolVersion, UiHostSolicitedEffectSchemaVersion, UiMountedFrameSchemaVersion,
        UiMountedPresentationSchemaVersion,
    };
    let current = UiHostProtocolContract::current();
    assert_eq!(current.protocol().revision(), 9);
    let mixed = UiHostProtocolContract::new(
        UiHostProtocolIdentity::worth_ui(),
        UiHostProtocolVersion::new(3),
        UiMountedFrameSchemaVersion::new(4),
        UiMountedPresentationSchemaVersion::new(4),
        UiHostObservationSchemaVersion::new(6),
        UiHostMeasurementSchemaVersion::new(4),
        UiHostSolicitedEffectSchemaVersion::new(1),
    );
    assert_eq!(
        mixed.negotiate(),
        UiHostProtocolNegotiation::Incompatible(UiHostProtocolDenial::ProtocolTooOld)
    );
}

#[test]
fn phase_one_consumer_inventory_rejects_legacy_protocol_branches() {
    assert_current_protocol_rejects_mixed_revision();
    let protocol = repository_document(
        "workspaces/worth-ui/crates/worth-ui-host-contract/src/mounted_frame/protocol.rs",
    );
    assert!(protocol.contains("const COMPATIBLE_FLOOR: u16 = 9;"));
    assert!(protocol.contains("const CURRENT: u16 = 9;"));
    assert!(protocol.contains("const CURRENT_FRAME_SCHEMA: u16 = 8;"));
    assert!(protocol.contains("const CURRENT_PRESENTATION_SCHEMA: u16 = 8;"));
    assert!(protocol.contains("const CURRENT_OBSERVATION_SCHEMA: u16 = 7;"));
    let inventory = workspace_source_inventory();
    let consumers = [
        (
            "crates/worth-ui-host-headless/src/headless_translation.rs",
            "UiMountedAppearancePresentationWork",
        ),
        (
            "crates/worth-ui-certification/tests/application_contracts/host_platform/world/production.rs",
            "WorthUiHeadlessRecorder",
        ),
    ];
    for (source, required) in consumers {
        let text = inventory.text(source);
        assert!(text.contains(required), "{source} omits {required}");
    }
    for root in ["crates/worth-ui-host-headless/src"] {
        for source in inventory.rust_files_under(root) {
            let path = source.relative_path().to_string_lossy();
            if path.ends_with("_tests.rs") || path.contains("/tests/") || path.contains("\\tests\\")
            {
                continue;
            }
            for forbidden in [
                "UiMountedFrameSchemaVersion::new(",
                "UiMountedPresentationSchemaVersion::new(",
                "mounted_frame().revision() == 3",
                "mounted_frame().revision() >= 3",
            ] {
                assert!(
                    !source.text().contains(forbidden),
                    "{} restores legacy protocol logic through {forbidden}",
                    source.relative_path().display()
                );
            }
        }
    }
}

#[test]
fn phase_one_product_preparation_is_effect_free_and_host_neutral() {
    let inventory = workspace_source_inventory();
    let application = inventory
        .source("crates/worth-ui-runtime/src/native_platform/application.rs")
        .expect("native application preparation owner");
    for required in [
        "pub enum UiNativeApplicationPreparationOutcome",
        "Prepared(Box<UiPreparedNativeApplication>)",
        "Denied(UiNativeApplicationPreparationDenial)",
        "WorthUiHostNeutralApp",
    ] {
        assert!(
            application.text().contains(required),
            "product preparation omits {required}"
        );
    }
    let application_effects = [
        "register_filesystem_source",
        "register_query_owner",
        "register_intent_owner",
        "register_inspection_owner",
        "register_readiness_owner",
        "UiNativePreparationActivation",
        "Condvar",
        "JoinHandle",
    ];
    for forbidden in application_effects {
        assert!(
            !application.text().contains(forbidden),
            "phase-one product preparation owns runtime effect {forbidden}"
        );
    }
    assert!(
        inventory
            .source("crates/worth-ui-runtime/src/native_platform/preparation_worker.rs")
            .is_none(),
        "retired generic preparation worker still exists"
    );
    let platform_preparation = inventory
        .source("crates/worth-ui-runtime/src/native_platform/platform/preparation.rs")
        .expect("native platform preparation transition");
    assert!(platform_preparation
        .text()
        .contains("impl WorthUiNativePlatform"));
    let profile = inventory
        .source("crates/worth-ui-runtime/src/native_platform/profile.rs")
        .expect("native platform profile validation owner");
    let native_profile = inventory
        .source("crates/worth-ui-host-native/src/native_profile/identity.rs")
        .expect("qualified native profile identity owner");
    let observed_effect_surfaces = preparation_call_graph::validate(
        platform_preparation.text(),
        profile.text(),
        native_profile.text(),
    )
    .expect("preparation call graph must remain closed over pure transitions");
    preparation_call_graph::assert_effectful_mutants_rejected();
    let prepared = worth_ui_native_platform::WorthUiNativePlatform::prepare(
        worth_ui_native_platform::UiNativePlatformProfile::single_window(
            worth_ui_native_platform::UiNativeWindowSpec::new("effect-free preparation", [160, 96]),
        ),
    )
    .expect("public preparation transition remains lawful and effect-free");
    assert_eq!(
        prepared.profile().window().initial_logical_size(),
        [160, 96]
    );
    let facade = inventory
        .source("crates/worth-ui-native-platform/src/lib.rs")
        .expect("native platform facade");
    for forbidden in [
        "UiNativeStoppedPreparationResource",
        "UiNativeStoppedReadinessOwner",
        "refusing_cleanup",
    ] {
        assert!(
            !facade.text().contains(forbidden),
            "facade exposes synthetic lifecycle lane {forbidden}"
        );
    }
    assert_eq!(observed_effect_surfaces, 0);
}

#[test]
fn independent_oracle_has_no_disputed_production_imports() {
    let oracle = workspace_source_inventory()
        .source("crates/worth-ui-certification/tests/application_contracts/host_platform/oracle.rs")
        .expect("independent host-platform oracle");
    for forbidden in [
        "worth_ui_runtime",
        "worth_ui_host_contract",
        "worth_ui_host_headless",
        "work_producer",
        "order_integrity",
    ] {
        assert!(
            !oracle.text().contains(forbidden),
            "oracle imports {forbidden}"
        );
    }
    let manifest = repository_document(
        "workspaces/worth-ui/crates/worth-ui-certification/tests/application_contracts/host_platform/control_points.toml",
    );
    assert!(manifest.contains("world_version = 1"));
    assert!(manifest.contains("maximum_surfaces = 2048"));
    assert_eq!(manifest.matches("[[surface]]").count(), 2);
}

#[test]
fn platform_and_presentation_issuers_have_exact_source_homes() {
    let inventory = workspace_source_inventory();
    let retired_native_issuer =
        ["UiNativePlatformBinding", "Issuer::for_prepared_platform"].concat();
    assert_exact_symbol_homes(inventory, &retired_native_issuer, &[]);
    let retired_presentation_issuer =
        ["UiMountedPresentationRuntimeAuthority", "::for_runtime"].concat();
    assert_exact_symbol_homes(
        inventory,
        &retired_presentation_issuer,
        &["crates/worth-ui-host-contract/tests/ui/presentation_work_issuance_requires_runtime_authority.rs"],
    );
}

pub(super) fn assert_exact_symbol_homes(
    inventory: &worth_ui_certification::topology::WorkspaceSourceInventory,
    symbol: &str,
    expected: &[&str],
) {
    let actual = inventory
        .rust_files_under("crates")
        .filter(|source| source.text().contains(symbol))
        .map(|source| source.relative_path().to_string_lossy().replace('\\', "/"))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        actual,
        expected.iter().map(|path| (*path).to_owned()).collect()
    );
}
