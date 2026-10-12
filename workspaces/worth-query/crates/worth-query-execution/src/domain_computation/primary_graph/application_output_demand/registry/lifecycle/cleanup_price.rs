//! Prepay custody pruning and the closed-row cascade before their first edit.
use super::super::refreshed_rejoin::{charge_whole_map, PricedMutation, WalkKind, WholeMapPasses};
use super::super::{DemandRegistryState, WorthQueryOutputDemandKey};
use super::closed_retirement::{closed_cached, closed_superseded};
use crate::domain_computation::primary_graph::application_query::observed_source;
use crate::domain_computation::primary_graph::output_family_identity::OutputFamilyIdentity;
use crate::domain_computation::primary_graph::InvalidationEditAdmission;
use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};
use std::collections::{BTreeMap, BTreeSet};

type Occurrence = (
    OutputFamilyIdentity,
    observed_source::source_identity::WorthQueryObservedSourceCoordinate,
);

type Walks = Vec<(WorthQueryOutputDemandKey, u64)>;

/// Quote a cascade whose caller leaves held custody for a later drain.
pub(in super::super) fn price_cascade_mutation<'a>(
    state: &'a mut DemandRegistryState,
    admission: &mut InvalidationEditAdmission,
    mut walks: Walks,
    roots: &[WorthQueryOutputDemandKey],
    kind: WalkKind,
) -> Result<PricedMutation<'a>, WorthQueryOutputDemandDenial> {
    walks.extend(cascade_walks(state, roots, admission, kind)?);
    super::super::refreshed_rejoin::price_mutation(state, admission, &walks, kind)
}

pub(in super::super) fn price_closed_cascade<'a>(
    state: &'a mut DemandRegistryState,
    admission: &mut InvalidationEditAdmission,
    roots: &[WorthQueryOutputDemandKey],
) -> Result<PricedMutation<'a>, WorthQueryOutputDemandDenial> {
    let kind = WalkKind::TerminalCleanup;
    for key in roots {
        let Some(row) = state.records.get(key) else {
            continue;
        };
        if row.terminal()
            && row.interests == 0
            && row.framework_required_count == 0
            && row.prepared_prerequisite_claims == 0
            && !row.performed_obligations.is_empty()
        {
            let work = row
                .performed_obligations
                .capacity()
                .checked_add(row.performed_obligations.len())
                .and_then(|n| {
                    n.checked_mul(std::mem::size_of::<super::super::PerformedOutputObligation>())
                })
                .and_then(|n| n.checked_add(8))
                .ok_or_else(|| kind.denial())?;
            admission
                .charge_external_work(u64::try_from(work).map_err(|_| kind.denial())?)
                .map_err(|_| kind.denial())?;
        }
    }
    let mut walks = cascade_walks(state, roots, admission, kind)?;
    walks.extend(roots.iter().cloned().map(|key| (key, 1)));
    super::super::refreshed_rejoin::price_mutation(state, admission, &walks, kind)
}

struct Candidate {
    cached: bool,
    superseded: bool,
    terminal: bool,
    prerequisites: Vec<WorthQueryOutputDemandKey>,
}

struct CascadeQuote {
    candidates: BTreeMap<WorthQueryOutputDemandKey, Candidate>,
    occurrences: BTreeMap<Occurrence, Vec<WorthQueryOutputDemandKey>>,
    incoming: BTreeMap<WorthQueryOutputDemandKey, u64>,
    edges: u64,
}

fn cascade_walks(
    state: &DemandRegistryState,
    roots: &[WorthQueryOutputDemandKey],
    admission: &mut InvalidationEditAdmission,
    kind: WalkKind,
) -> Result<Walks, WorthQueryOutputDemandDenial> {
    if !roots.iter().any(|key| {
        state
            .records
            .get(key)
            .is_some_and(|row| row.terminal() || closed_cached(row))
    }) {
        return Ok(Vec::new());
    }
    let quote = CascadeQuote::read(state, admission, kind)?;
    let selected = quote.reachable(roots);
    if selected
        .iter()
        .any(|key| quote.candidates.get(key).is_some_and(|row| row.superseded))
    {
        // The cascade's lazily built claimant index is another whole-map pass.
        admission
            .charge_external_work(
                (state.records.len() as u64)
                    .checked_add(quote.edges)
                    .ok_or_else(|| kind.denial())?,
            )
            .map_err(|_| kind.denial())?;
    }
    // Initial candidates, retired edges, and transferred edges bound retries.
    // Each original edge retires once and transfers at most once.
    let visits = quote
        .edges
        .checked_mul(2)
        .and_then(|count| count.checked_add(roots.len() as u64))
        .ok_or_else(|| kind.denial())?;
    quote.walks(selected, visits, kind)
}

