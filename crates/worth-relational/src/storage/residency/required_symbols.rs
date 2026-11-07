//! Dictionary closure of an admitted region view, including aspect revision
//! keys and fingerprint families as well as retained native field values.
use crate::storage::overlay::PartitionState;
use crate::storage::substrate::maintenance::ReconstructionLane;
use crate::symbols::data::Symbol;
use std::collections::BTreeSet;
use std::sync::Arc;
use worth_foundational::facade::{
    AspectValue, AuthoritativeRecordAspectState, ContractValidatedAspectValueView, InternedString,
};

#[derive(Debug, Default)]
pub(super) struct CollectedSymbols {
    pub(super) ids: BTreeSet<Symbol>,
    pub(super) visits: u64,
}
impl CollectedSymbols {
    fn insert(&mut self, symbol: Symbol) {
        self.visits += 1;
        self.ids.insert(symbol);
    }
    fn extend(&mut self, symbols: impl IntoIterator<Item = Symbol>) {
        for symbol in symbols {
            self.insert(symbol);
        }
    }
}
pub(super) fn collect_head(
    lane: &ReconstructionLane<'_>,
    partition: &PartitionState,
) -> BTreeSet<Symbol> {
    collect_counted(lane, partition, false).ids
}
pub(super) fn collect(
    lane: &ReconstructionLane<'_>,
    partition: &PartitionState,
) -> BTreeSet<Symbol> {
    collect_counted(lane, partition, true).ids
}
pub(super) fn collect_counted(
    _lane: &ReconstructionLane<'_>,
    partition: &PartitionState,
    full: bool,
) -> CollectedSymbols {
    let mut symbols = CollectedSymbols::default();
    for versions in partition
        .entity_arena
        .aspect_versions
        .iter()
        .chain(partition.relation_arena.aspect_versions.iter())
    {
        symbols.visits += 1;
        symbols.extend(versions.keys().copied());
    }
    for extra in partition.entity_arena.extra.iter() {
        symbols.visits += 1;
        if let Some(fingerprint) = extra.structural_fingerprint {
            symbols.insert(fingerprint.family);
        }
    }
    for revisions in partition
        .entity_arena
        .field_revisions
        .iter()
        .chain(partition.relation_arena.field_revisions.iter())
        .flatten()
    {
        for (aspect, field) in revisions.keys() {
            symbols.insert(*aspect);
            symbols.insert(*field);
        }
    }
    let entities = partition
        .entity_arena
        .extra
        .iter()
        .map(|extra| extra.authoritative_aspect_state.as_ref());
    let relations = partition
        .relation_arena
        .extra
        .iter()
        .map(|extra| extra.authoritative_aspect_state.as_ref());
    for state in entities.chain(relations).flatten() {
        symbols.visits += 1;
        visit_state(state, &mut symbols);
    }
    if !full {
        return symbols;
    }
    for history in partition.entity_arena.metadata_history.iter() {
        symbols.visits += 1;
        for metadata in history.iter() {
            symbols.visits += 1;
            if let Some(state) = &metadata.authoritative_aspect_state {
                visit_state(state, &mut symbols);
            }
        }
    }
    for history in partition.relation_arena.metadata_history.iter() {
        symbols.visits += 1;
        for metadata in history.iter() {
            symbols.visits += 1;
            if let Some(state) = &metadata.authoritative_aspect_state {
                visit_state(state, &mut symbols);
            }
        }
    }
    symbols
}

fn visit_state(state: &AuthoritativeRecordAspectState, symbols: &mut CollectedSymbols) {
    symbols.visits += 1;
    for (_, aspect) in state.aspects().entries() {
        symbols.visits += 1;
        match aspect.view() {
            ContractValidatedAspectValueView::Scalar(value) => visit_value(value, symbols),
            ContractValidatedAspectValueView::Struct(value) => {
                for (_, field) in value.fields() {
                    visit_value(field, symbols);
                }
            }
        }
    }
}

fn visit_value(value: &AspectValue, symbols: &mut CollectedSymbols) {
    symbols.visits += 1;
    if let AspectValue::String(InternedString::Symbol(symbol)) = value {
        symbols.insert(Symbol(symbol.0));
    }
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
                if let Some(state) = &new.authoritative_aspect_state {
                    visit_state(state, &mut result);
                }
            }
        },
    );
    current.relation_arena.extra.visit_changed_values(
        &previous.relation_arena.extra,
        &mut |_, new, _| {
            result.visits += 1;
            if let Some(state) = new.and_then(|extra| extra.authoritative_aspect_state.as_ref()) {
                visit_state(state, &mut result);
            }
        },
    );
    if full {
        history(
            &previous.entity_arena.metadata_history,
            &current.entity_arena.metadata_history,
            |meta| meta.authoritative_aspect_state.as_ref(),
            &mut result,
        );
        history(
            &previous.relation_arena.metadata_history,
            &current.relation_arena.metadata_history,
            |meta| meta.authoritative_aspect_state.as_ref(),
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
fn history<M: Clone>(
    previous: &crate::storage::substrate::SharedColumn<crate::storage::substrate::SharedColumn<M>>,
    current: &crate::storage::substrate::SharedColumn<crate::storage::substrate::SharedColumn<M>>,
    state: impl Fn(&M) -> Option<&AuthoritativeRecordAspectState>,
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
    pub(super) fn capture(
        lane: &ReconstructionLane<'_>,
        partition: &PartitionState,
        full: bool,
    ) -> Self {
        Self {
            head: Arc::new(collect_counted(lane, partition, false)),
            owned: Some(Arc::new(collect_counted(lane, partition, true))),
            full,
        }
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
