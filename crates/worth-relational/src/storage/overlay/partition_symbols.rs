//! The symbol closure of retained partition storage, including historical rows.
use super::PartitionState;
use crate::storage::substrate::{RecordArena, RecordKind};
use crate::symbols::data::Symbol;
use worth_foundational::facade::{
    AspectValue, AuthoritativeRecordAspectState, ContractValidatedAspectValueView, InternedString,
};

impl PartitionState {
    /// Cold checkpoint/readmission traversal. Pending preparation dictionaries
    /// are not storage; every symbol referenced by retained storage is visited.
    pub(crate) fn visit_retained_symbols<E>(
        &self,
        visit: &mut impl FnMut(Symbol) -> Result<(), E>,
    ) -> Result<(), E> {
        visit_columns(&self.entity_arena, visit)?;
        visit_columns(&self.relation_arena, visit)?;
        for extra in self.entity_arena.extra.iter() {
            if let Some(fingerprint) = extra.structural_fingerprint {
                visit(fingerprint.family)?;
            }
        }
        let entities = self
            .entity_arena
            .metadata_history
            .iter()
            .flat_map(|history| {
                history
                    .iter()
                    .map(|row| row.authoritative_aspect_state.as_ref())
            })
            .chain(
                self.entity_arena
                    .extra
                    .iter()
                    .map(|row| row.authoritative_aspect_state.as_ref()),
            );
        let relations = self
            .relation_arena
            .metadata_history
            .iter()
            .flat_map(|history| {
                history
                    .iter()
                    .map(|row| row.authoritative_aspect_state.as_ref())
            })
            .chain(
                self.relation_arena
                    .extra
                    .iter()
                    .map(|row| row.authoritative_aspect_state.as_ref()),
            );
        for state in entities.chain(relations).flatten() {
            visit_state(state, visit)?;
        }
        Ok(())
    }
}

fn visit_columns<K: RecordKind, E>(
    arena: &RecordArena<K>,
    visit: &mut impl FnMut(Symbol) -> Result<(), E>,
) -> Result<(), E> {
    for versions in arena.aspect_versions.iter() {
        for symbol in versions.keys() {
            visit(*symbol)?;
        }
    }
    for revisions in arena.field_revisions.iter().flatten() {
        for (aspect, field) in revisions.keys() {
            visit(*aspect)?;
            visit(*field)?;
        }
    }
    for enrichment in arena.diagnostics_enrichment.iter() {
        for symbol in enrichment.keys() {
            visit(*symbol)?;
        }
    }
    Ok(())
}

fn visit_state<E>(
    state: &AuthoritativeRecordAspectState,
    visit: &mut impl FnMut(Symbol) -> Result<(), E>,
) -> Result<(), E> {
    for (_, aspect) in state.aspects().entries() {
        match aspect.view() {
            ContractValidatedAspectValueView::Scalar(value) => visit_value(value, visit)?,
            ContractValidatedAspectValueView::Struct(value) => {
                for (_, field) in value.fields() {
                    visit_value(field, visit)?;
                }
            }
        }
    }
    Ok(())
}
fn visit_value<E>(
    value: &AspectValue,
    visit: &mut impl FnMut(Symbol) -> Result<(), E>,
) -> Result<(), E> {
    if let AspectValue::String(InternedString::Symbol(symbol)) = value {
        visit(Symbol(symbol.0))?;
    }
    Ok(())
}
