use crate::repository_document;

const CONTRACT: &str = "plans/worth-ui/milestone-3.12-phase-5-contract.toml";
const LIFECYCLE: &str = "workspaces/worth-ui/docs/application-lifecycle.md";
const HOT_REBIND: &str = "workspaces/worth-ui/docs/hot-rebind.md";
const INSPECTION: &str = "workspaces/worth-ui/docs/inspection.md";
const VISUAL: &str = "workspaces/worth-ui/docs/visual-inspection.md";
const AUTHORED: &str = "workspaces/worth-ui/docs/authored-composition.md";
const ARCHITECTURE: &str = "workspaces/worth-ui/docs/architecture.md";
const SUBSYSTEMS: &str = "workspaces/worth-ui/docs/runtime-subsystems.md";
const REBIND_PASS: &str =
    "workspaces/worth-ui/crates/worth-ui/tests/ui/rebind/pass/rebind_phase_progression_uses_owner_issued_values.rs";
const VISUAL_PASS: &str =
    "workspaces/worth-ui/crates/worth-ui/tests/ui/visual_snapshot/pass/governed_visual_snapshot_lifecycle.rs";
const EXECUTION_INVENTORY: &str =
    "workspaces/worth-ui/crates/worth-ui/tests/suites/compile_contract_execution.csv";
const HOT_REBIND_FENCE: &str =
    "<!-- compile-pass-source:tests/ui/rebind/pass/rebind_phase_progression_uses_owner_issued_values.rs -->";
const VISUAL_FENCE: &str =
    "<!-- compile-pass-source:tests/ui/visual_snapshot/pass/governed_visual_snapshot_lifecycle.rs -->";
const INSPECTION_FRAGMENT: &str = "<!-- compile-pass-fragment:inspect_rebind_decision -->";

const DOCUMENTS: [&str; 7] = [
    LIFECYCLE,
    HOT_REBIND,
    INSPECTION,
    VISUAL,
    AUTHORED,
    ARCHITECTURE,
    SUBSYSTEMS,
];

#[test]
fn milestone_312_phase5_documentation_contract_names_every_continuing_surface() {
    let contract: toml::Value =
        toml::from_str(&repository_document(CONTRACT)).expect("Phase 5 contract is TOML");
    let documentation = &contract["documentation"];
    let paths = documentation["documents"]
        .as_array()
        .expect("documentation paths are declared")
        .iter()
        .map(|entry| entry.as_str().expect("documentation path is text"))
        .collect::<Vec<_>>();
    assert_eq!(paths, DOCUMENTS);
    assert_eq!(
        documentation["rebind_compile_source"].as_str(),
        Some(REBIND_PASS)
    );
    assert_eq!(
        documentation["visual_compile_source"].as_str(),
        Some(VISUAL_PASS)
    );
    assert!(documentation["audit_command"]
        .as_str()
        .is_some_and(|command| command.contains("milestone_312_phase5_documentation")));
    assert_eq!(documentation["status"].as_str(), Some("closed"));
    assert!(documentation["evidence"]
        .as_str()
        .is_some_and(|evidence| evidence.contains("exactly two Cargo sessions")));
}

#[test]
fn milestone_312_phase5_documentation_matches_the_shipped_facades() {
    let docs = CurrentDocumentation::read();
    docs.assert_required_topics();
    docs.assert_no_predecessor_claims();
}

#[test]
fn milestone_312_phase5_documentation_examples_are_exact_compiled_sources() {
    assert_exact_fenced_source(HOT_REBIND, HOT_REBIND_FENCE, REBIND_PASS);
    assert_exact_fenced_source(VISUAL, VISUAL_FENCE, VISUAL_PASS);
    assert_compiled_fragment(INSPECTION, INSPECTION_FRAGMENT, REBIND_PASS);

    let inventory = repository_document(EXECUTION_INVENTORY);
    for row in [
        "pass,tests/ui/rebind/pass/rebind_phase_progression_uses_owner_issued_values.rs,facade_compile.rs",
        "pass,tests/ui/visual_snapshot/pass/governed_visual_snapshot_lifecycle.rs,visual_snapshot_facade_compile.rs",
    ] {
        assert!(
            inventory.lines().any(|line| line == row),
            "compile execution inventory misses `{row}`"
        );
    }
}

