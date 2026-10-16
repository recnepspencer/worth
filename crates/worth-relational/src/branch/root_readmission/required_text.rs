//! Dictionary closure for authoritative field text before native root readmission.
use std::collections::BTreeMap;

use crate::{
    branch::RelationalBranchRootCaptureDenial,
    identity::data::PartitionId,
    storage::overlay::PartitionState,
    symbols::data::{StringInterner, Symbol},
};
use worth_foundational::facade::{
    AspectValue, AuthoritativeRecordAspectState, ContractValidatedAspectValueView, InternedString,
};

/// Recovery already reconstructs each root. Validate its retained values without
/// allocating a second dictionary or treating numeric symbol IDs as text meaning.
pub(super) fn validate(
    partitions: &BTreeMap<PartitionId, PartitionState>,
    symbols: &StringInterner,
) -> Result<(), RelationalBranchRootCaptureDenial> {
    for partition in partitions.values() {
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
            validate_state(state, symbols)?;
        }
    }
    Ok(())
}

fn validate_state(
    state: &AuthoritativeRecordAspectState,
    symbols: &StringInterner,
) -> Result<(), RelationalBranchRootCaptureDenial> {
    for (_, aspect) in state.aspects().entries() {
        match aspect.view() {
            ContractValidatedAspectValueView::Scalar(value) => validate_value(value, symbols)?,
            ContractValidatedAspectValueView::Struct(value) => {
                for (_, field) in value.fields() {
                    validate_value(field, symbols)?;
                }
            }
        }
    }
    Ok(())
}

fn validate_value(
    value: &AspectValue,
    symbols: &StringInterner,
) -> Result<(), RelationalBranchRootCaptureDenial> {
    if let AspectValue::String(InternedString::Symbol(symbol)) = value {
        let symbol = Symbol(symbol.0);
        if symbols.retained_text(symbol).is_none() {
            return Err(RelationalBranchRootCaptureDenial::UnresolvedContentSymbol(
                symbol,
            ));
        }
    }
    Ok(())
}
