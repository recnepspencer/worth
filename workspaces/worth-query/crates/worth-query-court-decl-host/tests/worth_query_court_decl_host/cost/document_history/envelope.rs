//! One envelope step beside 20 or 80 members of another model, after 2 or 12 earlier steps.
use crate::certification::document_history::owned_outputs::history_envelope::output_forest::step_work::{
    measure, StepWork,
};

/// Charged work is not CPU time. Index paths and admission quotes may grow
/// slowly with the world, so the bound is coarse: under twice the work.
fn stays_under_double(more: StepWork, base: StepWork, what: &str) {
    for (name, more, base) in [
        ("recording", more.recording, base.recording),
        ("undo", more.undo, base.undo),
        ("redo", more.redo, base.redo),
    ] {
        assert!(more < base * 2, "{name} {what}: {more} against {base}");
    }
}

#[test]
fn an_envelope_step_and_its_navigation_do_not_scale_with_other_members_or_earlier_steps() {
    let few = measure(20, 2);
    let many = measure(80, 2);
    let deep = measure(20, 12);
    eprintln!("envelope step work: few={few:?} many={many:?} deep={deep:?}");
    assert!(
        few.recording > 0 && few.undo > 0 && few.redo > 0,
        "the measured requests must be accounted"
    );
    assert!(
        few.journal.iter().all(|probes| *probes > 0),
        "the journal's storage accesses must be observed"
    );
    assert_eq!(
        many.journal, few.journal,
        "the journal must not inspect another model's members"
    );
    assert_eq!(
        deep.journal, few.journal,
        "the journal must not inspect earlier steps"
    );
    stays_under_double(many, few, "beside 4x the members of another model");
    stays_under_double(deep, few, "after 6x the earlier steps");
}
