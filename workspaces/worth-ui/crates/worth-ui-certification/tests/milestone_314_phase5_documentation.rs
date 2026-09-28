use crate::repository_document;

const GUIDE: &str = "workspaces/worth-ui/docs/interaction-and-intents.md";
const LIFECYCLE: &str = "workspaces/worth-ui/docs/application-lifecycle.md";
const COMPILE_SOURCE: &str =
    "workspaces/worth-ui/crates/worth-ui/tests/ui/facade/intent/pass/typed_intent_relationships.rs";
const COMPILE_INVENTORY: &str =
    "workspaces/worth-ui/crates/worth-ui/tests/suites/compile_contract_execution.csv";
const PULSE_SOURCE: &str = "workspaces/worth-ui/apps/platform-pulse/app/main.wui";

#[test]
fn milestone_314_phase5_continuing_docs_name_the_shipped_authority_path() {
    let guide = repository_document(GUIDE);
    assert_contains_all(
        "intent guide",
        &guide,
        &[
            "## Stable Entry Points",
            "## Core Mental Model",
            "### Native units and IME",
            "### Gestures and drafts",
            "### Definition, declaration, and route",
            "### Operability, confirmation, and concurrency",
            "### Version coexistence",
            "## How It Executes",
            "## Small Example",
            "## Real Example",
            "## Inspection And Debugging",
            "## Cost Model",
            "## Anti-Patterns",
            "## Current Limits",
            "UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT",
            "UiHostImeRangeConversionReceipt",
            "TargetRouteSingleFlight",
            "UiIntentRecoveryHandle",
            "separate product or Query admission",
        ],
    );

    for path in [
        "workspaces/worth-ui/README.md",
        "workspaces/worth-ui/AI_README.md",
    ] {
        let document = repository_document(path);
        assert_contains_all(
            path,
            &document,
            &[
                "docs/interaction-and-intents.md",
                "semantic interaction",
                "UI admission",
                "provider",
                "Query",
            ],
        );
    }

    assert_contains_all(
        "architecture",
        &repository_document("workspaces/worth-ui/docs/architecture.md"),
        &[
            "## Interaction To Consequence Flow",
            "Intent admission owns",
            "Intent execution owns",
            "owner-admitted consequences",
            "Milestone 3.15",
        ],
    );
    assert_contains_all(
        "runtime subsystem map",
        &repository_document("workspaces/worth-ui/docs/runtime-subsystems.md"),
        &[
            "| interaction |",
            "| intent admission |",
            "| intent execution |",
            "separately admitted",
            "Milestone 3.15 providers",
        ],
    );
    assert_contains_all(
        "inspection",
        &repository_document("workspaces/worth-ui/docs/inspection.md"),
        &[
            "lookup_intent_causal_trace",
            "Expired",
            "UI_INTENT_INTERACTION_EVIDENCE_ENTRY_CAPACITY` (64)",
            "cannot construct an interaction, operability",
        ],
    );
    assert_contains_all(
        "DSL vision",
        &repository_document("plans/worth-ui/worth-ui-dsl-vision.md"),
        &[
            "## Interaction And Intent Are Separate Lanes",
            "Those interactions carry no product-effect authority",
            "does not author callbacks, executor code, Query mutation",
        ],
    );
    assert_contains_all(
        "AI diagnostics",
        &repository_document("plans/worth-ui/ai-diagnostics.md"),
        &[
            "### Implemented intent causal index",
            "64-entry semantic-only replacement ring",
            "expired records back into admission or execution authority",
        ],
    );
    assert_contains_all(
        "roadmap",
        &repository_document("plans/worth-ui/worth_ui_roadmap.md"),
        &[
            "### Milestone 3.14: Intent, Operability, and Interaction Substrate",
            "UI admission, provider execution, Query/domain admission",
            "Milestone 3.15 owns their execution",
        ],
    );
}

#[test]
fn milestone_314_phase5_lifecycle_is_executable_and_source_exact() {
    let lifecycle = repository_document(LIFECYCLE);
    assert_contains_all(
        "Platform Pulse lifecycle",
        &lifecycle,
        &[
            "--intent-source-root $intentRoot",
            "current version-9 lifecycle protocol",
            "Set-PulseIntent 2 ready released",
            "IntentInputAdmitted",
            "IntentExecutorStarted",
            "StaleConfirmation",
            "QueryAction",
            "IntentCausalTrace",
            "ShutdownCompleted",
            "zero live gesture, draft, admission, confirmation, execution",
        ],
    );
    assert!(!lifecycle.contains("version-3 lifecycle protocol"));
    assert!(!lifecycle.contains("terminal `Shutdown` observation"));
    assert_eq!(
        normalize(fenced_after(&lifecycle, "checked-in source exactly:")),
        normalize(&repository_document(PULSE_SOURCE)),
        "the documented recovery source drifted from the executable Pulse source",
    );
}

#[test]
fn milestone_314_phase5_public_example_remains_compiler_owned() {
    let guide = repository_document(GUIDE);
    let source = repository_document(COMPILE_SOURCE);
    let inventory = repository_document(COMPILE_INVENTORY);
    assert!(guide.contains("typed_intent_relationships.rs"));
    assert_contains_all(
        "compiled public example",
        &source,
        &[
            "impl UiIntent for Intent",
            "UiIntentDeclaration::<Intent>::activate",
            "register_intent_definition(UiIntentDefinition::<Intent>::application_effect())",
            "register_intent_provider(Provider)",
            "dispatch_admitted_intent",
            "retry_intent_recovery",
        ],
    );
    assert!(inventory.contains(
        "pass,tests/ui/facade/intent/pass/typed_intent_relationships.rs,facade_compile.rs"
    ));
}

#[test]
fn removing_any_required_documentary_claim_turns_the_audit_red() {
    let guide = repository_document(GUIDE);
    for required in [
        "### Native units and IME",
        "### Version coexistence",
        "separate product or Query admission",
        "UiIntentRecoveryHandle",
    ] {
        let hostile = guide.replacen(required, "removed", 1);
        assert!(require_all(&hostile, &[required]).is_err());
    }
}

fn assert_contains_all(label: &str, document: &str, required: &[&str]) {
    require_all(document, required)
        .unwrap_or_else(|missing| panic!("{label} misses required continuing truth `{missing}`"));
}

fn require_all(document: &str, required: &[&str]) -> Result<(), String> {
    let document = normalize_whitespace(document);
    required
        .iter()
        .find(|required| !document.contains(&normalize_whitespace(required)))
        .map_or(Ok(()), |missing| Err((*missing).to_owned()))
}

fn fenced_after<'a>(document: &'a str, marker: &str) -> &'a str {
    document
        .split_once(marker)
        .and_then(|(_, after)| after.split_once("```text"))
        .and_then(|(_, fenced)| fenced.split_once("```"))
        .map(|(body, _)| body)
        .expect("document owns one text fence after the source marker")
}

fn normalize(text: &str) -> String {
    text.replace("\r\n", "\n").trim().to_owned()
}

fn normalize_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
