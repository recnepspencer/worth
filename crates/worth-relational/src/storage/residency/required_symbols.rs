//! Dictionary closure of an admitted region view, including aspect revision
//! keys and fingerprint families as well as retained native field values.
use crate::storage::overlay::PartitionState;
use crate::symbols::data::Symbol;
use std::collections::BTreeSet;
use worth_foundational::facade::{
    AspectValue, AuthoritativeRecordAspectState, ContractValidatedAspectValueView, InternedString,
};

pub(super) fn collect(partition: &PartitionState) -> BTreeSet<Symbol> {
    let mut symbols = BTreeSet::new();
    for versions in partition
        .entity_arena
        .aspect_versions
        .iter()
        .chain(partition.relation_arena.aspect_versions.iter())
    {
        symbols.extend(versions.keys().copied());
    }
    for extra in partition.entity_arena.extra.iter() {
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
        .metadata_history
        .iter()
        .flat_map(|history| {
            history
                .iter()
                .map(|metadata| metadata.authoritative_aspect_state.as_ref())
        })
        .chain(
            partition
                .entity_arena
                .extra
                .iter()
                .map(|extra| extra.authoritative_aspect_state.as_ref()),
        );
    let relations = partition
        .relation_arena
        .metadata_history
        .iter()
        .flat_map(|history| {
            history
                .iter()
                .map(|metadata| metadata.authoritative_aspect_state.as_ref())
        })
        .chain(
            partition
                .relation_arena
                .extra
                .iter()
                .map(|extra| extra.authoritative_aspect_state.as_ref()),
        );
    for state in entities.chain(relations).flatten() {
        visit_state(state, &mut symbols);
    }
    symbols
}

fn visit_state(state: &AuthoritativeRecordAspectState, symbols: &mut BTreeSet<Symbol>) {
    for (_, aspect) in state.aspects().entries() {
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

fn visit_value(value: &AspectValue, symbols: &mut BTreeSet<Symbol>) {
    if let AspectValue::String(InternedString::Symbol(symbol)) = value {
        symbols.insert(Symbol(symbol.0));
    }
}
