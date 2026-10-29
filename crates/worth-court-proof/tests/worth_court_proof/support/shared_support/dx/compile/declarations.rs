pub(crate) use {crate::shared_support::compile_fail::bundle::CompileFailBundle, crate::shared_support::compile_fail::bundle::CompileFailCase};
pub(crate) use {crate::shared_support::compile_pass::bundle::CompilePassBundle, crate::shared_support::compile_pass::bundle::CompilePassCase};

pub(crate) fn compile_fail_bundle() -> CompileFailBundle {
    crate::shared_support::compile_fail::bundle::CompileFailBundle::new(
        "pleasant_lane_capability_overclaim_compile_boundaries_hold",
        vec![
            crate::shared_support::compile_fail::bundle::CompileFailCase::new(
                "pleasant_lane_cannot_skip_progression",
                concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_proof/sources/worth_proof/ui/dx/compile_fail/pleasant_lane_cannot_skip_progression.rs"),
            ),
            crate::shared_support::compile_fail::bundle::CompileFailCase::new(
                "missing_scoped_defaults",
                concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_proof/sources/worth_proof/ui/dx/compile_fail/scoped_defaults_missing_required_progression.rs"),
            ),
            crate::shared_support::compile_fail::bundle::CompileFailCase::new(
                "consumed_scoped_defaults_cannot_be_reused",
                concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_proof/sources/worth_proof/ui/dx/compile_fail/scoped_defaults_cannot_be_reused_after_consumption.rs"),
            ),
        ],
    )
}

pub(crate) fn compile_pass_bundle() -> CompilePassBundle {
    crate::shared_support::compile_pass::bundle::CompilePassBundle::new(
        "pleasant_lane_representative_workflows_compile_cleanly",
        vec![
            crate::shared_support::compile_pass::bundle::CompilePassCase::new(
                "prelude-plus-raw-escape-hatch",
                concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_proof/sources/worth_proof/ui/dx/compile_pass/prelude_plus_raw_lane_compiles.rs"),
            ),
            crate::shared_support::compile_pass::bundle::CompilePassCase::new(
                "verb-first-progression",
                concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_proof/sources/worth_proof/ui/dx/compile_pass/verb_first_progression_compiles.rs"),
            ),
            crate::shared_support::compile_pass::bundle::CompilePassCase::new(
                "checked-and-boundary-progression",
                concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_proof/sources/worth_proof/ui/dx/compile_pass/checked_and_boundary_progression_compiles.rs"),
            ),
            crate::shared_support::compile_pass::bundle::CompilePassCase::new(
                "composition-and-family-authoring",
                concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_proof/sources/worth_proof/ui/dx/compile_pass/composition_and_family_authoring_compiles.rs"),
            ),
            crate::shared_support::compile_pass::bundle::CompilePassCase::new(
                "grouped-read-inspectors",
                concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_proof/sources/worth_proof/ui/dx/compile_pass/grouped_read_inspectors_compiles.rs"),
            ),
            crate::shared_support::compile_pass::bundle::CompilePassCase::new(
                "scoped-progression-defaults",
                concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_proof/sources/worth_proof/ui/dx/compile_pass/scoped_progression_defaults_compiles.rs"),
            ),
        ],
    )
}