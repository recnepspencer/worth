use super::{repository_document, workspace_source_inventory};

const ROOT: &str = "crates/worth-ui-host-native/src/native/physical_work_signal";
const DECLARATIONS: &str =
    "crates/worth-ui-host-native/src/native/physical_work_signal/declarations";

const FILES: [&str; 18] = [
    "mod.rs",
    "construction.rs",
    "identity.rs",
    "declarations/mod.rs",
    "declarations/aspects.rs",
    "declarations/resources.rs",
    "routing/mod.rs",
    "routing/request.rs",
    "routing/external_observation.rs",
    "routing/progression.rs",
    "completion_reconciliation.rs",
    "wake_delivery.rs",
    "shutdown.rs",
    "observation.rs",
    "counters.rs",
    "temporal_progression.rs",
    "worker.rs",
    "worker_graph.rs",
];

const FORBIDDEN_PARALLEL_RUNTIME_FRAGMENTS: [&str; 7] = [
    "TimerWheel",
    "RetryQueue",
    "TimeoutRegistry",
    "PendingWorkRegistry",
    "LocalPhysicalWorkScheduler",
    "CallbackSettlement",
    "DuplicatePhysicalLifecycle",
];

const RAW_SIGNAL_CONSTRUCTORS: [&str; 4] = [
    "Aspect::new(",
    "Aspect::try_new(",
    "AspectMask::from_bits",
    "AspectMask::from_aspect",
];

#[test]
fn phase_five_physical_signal_owner_is_single_native_and_query_free() {
    let inventory = workspace_source_inventory();
    let paths = FILES.map(|name| format!("{ROOT}/{name}"));
    let present = paths
        .iter()
        .filter(|path| inventory.source(path).is_some())
        .count();
    assert!(
        present == 0 || present == paths.len(),
        "the physical Signal responsibility must appear complete or remain absent"
    );

    let native_manifest =
        repository_document("workspaces/worth-ui/crates/worth-ui-host-native/Cargo.toml");
    if present == 0 {
        let specification = repository_document("plans/worth-ui/milestone-3.14.1-phase-5.md");
        let plan =
            repository_document("plans/worth-ui/milestone-3.14.1-phase-5-implementation-plan.md");
        assert!(specification.contains("exactly one bounded host-native physical Signal runtime"));
        assert!(plan.contains("worth-signal.workspace = true"));
        return;
    }

    assert!(native_manifest.contains("worth-signal"));
    assert!(!native_manifest.contains("worth-query"));
    for source in inventory.rust_files_under("crates/worth-ui-host-native/src") {
        let path = normalized(source.relative_path());
        if source.text().contains("worth_signal") {
            assert!(
                path.starts_with(ROOT),
                "physical Signal access escaped its single native owner: {path}"
            );
        }
    }
    for source in inventory.rust_files_under("crates/worth-ui-runtime/src") {
        assert!(
            !source.text().contains("worth_signal"),
            "runtime constructed a second physical Signal graph: {}",
            source.relative_path().display()
        );
    }
}

#[test]
fn phase_five_physical_signal_owns_exact_basis_currentness_and_winit_wakes() {
    let inventory = workspace_source_inventory();
    let identity = inventory
        .source(format!("{ROOT}/identity.rs"))
        .expect("complete physical Signal topology must own exact work identities")
        .text();
    let request = inventory
        .source(format!("{ROOT}/routing/request.rs"))
        .expect("complete physical Signal topology must own routed requests")
        .text();
    let observation = inventory
        .source(format!("{ROOT}/routing/external_observation.rs"))
        .expect("complete physical Signal topology must own external observations")
        .text();
    assert!(identity.contains("UiNativePhysicalPresentationBasis"));
    assert!(identity.contains("UiNativePhysicalAtlasRequestIdentity"));
    assert!(identity.contains("basis_digest"));
    assert!(identity.contains("atlas_basis_digest"));
    for exact_atlas_part in [
        "host_session_identity",
        "demands",
        "pins",
        "canonical_raster_key_bytes",
        "UiNativeTextAtlasQualifiedCapacity",
    ] {
        assert!(
            identity.contains(exact_atlas_part),
            "atlas work omitted exact basis part {exact_atlas_part}"
        );
    }
    for field in [
        "host_session_identity",
        "attempt",
        "surface",
        "binding",
        "baseline",
    ] {
        assert!(
            identity.contains(field),
            "presentation work omitted {field}"
        );
    }
    for field in ["runtime", "work", "handle", "status"] {
        assert!(request.contains(field) && observation.contains(field));
    }

    let event_loop = inventory
        .source("crates/worth-ui-host-native/src/native/event_loop/physical_progression.rs")
        .expect("native event loop must transport physical Signal wakes")
        .text();
    assert!(event_loop.contains("request_physical_signal_redraw"));
    assert!(event_loop.contains("request_redraw"));

    let readiness = inventory
        .source("crates/worth-ui-host-native/src/native/readiness.rs")
        .expect("native readiness must transport the level physical wake")
        .text();
    for operation in ["register_level", "signal_level", "take_level"] {
        assert!(readiness.contains(operation));
    }
    let event_loop_run = inventory
        .source("crates/worth-ui-host-native/src/native/event_loop/run_preflight.rs")
        .expect("event-loop construction must register the physical level wake")
        .text();
    assert!(event_loop_run.contains("register_level"));
    let physical_progression = inventory
        .source("crates/worth-ui-host-native/src/native/event_loop/physical_progression.rs")
        .expect("event-loop progression must signal the physical level wake")
        .text();
    assert!(physical_progression.contains("signal_level_ready"));
    assert!(physical_progression.contains("progress_one_physical_signal_ready"));
    assert!(physical_progression.contains("take_level"));

    let atlas_port = inventory
        .source("crates/worth-ui-host-native/src/native/mechanics_adapter/text_atlas_upload.rs")
        .expect("native atlas upload port must be present")
        .text();
    assert!(atlas_port.contains("UiNativePhysicalSignalExternalBasis"));
    assert!(
        !atlas_port.contains("UiNativePhysicalSignalRequestToken"),
        "the external atlas port must echo an owner-issued basis, not receive Signal authority"
    );
    let presentation = inventory
        .source("crates/worth-ui-host-native/src/native/presentation/transaction_state.rs")
        .expect("pending native presentation owner must be present")
        .text();
    assert!(presentation.contains("poll_observation"));
    assert!(presentation.contains("external_basis()"));
    assert!(!presentation.contains("poll_status"));
}

