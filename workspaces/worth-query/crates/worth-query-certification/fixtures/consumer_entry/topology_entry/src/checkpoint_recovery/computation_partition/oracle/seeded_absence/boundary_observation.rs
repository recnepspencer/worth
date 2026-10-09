//! Observation boundary release and independent prime comparison.
use super::*;
pub(super) fn release_observation<
    const REUSE: bool,
    const WORK: usize,
    const RUNS: usize,
    const MODE: u8,
>(
    model: &mut Model,
    history: &mut differential::reference::Reference,
    request: &Request<'_, '_, '_>,
    application: &Application<REUSE, WORK, RUNS, MODE>,
    command: &mut u64,
) {
    for change in model.observation_release().changes {
        history.edit(&change);
        if let Change::Entry(change) = change {
            *command += 1;
            edit(request, application, change, *command);
        }
    }
}

/// A prime that reused its previous output still proves its retained result
/// and published state against a model-derived fresh computation.
pub(super) fn compare_prime(
    kept: &[OracleRun],
    reference: &[OracleRun],
    kept_publications: usize,
    fresh_publications: usize,
) {
    assert_eq!(
        kept.last().unwrap().outcome,
        reference.last().unwrap().outcome,
        "prime outcome"
    );
    compare_publications(kept, reference, kept_publications, fresh_publications);
}

/// Separate demand histories have independent model-derived cardinalities.
/// When the model gives equal counts, every publication is paired; otherwise
/// both complete histories and their final retained fields are still judged.
fn compare_publications(
    kept: &[OracleRun],
    reference: &[OracleRun],
    kept_publications: usize,
    fresh_publications: usize,
) {
    let kept_states = &kept.last().unwrap().published;
    let fresh_states = &reference.last().unwrap().published;
    assert_eq!(
        kept_states.len(),
        kept_publications,
        "kept model publications"
    );
    assert_eq!(
        fresh_states.len(),
        fresh_publications,
        "fresh model publications"
    );
    if kept_publications == fresh_publications {
        assert_published_state(kept, reference);
    } else {
        assert!(
            kept_states
                .last()
                .unwrap()
                .same_fields::<RegionKey, Entry, f64>(
                    fresh_states.last().unwrap(),
                    |a, b| a.0 == b.0,
                    Entry::same_binding_as,
                ),
            "final published retained fields differ"
        );
    }
}
