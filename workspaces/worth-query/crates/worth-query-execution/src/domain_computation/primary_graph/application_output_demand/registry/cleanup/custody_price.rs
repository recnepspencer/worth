//! Quote one noted entry without allocating a list of its occurrence walks.
use super::super::refreshed_rejoin::{
    occurrence_rows, source_rows, OccurrenceSearchCharge, SourceWalkAllowance, WalkAdmission,
    WalkKind,
};
use super::*;
use worth_runtime_world::facade::CompositeCommitIdentity;

pub(in super::super) fn entry_allowance(
    state: &DemandRegistryState,
    commit: &CompositeCommitIdentity,
    admission: &mut InvalidationEditAdmission,
    kind: WalkKind,
) -> Result<SourceWalkAllowance, WorthQueryOutputDemandDenial> {
    admission
        .charge_external_work(1)
        .map_err(|_| kind.denial())?;
    let mut allowance = SourceWalkAllowance {
        paths: 0,
        rows: 0,
        entries: 1,
        navigation: 0,
    };
    let custody = &state.source_custody[commit];
    for source in custody.bound_sources.iter().flatten() {
        let mut charge = WalkAdmission::new(admission, kind);
        let before = charge.admission_mut().charged_navigation();
        let rows = source_rows(&state.records, &source.identity, &mut charge)?;
        let descent = charge.admission_mut().charged_navigation() - before;
        add(&mut allowance.navigation, descent.checked_mul(2), kind)?;
        add(&mut allowance.paths, Some(2), kind)?;
        for (key, _) in rows {
            charge.inspect_row()?;
            add(&mut allowance.rows, Some(2), kind)?;
            // The held test can inspect every row's stale owners. Its quote
            // and execution are both paid, before any entry is unlinked.
            let before = charge.admission_mut().charged_navigation();
            let selected = occurrence_rows(&state.records, key, &mut charge)?;
            let descent = charge.admission_mut().charged_navigation() - before;
            add(&mut allowance.navigation, Some(descent), kind)?;
            add(&mut allowance.paths, Some(1), kind)?;
            for _ in selected {
                charge.inspect_row()?;
                add(&mut allowance.rows, Some(1), kind)?;
            }
        }
    }
    Ok(allowance)
}

pub(in super::super) fn noted_allowance(
    state: &DemandRegistryState,
    admission: &mut InvalidationEditAdmission,
    kind: WalkKind,
) -> Result<SourceWalkAllowance, WorthQueryOutputDemandDenial> {
    let mut total = SourceWalkAllowance {
        paths: 0,
        rows: 0,
        entries: 0,
        navigation: 0,
    };
    let mut next = state.custody_cleanup.head.as_ref();
    while let Some(commit) = next {
        let price = entry_allowance(state, commit, admission, kind)?;
        add(&mut total.paths, Some(price.paths), kind)?;
        add(&mut total.rows, Some(price.rows), kind)?;
        add(&mut total.entries, Some(price.entries), kind)?;
        add(&mut total.navigation, Some(price.navigation), kind)?;
        next = state.source_custody[commit].cleanup.next.as_ref();
    }
    Ok(total)
}

fn add(
    total: &mut u64,
    amount: Option<u64>,
    kind: WalkKind,
) -> Result<(), WorthQueryOutputDemandDenial> {
    *total = amount
        .and_then(|amount| total.checked_add(amount))
        .ok_or_else(|| kind.denial())?;
    Ok(())
}
