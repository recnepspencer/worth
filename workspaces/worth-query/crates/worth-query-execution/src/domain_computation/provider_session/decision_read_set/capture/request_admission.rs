//! Exact declared family and request admission for ordinary provider capture.
use super::*;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn admit_requests(
    authority: &WorthQuerySessionReadAuthority<'_>,
    requests: impl IntoIterator<Item = WorthQueryDecisionFactRequest>,
    request_scope: Option<&WorthQueryRequestScope>,
) -> Result<
    (
        BTreeSet<WorthQueryDecisionFactRequest>,
        WorthQueryDecisionReadSetCounters,
    ),
    WorthQueryDecisionReadSetFailure,
> {
    let mut selected = BTreeSet::new();
    for request in requests {
        check_request(request_scope)?;
        selected.insert(request);
    }
    let requests = selected;
    let mut family_counts = BTreeMap::<&str, usize>::new();
    for request in &requests {
        check_request(request_scope)?;
        let family = authority
            .plan()
            .decision_fact_families()
            .iter()
            .find(|family| family.identity() == request.family_identity())
            .ok_or_else(|| denial(WorthQueryDecisionReadSetDenialKind::UndeclaredFamily))?;
        if family.kind() != request.kind() {
            return Err(denial(
                WorthQueryDecisionReadSetDenialKind::FamilyKindMismatch,
            ));
        }
        let count = family_counts.entry(family.identity()).or_default();
        *count = count
            .checked_add(1)
            .ok_or_else(|| denial(WorthQueryDecisionReadSetDenialKind::FactCountOverflow))?;
    }
    for family in authority.plan().decision_fact_families() {
        check_request(request_scope)?;
        match family.cardinality() {
            WorthQueryDecisionFactCardinality::Exact(_)
                if !family_counts.contains_key(family.identity()) =>
            {
                return Err(denial(
                    WorthQueryDecisionReadSetDenialKind::IncompleteRequiredFamilies,
                ));
            }
            WorthQueryDecisionFactCardinality::Exact(expected)
                if family_counts[family.identity()] != expected =>
            {
                return Err(denial(
                    WorthQueryDecisionReadSetDenialKind::IncompleteRequiredFacts,
                ));
            }
            WorthQueryDecisionFactCardinality::Bounded { maximum }
                if family_counts
                    .get(family.identity())
                    .copied()
                    .unwrap_or_default()
                    > maximum =>
            {
                return Err(denial(
                    WorthQueryDecisionReadSetDenialKind::DecisionFactBudgetExceeded,
                ));
            }
            WorthQueryDecisionFactCardinality::Exact(_)
            | WorthQueryDecisionFactCardinality::Bounded { .. }
            | WorthQueryDecisionFactCardinality::Variable => {}
        }
    }
    let requested_facts = requests.len();
    Ok((
        requests,
        WorthQueryDecisionReadSetCounters {
            requested_facts,
            ..WorthQueryDecisionReadSetCounters::default()
        },
    ))
}