#[test]
fn removing_the_ordinary_rebind_root_turns_documentation_proof_red() {
    let docs = CurrentDocumentation::read();
    let mutated = docs.hot_rebind.replace(
        "WorthUiNativeApplicationShell::begin_source_rebind(...)",
        "hidden source swap",
    );
    assert!(!has_all(
        &mutated,
        &[
            "WorthUiNativeApplicationShell::begin_source_rebind(...)",
            "WorthUiActiveApplicationSession::begin_observation_turn()",
        ]
    ));
}

#[test]
fn restoring_the_visual_comparison_deferral_turns_documentation_proof_red() {
    let docs = CurrentDocumentation::read();
    let mutated = format!(
        "{}\nThe comparison of predecessor and successor snapshots is deferred.",
        docs.visual
    );
    assert!(contains_predecessor_claim(&mutated));
}

struct CurrentDocumentation {
    lifecycle: String,
    hot_rebind: String,
    inspection: String,
    visual: String,
    authored: String,
    architecture: String,
    subsystems: String,
}

impl CurrentDocumentation {
    fn read() -> Self {
        Self {
            lifecycle: repository_document(LIFECYCLE),
            hot_rebind: repository_document(HOT_REBIND),
            inspection: repository_document(INSPECTION),
            visual: repository_document(VISUAL),
            authored: repository_document(AUTHORED),
            architecture: repository_document(ARCHITECTURE),
            subsystems: repository_document(SUBSYSTEMS),
        }
    }

    fn assert_required_topics(&self) {
        self.assert_lifecycle_and_rebind_topics();
        self.assert_inspection_and_visual_topics();
        self.assert_authoring_and_topology_topics();
    }

    fn assert_lifecycle_and_rebind_topics(&self) {
        assert_topics(
            LIFECYCLE,
            &self.lifecycle,
            &[
                "begin_source_rebind",
                "FirstFramePublished",
                "RebindPublished",
                "RebindDeniedPreserving",
                "VisualComparison",
                "same process and window",
            ],
        );
        assert_topics(
            HOT_REBIND,
            &self.hot_rebind,
            &[
                "WorthUiNativeApplicationShell::begin_source_rebind(...)",
                "WorthUiActiveApplicationSession::begin_observation_turn()",
                "UiChangeClassificationOutcome",
                "UiResolvedAffectedScope::resolve_identity_lifecycle()",
                "UiPreparedRebind::execute(...)",
                "O + F + A + C + R + G + M + B",
            ],
        );
    }

    fn assert_inspection_and_visual_topics(&self) {
        assert_topics(
            INSPECTION,
            &self.inspection,
            &[
                "UiRebindReceipt::decision_record()",
                "UiRebindDecisionLookup",
                "`Expired`",
                "replay/reconstruction remain deferred",
            ],
        );
        assert_topics(
            VISUAL,
            &self.visual,
            &[
                "issue_comparison_grant()",
                "compare_visual_snapshots(...)",
                "UiUnbudgetedVisualSnapshotComparisonRequest::between(...)",
                "Compared`, `Omitted`, `Expired`, `Incompatible`, or `Denied",
                "Comparison never recaptures",
            ],
        );
    }

