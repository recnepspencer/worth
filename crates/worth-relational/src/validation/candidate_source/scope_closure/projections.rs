//! Endpoint lifecycle, scalar writes and rewiring pull back to actual Source rows.
use super::*;

pub(super) fn pullback(
    registry: &RelationalSchemaRegistry,
    program: &ManagedIndexSourceProgram,
    records: &[RecordRef],
    adjacency: &[AdjacencyDelta],
    [old, new]: [&mut LifecycleSource<'_, '_>; 2],
    closure: &mut PreparedManagedIndexScopeClosure,
    rows: &mut BTreeSet<NativeMemberKey>,
) -> Result<(), Denial> {
    for route in program.projection_routes() {
        let affected = crate::derived_relation::affected::collect(
            registry,
            &route.path,
            adjacency,
            records,
            old,
            new,
        )?;
        if rows.is_empty() {
            *rows = affected.roots;
        } else {
            for row in affected.roots {
                insert_row(new, rows, row)?;
            }
        }
        crate::derived_relation::support::append(new, &mut closure.support, affected.support)?;
        // The generic lifecycle owner intentionally ignores unchanged kinds.
        // Projected scalar writes must independently reacquire the inverse
        // endpoint route on both images, including an empty inverse read.
        for record in records {
            let RecordRef::Entity(entity) = *record else {
                continue;
            };
            for source in [&mut *old, &mut *new] {
                match source.entity_kind(entity) {
                    Ok(kind) if kind == route.endpoint_kind() => {}
                    Ok(_) | Err(Denial::Definition(Definition::RecordUnavailable)) => continue,
                    Err(cause) => return Err(cause),
                }
                let data = evaluate_position(
                    registry,
                    &route.path,
                    route.path.parts().nodes.len() - 1,
                    true,
                    entity.into(),
                    source,
                )?;
                for row in data.endpoints() {
                    let row = row.entity().ok_or(Definition::InvalidMemberDomain)?;
                    insert_row(source, rows, row.into())?;
                }
                crate::derived_relation::support::push(source, &mut closure.support, data)?;
            }
        }
    }
    Ok(())
}

fn insert_row(
    source: &mut LifecycleSource<'_, '_>,
    rows: &mut BTreeSet<NativeMemberKey>,
    row: NativeMemberKey,
) -> Result<(), Denial> {
    if !rows.contains(&row) {
        let after = rows
            .len()
            .checked_add(1)
            .ok_or(Definition::WorkBudgetExceeded)?;
        let levels = (usize::BITS - after.leading_zeros()) as usize + 1;
        let bytes = size_of::<NativeMemberKey>()
            .checked_mul(11)
            .and_then(|bytes| bytes.checked_add(14 * size_of::<usize>()))
            .and_then(|bytes| bytes.checked_mul(levels))
            .ok_or(Definition::WorkBudgetExceeded)?;
        source.retain(bytes)?;
        rows.insert(row);
    }
    Ok(())
}
