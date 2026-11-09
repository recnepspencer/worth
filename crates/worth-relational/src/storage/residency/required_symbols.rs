//! Cached dictionary closure and actual visits of an admitted region view.
use crate::storage::overlay::PartitionState;
use crate::storage::substrate::maintenance::ReconstructionLane;
use crate::symbols::data::Symbol;
use std::collections::BTreeSet;
use std::sync::Arc;
use worth_foundational::facade::{
    AspectValue, AuthoritativeRecordAspectState, ContractValidatedAspectValueView, InternedString,
};
type Failure = super::RelationalResidencyDemandFailure;
type Checkpoint<'a> = dyn FnMut(usize) -> Result<(), Failure> + 'a;
#[derive(Debug, Default)]
pub(super) struct CollectedSymbols {
    pub(super) ids: BTreeSet<Symbol>,
    pub(super) visits: u64,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl CollectedSymbols {
    fn insert(&mut self, symbol: Symbol) {
        self.visits += 1;
        self.ids.insert(symbol);
    }
    fn extend(&mut self, values: impl IntoIterator<Item = Symbol>) {
        for value in values {
            self.insert(value);
        }
    }
}
fn step(symbols: &mut CollectedSymbols, checkpoint: &mut Checkpoint<'_>) -> Result<(), Failure> {
    checkpoint(
        symbols
            .ids
            .len()
            .checked_add(1)
            .ok_or_else(super::region_load::materialization_funding::overflow)?,
    )?;
    symbols.visits += 1;
    Ok(())
}
#[cfg(test)]
pub(super) fn collect_head(
    lane: &ReconstructionLane<'_>,
    partition: &PartitionState,
) -> BTreeSet<Symbol> {
    collect_counted(lane, partition, false).ids
}
#[cfg(test)]
pub(super) fn collect(
    lane: &ReconstructionLane<'_>,
    partition: &PartitionState,
) -> BTreeSet<Symbol> {
    collect_counted(lane, partition, true).ids
}
#[cfg(test)]
pub(super) fn collect_counted(
    _lane: &ReconstructionLane<'_>,
    partition: &PartitionState,
    full: bool,
) -> CollectedSymbols {
    collect_using(partition, full, &mut |_| Ok(()), &mut |_| Ok(()))
        .expect("uncontrolled symbol oracle cannot stop")
}
pub(super) fn collect_for_request(
    _lane: &ReconstructionLane<'_>,
    partition: &PartitionState,
    full: bool,
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<CollectedSymbols, Failure> {
    use super::region_load::materialization_funding as funding;
    let mut memory = funding::Funding::reserve(funding::shared::<CollectedSymbols>()?, request)?;
    let mut retained = 0;
    let mut result = collect_using(
        partition,
        full,
        &mut |count| {
            let bytes = funding::tree::<Symbol, ()>(count)?;
            if bytes > retained {
                memory.grow(bytes - retained, request)?;
                retained = bytes;
            }
            funding::work(request, 1 + 22 * (count.max(1).ilog2() as u64 + 1))
        },
        &mut |_| Ok(()),
    )?;
    result._memory = memory.into_memory();
    Ok(result)
}
fn collect_using(
    partition: &PartitionState,
    full: bool,
    checkpoint: &mut Checkpoint<'_>,
    head: &mut dyn FnMut(&CollectedSymbols) -> Result<(), Failure>,
) -> Result<CollectedSymbols, Failure> {
    let mut symbols = CollectedSymbols::default();
    for versions in partition
        .entity_arena
        .aspect_versions
        .iter()
        .chain(partition.relation_arena.aspect_versions.iter())
    {
        step(&mut symbols, checkpoint)?;
        for &symbol in versions.keys() {
            checkpoint(symbols.ids.len() + 1)?;
            symbols.insert(symbol);
        }
    }
    for extra in partition.entity_arena.extra.iter() {
        step(&mut symbols, checkpoint)?;
        if let Some(fingerprint) = extra.structural_fingerprint {
            checkpoint(symbols.ids.len() + 1)?;
            symbols.insert(fingerprint.family);
        }
    }
    for revisions in partition
        .entity_arena
        .field_revisions
        .iter()
        .chain(partition.relation_arena.field_revisions.iter())
    {
        step(&mut symbols, checkpoint)?;
        if let Some(revisions) = revisions {
            for (aspect, field) in revisions.keys() {
                checkpoint(symbols.ids.len() + 2)?;
                symbols.insert(*aspect);
                symbols.insert(*field);
            }
        }
    }
    for state in partition
        .entity_arena
        .extra
        .iter()
        .map(|extra| extra.authoritative_aspect_state())
        .chain(
            partition
                .relation_arena
                .extra
                .iter()
                .map(|extra| extra.authoritative_aspect_state()),
        )
        .flatten()
    {
        step(&mut symbols, checkpoint)?;
        visit_state_using(state, &mut symbols, checkpoint)?;
    }
    head(&symbols)?;
    if full {
        for history in partition.entity_arena.metadata_history.iter() {
            step(&mut symbols, checkpoint)?;
            for metadata in history.iter() {
                step(&mut symbols, checkpoint)?;
                if let Some(state) = metadata.authoritative_aspect_state() {
                    visit_state_using(state, &mut symbols, checkpoint)?;
                }
            }
        }
        for history in partition.relation_arena.metadata_history.iter() {
            step(&mut symbols, checkpoint)?;
            for metadata in history.iter() {
                step(&mut symbols, checkpoint)?;
                if let Some(state) = metadata.authoritative_aspect_state() {
                    visit_state_using(state, &mut symbols, checkpoint)?;
                }
            }
        }
    }
    Ok(symbols)
}
fn visit_state(state: &AuthoritativeRecordAspectState, symbols: &mut CollectedSymbols) {
    visit_state_using(state, symbols, &mut |_| Ok(()))
        .expect("ordinary symbol delta traversal cannot stop")
}
fn visit_state_using(
    state: &AuthoritativeRecordAspectState,
    symbols: &mut CollectedSymbols,
    checkpoint: &mut Checkpoint<'_>,
) -> Result<(), Failure> {
    step(symbols, checkpoint)?;
    for (_, aspect) in state.aspects().entries() {
        step(symbols, checkpoint)?;
        match aspect.view() {
            ContractValidatedAspectValueView::Scalar(value) => {
                visit_value(value, symbols, checkpoint)?
            }
            ContractValidatedAspectValueView::Struct(value) => {
                for (_, field) in value.fields() {
                    visit_value(field, symbols, checkpoint)?;
                }
            }
        }
    }
    Ok(())
}
fn visit_value(
    value: &AspectValue,
    symbols: &mut CollectedSymbols,
    checkpoint: &mut Checkpoint<'_>,
) -> Result<(), Failure> {
    step(symbols, checkpoint)?;
    if let AspectValue::String(InternedString::Symbol(symbol)) = value {
        checkpoint(symbols.ids.len() + 1)?;
        symbols.insert(Symbol(symbol.0));
    }
    Ok(())
}
/// Only values whose persistent owners changed can introduce new meanings.
pub(super) fn collect_delta(
    previous: &PartitionState,
    current: &PartitionState,
    full: bool,
) -> CollectedSymbols {
    let mut result = CollectedSymbols::default();
    arena_keys(&previous.entity_arena, &current.entity_arena, &mut result);
    arena_keys(
        &previous.relation_arena,
        &current.relation_arena,
        &mut result,
    );
    current.entity_arena.extra.visit_changed_values(
        &previous.entity_arena.extra,
        &mut |_, new, _| {
            result.visits += 1;
            if let Some(new) = new {
                if let Some(fingerprint) = new.structural_fingerprint {
                    result.insert(fingerprint.family);
                }
                if let Some(state) = new.authoritative_aspect_state() {
                    visit_state(state, &mut result);
                }
            }
        },
    );
    current.relation_arena.extra.visit_changed_values(
        &previous.relation_arena.extra,
        &mut |_, new, _| {
            result.visits += 1;
            if let Some(state) = new.and_then(|extra| extra.authoritative_aspect_state()) {
                visit_state(state, &mut result);
            }
        },
    );
    if full {
        history(
            &previous.entity_arena.metadata_history,
            &current.entity_arena.metadata_history,
            |meta| meta.authoritative_aspect_state(),
            &mut result,
        );
        history(
            &previous.relation_arena.metadata_history,
            &current.relation_arena.metadata_history,
            |meta| meta.authoritative_aspect_state(),
            &mut result,
        );
    }
    result
}
fn arena_keys<K: crate::storage::substrate::RecordKind>(
    previous: &crate::storage::substrate::RecordArena<K>,
    current: &crate::storage::substrate::RecordArena<K>,
    result: &mut CollectedSymbols,
) {
    current
        .aspect_versions
        .visit_changed_values(&previous.aspect_versions, &mut |_, new, _| {
            result.visits += 1;
            if let Some(new) = new {
                result.extend(new.keys().copied());
            }
        });
    current
        .field_revisions
        .visit_changed_values(&previous.field_revisions, &mut |_, new, _| {
            result.visits += 1;
            if let Some(Some(new)) = new {
                for (aspect, field) in new.keys() {
                    result.insert(*aspect);
                    result.insert(*field);
                }
            }
        });
}
fn history<M: crate::storage::substrate::StoredValue>(
    previous: &crate::storage::substrate::SharedColumn<crate::storage::substrate::SharedColumn<M>>,
    current: &crate::storage::substrate::SharedColumn<crate::storage::substrate::SharedColumn<M>>,
    state: impl Fn(&M::Value) -> Option<&AuthoritativeRecordAspectState>,
    result: &mut CollectedSymbols,
) {
    current.visit_changed_values(previous, &mut |_, new, old| {
        result.visits += 1;
        if let Some(new) = new {
            let empty = crate::storage::substrate::SharedColumn::with_capacity(0);
            new.visit_changed_values(old.unwrap_or(&empty), &mut |_, new, _| {
                result.visits += 1;
                if let Some(state) = new.and_then(&state) {
                    visit_state(state, result);
                }
            });
        }
    });
}

/// Only admitted cold/initial images may create a complete requirement seed.
#[derive(Debug)]
pub(super) struct CapturedSymbolRequirements {
    head: Arc<CollectedSymbols>,
    owned: Option<Arc<CollectedSymbols>>,
    full: bool,
}
impl CapturedSymbolRequirements {
    pub(super) fn capture_for_request(
        _lane: &ReconstructionLane<'_>,
        partition: &PartitionState,
        full: bool,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, super::RelationalResidencyDemandFailure> {
        use super::region_load::materialization_funding as funding;
        let mut memory =
            funding::Funding::reserve(funding::shared::<CollectedSymbols>()?, request)?;
        let mut retained = 0;
        let mut head = None;
        let mut owned = collect_using(
            partition,
            true,
            &mut |count| {
                let bytes = funding::tree::<Symbol, ()>(count)?;
                if bytes > retained {
                    memory.grow(bytes - retained, request)?;
                    retained = bytes;
                }
                funding::work(request, 1 + 22 * (count.max(1).ilog2() as u64 + 1))
            },
            &mut |symbols| {
                let bytes = funding::sum(
                    funding::shared::<CollectedSymbols>()?,
                    funding::tree::<Symbol, ()>(symbols.ids.len())?,
                )?;
                let backing = funding::Funding::reserve(bytes, request)?;
                funding::work(request, funding::sum(symbols.ids.len() as u64, bytes)?)?;
                head = Some(Arc::new(CollectedSymbols {
                    ids: symbols.ids.clone(),
                    visits: symbols.visits,
                    _memory: backing.into_memory(),
                }));
                Ok(())
            },
        )?;
        owned._memory = memory.into_memory();
        Ok(Self {
            head: head.expect("the completed symbol traversal captures its current prefix"),
            owned: Some(Arc::new(owned)),
            full,
        })
    }
    pub(super) fn changes(&self) -> Option<Arc<super::RegionSymbolChanges>> {
        Some(Arc::new(super::RegionSymbolChanges {
            head: self.head.clone(),
            owned: self.owned.clone()?,
            prefixes_changed: false,
        }))
    }
    pub(super) fn head_only(&self) -> Self {
        Self {
            head: self.head.clone(),
            owned: None,
            full: false,
        }
    }
    pub(super) fn with_omitted_history(&self) -> Self {
        Self {
            head: self.head.clone(),
            owned: self.owned.clone(),
            full: false,
        }
    }
    pub(super) fn owned(&self) -> Option<&CollectedSymbols> {
        self.owned.as_deref()
    }
    pub(super) fn get(&self, full: bool) -> Option<&CollectedSymbols> {
        if full {
            self.full.then(|| self.owned.as_deref()).flatten()
        } else {
            Some(&self.head)
        }
    }
}