    fn assert_authoring_and_topology_topics(&self) {
        assert_topics(
            AUTHORED,
            &self.authored,
            &[
                "WorthUiSettledSourceSnapshot::attempt_source_rebind(...)",
                "one DSL compile",
                "sealed candidate submission | typed compile report",
                "malformed",
                "begin_source_rebind",
            ],
        );
        assert_topics(
            ARCHITECTURE,
            &self.architecture,
            &[
                "## Observation To Rebind Flow",
                "UiRebindRuntimeState",
                "canonical host effects",
                "O + F + A + C + R + G + M + B",
            ],
        );
        assert_topics(
            SUBSYSTEMS,
            &self.subsystems,
            &[
                "rebind lifecycle",
                "`UiRebindRuntimeState`",
                "## Successor Homes",
                "The legacy whole-application replacement facade is not an alternate route.",
            ],
        );
    }

    fn assert_no_predecessor_claims(&self) {
        for (path, document) in [
            (LIFECYCLE, self.lifecycle.as_str()),
            (HOT_REBIND, self.hot_rebind.as_str()),
            (INSPECTION, self.inspection.as_str()),
            (VISUAL, self.visual.as_str()),
            (AUTHORED, self.authored.as_str()),
            (ARCHITECTURE, self.architecture.as_str()),
            (SUBSYSTEMS, self.subsystems.as_str()),
        ] {
            assert!(
                !contains_predecessor_claim(document),
                "`{path}` retains obsolete pre-3.12 guidance"
            );
        }
    }
}

fn assert_topics(path: &str, document: &str, topics: &[&str]) {
    let document = normalized_whitespace(document);
    for topic in topics {
        assert!(
            document.contains(&normalized_whitespace(topic)),
            "`{path}` misses required topic `{topic}`"
        );
    }
}

fn has_all(document: &str, topics: &[&str]) -> bool {
    let document = normalized_whitespace(document);
    topics
        .iter()
        .all(|topic| document.contains(&normalized_whitespace(topic)))
}

fn contains_predecessor_claim(document: &str) -> bool {
    [
        "The comparison of predecessor and successor snapshots is deferred.",
        "Do not look for a comparison API",
        "This is whole-application replacement, not Milestone 3.12",
        "version-2 sequence",
        "lower_to_candidate_submission",
        "## The Seven Owners",
        "WORTH_UI_PLATFORM_PULSE_PUBLISHED",
        "WORTH_UI_PLATFORM_PULSE_REPLACED",
    ]
    .iter()
    .any(|claim| document.contains(claim))
}

fn assert_exact_fenced_source(document_path: &str, marker: &str, source_path: &str) {
    let document = repository_document(document_path);
    assert_eq!(
        document.matches(marker).count(),
        1,
        "`{document_path}` must own one exact compile-source marker"
    );
    let documented = fenced_rust_after(&document, marker);
    let source = repository_document(source_path);
    assert_eq!(
        normalized_source(documented),
        normalized_source(&source),
        "`{document_path}` public example drifted from `{source_path}`"
    );
}

fn assert_compiled_fragment(document_path: &str, marker: &str, source_path: &str) {
    let document = repository_document(document_path);
    let fragment = normalized_source(fenced_rust_after(&document, marker));
    let source = normalized_source(&repository_document(source_path));
    assert!(
        source.contains(&fragment),
        "`{document_path}` fragment is not compiled by `{source_path}`"
    );
}

fn fenced_rust_after<'a>(document: &'a str, marker: &str) -> &'a str {
    let after_marker = document
        .split_once(marker)
        .expect("compile-source marker exists")
        .1;
    let after_open = after_marker
        .split_once("```rust")
        .expect("marker is followed by a Rust fence")
        .1;
    let after_open = after_open
        .strip_prefix("\r\n")
        .or_else(|| after_open.strip_prefix('\n'))
        .expect("Rust fence body starts on the next line");
    after_open
        .split_once("\r\n```")
        .or_else(|| after_open.split_once("\n```"))
        .expect("Rust fence closes")
        .0
}

fn normalized_source(text: &str) -> String {
    text.replace("\r\n", "\n").trim_end_matches('\n').to_owned()
}

fn normalized_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