impl CascadeQuote {
    fn read(
        state: &DemandRegistryState,
        admission: &mut InvalidationEditAdmission,
        kind: WalkKind,
    ) -> Result<Self, WorthQueryOutputDemandDenial> {
        // Count inline row headers before admitting the plan's cloned keys,
        // edge lists, ordered indices and cascade worklists.
        let mut edges = Some(0usize);
        let mut producer = 0usize;
        charge_whole_map(&state.records, admission, WholeMapPasses::One, 0)
            .map_err(|_| kind.denial())?
            .visit_rows(|key, row| {
                edges = edges.and_then(|n| n.checked_add(row.prerequisites.len()));
                producer = producer.max(key.producer.len());
            })?;
        let edges = edges.ok_or_else(|| kind.denial())?;
        let slots = state
            .records
            .len()
            .checked_mul(24)
            .and_then(|n| edges.checked_mul(12).and_then(|edges| n.checked_add(edges)))
            .and_then(|n| n.checked_add(8))
            .ok_or_else(|| kind.denial())?;
        let slot = std::mem::size_of::<WorthQueryOutputDemandKey>()
            .checked_add(producer)
            .and_then(|n| n.checked_add(std::mem::size_of::<Candidate>()))
            .and_then(|n| n.checked_add(12 * std::mem::size_of::<usize>()))
            .ok_or_else(|| kind.denial())?;
        let scratch = slots.checked_mul(slot).ok_or_else(|| kind.denial())?;
        admission
            .admit_read_scratch(u64::try_from(scratch).map_err(|_| kind.denial())?)
            .map_err(|_| {
                super::super::WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
                    "closed-row cascade scratch exceeds request preparation",
                )
            })?;
        // Both the edge index and reachability traverse these existing edges.
        admission
            .charge_external_work(
                u64::try_from(edges)
                    .ok()
                    .and_then(|n| n.checked_mul(2))
                    .ok_or_else(|| kind.denial())?,
            )
            .map_err(|_| kind.denial())?;
        let mut quote = Self {
            candidates: BTreeMap::new(),
            occurrences: BTreeMap::new(),
            incoming: BTreeMap::new(),
            edges: u64::try_from(edges).map_err(|_| kind.denial())?,
        };
        // The receipt supplies this map and is consumed by the plan's scan.
        charge_whole_map(&state.records, admission, WholeMapPasses::One, 0)
            .map_err(|_| kind.denial())?
            .visit_rows(|key, row| {
                quote
                    .occurrences
                    .entry((key.family.clone(), key.source.occurrence_coordinate()))
                    .or_default()
                    .push(key.clone());
                let prerequisites: Vec<_> = row
                    .prerequisites
                    .iter()
                    .map(|key| key.as_ref().clone())
                    .collect();
                for upstream in &prerequisites {
                    *quote.incoming.entry(upstream.clone()).or_default() += 1;
                }
                quote.candidates.insert(
                    key.clone(),
                    Candidate {
                        cached: closed_cached(row),
                        superseded: closed_superseded(row),
                        terminal: row.terminal(),
                        prerequisites,
                    },
                );
            })?;
        Ok(quote)
    }

    fn reachable(
        &self,
        roots: &[WorthQueryOutputDemandKey],
    ) -> BTreeSet<WorthQueryOutputDemandKey> {
        let mut selected = BTreeSet::new();
        let mut expanded = BTreeSet::new();
        let mut pending = roots.to_vec();
        while let Some(key) = pending.pop() {
            if !selected.insert(key.clone()) {
                continue;
            }
            let coordinate = (key.family.clone(), key.source.occurrence_coordinate());
            if expanded.insert(coordinate.clone()) {
                pending.extend(
                    self.occurrences
                        .get(&coordinate)
                        .into_iter()
                        .flatten()
                        .cloned(),
                );
            }
            if let Some(row) = self.candidates.get(&key) {
                pending.extend(row.prerequisites.iter().cloned());
            }
        }
        selected
    }

    fn walks(
        &self,
        selected: BTreeSet<WorthQueryOutputDemandKey>,
        visits: u64,
        kind: WalkKind,
    ) -> Result<Walks, WorthQueryOutputDemandDenial> {
        let mut walks = Vec::new();
        for key in selected {
            let Some(row) = self.candidates.get(&key) else {
                continue;
            };
            // held, settlement handoff, own membership; three per claimant.
            let own = 3 * u64::from(row.terminal);
            let claimant = if row.superseded {
                self.incoming
                    .get(&key)
                    .copied()
                    .unwrap_or(0)
                    .checked_mul(3)
                    .ok_or_else(|| kind.denial())?
            } else {
                0
            };
            let count = own
                .checked_add(claimant)
                .and_then(|count| count.checked_mul(visits))
                .ok_or_else(|| kind.denial())?;
            if count != 0 {
                walks.push((key, count));
            }
            // A cached row follows its edges; a retiring row releases them.
            let edge_walks = 3 * (u64::from(row.cached) + u64::from(row.terminal));
            let count = edge_walks
                .checked_mul(visits)
                .ok_or_else(|| kind.denial())?;
            if count != 0 {
                walks.extend(row.prerequisites.iter().cloned().map(|key| (key, count)));
            }
        }
        Ok(walks)
    }
}
