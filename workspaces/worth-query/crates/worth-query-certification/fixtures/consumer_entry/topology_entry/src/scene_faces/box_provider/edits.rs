//! One controlled numerical edit per genuine Native before/after occurrence.
use super::*;
use worth_query_consumer_values::{insert_box_source, remove_box_source, replace_box_source};
pub(super) fn build<S: SceneFaceSchemaBinding>(
    sources: &WorthQueryManagedIndexSourceRows<'_, S, SceneFaceBoxIndex<S>>,
    nodes: &mut WorthQueryManagedIndexNodeWriter<'_, Node>,
    control: &WorthQueryManagedIndexControl<'_>,
) -> Result<Option<WorthQueryManagedIndexNodeId>, Stop> {
    let mut root = None;
    for slot in sources.slots() {
        let slot = slot?;
        let bounds = row_bounds(sources.get(slot)?)?;
        root = Some(insert_box_source(
            root,
            slot,
            bounds,
            &mut storage::Edit { nodes, control },
        )?);
    }
    Ok(root)
}
pub(super) fn update<S: SceneFaceSchemaBinding>(
    after: &WorthQueryManagedIndexSourceRows<'_, S, SceneFaceBoxIndex<S>>,
    changes: &[WorthQueryManagedIndexSourceChange],
    mut root: Option<WorthQueryManagedIndexNodeId>,
    nodes: &mut WorthQueryManagedIndexNodeWriter<'_, Node>,
    control: &WorthQueryManagedIndexControl<'_>,
) -> Result<Option<WorthQueryManagedIndexNodeId>, Stop> {
    for change in changes {
        let mut edit = storage::Edit { nodes, control };
        match (change.before(), change.after()) {
            (Some(before), Some(next)) if before == next => {
                root = Some(replace_box_source(
                    root.ok_or_else(storage::invalid)?,
                    next,
                    row_bounds(after.get(next)?)?,
                    &mut edit,
                )?);
            }
            (before, next) => {
                if let Some(before) = before {
                    root =
                        remove_box_source(root.ok_or_else(storage::invalid)?, before, &mut edit)?;
                }
                if let Some(next) = next {
                    root = Some(insert_box_source(
                        root,
                        next,
                        row_bounds(after.get(next)?)?,
                        &mut edit,
                    )?);
                }
            }
        }
    }
    Ok(root)
}
