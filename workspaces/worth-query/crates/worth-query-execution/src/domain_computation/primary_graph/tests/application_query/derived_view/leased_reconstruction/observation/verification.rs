//! Judgments of canonical application, public values and retained state.
use super::*;
pub(super) struct ApplicationEvidence {
    pub(super) entries: Vec<worth_relational::facade::identity::EntityId>,
    pub(super) completions: Vec<worth_relational::facade::identity::EntityId>,
    pub(super) later: Option<worth_relational::facade::identity::EntityId>,
    pub(super) projected: Vec<(worth_relational::facade::identity::EntityId, String)>,
    pub(super) published_values: Option<Vec<String>>,
    pub(super) prior_values: Vec<String>,
}
pub(super) fn verify(
    observed: Observation,
    evidence: ApplicationEvidence,
    case: &Case,
    count: usize,
    duplicate: bool,
    failures: &[usize],
) -> Observation {
    let ApplicationEvidence {
        entries,
        completions,
        later,
        projected,
        published_values,
        prior_values,
    } = evidence;
    if failures.is_empty() && case.interrupt.is_none() {
        assert_eq!(
            entries.len(),
            if duplicate { 0 } else { count },
            "each unique root must enter exactly once"
        );
    }
    assert_eq!(
        entries
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        entries.len()
    );
    if let Some(later) = later {
        assert_eq!(
            completions[0], later,
            "the rendezvous forces completion against canonical order"
        );
    }
    let mut canonical = projected.clone();
    canonical.sort_by_key(|(root, _)| *root);
    assert_eq!(
        projected, canonical,
        "projection must follow EntityId order"
    );

    if let Some(values) = published_values {
        assert_eq!(values.len(), count);
        assert_eq!(
            values, observed.values,
            "public retention must contain the projected values"
        );
    }
    assert_eq!(
        observed.retained_values, prior_values,
        "failed reconstruction cannot replace same-basis retention"
    );
    if observed.denial.is_some() && !case.warm {
        assert!(observed.retention.0);
        assert_eq!(
            observed.retention.1, 0,
            "a failed cold reconstruction publishes no entries"
        );
    }
    observed
}
