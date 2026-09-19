use crate::repository_document;

const README: &str = "workspaces/worth-ui/README.md";
const AI_README: &str = "workspaces/worth-ui/AI_README.md";
const QUERY_BINDING: &str = "workspaces/worth-ui/docs/query-binding.md";
const LIFECYCLE: &str = "workspaces/worth-ui/docs/application-lifecycle.md";
const ARCHITECTURE: &str = "workspaces/worth-ui/docs/architecture.md";
const SUBSYSTEMS: &str = "workspaces/worth-ui/docs/runtime-subsystems.md";
const INSPECTION: &str = "workspaces/worth-ui/docs/inspection.md";
const DSL_VISION: &str = "_docs/worth-ui/worth-ui-dsl-vision.md";
const DIAGNOSTICS: &str = "_docs/worth-ui/ai-diagnostics.md";
const ADVANCED: &str = "workspaces/worth-ui/docs/worth-ui-readme.md";
const MILESTONE: &str = "_docs/worth-ui/milestone-3.13.md";
const PROJECTION_PASS: &str =
    "workspaces/worth-ui/crates/worth-ui/tests/ui/facade/query_binding/pass/projection_contract_is_shape_specific.rs";
const EXECUTION_INVENTORY: &str =
    "workspaces/worth-ui/crates/worth-ui/tests/suites/compile_contract_execution.csv";
const PULSE_INSTALLATION: &str =
    "workspaces/worth-ui/apps/platform-pulse/src/query_source/installation.rs";
const PULSE_SOURCE: &str = "workspaces/worth-ui/apps/platform-pulse/app/main.wui";
const REGISTRATION_FRAGMENT: &str = "<!-- compile-pass-fragment:register_scalar_projection -->";
const PASS_ROW: &str =
    "pass,tests/ui/facade/query_binding/pass/projection_contract_is_shape_specific.rs,view_binding_facade_compile.rs";

#[test]
fn milestone_313_continuing_documentation_matches_the_shipped_projection_path() {
    assert_topics(
        README,
        &[
            "worth_ui::facade::query_binding",
            "does not import the binding crate",
        ],
    );
    assert_topics(
        AI_README,
        &[
            "host installation plan",
            "shape-specific affine fact receipt",
            "literal field lookup",
        ],
    );
    assert_query_guide(&repository_document(QUERY_BINDING));
    assert_topics(
        LIFECYCLE,
        &[
            "--query-source-root $queryRoot",
            "QueryProjectionIssued",
            "QueryProjectionPublished",
            r#"{"status":"ONLINE","revision":1}"#,
            r#"{"status":"UPDATED-LONG","revision":2}"#,
            "zero live Query",
        ],
    );
    assert_topics(
        ARCHITECTURE,
        &[
            "Projection fact flow is deliberately one-way",
            "The active session owns no Query result cache",
            "Semantic text is complete before the host boundary",
        ],
    );
    assert_topics(
        SUBSYSTEMS,
        &[
            "Query projection state is not another session owner",
            "session data cache",
            "renderer-side field lookup",
        ],
    );
    assert_topics(
        INSPECTION,
        &[
            "projection fact, mounted node, frame, and presentation",
            "Disclosure and retention posture",
            "cannot construct a binding",
        ],
    );
    assert_topics(
        DSL_VISION,
        &[
            "## Direct Projection Binding",
            "query_scalar platform.pulse.status",
            "try_with_query_collection_*",
            "General Query authoring",
        ],
    );
    assert_topics(
        DIAGNOSTICS,
        &[
            "shape-specific affine fact",
            "materialized lazily",
            "cannot construct a binding",
        ],
    );
    assert_topics(
        ADVANCED,
        &[
            "WorthUiStatusSourceOwner",
            "UiProjectionAvailability",
            "stale-generation",
            "additive successors",
        ],
    );
    assert_topics(
        MILESTONE,
        &[
            "Status: Complete. Phases 1 through 5 closed on 2026-07-29.",
            "All `QP-01` through `QP-10` rows are proved.",
            "explicit closure-stress lane",
        ],
    );
}

#[test]
fn milestone_313_registration_example_is_compiled_and_pulse_installation_is_real() {
    let guide = repository_document(QUERY_BINDING);
    let fragment = fenced_rust_after(&guide, REGISTRATION_FRAGMENT);
    let pass_source = repository_document(PROJECTION_PASS);
    assert!(
        normalized_source(&pass_source).contains(&normalized_source(fragment)),
        "Query registration fragment drifted from the compiled public source"
    );
    assert!(
        repository_document(EXECUTION_INVENTORY)
            .lines()
            .any(|line| line == PASS_ROW),
        "compiled projection source is absent from the existing execution inventory"
    );

    assert_topics(
        PULSE_INSTALLATION,
        &[
            "WorthUiStatusSourceOwner::install()",
            "WorthUiPresentationAsyncHostPlan::prepare()",
            "WorthQueryExecutionRuntimeInstaller::new()",
            ".complete(installation)",
        ],
    );
    assert_topics(
        PULSE_SOURCE,
        &[
            "content projection platform.pulse.status",
            "query_scalar platform.pulse.status",
            "field status",
            "lifecycle live",
        ],
    );
}

#[test]
fn reintroducing_the_false_workspace_extension_turns_documentation_proof_red() {
    let guide = repository_document(QUERY_BINDING);
    assert!(query_guide_is_current(&guide));
    let mutated = guide.replace(
        "There is no product `WorthUiQueryWorkspaceExt` import.",
        "Import `WorthUiQueryWorkspaceExt` and call `workspace.worth_ui()`.",
    );
    assert!(!query_guide_is_current(&mutated));
}

fn assert_query_guide(guide: &str) {
    assert!(
        query_guide_is_current(guide),
        "Query guide retains false or superseded product guidance"
    );
    for topic in [
        "UiScalarProjectionRegistration",
        "UiCollectionProjectionRegistration",
        "UiProjectionAvailability",
        "WorthUiStatusSourceOwner::install()",
        "WorthUiPresentationAsyncHostPlan::prepare()",
        "begin_projection_rebind(...)",
        "Query-free and unchanged turns perform zero projection/content work.",
        "Milestone 3.14",
    ] {
        assert!(guide.contains(topic), "Query guide misses `{topic}`");
    }
}

fn query_guide_is_current(guide: &str) -> bool {
    [
        "use worth_ui::facade::query_binding::WorthUiQueryWorkspaceExt",
        "workspace.worth_ui()",
        "settle_projection(",
        "register_query_view(",
    ]
    .iter()
    .all(|obsolete| !guide.contains(obsolete))
}

fn assert_topics(path: &str, topics: &[&str]) {
    let document = repository_document(path);
    let normalized = normalized_prose(&document);
    for topic in topics {
        assert!(
            normalized.contains(&normalized_prose(topic)),
            "`{path}` misses `{topic}`"
        );
    }
}

fn normalized_prose(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn fenced_rust_after<'a>(document: &'a str, marker: &str) -> &'a str {
    let after_marker = document
        .split_once(marker)
        .expect("compile fragment marker exists")
        .1;
    let after_open = after_marker
        .split_once("```rust")
        .expect("marker is followed by a Rust fence")
        .1;
    let after_open = after_open
        .strip_prefix("\r\n")
        .or_else(|| after_open.strip_prefix('\n'))
        .expect("Rust fence body begins on the next line");
    after_open
        .split_once("\r\n```")
        .or_else(|| after_open.split_once("\n```"))
        .expect("Rust fence closes")
        .0
}

fn normalized_source(text: &str) -> String {
    text.replace("\r\n", "\n").trim_end_matches('\n').to_owned()
}
