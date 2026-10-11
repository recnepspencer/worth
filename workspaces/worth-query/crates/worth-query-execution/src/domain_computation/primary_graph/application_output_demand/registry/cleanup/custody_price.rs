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
            add(&mut allowance.navigation, descent.checked_mul(3), kind)?;
            add(&mut allowance.paths, Some(3), kind)?;
            for (_, row) in selected {
                charge.inspect_row()?;
                add(&mut allowance.rows, Some(3), kind)?;
                let claims = row.closed_cleanup.node().claimants.lock().unwrap().len();
                add(
                    &mut allowance.rows,
                    u64::try_from(claims).ok().and_then(|n| n.checked_mul(2)),
                    kind,
                )?;
            }
        }
    }
    Ok(allowance)
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
