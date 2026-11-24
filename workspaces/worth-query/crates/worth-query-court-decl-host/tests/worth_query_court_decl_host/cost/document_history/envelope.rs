//! One envelope step beside 20 or 80 members of another model, after 2, 12 or 72 earlier steps.
use crate::certification::document_history::owned_outputs::history_envelope::output_forest::step_work::{
    measure, StepWork,
};

/// The charged work of the recording, its undo and its redo. Charged work is
/// not CPU time.
fn operations(step: &StepWork) -> [(&'static str, u64); 3] {
    [
        ("recording", step.recording),
        ("undo", step.undo),
        ("redo", step.redo),
    ]
}

#[test]
fn an_envelope_step_and_its_navigation_do_not_scale_with_other_members_or_earlier_steps() {
    let few = measure(20, 2);
    let many = measure(80, 2);
    let deeper = measure(20, 12);
    let deepest = measure(20, 72);
    eprintln!(
        "envelope step work: few={few:?} many={many:?} deeper={deeper:?} deepest={deepest:?}"
    );
    assert!(
        operations(&few).iter().all(|(_, work)| *work > 0),
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
        deeper.journal, few.journal,
        "the journal must not inspect earlier steps"
    );
    assert_eq!(
        deepest.journal, few.journal,
        "the journal must not inspect earlier steps"
    );
    for ((name, many), (_, few)) in operations(&many).into_iter().zip(operations(&few)) {
        assert!(
            many <= few + few / 10,
            "{name} beside 4x the members of another model: {many} against {few}"
        );
    }
    // Each request pays branch and index lookups that grow with the logarithm
    // of the retained history. The largest is the relational index's basis
    // check: (bit length of its base count + 1) x 128 x (branch name length + 1)
    // units, and every step retains bases. Work in proportion to depth would
    // add six times as much over 60 more steps as over 10 more. A single
    // `.edit` step grows the same way, with a small linear term (public #321).
    for (((name, few), (_, deeper)), (_, deepest)) in operations(&few)
        .into_iter()
        .zip(operations(&deeper))
        .zip(operations(&deepest))
    {
        let ten_more = deeper.saturating_sub(few);
        let sixty_more = deepest.saturating_sub(deeper);
        assert!(
            sixty_more <= ten_more * 3,
            "{name} grows in proportion to earlier steps: {few}, {deeper}, {deepest}"
        );
    }
}
