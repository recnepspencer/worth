use crate::shared_support::compile_fail::{CompileFailBundle, CompileFailCase};
use crate::shared_support::compile_pass::{CompilePassBundle, CompilePassCase};

pub fn compile_fail_bundle() -> CompileFailBundle {
    CompileFailBundle::new(
        "pleasant_lane_capability_overclaim_compile_boundaries_hold",
        vec![
            CompileFailCase::new(
                "pleasant_lane_cannot_skip_progression",
                "tests/ui/dx/compile_fail/pleasant_lane_cannot_skip_progression.rs",
            ),
            CompileFailCase::new(
                "missing_scoped_defaults",
                "tests/ui/dx/compile_fail/scoped_defaults_missing_required_progression.rs",
            ),
            CompileFailCase::new(
                "consumed_scoped_defaults_cannot_be_reused",
                "tests/ui/dx/compile_fail/scoped_defaults_cannot_be_reused_after_consumption.rs",
            ),
        ],
    )
}

pub fn compile_pass_bundle() -> CompilePassBundle {
    CompilePassBundle::new(
        "pleasant_lane_representative_workflows_compile_cleanly",
        vec![
            CompilePassCase::new(
                "prelude-plus-raw-escape-hatch",
                "tests/ui/dx/compile_pass/prelude_plus_raw_lane_compiles.rs",
            ),
            CompilePassCase::new(
                "verb-first-progression",
                "tests/ui/dx/compile_pass/verb_first_progression_compiles.rs",
            ),
            CompilePassCase::new(
                "checked-and-boundary-progression",
                "tests/ui/dx/compile_pass/checked_and_boundary_progression_compiles.rs",
            ),
            CompilePassCase::new(
                "composition-and-family-authoring",
                "tests/ui/dx/compile_pass/composition_and_family_authoring_compiles.rs",
            ),
            CompilePassCase::new(
                "grouped-read-inspectors",
                "tests/ui/dx/compile_pass/grouped_read_inspectors_compiles.rs",
            ),
            CompilePassCase::new(
                "scoped-progression-defaults",
                "tests/ui/dx/compile_pass/scoped_progression_defaults_compiles.rs",
            ),
        ],
    )
}
