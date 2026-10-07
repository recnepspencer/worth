use super::super::super::index::{append_index_for_field, IndexTouchInput};
use super::*;
use crate::identity::data::KindId;
use crate::storage::data::{RelationalFieldPresence, RelationalFieldRevision};
use crate::symbols::data::Symbol;
use std::mem::size_of;
use worth_foundational::facade::{AspectKey, CanonicalFieldPath, FieldKey};

pub(super) fn append_aspect(
    runtime: &crate::runtime::RelationalPreparationRuntime,
    symbol: Symbol,
    record: &RecordRef,
    touches: &mut Vec<Touch>,
    budget: &mut TouchBudget<'_, '_, '_, '_>,
) -> Result<bool, Failure> {
    let Some(aspect) = resolve_aspect(runtime, symbol, budget)? else {
        return Ok(false);
    };
    budget.claim(aspect.owned_allocation_capacity_bytes() as u64)?;
    push(
        touches,
        Touch::AspectRevision {
            record: record.clone(),
            aspect,
        },
        budget,
    )?;
    Ok(true)
}

pub(super) struct FieldDifference<'a> {
    pub(super) selected: &'a crate::branch::SelectedRelationalBranchState,
    pub(super) working: &'a crate::runtime::WorkingState,
    pub(super) symbols: (Symbol, Symbol),
    pub(super) old_record: Option<&'a RecordRef>,
    pub(super) new_record: Option<&'a RecordRef>,
    pub(super) old_kind: Option<KindId>,
    pub(super) new_kind: Option<KindId>,
    pub(super) new_revision: Option<&'a RelationalFieldRevision>,
}

pub(super) fn append_field(
    runtime: &crate::runtime::RelationalPreparationRuntime,
    difference: FieldDifference<'_>,
    touches: &mut Vec<Touch>,
    budget: &mut TouchBudget<'_, '_, '_, '_>,
) -> Result<bool, Failure> {
    let FieldDifference {
        selected,
        working,
        symbols,
        old_record,
        new_record,
        old_kind,
        new_kind,
        new_revision,
    } = difference;
    let Some(record) = (if new_revision.is_some() {
        new_record.or(old_record)
    } else {
        old_record.or(new_record)
    }) else {
        return Ok(false);
    };
    let Some(aspect) = resolve_aspect(runtime, symbols.0, budget)? else {
        return Ok(false);
    };
    let Some(field) = resolve_field(runtime, symbols.1, budget)? else {
        return Ok(false);
    };
    let Some(kind) = (if new_revision.is_some() {
        new_kind.or(old_kind)
    } else {
        old_kind.or(new_kind)
    }) else {
        return Ok(false);
    };
    budget.claim(
        (aspect.owned_allocation_capacity_bytes()
            + field.owned_allocation_capacity_bytes()
            + size_of::<FieldKey>()) as u64,
    )?;
    push(
        touches,
        Touch::FieldRevision {
            record: record.clone(),
            kind,
            aspect: aspect.clone(),
            path: CanonicalFieldPath::single(field.clone()),
            presence: new_revision.map_or(RelationalFieldPresence::Absent, |revision| {
                revision.presence()
            }),
        },
        budget,
    )?;
    runtime
        .index_definitions
        .with_field(&aspect, &field, |definitions| {
            for definition in definitions {
                let locator = match &definition.kind {
                    crate::indexes::data::DerivedIndexKind::EntityField { field_locator }
                        if matches!(record, RecordRef::Entity(_)) =>
                    {
                        field_locator
                    }
                    crate::indexes::data::DerivedIndexKind::RelationField { field_locator }
                        if matches!(record, RecordRef::Relation(_)) =>
                    {
                        field_locator
                    }
                    _ => continue,
                };
                append_index_for_field(
                    IndexTouchInput {
                        selected,
                        working,
                        old_record: old_record.unwrap_or(record),
                        new_record: new_record.unwrap_or(record),
                        old_kind,
                        new_kind,
                        aspect: &aspect,
                        field: &field,
                        definition,
                        locator,
                    },
                    touches,
                    budget,
                )?;
            }
            Ok::<(), Failure>(())
        })?;
    Ok(true)
}

fn resolve_aspect(
    runtime: &crate::runtime::RelationalPreparationRuntime,
    symbol: Symbol,
    budget: &mut TouchBudget<'_, '_, '_, '_>,
) -> Result<Option<AspectKey>, Failure> {
    budget.checkpoint(1)?;
    runtime.services.symbols.with_read(|symbols| {
        let Some(name) = symbols.resolve(symbol) else {
            return Ok(None);
        };
        budget.checkpoint(1 + (name.len() as u64 / 64))?;
        budget.claim(name.len() as u64)?;
        Ok(AspectKey::new(name.to_owned()))
    })
}

fn resolve_field(
    runtime: &crate::runtime::RelationalPreparationRuntime,
    symbol: Symbol,
    budget: &mut TouchBudget<'_, '_, '_, '_>,
) -> Result<Option<FieldKey>, Failure> {
    budget.checkpoint(1)?;
    runtime.services.symbols.with_read(|symbols| {
        let Some(name) = symbols.resolve(symbol) else {
            return Ok(None);
        };
        budget.checkpoint(1 + (name.len() as u64 / 64))?;
        budget.claim(name.len() as u64)?;
        Ok(FieldKey::new(name.to_owned()))
    })
}
