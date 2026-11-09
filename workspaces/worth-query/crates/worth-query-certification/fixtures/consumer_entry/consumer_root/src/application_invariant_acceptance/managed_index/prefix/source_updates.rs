//! Opt-in observation of the actual provider update receiver, through ordinary rows.
use super::*;
pub type RowChanges = Vec<(query::ItemRow, query::ItemRow)>;
pub type Observer = Box<dyn FnMut(Vec<query::ItemRow>, Vec<query::ItemRow>, RowChanges) + Send>;
pub(super) fn record<I: PrefixSourceIndex>(
    calls: &Calls,
    before: &WorthQueryManagedIndexSourceRows<'_, Catalog, I>,
    after: &WorthQueryManagedIndexSourceRows<'_, Catalog, I>,
    changes: &[WorthQueryManagedIndexSourceChange],
) -> Result<(), Stop> {
    let mut records = calls.source_updates.lock().unwrap();
    let Some(observe) = records.as_mut() else {
        return Ok(());
    };
    let read = |rows: &WorthQueryManagedIndexSourceRows<'_, Catalog, I>| -> Result<Vec<_>, Stop> {
        rows.slots()
            .map(|slot| {
                let row = rows.get(slot?)?;
                query::ItemRow::project(&row)
                    .map_err(|_| Stop::Domain(PrefixIndexDenial::InvalidTopology))
            })
            .collect()
    };
    let changed = changes
        .iter()
        .map(|change| {
            let before = before.get(change.before().expect("existing row"))?;
            let after = after.get(change.after().expect("surviving row"))?;
            Ok((
                query::ItemRow::project(&before)
                    .map_err(|_| Stop::Domain(PrefixIndexDenial::InvalidTopology))?,
                query::ItemRow::project(&after)
                    .map_err(|_| Stop::Domain(PrefixIndexDenial::InvalidTopology))?,
            ))
        })
        .collect::<Result<_, Stop>>()?;
    observe(read(before)?, read(after)?, changed);
    Ok(())
}
