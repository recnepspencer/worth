use crate::identity::data::{EntityId, KindId};
use crate::indexes::data::{
    BoundedEntityFieldLookupAdmissionStop as Stop, BoundedEntityFieldLookupDenial,
    BoundedEntityFieldLookupDenialKind as Denial, DerivedIndexId, DerivedIndexRows,
};

/// Each entry lane owns its existing admission and exact-root comparison.
pub(super) trait EntityFieldCollectionRead<E> {
    fn prepare_results(&mut self, examined: usize) -> Result<(), Stop<E>>;
    fn compare(
        &mut self,
        entity: EntityId,
        previous: usize,
    ) -> Result<Option<(KindId, bool)>, Stop<E>>;
}

/// Collect the complete bounded prefix without promoting corrupt or duplicate
/// index entries into candidates. Overflow remains separate from that prefix.
pub(super) fn collect<E>(
    rows: Option<&DerivedIndexRows<EntityId>>,
    index: DerivedIndexId,
    kind: KindId,
    limit: usize,
    read: &mut impl EntityFieldCollectionRead<E>,
) -> Result<(Vec<EntityId>, usize, bool), Stop<E>> {
    let count = rows.map_or(0, |rows| rows.len());
    let examined = count.min(limit);
    read.prepare_results(examined)?;
    let mut seen = Vec::with_capacity(examined);
    let mut candidates = Vec::with_capacity(examined);
    if let Some(rows) = rows {
        for (ordinal, entity) in rows.iter().take(limit).enumerate() {
            let Some((actual_kind, matches)) = read.compare(*entity, seen.len())? else {
                return Err(corrupt(index, ordinal + 1));
            };
            if seen.contains(entity) || !matches {
                return Err(corrupt(index, ordinal + 1));
            }
            seen.push(*entity);
            if actual_kind == kind {
                candidates.push(*entity);
            }
        }
    }
    Ok((candidates, examined, count > limit))
}

fn corrupt<E>(index: DerivedIndexId, examined: usize) -> Stop<E> {
    Stop::Lookup(
        BoundedEntityFieldLookupDenial::new(Denial::CorruptIndexEntries, index)
            .with_examined_entry_count(examined),
    )
}
