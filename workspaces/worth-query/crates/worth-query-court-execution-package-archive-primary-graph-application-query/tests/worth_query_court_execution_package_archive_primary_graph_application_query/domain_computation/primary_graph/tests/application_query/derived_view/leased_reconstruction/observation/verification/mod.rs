//! Judgments of canonical application, public values and retained state.
use crate::domain_computation::primary_graph::tests::application_query::derived_view::leased_reconstruction::observation::*;
pub struct ApplicationEvidence {
    pub entries: Vec<worth_relational::facade::identity::EntityId>,
    pub completions: Vec<worth_relational::facade::identity::EntityId>,
    pub later: Option<worth_relational::facade::identity::EntityId>,
    pub projected: Vec<(worth_relational::facade::identity::EntityId, String)>,
    pub published_values: Option<Vec<String>>,
    pub prior_values: Vec<String>,
}
pub fn verify(
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

    if let Some(values) = &published_values {
        assert_eq!(values.len(), count);
    }

    if observed.denial.is_some() && !case.warm {
        assert_eq!(
            published_values.as_deref().unwrap_or_default(),
            prior_values,
            "a refused cold reconstruction cannot publish partial values"
        );
    }
    observed
}
