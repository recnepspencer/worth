//! Dictionary closure of an admitted region view, including aspect revision
//! keys and fingerprint families as well as retained native field values.
use crate::storage::overlay::PartitionState;
use crate::symbols::data::Symbol;
use std::collections::BTreeSet;
use worth_foundational::facade::{
    AspectValue, AuthoritativeRecordAspectState, ContractValidatedAspectValueView, InternedString,
};

pub(super) fn collect_head(partition: &PartitionState) -> BTreeSet<Symbol> {
    collect_head_using(partition, &mut |_| Ok(())).expect("ordinary symbol collection cannot stop")
}
fn collect_head_using(
    partition: &PartitionState,
    checkpoint: &mut dyn FnMut(usize) -> Result<(), super::RelationalResidencyDemandFailure>,
) -> Result<BTreeSet<Symbol>, super::RelationalResidencyDemandFailure> {
    let mut symbols = BTreeSet::new();
    for versions in partition
        .entity_arena
        .aspect_versions
        .iter()
        .chain(partition.relation_arena.aspect_versions.iter())
    {
        checkpoint(symbols.len())?;
        for &symbol in versions.keys() {
            checkpoint(symbols.len())?;
            symbols.insert(symbol);
        }
    }
    for extra in partition.entity_arena.extra.iter() {
        checkpoint(symbols.len())?;
        if let Some(fingerprint) = extra.structural_fingerprint {
            symbols.insert(fingerprint.family);
        }
    }
    for revisions in partition
        .entity_arena
        .field_revisions
        .iter()
        .chain(partition.relation_arena.field_revisions.iter())
    {
        checkpoint(symbols.len())?;
        let Some(revisions) = revisions else {
            continue;
        };
        for (aspect, field) in revisions.keys() {
            checkpoint(symbols.len())?;
            symbols.insert(*aspect);
            symbols.insert(*field);
        }
    }
    let entities = partition
        .entity_arena
        .extra
        .iter()
        .map(|extra| extra.authoritative_aspect_state());
    let relations = partition
        .relation_arena
        .extra
        .iter()
        .map(|extra| extra.authoritative_aspect_state());
    for state in entities.chain(relations) {
        checkpoint(symbols.len())?;
        if let Some(state) = state {
            visit_state(state, &mut symbols, checkpoint)?;
        }
    }
    Ok(symbols)
}

pub(super) fn collect(partition: &PartitionState) -> BTreeSet<Symbol> {
    collect_using(partition, &mut |_| Ok(())).expect("ordinary symbol collection cannot stop")
}
pub(super) fn collect_for_request(
    partition: &PartitionState,
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<BTreeSet<Symbol>, super::RelationalResidencyDemandFailure> {
    collect_using(partition, &mut |count| {
        super::region_load::materialization_funding::work(
            request,
            1 + 22 * (count + 1).ilog2() as u64,
        )
    })
}
fn collect_using(
    partition: &PartitionState,
    checkpoint: &mut dyn FnMut(usize) -> Result<(), super::RelationalResidencyDemandFailure>,
) -> Result<BTreeSet<Symbol>, super::RelationalResidencyDemandFailure> {
    let mut symbols = collect_head_using(partition, checkpoint)?;
    for state in partition
        .entity_arena
        .metadata_history
        .iter()
        .flat_map(|history| {
            history
                .iter()
                .map(|metadata| metadata.authoritative_aspect_state())
        })
        .chain(
            partition
                .relation_arena
                .metadata_history
                .iter()
                .flat_map(|history| {
                    history
                        .iter()
                        .map(|metadata| metadata.authoritative_aspect_state())
                }),
        )
    {
        checkpoint(symbols.len())?;
        if let Some(state) = state {
            visit_state(state, &mut symbols, checkpoint)?;
        }
    }
    Ok(symbols)
}

fn visit_state(
    state: &AuthoritativeRecordAspectState,
    symbols: &mut BTreeSet<Symbol>,
    checkpoint: &mut dyn FnMut(usize) -> Result<(), super::RelationalResidencyDemandFailure>,
) -> Result<(), super::RelationalResidencyDemandFailure> {
    checkpoint(symbols.len())?;
    for (_, aspect) in state.aspects().entries() {
        checkpoint(symbols.len())?;
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
    symbols: &mut BTreeSet<Symbol>,
    checkpoint: &mut dyn FnMut(usize) -> Result<(), super::RelationalResidencyDemandFailure>,
) -> Result<(), super::RelationalResidencyDemandFailure> {
    checkpoint(symbols.len())?;
    if let AspectValue::String(InternedString::Symbol(symbol)) = value {
        symbols.insert(Symbol(symbol.0));
    }
    Ok(())
}