#[test]
fn phase_five_physical_signal_has_no_public_atlas_control_or_close_time_scheduler() {
    let inventory = workspace_source_inventory();
    let public_facade = inventory
        .source("crates/worth-ui-host-native/src/lib.rs")
        .expect("native public facade must exist")
        .text();
    let operational_adapter = inventory
        .source("crates/worth-ui-host-contract/src/operational_adapter.rs")
        .expect("host operational adapter must exist")
        .text();
    assert!(!public_facade.contains("UiNativeTextAtlas,"));
    assert!(!public_facade.contains("UiAtlasEntryIdentity"));
    assert!(!public_facade.contains("UiNativePlatformEffectAuthority"));
    assert!(!public_facade.contains("WorthUiNativeMechanicsAdapter"));
    assert!(!operational_adapter.contains("text_atlas_transaction"));

    let prepared_host = inventory
        .source("crates/worth-ui-host-native/src/prepared_host.rs")
        .expect("prepared native host must retain the combined mechanics facade")
        .text();
    assert!(!prepared_host.contains("pub fn into_runtime_binding"));
    for forbidden in [
        "pub fn perform_mounted_text_raster_preparation",
        "pub fn perform_mounted_text_raster_completion",
        "pub fn perform_mounted_text_raster_cancellation",
        "pub fn perform_mounted_text_pin_release",
    ] {
        assert!(
            !prepared_host.contains(forbidden),
            "ordinary native mechanics must not expose {forbidden}"
        );
    }

    let presentation_adapter = inventory
        .source("crates/worth-ui-host-native/src/native/mechanics_adapter/presentation.rs")
        .expect("native presentation adapter must exist")
        .text();
    assert!(
        !presentation_adapter.contains("pending_presentations.is_empty()"),
        "the retained resource registry must not become a global presentation scheduler"
    );

    let host_state = inventory
        .source("crates/worth-ui-host-native/src/native/host_state.rs")
        .expect("native host state must own physical shutdown")
        .text();
    assert!(!host_state.contains("progress_pending_presentations"));
    assert_eq!(
        host_state
            .matches("UiNativePhysicalSignalOwner::new()")
            .count(),
        1
    );
    let close = function_prefix(
        host_state,
        "pub(crate) fn close",
        "pub(crate) fn progress_one_physical_signal_ready",
    );
    assert!(!close.contains("progress_one_physical_signal_ready"));
    assert!(!close.contains("try_settle"));
}

#[test]
fn phase_five_physical_signal_seals_raw_aspects_and_parallel_runtime_fragments() {
    let inventory = workspace_source_inventory();
    for source in inventory.rust_files_under("crates/worth-ui-host-native/src") {
        let path = normalized(source.relative_path());
        if !path.starts_with(DECLARATIONS) && !path.starts_with(&format!("{ROOT}/tests")) {
            for constructor in RAW_SIGNAL_CONSTRUCTORS {
                assert!(
                    !source.text().contains(constructor),
                    "raw Signal aspect construction escaped {DECLARATIONS}: {path}"
                );
            }
        }
        if !path.starts_with(ROOT) {
            for fragment in FORBIDDEN_PARALLEL_RUNTIME_FRAGMENTS {
                assert!(
                    !source.text().contains(fragment),
                    "parallel physical lifecycle fragment {fragment} escaped {ROOT}: {path}"
                );
            }
        }
    }
}

fn normalized(path: &std::path::Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn function_prefix<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start = source.find(start).expect("expected function start");
    let tail = &source[start..];
    let end = tail.find(end).expect("expected following function");
    &tail[..end]
}
