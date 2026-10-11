//! Prepay custody pruning and the closed-row cascade before their first edit.
use super::super::refreshed_rejoin::{
    charge_whole_map, price_mutation_with_source_allowance, source_rows, OccurrenceSearchCharge,
    PricedMutation, SourceWalkAllowance, WalkAdmission, WalkKind, WholeMapPasses,
};
use super::super::{DemandRegistryState, WorthQueryOutputDemandKey};
use super::closed_retirement::{closed_cached, closed_superseded};
use crate::domain_computation::primary_graph::application_query::observed_source;
use crate::domain_computation::primary_graph::output_family_identity::OutputFamilyIdentity;
use crate::domain_computation::primary_graph::InvalidationEditAdmission;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;
use std::collections::{BTreeMap, BTreeSet};

type Occurrence = (
    OutputFamilyIdentity,
    observed_source::source_identity::WorthQueryObservedSourceCoordinate,
);

type Walks = Vec<(WorthQueryOutputDemandKey, u64)>;

pub(in super::super) fn price_cleanup<'a>(
    state: &'a mut DemandRegistryState,
    admission: &mut InvalidationEditAdmission,
    mut walks: Walks,
    cascade: &[WorthQueryOutputDemandKey],
    kind: WalkKind,
) -> Result<PricedMutation<'a>, WorthQueryOutputDemandDenial> {
    walks.extend(cascade_walks(state, cascade, admission, kind)?);
    let allowance = custody_walks(state, &mut walks, admission, kind)?;
    price_mutation_with_source_allowance(state, admission, &walks, kind, allowance)
}

pub(in super::super) fn custody_walks(
    state: &DemandRegistryState,
    walks: &mut Walks,
    admission: &mut InvalidationEditAdmission,
    kind: WalkKind,
) -> Result<SourceWalkAllowance, WorthQueryOutputDemandDenial> {
    let mut allowance = SourceWalkAllowance {
        paths: 0,
        rows: 0,
        entries: state.source_custody.len() as u64,
        navigation: 0,
    };
    // The quote's custody-entry scan is paid as well as the later sweep.
    admission
        .charge_external_work(allowance.entries)
        .map_err(|_| kind.denial())?;
    for custody in state.source_custody.values() {
        for source in custody.bound_sources.iter().flatten() {
            let mut charge = WalkAdmission::new(admission, kind);
            let mut count = 0u64;
            let before = charge.admission_mut().charged_navigation();
            let rows = source_rows(&state.records, &source.identity, &mut charge)?;
            let descent = charge.admission_mut().charged_navigation() - before;
            allowance.navigation = descent
                .checked_mul(2)
                .and_then(|price| allowance.navigation.checked_add(price))
                .ok_or_else(|| kind.denial())?;
            for (key, _) in rows {
                charge.inspect_row()?;
                count = count.checked_add(1).ok_or_else(|| kind.denial())?;
                walks.push((key.clone(), 1));
            }
            // Pruning checks the source, then may gather its retiring rows.
            allowance.paths = allowance
                .paths
                .checked_add(2)
                .ok_or_else(|| kind.denial())?;
            allowance.rows = count
                .checked_mul(2)
                .and_then(|rows| allowance.rows.checked_add(rows))
                .ok_or_else(|| kind.denial())?;
        }
    }
    Ok(allowance)
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
        let mut quote = Self {
            candidates: BTreeMap::new(),
            occurrences: BTreeMap::new(),
            incoming: BTreeMap::new(),
            edges: 0,
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
                quote.edges = quote.edges.saturating_add(prerequisites.len() as u64);
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
        // Both the read-only edge index and reachability traverse the edges.
        admission
            .charge_external_work(quote.edges.checked_mul(2).ok_or_else(|| kind.denial())?)
            .map_err(|_| kind.denial())?;
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
